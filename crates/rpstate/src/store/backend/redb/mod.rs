use crate::store::{
    SchemaAwareStore, Store, StoreCallback, StoreEvent, StoreOp, SubscriptionEntry, SubscriptionId,
    SubscriptionKind,
};
use error::RedbStoreError;
use raw_storage::RedbRawStorage;
use redb::{Database, ReadableDatabase, WriteTransaction};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use tables::{
    TABLE_DATA, TABLE_DIFF_LOG, TABLE_LOG, TABLE_META, TABLE_MIGRATION_LOG, TableReader,
    TableWriter,
};

use crate::store::config::StoreConfig;
use crate::{MigrationContext, MigrationError, MigrationReport, Migrator, Result};

use crate::codec::CodecError;
use crate::migration::fields::FieldDescriptor;
use crate::migration::meta::{PrefixMeta, SchemaSnapshot, StoredFieldEntry};
use crate::migration::set::MigrationSet;
use crate::migration::{
    AppliedStep, ComponentOutcome, ComponentResult, FieldTypeChange, NaggingRecord, SchemaDiff,
};
use crate::store::backend::redb::tables::TABLE_SCHEMA_SNAPSHOT;
use crate::store::util::debouncer::Debouncer;
use bytes::Bytes;
use rmp_serde::Serializer;
use rmp_serde::config::BytesMode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::{thread, time::Duration};
use tracing::{info, warn};

pub mod error;
mod events;
mod raw_storage;
mod tables;

const BUF_SIZE: usize = 64 * 1024;

thread_local! {
    static SERIALIZATION_BUFFER: std::cell::RefCell<Vec<u8>> =
        std::cell::RefCell::new(Vec::with_capacity(BUF_SIZE));
}

pub struct RedbStore {
    db: Arc<Database>,
    pending: Arc<Mutex<HashMap<Arc<str>, Option<Bytes>>>>,
    debouncer: Debouncer,
    subscriptions: Arc<RwLock<Vec<SubscriptionEntry>>>,
    next_sub_id: AtomicU64,

