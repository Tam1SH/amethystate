use crate::store::{
    Store, StoreCallback, StoreEvent, StoreOp, SubscriptionId, SubscriptionKind,
    debouncer::Debouncer,
};
use anyhow::{Context, anyhow};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Result, error::Error};
use crate::store::config::StoreConfig;
use crate::store::migration::RawStorage;
use crate::store::shared::{DiffEntry, PrefixMeta, SubscriptionEntry, matches_kind};
use rmp_serde::Serializer;
use rmp_serde::config::BytesMode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time::Duration};
use tracing::{error, info, warn};

const TABLE_DATA: TableDefinition<&str, &[u8]> = TableDefinition::new("data");
const TABLE_LOG: TableDefinition<u64, &str> = TableDefinition::new("inbox_log");
const TABLE_META: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
const TABLE_DIFF_LOG: TableDefinition<&str, &[u8]> = TableDefinition::new("diff_log");

pub struct RedbStore {
    db: Arc<Database>,
    pending: Arc<Mutex<HashMap<String, Option<Vec<u8>>>>>,
    debouncer: Debouncer,
    subscriptions: Arc<RwLock<Vec<SubscriptionEntry>>>,
    next_sub_id: AtomicU64,
}

impl RedbStore {
    pub fn open(config: StoreConfig) -> Result<Self> {
        let db = Arc::new(Database::create(&config.path).map_err(map_db_err)?);

        let write_txn = db.begin_write().map_err(map_db_err)?;
        {
            let _ = write_txn.open_table(TABLE_DATA).map_err(map_db_err)?;
            let _ = write_txn.open_table(TABLE_LOG).map_err(map_db_err)?;
            let _ = write_txn.open_table(TABLE_META).map_err(map_db_err)?;
        }
        write_txn.commit().map_err(map_db_err)?;

        let pending = Arc::new(Mutex::new(HashMap::<String, Option<Vec<u8>>>::new()));
        let subscriptions = Arc::new(RwLock::new(Vec::new()));

        let db_save = db.clone();
        let pending_save = pending.clone();
        let debouncer = Debouncer::new(Duration::from_millis(300), move || {
            let changes = {
                let mut lock = pending_save.lock().unwrap();
                if lock.is_empty() {
                    return;
                }
                std::mem::take(&mut *lock)
            };

            let Ok(write_txn) = db_save.begin_write() else {
                return;
            };
            {
                let mut data_table = write_txn.open_table(TABLE_DATA).unwrap();
                for (path, opt_bytes) in changes {
                    match opt_bytes {
                        Some(b) => {
                            data_table.insert(path.as_str(), b.as_slice()).unwrap();
                        }
                        None => {
                            data_table.remove(path.as_str()).unwrap();
                        }
                    }
                }
            }
            let _ = write_txn.commit();
        });

        let store = Self {
            db: db.clone(),
            pending,
            debouncer,
            subscriptions: subscriptions.clone(),
            next_sub_id: AtomicU64::new(1),
        };

        let watch_db = db.clone();
        let watch_subs = subscriptions.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(300));
                let _ = Self::process_inbox(&watch_db, &watch_subs);
            }
        });

        Ok(store)
    }

    fn process_inbox(db: &Database, subs: &RwLock<Vec<SubscriptionEntry>>) -> Result<()> {
        let write_txn = db.begin_write().map_err(map_db_err)?;
        let mut events = Vec::new();

        {
            let mut log_table = write_txn.open_table(TABLE_LOG).map_err(map_db_err)?;
            let data_table = write_txn.open_table(TABLE_DATA).map_err(map_db_err)?;

            let mut to_delete = Vec::new();
            for result in log_table.iter().map_err(map_db_err)? {
                let (id, path_guard) = result.map_err(map_db_err)?;
                let path = path_guard.value();

                let current_val = data_table.get(path).map_err(map_db_err)?;

                events.push(StoreEvent {
                    path: path.to_string(),
                    op: if current_val.is_some() {
                        StoreOp::Set
                    } else {
                        StoreOp::Delete
                    },
                    old: None,
                    new: current_val.map(|v| v.value().to_vec()),
                });

                to_delete.push(id.value());
            }

            for id in to_delete {
                log_table.remove(id).map_err(map_db_err)?;
            }
        }
        write_txn.commit().map_err(map_db_err)?;

        for event in events {
            Self::emit_local(subs, event);
        }

        Ok(())
    }

    fn emit_local(subs_lock: &RwLock<Vec<SubscriptionEntry>>, event: StoreEvent) {
        let callbacks = {
            let guard = subs_lock.read().unwrap();
            guard
                .iter()
                .filter(|s| matches_kind(&s.kind, &event.path))
                .map(|s| s.callback.clone())
                .collect::<Vec<_>>()
        };
        for cb in callbacks {
            cb(&event);
        }
    }
}

