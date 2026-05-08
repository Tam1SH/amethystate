use crate::store::{
    debouncer::Debouncer, Store, StoreCallback, StoreEvent, StoreOp, SubscriptionId,
    SubscriptionKind,
};
use anyhow::{anyhow, Context};
use redb::{
    Database, ReadTransaction, ReadableDatabase, ReadableTable, TableDefinition, WriteTransaction,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{error::Error, Result};
use crate::store::config::StoreConfig;
use crate::store::migration::set::MigrationSet;
use crate::store::migration::{
    AppliedStep, ComponentOutcome, ComponentResult, MigrationContext, MigrationReport, Migrator,
    NaggingRecord, RawStorage,
};
use crate::store::shared::{matches_kind, DiffEntry, PrefixMeta, SubscriptionEntry};
use rmp_serde::config::BytesMode;
use rmp_serde::Serializer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time::Duration};
use tracing::{error, info, warn};

const TABLE_DATA: TableDefinition<&str, &[u8]> = TableDefinition::new("data");
const TABLE_LOG: TableDefinition<u64, &str> = TableDefinition::new("inbox_log");
const TABLE_META: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
const TABLE_DIFF_LOG: TableDefinition<&str, &[u8]> = TableDefinition::new("diff_log");
const TABLE_MIGRATION_LOG: TableDefinition<&str, &[u8]> = TableDefinition::new("migration_log");

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
            let _ = write_txn.open_table(TABLE_DIFF_LOG).map_err(map_db_err)?;
            let _ = write_txn
                .open_table(TABLE_MIGRATION_LOG)
                .map_err(map_db_err)?;
        }
        write_txn.commit().map_err(map_db_err)?;

        let pending = Arc::new(Mutex::new(HashMap::<String, Option<Vec<u8>>>::new()));
        let subscriptions = Arc::new(RwLock::new(Vec::new()));

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
                                table.insert(path.as_str(), b.as_slice()).ok();
                            }
                            None => {
                                table.remove(path.as_str()).ok();
                            }
                        }
                    }
                }
                let _ = txn.commit();
            }
        });

        let store = Self {
            db: db.clone(),
            pending,
            debouncer,
            subscriptions: subscriptions.clone(),
            next_sub_id: AtomicU64::new(1),
        };

        store.spawn_watcher();

        Ok(store)
    }

    fn spawn_watcher(&self) {
        let db = self.db.clone();
        let subs = self.subscriptions.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(300));
                let _ = Self::process_inbox(&db, &subs);
            }
        });
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

    pub fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport> {
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
        let read_txn = self.db.begin_read().map_err(map_db_err)?;

        for prefix in prefixes {
            let meta: Option<PrefixMeta> = read_txn.load_typed(TABLE_META, prefix)?;

            let current_v = meta.as_ref().map(|m| m.version).unwrap_or(0);
            let current_h = meta.as_ref().map(|m| m.hash).unwrap_or(0);
            let (target_v, target_h) = mset.get_target(prefix);

            if target_v > current_v || (target_v == current_v && target_h != current_h) {
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
        let write_txn = self.db.begin_write().map_err(map_db_err)?;
        let mut all_steps = Vec::new();
        let mut all_nagging = Vec::new();

        {
            let mut storage = RedbRawStorage { txn: &write_txn };
            for prefix in prefixes {
                let (steps, nagging) =
                    self.migrate_prefix(prefix, &write_txn, mset, &mut storage)?;
                all_steps.extend(steps);
                all_nagging.extend(nagging);
            }
        }

        write_txn.commit().map_err(map_db_err)?;
        Ok((all_steps, all_nagging))
    }
    fn migrate_prefix(
        &self,
        prefix: &str,
        txn: &WriteTransaction,
        mset: &MigrationSet,
        storage: &mut RedbRawStorage,
    ) -> Result<(Vec<AppliedStep>, Vec<NaggingRecord>)> {
        let (target_v, target_hash) = mset.get_target(prefix);

        let mut meta: PrefixMeta = txn.load_typed(TABLE_META, prefix)?.unwrap_or_default();
        let mut nagging = Vec::new();

        if target_v < meta.version {
            return Err(Error::Downgrade {
                prefix: prefix.to_string(),
                db_version: meta.version,
                code_version: target_v,
            });
        }

        if target_hash != 0 && target_v == meta.version && target_hash != meta.hash {
            nagging.push(NaggingRecord {
                prefix: prefix.to_string(),
                old_hash: meta.hash,
                new_hash: target_hash,
            });
            self.log_hash_diff(prefix, txn, meta.hash, target_hash)?;
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
            return Err(Error::MigrationGap {
                prefix: prefix.to_string(),
                reached_version: meta.version,
                expected_version: target_v,
            });
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
                return Err(Error::MigrationGap {
                    prefix: prefix.to_string(),
                    reached_version: meta.version,
                    expected_version: meta.version + 1,
                });
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

    fn log_hash_diff(
        &self,
        prefix: &str,
        txn: &WriteTransaction,
        old_h: u64,
        new_h: u64,
    ) -> Result<()> {
        let mut table = txn.open_table(TABLE_DIFF_LOG).map_err(map_db_err)?;
        let mut history: Vec<DiffEntry> =
            txn.load_typed(TABLE_DIFF_LOG, prefix)?.unwrap_or_default();

        if history.last().map_or(true, |l| l.new_hash != new_h) {
            history.push(DiffEntry {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                old_hash: old_h,
                new_hash: new_h,
            });
            let bytes =
                rmp_serde::to_vec(&history).map_err(|e| Error::Serialization(e.to_string()))?;
            table.insert(prefix, bytes.as_slice()).map_err(map_db_err)?;
        }
        Ok(())
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

pub trait TableReader {
    fn load_typed<T: DeserializeOwned>(
        &self,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<Option<T>>;
}

pub trait TableWriter {
    fn save_typed<T: Serialize>(
        &self,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
        val: &T,
    ) -> Result<()>;
}

fn deserialize_from_table<T: DeserializeOwned>(
    table: impl redb::ReadableTable<&'static str, &'static [u8]>,
    key: &str,
) -> Result<Option<T>> {
    table
        .get(key)
        .map_err(map_db_err)?
        .map(|v| rmp_serde::from_slice(v.value()).map_err(|e| Error::Serialization(e.to_string())))
        .transpose()
}

impl TableReader for ReadTransaction {
    fn load_typed<T: DeserializeOwned>(
        &self,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<Option<T>> {
        let table = self.open_table(table_def).map_err(map_db_err)?;
        deserialize_from_table(table, key)
    }
}

impl TableReader for WriteTransaction {
    fn load_typed<T: DeserializeOwned>(
        &self,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<Option<T>> {
        let table = self.open_table(table_def).map_err(map_db_err)?;
        deserialize_from_table(table, key)
    }
}

impl TableWriter for WriteTransaction {
    fn save_typed<T: Serialize>(
        &self,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
        val: &T,
    ) -> Result<()> {
        let mut table = self.open_table(table_def).map_err(map_db_err)?;
        let bytes = rmp_serde::to_vec(val).map_err(|e| Error::Serialization(e.to_string()))?;
        table.insert(key, bytes.as_slice()).map_err(map_db_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::migration::Migrator;
    use redb::ReadableTableMetadata;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
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

    #[test]
    fn test_component_atomic_rollback() {
        let path = unique_path("rollback");
        let mut cfg = StoreConfig::new(path);
        cfg.save_debounce = Duration::from_millis(50);
        let store = RedbStore::open(cfg).unwrap();

        store.set("net.ip", &"1.1.1.1".to_string()).unwrap();
        thread::sleep(Duration::from_millis(60));

        let mset = MigrationSet::default()
            .add(
                "net",
                Migrator::new().step(1, "ok", |ctx| ctx.set("ip", &"8.8.8.8".to_string())),
                &[],
            )
            .add(
                "ui",
                Migrator::new().step(1, "fail", |_| Err(Error::Backend("crash".into()))),
                &["net"],
            );

        let report = store.run_migrations(mset).unwrap();
        assert!(report.has_failures());

        let val: String = store.get("net.ip").unwrap().unwrap();
        assert_eq!(val, "1.1.1.1");
    }

    #[test]
    fn test_independent_components_success() {
        let path = unique_path("independent");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        let mset = MigrationSet::default()
            .add(
                "a",
                Migrator::new().step(1, "ok", |ctx| ctx.set("v", &1)),
                &[],
            )
            .add(
                "b",
                Migrator::new().step(1, "fail", |_| Err(Error::Backend("err".into()))),
                &[],
            );

        let report = store.run_migrations(mset).unwrap();

        assert_eq!(store.get::<i32>("a.v").unwrap(), Some(1));
        assert!(report.has_failures());
    }

    #[test]
    fn test_idle_migration_skipped() {
        let path = unique_path("idle");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        let mset1 = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", |ctx| ctx.set("v", &1)),
            &[],
        );

        store.run_migrations(mset1).unwrap();

        let mset2 =
            MigrationSet::default().add("app", Migrator::new().step(1, "init", |_| Ok(())), &[]);

        let report2 = store.run_migrations(mset2).unwrap();
        assert!(matches!(
            report2.components[0].outcome,
            ComponentOutcome::Skipped
        ));
    }

    #[test]
    fn test_partial_migration_within_component() {
        let path = unique_path("partial");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        {
            let write_txn = store.db.begin_write().unwrap();
            let mut meta_table = write_txn.open_table(TABLE_META).unwrap();
            let meta = PrefixMeta {
                version: 1,
                hash: 0,
            };
            meta_table
                .insert("a", rmp_serde::to_vec(&meta).unwrap().as_slice())
                .unwrap();
            drop(meta_table);
            write_txn.commit().unwrap();
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
                &[],
            )
            .add(
                "b",
                Migrator::new().step(1, "v1", move |_| {
                    b_cap.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
                &["a"],
            );

        let _ = store.run_migrations(mset).unwrap();

        assert_eq!(a_calls.load(Ordering::SeqCst), 0);
        assert_eq!(b_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_steps_migration_order() {
        let path = unique_path("multi_steps_order");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

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
            &[],
        );

        let _ = store.run_migrations(mset).unwrap();

        // Toposort guarantees order, but checking anyway just in case the graphs have decided to revolt.
        let final_log: String = store.get("app.log").unwrap().unwrap();
        assert_eq!(final_log, "123");
    }

    #[test]
    fn test_migration_resume_from_version() {
        let path = unique_path("resume_v");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        let mset1 = MigrationSet::default().add(
            "app",
            Migrator::new().step(1, "init", |ctx| ctx.set("log", &"1".to_string())),
            &[],
        );
        store.run_migrations(mset1).unwrap();

        let mset2 = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "init", |_| panic!("Step 1 should be skipped"))
                .step(2, "next", |ctx| {
                    let mut s: String = ctx.get("log")?.unwrap();
                    s.push('2');
                    ctx.set("log", &s)
                }),
            &[],
        );

        let _ = store.run_migrations(mset2).unwrap();

        let final_log: String = store.get("app.log").unwrap().unwrap();
        assert_eq!(final_log, "12");
    }
    #[test]
    fn test_migration_gap_detection() {
        let path = unique_path("gap_failure");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        let mset = MigrationSet::default().add(
            "app",
            Migrator::new()
                .step(1, "v1", |_| Ok(()))
                .step(3, "v3", |_| Ok(())),
            &[],
        );

        let report = store.run_migrations(mset).unwrap();
        let ComponentOutcome::Failed { error } = &report.components[0].outcome else {
            panic!("a")
        };

        let Error::MigrationGap {
            prefix,
            reached_version,
            expected_version,
        } = error
        else {
            panic!("b")
        };
        assert_eq!(prefix, "app");
        assert_eq!(*reached_version, 1);
        assert_eq!(*expected_version, 2);
    }

    #[test]
    fn test_migration_gap_at_start() {
        let path = unique_path("gap_start");
        let store = RedbStore::open(StoreConfig::new(path)).unwrap();

        let mset =
            MigrationSet::default().add("app", Migrator::new().step(2, "v2", |_| Ok(())), &[]);

        let report = store.run_migrations(mset).unwrap();
        let ComponentOutcome::Failed { error } = &report.components[0].outcome else {
            panic!()
        };
        let Error::MigrationGap {
            reached_version,
            expected_version,
            ..
        } = error
        else {
            panic!()
        };

        assert_eq!(*reached_version, 0);
        assert_eq!(*expected_version, 1);
    }
}