    watcher_tx: std::sync::mpsc::Sender<()>,
    watcher_handle: Option<JoinHandle<()>>,
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
        let debouncer = Debouncer::new(config.save_debounce, move || {
            let changes = {
                let mut lock = pending_save.lock().unwrap();
                if lock.is_empty() {
                    return;
                }
                std::mem::take(&mut *lock)
            };

            if let Ok(txn) = db_save.begin_write() {
                if let Ok(mut table) = txn.open_table(TABLE_DATA) {
                    for (path, opt_bytes) in changes {
                        match opt_bytes {
                            Some(b) => {
                                table.insert(&*path, &b[..]).ok();
                            }
                            None => {
                                table.remove(&*path).ok();
                            }
                        }
                    }
                }
                let _ = txn.commit();
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
            debouncer,
            subscriptions: subscriptions.clone(),
            next_sub_id: AtomicU64::new(1),
            watcher_tx: w_tx,
            watcher_handle: Some(watcher_handle),
        };

        let report = store.run_migrations(migration_set)?;

        Ok((store, report))
    }

    pub fn close(&mut self) -> Result<()> {
        info!("Closing RedbStore explicitly...");

        let _ = self.watcher_tx.send(());
        if let Some(handle) = self.watcher_handle.take() {
            let _ = handle.join();
        }

        self.save_now()?;

        Ok(())
    }

    pub fn save_now(&self) -> Result<()> {
        let changes = {
            let mut lock = self.pending.lock().map_err(|_| RedbStoreError::Poisoned)?;
            if lock.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *lock)
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

    fn run_migrations_impl(&self, mset: MigrationSet) -> Result<MigrationReport> {
        let mut report = MigrationReport::default();
        let components = mset.find_components();

        for component_prefixes in components {
            let sorted_prefixes = mset.topo_sort_component(&component_prefixes)?;

            if !self.component_needs_work(&sorted_prefixes, &mset)? {
                report.components.push(ComponentResult {
                    prefixes: component_prefixes,
                    outcome: ComponentOutcome::Skipped,
                    nagging: Vec::new(),
                });
                continue;
            }

            match self.execute_component_migration(&sorted_prefixes, &mset) {
                Ok((steps, nagging)) => {
                    report.components.push(ComponentResult {
                        prefixes: component_prefixes,
                        outcome: ComponentOutcome::Committed { steps },
                        nagging,
                    });
                }
                Err(e) => {
                    report.components.push(ComponentResult {
                        prefixes: component_prefixes,
                        outcome: ComponentOutcome::Failed { error: e },
                        nagging: Vec::new(),
                    });
                }
            }
        }
        Ok(report)
    }

    fn component_needs_work(&self, prefixes: &[String], mset: &MigrationSet) -> Result<bool> {
        let read_txn = self.db.begin_read().map_err(RedbStoreError::from)?;

        for prefix in prefixes {
            let meta: Option<PrefixMeta> = read_txn.load_typed(TABLE_META, prefix)?;

            let current_v = meta.as_ref().map(|m| m.version).unwrap_or(0);
            let current_h = meta.as_ref().map(|m| m.hash).unwrap_or(0);
            let (target_v, target_h, _) = mset.get_target(prefix);

            if target_v != current_v || target_h != 0 && target_h != current_h {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn execute_component_migration(
        &self,
        prefixes: &[String],
        mset: &MigrationSet,
    ) -> Result<(Vec<AppliedStep>, Vec<NaggingRecord>)> {
        let write_txn = self.db.begin_write().map_err(RedbStoreError::from)?;
        let mut all_steps = Vec::new();
        let mut all_nagging = Vec::new();

        {
            let mut storage = RedbRawStorage::new(&write_txn);
            for prefix in prefixes {
                let (steps, nagging) =
                    self.migrate_prefix(prefix, &write_txn, mset, &mut storage)?;
                all_steps.extend(steps);
                all_nagging.extend(nagging);
            }
        }

        write_txn.commit().map_err(RedbStoreError::from)?;
        Ok((all_steps, all_nagging))
    }

    fn calculate_drift(
        &self,
        prefix: &str,
        txn: &redb::WriteTransaction,
        current_fields: &[FieldDescriptor],
    ) -> Result<Option<SchemaDiff>> {
        let snapshot: Option<SchemaSnapshot> = txn.load_typed(TABLE_SCHEMA_SNAPSHOT, prefix)?;
        let Some(old) = snapshot else {
            return Ok(None);
        };

        let mut diff = SchemaDiff {
            added: vec![],
            removed: vec![],
            type_changed: vec![],
        };
        let mut old_fields: HashMap<String, StoredFieldEntry> = old
            .fields
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        for f in current_fields {
            if let Some(old_f) = old_fields.remove(f.name) {
                if old_f.type_hash != f.type_hash {
                    diff.type_changed.push(FieldTypeChange {
                        name: f.name.to_string(),
                        old_type: old_f.type_name,
                        new_type: f.type_name.to_string(),
                    });
                }
            } else {
                diff.added.push(StoredFieldEntry {
                    name: f.name.to_string(),
                    type_name: f.type_name.to_string(),
                    type_hash: f.type_hash,
                });
            }
        }

        diff.removed = old_fields.into_values().collect();

        if diff.added.is_empty() && diff.removed.is_empty() && diff.type_changed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(diff))
        }
    }

    fn migrate_prefix(
        &self,
        prefix: &str,
        txn: &WriteTransaction,
        mset: &MigrationSet,
        storage: &mut RedbRawStorage,
    ) -> Result<(Vec<AppliedStep>, Vec<NaggingRecord>)> {
        let (target_v, target_hash, target_fields) = mset.get_target(prefix);

        let meta_opt: Option<PrefixMeta> = txn.load_typed(TABLE_META, prefix)?;

        let mut meta = match meta_opt {
            Some(m) => m,
            None => {
                let start_v = mset
                    .get_migrator(prefix)
                    .and_then(|m| m.steps.iter().map(|s| s.target_version()).min())
                    .map(|v| v.saturating_sub(1))
                    .unwrap_or(target_v);

                if start_v == target_v {
                    txn.save_typed(
                        TABLE_META,
                        prefix,
                        &PrefixMeta {
                            version: target_v,
                            hash: target_hash,
                        },
                    )?;
                    return Ok((vec![], vec![]));
                }

                PrefixMeta {
                    version: start_v,
                    hash: 0,
                }
            }
        };

        let mut nagging = Vec::new();

        if target_v < meta.version {
            return Err(MigrationError::Downgrade {
                prefix: prefix.to_string(),
                db_version: meta.version,
                code_version: target_v,
            }
            .into());
        }

        if target_hash != 0 && target_v == meta.version && target_hash != meta.hash {
            let diff = self.calculate_drift(prefix, txn, target_fields)?;

            nagging.push(NaggingRecord {
                prefix: prefix.to_string(),
                old_hash: meta.hash,
                new_hash: target_hash,
                diff,
            });

            meta.hash = target_hash;
            txn.save_typed(TABLE_META, prefix, &meta)?;
        }

        let mut applied_steps = Vec::new();
        if let Some(migrator) = mset.get_migrator(prefix) {
            let mut history = txn
                .load_typed(TABLE_MIGRATION_LOG, prefix)?
                .unwrap_or_default();

            applied_steps = self.run_migrator_steps(
                prefix,
                migrator,
                &mut meta,
                target_v,
                storage,
                &mut history,
            )?;

            if !applied_steps.is_empty() {
                meta.hash = target_hash;
                txn.save_typed(TABLE_META, prefix, &meta)?;
                txn.save_typed(TABLE_MIGRATION_LOG, prefix, &history)?;
            }
        }

        if meta.version < target_v {
            return Err(MigrationError::Gap {
                prefix: prefix.to_string(),
                reached_version: meta.version,
                expected_version: target_v,
            }
            .into());
        }

        if meta.version == target_v && !target_fields.is_empty() {
            let new_snapshot = SchemaSnapshot {
                version: target_v,
                fields: target_fields
                    .iter()
                    .map(|f| StoredFieldEntry {
                        name: f.name.to_string(),
                        type_name: f.type_name.to_string(),
                        type_hash: f.type_hash,
                    })
                    .collect(),
            };

            txn.save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, &new_snapshot)?;
        }

        Ok((applied_steps, nagging))
    }

    fn run_migrator_steps(
        &self,
        prefix: &str,
        migrator: &Migrator,
        meta: &mut PrefixMeta,
        target_v: u32,
        storage: &mut RedbRawStorage,
        history: &mut Vec<AppliedStep>,
    ) -> Result<Vec<AppliedStep>> {
        let mut new_steps = Vec::new();
        let mut ctx = MigrationContext::new(prefix.to_string(), storage);

        for step in &migrator.steps {
            let sv = step.target_version();
            if sv <= meta.version {
                continue;
            }
            if sv > target_v {
                break;
            }

            if sv != meta.version + 1 {
                return Err(MigrationError::Gap {
                    prefix: prefix.to_string(),
                    reached_version: meta.version,
                    expected_version: meta.version + 1,
                }
                .into());
            }

            step.run(&mut ctx)?;

            let applied = AppliedStep {
                prefix: prefix.to_string(),
                target_version: sv,
                description: step.description().map(|s| s.to_string()),
                applied_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            meta.version = sv;
            history.push(applied.clone());
            new_steps.push(applied);
        }
        Ok(new_steps)
    }
}

impl Drop for RedbStore {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl SchemaAwareStore for RedbStore {
    fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport> {
        self.run_migrations_impl(mset)
    }
}

impl Store for RedbStore {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        {
            let lock = self.pending.lock().map_err(|_| RedbStoreError::Poisoned)?;
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
        let bytes = SERIALIZATION_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            b.clear();
            let mut ser = Serializer::new(&mut *b).with_bytes(BytesMode::ForceAll);
            value.serialize(&mut ser).map_err(CodecError::from)?;

            Ok::<Bytes, RedbStoreError>(Bytes::copy_from_slice(&b))
        })?;

        {
            let mut lock = self.pending.lock().map_err(|_| RedbStoreError::Poisoned)?;
            lock.insert(path.clone(), Some(bytes.clone()));
        }

        events::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: path.clone(),
                op: StoreOp::Set,
                old: None,
                new: Some(bytes),
            },
        );

        self.debouncer.schedule();
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<()> {
        {
            let mut lock = self.pending.lock().map_err(|_| RedbStoreError::Poisoned)?;
            lock.insert(Arc::from(path), None);
        }

        events::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: Arc::from(path),
                op: StoreOp::Delete,
                old: None,
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
            .unwrap()
            .push(SubscriptionEntry { id, kind, callback });
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.subscriptions.write().unwrap().retain(|s| s.id != id);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StoreBuilder;
    use crate::error::Error;
    use crate::migration::ComponentOutcome;
    use redb::ReadableTableMetadata;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_test::traced_test;

    const EMPTY_FIELDS: &[crate::migration::fields::FieldDescriptor] = &[];

    fn unique_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rpstate-redb-{suffix}-{nanos}.redb"))
    }

    fn write_prefix_meta(store: &RedbStore, prefix: &str, version: u32, hash: u64) {
        let write_txn = store.db.begin_write().unwrap();
        {
            let mut meta_table = write_txn.open_table(TABLE_META).unwrap();
            let meta = PrefixMeta { version, hash };
            meta_table
                .insert(prefix, rmp_serde::to_vec(&meta).unwrap().as_slice())
                .unwrap();
        }
        write_txn.commit().unwrap();
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
                let mut guard = hit_inner.lock().unwrap();
                *guard = true;
            }),
        );

        store.set("ui.theme", &"dark".to_string()).unwrap();

        assert!(*hit.lock().unwrap());
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
    fn test_first_initialization() {
        let path = unique_path("init");
        let mset = MigrationSet::default().add(
            "ui",
            Migrator::new().step(1, "init", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );
        let (store, report) = RedbStore::open(StoreConfig::new(path), mset).unwrap();

        assert!(!report.has_failures());

        let read_txn = store.db.begin_read().unwrap();
        let meta_table = read_txn.open_table(TABLE_META).unwrap();
        let meta: PrefixMeta =
            rmp_serde::from_slice(meta_table.get("ui").unwrap().unwrap().value()).unwrap();

        assert_eq!(meta.version, 1);
        assert_eq!(meta.hash, 0);
    }

    #[test]
    fn test_missing_migration_step_does_not_advance_meta() {
        let path = unique_path("mig_req");
        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, "app", 1, 100);
        }

        let mset = MigrationSet::default().add(
            "app",
            Migrator::new().step(3, "v3", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (store, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
        let ComponentOutcome::Failed { error } = &report.components[0].outcome else {
            panic!("Expected failed migration component");
        };

        let Error::Migration(MigrationError::Gap {
            prefix,
            reached_version,
            expected_version,
        }) = error
        else {
            panic!("Expected migration gap, got {error:?}");
        };
        assert_eq!(prefix, "app");
        assert_eq!(*reached_version, 1);
        assert_eq!(*expected_version, 2);

        let read_txn = store.db.begin_read().unwrap();
        let meta_table = read_txn.open_table(TABLE_META).unwrap();
        let meta: PrefixMeta =
            rmp_serde::from_slice(meta_table.get("app").unwrap().unwrap().value()).unwrap();
        assert_eq!(meta.version, 1);
        assert_eq!(meta.hash, 100);
    }

    #[test]
    fn test_hashless_target_ignores_saved_hash() {
        let path = unique_path("hashless");
        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, "net", 1, 100);
        }

        let mset = MigrationSet::default().add(
            "net",
            Migrator::new().step(1, "init", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );
        let (store, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        assert!(matches!(
            report.components[0].outcome,
            ComponentOutcome::Skipped
        ));

        let read_txn = store.db.begin_read().unwrap();
        let meta_table = read_txn.open_table(TABLE_META).unwrap();
        let meta: PrefixMeta =
            rmp_serde::from_slice(meta_table.get("net").unwrap().unwrap().value()).unwrap();
        assert_eq!(meta.version, 1);
        assert_eq!(meta.hash, 100);

        let diff_table = read_txn.open_table(TABLE_DIFF_LOG).unwrap();
        assert!(diff_table.get("net").unwrap().is_none());
    }

    #[test]
    fn test_downgrade_error() {
        let path = unique_path("downgrade");
        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, "app", 5, 500);
        }

        let mset = MigrationSet::default().add(
            "app",
            Migrator::new().step(4, "v4", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );
        let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        let ComponentOutcome::Failed { error } = &report.components[0].outcome else {
            panic!("Expected failed migration component");
        };

        match error {
            Error::Migration(MigrationError::Downgrade {
                prefix,
                db_version,
                code_version,
            }) => {
                assert_eq!(prefix, "app");
                assert_eq!(*db_version, 5);
                assert_eq!(*code_version, 4);
            }
            _ => panic!("Expected Downgrade error, got {:?}", error),
        }
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
                Migrator::new().step(1, "ok", |ctx| ctx.set("ip", &"8.8.8.8".to_string())),
                0,
                EMPTY_FIELDS,
                &[],
            )
            .add(
                "ui",
                Migrator::new().step(1, "fail", |_| {
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
    fn test_independent_components_success() {
        let path = unique_path("independent");
        let mset = MigrationSet::default()
            .add(
                "a",
                Migrator::new().step(1, "ok", |ctx| ctx.set("v", &1)),
                0,
                EMPTY_FIELDS,
                &[],
            )
            .add(
                "b",
                Migrator::new().step(1, "fail", |_| {
                    Err(MigrationError::Custom("err".into()).into())
                }),
                0,
                EMPTY_FIELDS,
                &[],
            );

        let (store, report) = RedbStore::open(StoreConfig::new(path), mset).unwrap();

        assert_eq!(store.get::<i32>("a.v").unwrap(), Some(1));
        assert!(report.has_failures());
    }

    #[test]
    fn test_idle_migration_skipped() {
        let path = unique_path("idle");
        let mset1 = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", |ctx| ctx.set("v", &1)),
            0,
            EMPTY_FIELDS,
            &[],
        );

        {
            let (store, _) = RedbStore::open(StoreConfig::new(&path), mset1).unwrap();
            assert_eq!(store.get::<i32>("app.v").unwrap(), Some(1));
        }

        let mset2 = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (_, report2) = RedbStore::open(StoreConfig::new(&path), mset2).unwrap();
        assert!(matches!(
            report2.components[0].outcome,
            ComponentOutcome::Skipped
        ));
    }

    #[test]
    fn test_partial_migration_within_component() {
        let path = unique_path("partial");
        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, "a", 1, 0);
        }

        let a_calls = Arc::new(AtomicUsize::new(0));
        let b_calls = Arc::new(AtomicUsize::new(0));

        let a_cap = a_calls.clone();
        let b_cap = b_calls.clone();

        let mset = MigrationSet::default()
            .add(
                "a",
                Migrator::new().step(1, "v1", move |_| {
                    a_cap.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
                0,
                EMPTY_FIELDS,
                &[],
            )
            .add(
                "b",
                Migrator::new().step(1, "v1", move |_| {
                    b_cap.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
                0,
                EMPTY_FIELDS,
                &["a"],
            );

        let _ = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        assert_eq!(a_calls.load(Ordering::SeqCst), 0);
        assert_eq!(b_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_steps_migration_order() {
        let path = unique_path("multi_steps_order");
        let mset = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "one", |ctx| ctx.set("log", &"1".to_string()))
                .step(2, "two", |ctx| {
                    let mut s: String = ctx.get("log")?.unwrap();
                    s.push('2');
                    ctx.set("log", &s)
                })
                .step(3, "three", |ctx| {
                    let mut s: String = ctx.get("log")?.unwrap();
                    s.push('3');
                    ctx.set("log", &s)
                }),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (store, _) = RedbStore::open(StoreConfig::new(path), mset).unwrap();
        let final_log: String = store.get("app.log").unwrap().unwrap();
        assert_eq!(final_log, "123");
    }

    #[test]
    fn test_migration_resume_from_version() {
        let path = unique_path("resume_v");
        let mset1 = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", |ctx| ctx.set("log", &"1".to_string())),
            0,
            EMPTY_FIELDS,
            &[],
        );
        {
            let _ = RedbStore::open(StoreConfig::new(&path), mset1).unwrap();
        }

        let mset2 = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "init", |_| panic!("Step 1 should be skipped"))
                .step(2, "next", |ctx| {
                    let mut s: String = ctx.get("log")?.unwrap();
                    s.push('2');
                    ctx.set("log", &s)
                }),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (store, _) = RedbStore::open(StoreConfig::new(&path), mset2).unwrap();
        let final_log: String = store.get("app.log").unwrap().unwrap();
        assert_eq!(final_log, "12");
    }

    #[test]
    fn test_migration_gap_detection() {
        let path = unique_path("gap_failure");
        let mset = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "v1", |_| Ok(()))
                .step(3, "v3", |_| Ok(())),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (_, report) = RedbStore::open(StoreConfig::new(path), mset).unwrap();
        let ComponentOutcome::Failed { error } = &report.components[0].outcome else {
            panic!("Expected failure")
        };

        let Error::Migration(MigrationError::Gap {
            prefix,
            reached_version,
            expected_version,
        }) = error
        else {
            panic!("Expected Gap error");
        };

        assert_eq!(prefix, "app");
        assert_eq!(*reached_version, 1);
        assert_eq!(*expected_version, 2);
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
    fn test_fresh_db_runs_migration_steps() {
        let path = unique_path("fresh_runs_steps");
        let mset = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "init", |ctx| ctx.set("v", &1u32))
                .step(2, "next", |ctx| {
                    let v: u32 = ctx.get("v")?.unwrap();
                    ctx.set("v", &(v + 1))
                }),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let (store, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
        assert!(!report.has_failures());
        assert_eq!(store.get::<u32>("app.v").unwrap(), Some(2));
    }

    #[test]
    fn test_fresh_db_no_migrator_is_skipped() {
        let path = unique_path("fresh_no_migrator");
        let mset = MigrationSet::default().add("app", Migrator::new(), 0, EMPTY_FIELDS, &[]);
        let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        assert!(!report.has_failures());
        assert!(matches!(
            report.components[0].outcome,
            ComponentOutcome::Skipped
        ));
    }

    #[test]
    fn test_fresh_db_step_is_not_skipped() {
        let path = unique_path("fresh_not_skipped");
        let ran = Arc::new(AtomicUsize::new(0));
        let ran_cap = ran.clone();

        let mset = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", move |_| {
                ran_cap.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
            0,
            EMPTY_FIELDS,
            &[],
        );

        let _ = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_drift_detection_field_added() {
        let path = unique_path("drift_add");
        let prefix = "profile";

        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();

            write_prefix_meta(&store, prefix, 1, 111);

            let write_txn = store.db.begin_write().unwrap();

            let snapshot = SchemaSnapshot {
                version: 1,
                fields: vec![StoredFieldEntry {
                    name: "name".to_string(),
                    type_name: "String".to_string(),
                    type_hash: 1,
                }],
            };
            write_txn
                .save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, &snapshot)
                .unwrap();
            write_txn.commit().unwrap();
        }

        let current_fields: &'static [FieldDescriptor] = &[
            FieldDescriptor {
                name: "name",
                type_hash: 1,
                type_name: "String",
            },
            FieldDescriptor {
                name: "age",
                type_hash: 2,
                type_name: "u32",
            },
        ];
        let new_hash = 222;

        let mset = MigrationSet::default().add(
            prefix,
            Migrator::new().step(1, "v1", |_| Ok(())),
            new_hash,
            current_fields,
            &[],
        );

        let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        assert!(report.has_drift(), "Report should contain drift nagging");
        let nag = &report.components[0].nagging[0];
        assert_eq!(nag.diff.as_ref().unwrap().added.len(), 1);
        assert_eq!(nag.diff.as_ref().unwrap().added[0].name, "age");
    }

    #[test]
    fn test_drift_detection_type_changed() {
        let path = unique_path("drift_type");
        let prefix = "settings";

        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, prefix, 1, 10);
            let write_txn = store.db.begin_write().unwrap();

            let snapshot = SchemaSnapshot {
                version: 1,
                fields: vec![StoredFieldEntry {
                    name: "port".to_string(),
                    type_name: "u16".to_string(),
                    type_hash: 100,
                }],
            };
            write_txn
                .save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, &snapshot)
                .unwrap();
            write_txn.commit().unwrap();
        }

        let current_fields: &'static [FieldDescriptor] = &[FieldDescriptor {
            name: "port",
            type_hash: 200,
            type_name: "u32",
        }];

        let mset = MigrationSet::default().add(
            prefix,
            Migrator::new().step(1, "v1", |_| Ok(())),
            20,
            current_fields,
            &[],
        );

        let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        let diff = report.components[0].nagging[0].diff.as_ref().unwrap();
        assert_eq!(diff.type_changed.len(), 1);
        assert_eq!(diff.type_changed[0].old_type, "u16");
        assert_eq!(diff.type_changed[0].new_type, "u32");
    }

    #[test]
    fn test_drift_acknowledgment_silence() {
        let path = unique_path("drift_silence");
        let prefix = "app";

        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();
            write_prefix_meta(&store, prefix, 1, 1);
            let write_txn = store.db.begin_write().unwrap();
            let snap = SchemaSnapshot {
                version: 1,
                fields: vec![],
            };
            write_txn
                .save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, &snap)
                .unwrap();
            write_txn.commit().unwrap();
        }

        let fields: &'static [FieldDescriptor] = &[FieldDescriptor {
            name: "new",
            type_hash: 9,
            type_name: "i32",
        }];

        {
            let mset = MigrationSet::default().add(
                prefix,
                Migrator::new().step(1, "v1", |_| Ok(())),
                99,
                fields,
                &[],
            );
            let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
            assert!(report.has_drift());
        }

        {
            let mset = MigrationSet::default().add(
                prefix,
                Migrator::new().step(1, "v1", |_| Ok(())),
                99,
                fields,
                &[],
            );
            let (_, report) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();
            assert!(!report.has_drift(), "Should not nag again");
        }
    }

    #[test]
    fn test_migration_updates_snapshot() {
        let path = unique_path("mig_snapshot");
        let prefix = "data";

        {
            let (store, _) =
                RedbStore::open(StoreConfig::new(&path), MigrationSet::default()).unwrap();

            write_prefix_meta(&store, prefix, 1, 111);

            let snap = SchemaSnapshot {
                version: 1,
                fields: vec![StoredFieldEntry {
                    name: "old_f".into(),
                    type_name: "u8".into(),
                    type_hash: 1,
                }],
            };

            let write_txn = store.db.begin_write().unwrap();

            write_txn
                .save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, &snap)
                .unwrap();
            write_txn.commit().unwrap();
        }

        let v2_fields: &'static [FieldDescriptor] = &[FieldDescriptor {
            name: "new_f",
            type_hash: 2,
            type_name: "u16",
        }];

        let v2_hash = 222;

        let mset = MigrationSet::default().add(
            prefix,
            Migrator::new().step(2, "v2", |ctx| ctx.set("new_f", &10u16)),
            v2_hash,
            v2_fields,
            &[],
        );

        let (store, _) = RedbStore::open(StoreConfig::new(&path), mset).unwrap();

        let read_txn = store.db.begin_read().unwrap();
        let snap: SchemaSnapshot = read_txn
            .load_typed(TABLE_SCHEMA_SNAPSHOT, prefix)
            .unwrap()
            .unwrap();

        assert_eq!(snap.version, 2);
        assert_eq!(snap.fields.len(), 1);
        assert_eq!(snap.fields[0].name, "new_f");
        assert_eq!(snap.fields[0].type_name, "u16");
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
                Migrator::new().step(1, "v1", |_| Ok(())),
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
}
