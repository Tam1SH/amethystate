use crate::store::{
    RawStorage, SchemaAwareStore, Store, StoreCallback, StoreEvent, StoreOp, SubscriptionEntry,
    SubscriptionId, SubscriptionKind,
};
use error::RedbStoreError;
use raw_storage::RedbRawStorage;
use redb::{Database, ReadableDatabase};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use tables::{TABLE_DATA, TABLE_DIFF_LOG, TABLE_LOG, TABLE_META, TABLE_MIGRATION_LOG};

use crate::store::config::StoreConfig;
use crate::{MigrationReport, Result};

use crate::codec::CodecError;
use crate::migration::engine::{MigrationEngine, StorageProvider};
use crate::migration::set::MigrationSet;
use crate::store::backend::redb::tables::TABLE_SCHEMA_SNAPSHOT;
use crate::store::util::debouncer::Debouncer;
use bytes::Bytes;
use parking_lot::{Mutex, RwLock};
use rmp_serde::Serializer;
use rmp_serde::config::BytesMode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::{thread, time::Duration};
use tracing::{info, warn};

pub mod error;
mod events;
mod raw_storage;
mod tables;

const BUF_SIZE: usize = 64 * 1024;

#[cfg(test)]
static SIMULATE_WRITE_FAILURE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

thread_local! {
    static SERIALIZATION_BUFFER: std::cell::RefCell<Vec<u8>> =
        std::cell::RefCell::new(Vec::with_capacity(BUF_SIZE));
}

#[derive(Clone)]
pub struct RedbStore {
    db: Arc<Database>,
    //TODO: ordering?
    pending: Arc<Mutex<HashMap<Arc<str>, Option<Bytes>>>>,
    debouncer: Arc<Debouncer>,
    subscriptions: Arc<RwLock<Vec<SubscriptionEntry>>>,
    next_sub_id: Arc<AtomicU64>,