struct RedbRawStorage<'a> {
    txn: &'a redb::WriteTransaction,
}

impl<'a> RawStorage for RedbRawStorage<'a> {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let table = self.txn.open_table(TABLE_DATA).map_err(map_db_err)?;
        Ok(table
            .get(key)
            .map_err(map_db_err)?
            .map(|v| v.value().to_vec()))
    }
    fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let mut table = self.txn.open_table(TABLE_DATA).map_err(map_db_err)?;
        table.insert(key, value).map_err(map_db_err)?;
        Ok(())
    }
    fn delete(&mut self, key: &str) -> Result<()> {
        let mut table = self.txn.open_table(TABLE_DATA).map_err(map_db_err)?;
        table.remove(key).map_err(map_db_err)?;
        Ok(())
    }
}

impl Store for RedbStore {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        {
            let lock = self.pending.lock().map_err(|_| Error::Poisoned)?;
            if let Some(opt_bytes) = lock.get(path) {
                return match opt_bytes {
                    Some(bytes) => Ok(Some(
                        rmp_serde::from_slice(bytes)
                            .map_err(|e| Error::Serialization(e.to_string()))?,
                    )),
                    None => Ok(None),
                };
            }
        }

        let read_txn = self.db.begin_read().map_err(map_db_err)?;
        let table = read_txn.open_table(TABLE_DATA).map_err(map_db_err)?;
        match table.get(path).map_err(map_db_err)? {
            Some(access_guard) => {
                let bytes = access_guard.value();
                Ok(Some(
                    rmp_serde::from_slice(bytes)
                        .map_err(|e| Error::Serialization(e.to_string()))?,
                ))
            }
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()> {
        let mut bytes = Vec::new();
        let mut ser = Serializer::new(&mut bytes).with_bytes(BytesMode::ForceAll);

        value
            .serialize(&mut ser)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        {
            let mut lock = self.pending.lock().map_err(|_| Error::Poisoned)?;
            lock.insert(path.to_string(), Some(bytes.clone()));
        }

        Self::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: path.to_string(),
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
            let mut lock = self.pending.lock().map_err(|_| Error::Poisoned)?;
            lock.insert(path.to_string(), None);
        }

        Self::emit_local(
            &self.subscriptions,
            StoreEvent {
                path: path.to_string(),
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

    fn evolve_prefix(&self, prefix: &str, version: u32, hash: u64) -> Result<()> {
        let write_txn = self.db.begin_write().map_err(map_db_err)?;

        {
            let mut meta_table = write_txn.open_table(TABLE_META).map_err(map_db_err)?;

            let saved_meta: Option<PrefixMeta> = meta_table
                .get(prefix)
                .map_err(map_db_err)?
                .map(|m| rmp_serde::from_slice(m.value()))
                .transpose()
                .map_err(|e| Error::Serialization(e.to_string()))?;

            if let Some(meta) = saved_meta.as_ref() {
                if version < meta.version {
                    return Err(Error::Downgrade {
                        prefix: prefix.to_string(),
                        db_version: meta.version,
                        code_version: version,
                    });
                }

                if version > meta.version {
                    return Err(Error::MigrationRequired {
                        prefix: prefix.to_string(),
                        db_version: meta.version,
                        code_version: version,
                    });
                } else if hash != meta.hash {
                    error!(
                        target: "rpstate",
                        "SCHEMA DIFF TRACE: Section [{}] (hash {} -> {}). Undocumented mutation!",
                        prefix, meta.hash, hash
                    );

                    let mut diff_table =
                        write_txn.open_table(TABLE_DIFF_LOG).map_err(map_db_err)?;

                    let mut history: Vec<DiffEntry> = diff_table
                        .get(prefix)
                        .map_err(map_db_err)?
                        .map(|m| rmp_serde::from_slice(m.value()))
                        .transpose()
                        .map_err(|e| Error::Serialization(e.to_string()))?
                        .unwrap_or_default();

                    if history.last().map_or(true, |l| l.new_hash != hash) {
                        history.push(DiffEntry {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            old_hash: meta.hash,
                            new_hash: hash,
                        });

                        let b = rmp_serde::to_vec(&history)
                            .map_err(|e| Error::Serialization(e.to_string()))?;
                        diff_table
                            .insert(prefix, b.as_slice())
                            .map_err(map_db_err)?;
                    }
                }
            }

            let needs_meta_update = saved_meta.as_ref().map_or(true, |m| {
                m.version != version || (m.version == version && m.hash == hash)
            });

            if needs_meta_update {
                let new_meta = PrefixMeta { version, hash };

                let b = rmp_serde::to_vec(&new_meta)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                meta_table
                    .insert(prefix, b.as_slice())
                    .map_err(map_db_err)?;
            }
        }

        write_txn.commit().map_err(map_db_err)?;
        Ok(())
    }
}

fn map_db_err(e: impl std::fmt::Display) -> Error {
    Error::Backend(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::ReadableTableMetadata;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.set("user.name", &"Alice".to_string()).unwrap();

        let val: Option<String> = store.get("user.name").unwrap();
        assert_eq!(val, Some("Alice".to_string()));
    }

    #[test]
    fn test_debouncer_persistence() {
        let path = unique_path("debounce");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

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
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

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
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

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

        assert_eq!(event.path, "app.version");
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
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.set("temp.key", &1).unwrap();
        thread::sleep(Duration::from_millis(400));

        store.delete("temp.key").unwrap();
        assert_eq!(store.get::<i32>("temp.key").unwrap(), None);

        thread::sleep(Duration::from_millis(400));

        let read_txn = store.db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE_DATA).unwrap();
        assert!(table.get("temp.key").unwrap().is_none());
    }

    #[test]
    fn test_first_initialization() {
        let path = unique_path("init");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.evolve_prefix("ui", 1, 123).unwrap();

        let read_txn = store.db.begin_read().unwrap();
        let meta_table = read_txn.open_table(TABLE_META).unwrap();
        let meta: PrefixMeta =
            rmp_serde::from_slice(meta_table.get("ui").unwrap().unwrap().value()).unwrap();

        assert_eq!(meta.version, 1);
        assert_eq!(meta.hash, 123);
    }

    #[test]
    fn test_migration_required_error() {
        let path = unique_path("mig_req");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.evolve_prefix("app", 1, 100).unwrap();

        let result = store.evolve_prefix("app", 2, 200);

        match result {
            Err(Error::MigrationRequired {
                prefix,
                db_version,
                code_version,
            }) => {
                assert_eq!(prefix, "app");
                assert_eq!(db_version, 1);
                assert_eq!(code_version, 2);
            }
            _ => panic!("Expected MigrationRequired error, got {:?}", result),
        }
    }

    #[test]
    fn test_downgrade_error() {
        let path = unique_path("downgrade");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.evolve_prefix("app", 5, 500).unwrap();

        let result = store.evolve_prefix("app", 4, 400);

        match result {
            Err(Error::Downgrade {
                prefix,
                db_version,
                code_version,
            }) => {
                assert_eq!(prefix, "app");
                assert_eq!(db_version, 5);
                assert_eq!(code_version, 4);
            }
            _ => panic!("Expected Downgrade error, got {:?}", result),
        }
    }

    #[test]
    fn test_nagging_does_not_update_meta() {
        let path = unique_path("nagging");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        store.evolve_prefix("net", 1, 100).unwrap();

        store.evolve_prefix("net", 1, 200).unwrap();

        let read_txn = store.db.begin_read().unwrap();
        let meta_table = read_txn.open_table(TABLE_META).unwrap();
        let meta: PrefixMeta =
            rmp_serde::from_slice(meta_table.get("net").unwrap().unwrap().value()).unwrap();

        assert_eq!(meta.hash, 100);

        let diff_table = read_txn.open_table(TABLE_DIFF_LOG).unwrap();
        let history: Vec<DiffEntry> =
            rmp_serde::from_slice(diff_table.get("net").unwrap().unwrap().value()).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].old_hash, 100);
        assert_eq!(history[0].new_hash, 200);
    }

    #[test]
    fn test_smart_recovery_decode() {
        let path = unique_path("recovery");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();
        let garbage = vec![0x00, 0x01, 0x02];

        let result: String = store.decode(&garbage).unwrap();
        assert_eq!(result, String::default());
    }
}