    write_lock: Arc<Mutex<()>>,
    watcher_tx: Arc<std::sync::mpsc::Sender<()>>,
    watcher_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RedbStore {
    pub fn open(
        config: StoreConfig,
        migration_set: MigrationSet,
    ) -> Result<(Self, MigrationReport)> {
        let db = Arc::new(Database::create(&config.path).map_err(RedbStoreError::from)?);

        let write_txn = db.begin_write().map_err(RedbStoreError::from)?;
        {
            let _ = write_txn
                .open_table(TABLE_DATA)
                .map_err(RedbStoreError::from)?;
            let _ = write_txn
                .open_table(TABLE_LOG)
                .map_err(RedbStoreError::from)?;
            let _ = write_txn
                .open_table(TABLE_META)
                .map_err(RedbStoreError::from)?;
            let _ = write_txn
                .open_table(TABLE_DIFF_LOG)
                .map_err(RedbStoreError::from)?;
            let _ = write_txn
                .open_table(TABLE_MIGRATION_LOG)
                .map_err(RedbStoreError::from)?;
            let _ = write_txn
                .open_table(TABLE_SCHEMA_SNAPSHOT)
                .map_err(RedbStoreError::from)?;
        }
        write_txn.commit().map_err(RedbStoreError::from)?;

        let pending = Arc::new(Mutex::new(HashMap::<Arc<str>, Option<Bytes>>::new()));
        let subscriptions = Arc::new(RwLock::new(Vec::new()));
        let (w_tx, w_rx) = std::sync::mpsc::channel();
        let db_inner = db.clone();
        let subs_inner = subscriptions.clone();

        let db_save = db.clone();
        let pending_save = pending.clone();

        let write_lock = Arc::new(Mutex::new(()));
        let write_lock_save = write_lock.clone();

        let debouncer = Debouncer::new(config.save_debounce, move || {
            let _write_guard = write_lock_save.lock();

            let changes = {
                let lock = pending_save.lock();
                if lock.is_empty() {
                    return;
                }
                lock.clone()
            };

            let success = (|| -> Option<bool> {
                #[cfg(test)]
                if SIMULATE_WRITE_FAILURE.load(Ordering::Relaxed) {
                    return None;
                }

                let txn = db_save.begin_write().ok()?;
                {
                    let mut table = txn.open_table(TABLE_DATA).ok()?;
                    for (path, opt_bytes) in &changes {
                        match opt_bytes {
                            Some(b) => {
                                table.insert(&**path, &b[..]).ok()?;
                            }
                            None => {
                                table.remove(&**path).ok()?;
                            }
                        }
                    }
                }
                txn.commit().ok()?;
                Some(true)
            })()
            .unwrap_or(false);

            if success {
                let mut lock = pending_save.lock();
                for key in changes.keys() {
                    lock.remove(key);
                }
            }
        });

        let watcher_handle = thread::spawn(move || {
            while w_rx.recv_timeout(Duration::from_millis(300)).is_err() {
                let _ = events::process_inbox(&db_inner, &subs_inner);
            }
        });

        let store = Self {
            db: db.clone(),
            pending,
            debouncer: Arc::new(debouncer),
            subscriptions: subscriptions.clone(),
            next_sub_id: Arc::new(AtomicU64::new(1)),
            write_lock,
            watcher_tx: Arc::new(w_tx),
            watcher_handle: Arc::new(Mutex::new(Some(watcher_handle))),
        };

        let report = store.run_migrations(migration_set)?;

        Ok((store, report))
    }

    pub fn close(&mut self) -> Result<()> {
        info!("Closing RedbStore explicitly...");

        let _ = self.watcher_tx.send(());
        if let Some(handle) = self.watcher_handle.lock().take() {
            let _ = handle.join();
        }

        self.save_now()?;

        Ok(())
    }

    pub fn flush_prefix(&self, prefix: &str) -> Result<()> {
        let _write_guard = self.write_lock.lock();

        let changes = {
            let mut lock = self.pending.lock();
            if lock.is_empty() {
                return Ok(());
            }

            let mut matched = HashMap::new();

            if prefix.is_empty() {
                matched = std::mem::take(&mut *lock);
            } else {
                let prefix_dot = format!("{}.", prefix);
                let keys_to_remove: Vec<Arc<str>> = lock
                    .keys()
                    .filter(|k| k.starts_with(&prefix_dot) || &***k == prefix)
                    .cloned()
                    .collect();

                for k in keys_to_remove {
                    if let Some(v) = lock.remove(&k) {
                        matched.insert(k, v);
                    }
                }
            }

            if matched.is_empty() {
                return Ok(());
            }
            matched
        };

        let txn = self.db.begin_write().map_err(RedbStoreError::from)?;
        {
            let mut table = txn.open_table(TABLE_DATA).map_err(RedbStoreError::from)?;
            for (path, opt_bytes) in changes {
                match opt_bytes {
                    Some(b) => {
                        table.insert(&*path, &b[..]).map_err(RedbStoreError::from)?;
                    }
                    None => {
                        table.remove(&*path).map_err(RedbStoreError::from)?;
                    }
                }
            }
        }
        txn.commit().map_err(RedbStoreError::from)?;
        Ok(())
    }

    fn check_debouncer(&self) {
        if self.debouncer.is_poisoned() {
            panic!("debouncer thread is dead — store integrity cannot be guaranteed");
        }
    }
}

impl Drop for RedbStore {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl SchemaAwareStore for RedbStore {
    fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport> {
        struct RedbProvider<'a> {
            db: &'a Database,
        }

        impl<'a> StorageProvider for RedbProvider<'a> {
            fn atomic<F, T>(&self, f: F) -> Result<T>
            where
                F: FnOnce(&mut dyn RawStorage) -> Result<T>,
            {
                let write_txn = self.db.begin_write().map_err(RedbStoreError::from)?;

                let res = {
                    let mut storage = RedbRawStorage::new(&write_txn);
                    f(&mut storage)?
                };

                write_txn.commit().map_err(RedbStoreError::from)?;
                Ok(res)
            }
        }

        let provider = RedbProvider { db: &self.db };
        let engine = MigrationEngine::new(&provider);
        engine.run(mset)
    }
}

impl Store for RedbStore {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        {
            let lock = self.pending.lock();
            if let Some(opt_bytes) = lock.get(path) {
                return match opt_bytes {
                    Some(bytes) => Ok(Some(
                        rmp_serde::from_slice(bytes).map_err(CodecError::from)?,
                    )),
                    None => Ok(None),
                };
            }
        }

        let read_txn = self.db.begin_read().map_err(RedbStoreError::from)?;
        let table = read_txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;
        match table.get(path).map_err(RedbStoreError::from)? {
            Some(access_guard) => {
                let bytes = access_guard.value();
                Ok(Some(
                    rmp_serde::from_slice(bytes).map_err(CodecError::from)?,
                ))
            }
            None => Ok(None),
        }
    }
    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()> {
        self.set_owned(Arc::from(path), value)
    }

    fn set_owned<T: Serialize>(&self, path: Arc<str>, value: &T) -> Result<()> {
        self.check_debouncer();
        let bytes = SERIALIZATION_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            b.clear();
            let mut ser = Serializer::new(&mut *b).with_bytes(BytesMode::ForceAll);
            value.serialize(&mut ser).map_err(CodecError::from)?;

            Ok::<Bytes, RedbStoreError>(Bytes::copy_from_slice(&b))
        })?;

        let old_bytes = {
            let lock = self.pending.lock();
            lock.get(&*path).cloned().flatten()
        };

        {
            let mut lock = self.pending.lock();
            lock.insert(path.clone(), Some(bytes.clone()));
        }

        events::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: path.clone(),
                op: StoreOp::Set,
                old: old_bytes,
                new: Some(bytes),
            },
        );

        self.debouncer.schedule();
        Ok(())
    }

    fn save_now(&self) -> Result<()> {
        self.flush_prefix("")
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Bytes)>> {
        let mut results = Vec::new();

        let read_txn = self.db.begin_read().map_err(RedbStoreError::from)?;
        let table = read_txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;

        let range = prefix..;
        for result in table.range(range).map_err(RedbStoreError::from)? {
            let (k, v) = result.map_err(RedbStoreError::from)?;
            let key_str = k.value();
            if key_str.starts_with(prefix) {
                results.push((key_str.to_string(), Bytes::copy_from_slice(v.value())));
            } else {
                break;
            }
        }

        let mut pending_map = HashMap::new();
        {
            let lock = self.pending.lock();
            for (k, opt_v) in lock.iter() {
                if k.starts_with(prefix) {
                    pending_map.insert(k.to_string(), opt_v.clone());
                }
            }
        }

        for (k, opt_v) in pending_map {
            if let Some(v) = opt_v {
                if let Some(pos) = results.iter().position(|(rk, _)| *rk == k) {
                    results[pos].1 = v;
                } else {
                    results.push((k, v));
                }
            } else {
                results.retain(|(rk, _)| *rk != k);
            }
        }

        Ok(results)
    }

    fn delete(&self, path: &str) -> Result<()> {
        self.check_debouncer();
        let path_arc: Arc<str> = Arc::from(path);

        let old_bytes = {
            let lock = self.pending.lock();
            if let Some(p) = lock.get(path) {
                p.clone()
            } else {
                let read_txn = self.db.begin_read().map_err(RedbStoreError::from)?;
                let table = read_txn
                    .open_table(TABLE_DATA)
                    .map_err(RedbStoreError::from)?;
                table
                    .get(path)
                    .map_err(RedbStoreError::from)?
                    .map(|v| Bytes::copy_from_slice(v.value()))
            }
        };

        {
            let mut lock = self.pending.lock();
            lock.insert(path_arc.clone(), None);
        }

        events::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: path_arc,
                op: StoreOp::Delete,
                old: old_bytes,
                new: None,
            },
        );

        self.debouncer.schedule();
        Ok(())
    }

    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId {
        let id = self.next_sub_id.fetch_add(1, Ordering::Relaxed);
        self.subscriptions
            .write()
            .push(SubscriptionEntry { id, kind, callback });
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.subscriptions.write().retain(|s| s.id != id);
    }

    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T> {
        match rmp_serde::from_slice(bytes) {
            Ok(val) => Ok(val),
            Err(e) => {
                warn!(
                    target: "rpstate",
                    "Failed to decode field. Data is corrupted or type changed. \
                    Using Default value. Error: {e}"
                );
                Ok(T::default())
            }
        }
    }

    fn flush_prefix(&self, prefix: &str) -> Result<()> {
        Self::flush_prefix(self, prefix)
    }

    fn is_initialized(&self, namespace: &str) -> Result<bool> {
        let key = format!("__init::{namespace}");
        let read_txn = self.db.begin_read().map_err(RedbStoreError::from)?;
        let table = read_txn
            .open_table(TABLE_META)
            .map_err(RedbStoreError::from)?;
        Ok(table
            .get(key.as_str())
            .map_err(RedbStoreError::from)?
            .is_some())
    }

    fn mark_initialized(&self, namespace: &str) -> Result<()> {
        let key = format!("__init::{namespace}");
        let write_txn = self.db.begin_write().map_err(RedbStoreError::from)?;
        {
            let mut table = write_txn
                .open_table(TABLE_META)
                .map_err(RedbStoreError::from)?;
            table
                .insert(key.as_str(), &[][..])
                .map_err(RedbStoreError::from)?;
        }
        write_txn.commit().map_err(RedbStoreError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StoreBuilder;
    use crate::migration::fields::FieldDescriptor;
    use crate::migration::{MigrationError, MigrationPlan};
    use redb::ReadableTableMetadata;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_test::traced_test;

    const EMPTY_FIELDS: &[FieldDescriptor] = &[];

    fn unique_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rpstate-redb-{suffix}-{nanos}.redb"))
    }

    #[test]
    fn test_set_get_immediate() {
        let path = unique_path("immediate");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();

        store.set("user.name", &"Alice".to_string()).unwrap();

        let val: Option<String> = store.get("user.name").unwrap();
        assert_eq!(val, Some("Alice".to_string()));
    }

    #[test]
    fn test_debouncer_persistence() {
        let path = unique_path("debounce");

        let mut config = StoreConfig::new(path);
        config.save_debounce = Duration::from_millis(50);

        let (store, _) = RedbStore::open(config, MigrationSet::default()).unwrap();

        store.set("config.port", &8080u16).unwrap();

        {
            let read_txn = store.db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE_DATA).unwrap();
            assert!(table.get("config.port").unwrap().is_none());
        }

        thread::sleep(Duration::from_millis(500));

        {
            let read_txn = store.db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE_DATA).unwrap();
            assert!(table.get("config.port").unwrap().is_some());
        }
    }

    #[test]
    fn test_local_reactivity() {
        let path = unique_path("reactivity");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();

        let hit = Arc::new(Mutex::new(false));
        let hit_inner = hit.clone();

        store.subscribe(
            SubscriptionKind::ExactPath(Arc::from("ui.theme")),
            Arc::new(move |_| {
                let mut guard = hit_inner.lock();
                *guard = true;
            }),
        );

        store.set("ui.theme", &"dark".to_string()).unwrap();

        assert!(*hit.lock());
    }

    #[test]
    fn test_inbox_watcher_sync() {
        let path = unique_path("inbox");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        store.subscribe(
            SubscriptionKind::Any,
            Arc::new(move |evt| {
                let _ = tx.send(evt.clone());
            }),
        );

        {
            let write_txn = store.db.begin_write().unwrap();
            {
                let mut data_table = write_txn.open_table(TABLE_DATA).unwrap();
                let mut log_table = write_txn.open_table(TABLE_LOG).unwrap();

                let val = rmp_serde::to_vec(&"external_change").unwrap();
                data_table.insert("app.version", val.as_slice()).unwrap();

                log_table.insert(1u64, "app.version").unwrap();
            }
            write_txn.commit().unwrap();
        }

        let event = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Watcher should detect external change");

        assert_eq!(&*event.path, "app.version");
        assert_eq!(event.op, StoreOp::Set);

        let val: String = store.get("app.version").unwrap().unwrap();
        assert_eq!(val, "external_change");

        thread::sleep(Duration::from_millis(100));
        {
            let read_txn = store.db.begin_read().unwrap();
            let log_table = read_txn.open_table(TABLE_LOG).unwrap();
            assert_eq!(log_table.len().unwrap(), 0);
        }
    }

    #[test]
    fn test_delete_flow() {
        let path = unique_path("delete");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();

        store.set("temp.key", &1).unwrap();

        store.save_now().unwrap();
        store.delete("temp.key").unwrap();
        assert_eq!(store.get::<i32>("temp.key").unwrap(), None);

        store.save_now().unwrap();

        let read_txn = store.db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE_DATA).unwrap();
        assert!(table.get("temp.key").unwrap().is_none());
    }

    #[test]
    fn test_smart_recovery_decode() {
        let path = unique_path("recovery");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();
        let garbage = vec![0x00, 0x01, 0x02];

        let result: String = store.decode(&garbage).unwrap();
        assert_eq!(result, String::default());
    }

    #[test]
    fn test_deterministic_closure_and_reopen() {
        let path = unique_path("closure");
        {
            let (mut store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            store.set("test.key", &"hello".to_string()).unwrap();
            store.close().expect("Explicit close failed");
        }

        let (store_reopened, _) = RedbStore::open(StoreConfig::new(&path), MigrationSet::default())
            .expect("Database should be available immediately after close");

        let val: Option<String> = store_reopened.get("test.key").unwrap();
        assert_eq!(val, Some("hello".to_string()));
    }

    #[test]
    fn test_drop_behavior_is_deterministic() {
        let path = unique_path("drop_logic");
        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            store.set("drop.test", &42u32).unwrap();
        }

        let (store_reopened, _) = RedbStore::open(StoreConfig::new(&path), MigrationSet::default())
            .expect("Drop must release file lock deterministically");

        assert_eq!(store_reopened.get::<u32>("drop.test").unwrap(), Some(42));
    }

    #[test]
    fn test_close_saves_pending_data() {
        let path = unique_path("save_on_close");
        let mut config = StoreConfig::new(&path);
        config.save_debounce = Duration::from_secs(3600);

        {
            let (mut store, _) = RedbStore::open(config, MigrationSet::default()).unwrap();
            store.set("urgent.data", &true).unwrap();
            store.close().unwrap();
        }

        let (store, _) = RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
        assert_eq!(store.get::<bool>("urgent.data").unwrap(), Some(true));
    }

    #[test]
    fn test_granular_flush_prefix_drains_buffer() {
        let path = unique_path("granular_flush");
        let mut config = StoreConfig::new(&path);

        config.save_debounce = Duration::from_secs(3600);

        let (store, _) = RedbStore::open(config, MigrationSet::default()).unwrap();

        store.set("net.host", &"127.0.0.1".to_string()).unwrap();
        store.set("net.port", &8080u16).unwrap();
        store.set("ui.theme", &"dark".to_string()).unwrap();

        {
            let pending = store.pending.lock();
            assert_eq!(pending.len(), 3);
        }
        {
            let read_txn = store.db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE_DATA).unwrap();
            assert!(table.get("net.host").unwrap().is_none());
            assert!(table.get("ui.theme").unwrap().is_none());
        }

        store.flush_prefix("net").unwrap();

        {
            let read_txn = store.db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE_DATA).unwrap();
            assert_eq!(
                store
                    .decode::<String>(table.get("net.host").unwrap().unwrap().value())
                    .unwrap(),
                "127.0.0.1"
            );
            assert_eq!(
                store
                    .decode::<u16>(table.get("net.port").unwrap().unwrap().value())
                    .unwrap(),
                8080
            );
            assert!(
                table.get("ui.theme").unwrap().is_none(),
                "UI should remain in the RAM buffer"
            );
        }

        {
            let pending = store.pending.lock();
            assert_eq!(
                pending.len(),
                1,
                "Only ui.theme should remain in the buffer"
            );
            assert!(pending.contains_key("ui.theme"));
            assert!(!pending.contains_key("net.host"));
            assert!(!pending.contains_key("net.port"));
        }

        store.flush_prefix("").unwrap();
        {
            let pending = store.pending.lock();
            assert!(
                pending.is_empty(),
                "Pending buffer should be completely empty"
            );
        }
        {
            let read_txn = store.db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE_DATA).unwrap();
            assert!(
                table.get("ui.theme").unwrap().is_some(),
                "UI should now be persisted on disk"
            );
        }
    }

    #[test]
    fn test_is_initialized_false_on_fresh_store() {
        let path = unique_path("init_fresh");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();
        assert!(!store.is_initialized("settings").unwrap());
    }

    #[test]
    fn test_mark_and_is_initialized() {
        let path = unique_path("init_mark");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();
        assert!(!store.is_initialized("settings").unwrap());
        store.mark_initialized("settings").unwrap();
        assert!(store.is_initialized("settings").unwrap());
    }

    #[test]
    fn test_initialized_namespaces_are_independent() {
        let path = unique_path("init_namespaces");
        let (store, _) = RedbStore::open(StoreConfig::new(path), MigrationSet::default()).unwrap();
        store.mark_initialized("settings").unwrap();
        assert!(store.is_initialized("settings").unwrap());
        assert!(!store.is_initialized("other").unwrap());
    }

    #[traced_test]
    #[test]
    fn test_drift_automatic_warning_log() {
        let path = unique_path("tracing_drift");
        let prefix = "app_settings";

        {
            let fields_v1: &'static [FieldDescriptor] = &[
                FieldDescriptor {
                    name: "port",
                    type_hash: 10,
                    type_name: "u16",
                },
                FieldDescriptor {
                    name: "host",
                    type_hash: 20,
                    type_name: "String",
                },
            ];
            let hash_v1 = 111;

            let mset = MigrationSet::default().add(
                prefix,
                MigrationPlan::new().step(1, "v1", |_| Ok(())),
                hash_v1,
                fields_v1,
                &[],
            );

            let (store, _) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
            store.save_now().unwrap();
        }

        let fields_v2: &'static [FieldDescriptor] = &[
            FieldDescriptor {
                name: "port",
                type_hash: 30,
                type_name: "u32",
            },
            FieldDescriptor {
                name: "timeout",
                type_hash: 40,
                type_name: "Duration",
            },
        ];
        let hash_v2 = 222;

        let (_store, report) = StoreBuilder::new(&path)
            .migrations(|m| {
                m.for_prefix(prefix)
                    .step(1, "v1", |_| Ok(()))
                    .depends_on_raw("none");

                let plan = m.prefix_plan(prefix);
                plan.schema_hash = hash_v2;
                plan.fields = fields_v2;
            })
            .build()
            .unwrap();

        assert!(report.has_drift(), "Report should detect drift");

        assert!(logs_contain(&format!(
            "Schema drift detected in prefix '{}'",
            prefix
        )));
        assert!(logs_contain("+ field 'timeout': Duration"));
        assert!(logs_contain("- field 'host'"));
        assert!(logs_contain("~ field 'port': u16 -> u32"));
        assert!(logs_contain("Suggestion: increment version"));
    }

    #[test]
    fn test_component_atomic_rollback() {
        let path = unique_path("rollback");
        let mut cfg = StoreConfig::new(&path);
        cfg.save_debounce = Duration::from_millis(50);
        {
            let (store, _) = RedbStore::open(cfg, MigrationSet::default()).unwrap();
            store.set("net.ip", &"1.1.1.1".to_string()).unwrap();
            store.save_now().unwrap();
        }

        let mset = MigrationSet::default()
            .add(
                "net",
                MigrationPlan::new().step(1, "ok", |ctx| ctx.set("ip", &"8.8.8.8".to_string())),
                0,
                EMPTY_FIELDS,
                &[],
            )
            .add(
                "ui",
                MigrationPlan::new().step(1, "fail", |_| {
                    Err(MigrationError::Custom("crash".into()).into())
                }),
                0,
                EMPTY_FIELDS,
                &["net"],
            );

        let (store, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
        assert!(report.has_failures());

        let val: String = store.get("net.ip").unwrap().unwrap();
        assert_eq!(val, "1.1.1.1");
    }
    #[test]
    fn test_debouncer_retains_buffer_on_simulated_transaction_failure() {
        let path = unique_path("debouncer_simulated_fail");

        let mut config = StoreConfig::new(&path);
        config.save_debounce = Duration::from_millis(50);

        SIMULATE_WRITE_FAILURE.store(true, Ordering::Relaxed);

        let (store, _) = RedbStore::open(config, MigrationSet::default()).unwrap();

        let test_key = "system.critical_update";
        let test_value = "payload_data".to_string();
        store.set(test_key, &test_value).unwrap();

        {
            let pending = store.pending.lock();
            assert!(pending.contains_key(test_key));
        }

        thread::sleep(Duration::from_millis(150));

        SIMULATE_WRITE_FAILURE.store(false, Ordering::Relaxed);

        {
            let pending = store.pending.lock();
            assert!(
                pending.contains_key(test_key),
                "The pending changes buffer should not be cleared when a transaction fails!"
            );
        }

        let retrieved: Option<String> = store.get(test_key).unwrap();
        assert_eq!(retrieved, Some(test_value));
    }
}
