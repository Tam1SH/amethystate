use super::document::TextDocument;
use super::error::TextStoreError;
use crate::codec::CodecError;
use crate::migration::engine::{MigrationEngine, StorageProvider};
use crate::migration::set::MigrationSet;
use crate::store::backend::text::events;
use crate::store::backend::text::raw_storage::TextRawStorage;
use crate::store::config::StoreConfig;
use crate::store::util::debouncer::Debouncer;
use crate::store::util::ticker::Ticker;
use crate::store::{
    RawStorage, SchemaAwareStore, Store, StoreCallback, StoreEvent, StoreOp, SubscriptionEntry,
    SubscriptionId, SubscriptionKind, matches_kind,
};
use crate::{MigrationReport, Result};
use bytes::Bytes;
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tempfile::NamedTempFile;
use tracing::{info, warn};

/// Tracks what happened to a key since the last persist.
/// `Some(())` = Set, `None` = Delete.
type DirtyKeys = Arc<Mutex<HashMap<String, Option<()>>>>;

pub struct StoreFile<D> {
    pub path: PathBuf,
    pub backup_path: PathBuf,
    pub doc: Arc<RwLock<D>>,
    pub last_write_mtime: Arc<Mutex<Option<SystemTime>>>,
    pub dirty_keys: DirtyKeys,
}

impl<D> Clone for StoreFile<D> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            backup_path: self.backup_path.clone(),
            doc: self.doc.clone(),
            last_write_mtime: self.last_write_mtime.clone(),
            dirty_keys: self.dirty_keys.clone(),
        }
    }
}

impl<D: TextDocument> StoreFile<D> {
    pub fn new(path: PathBuf, initial_doc: D) -> Self {
        let backup_path = path.with_extension("bak");
        Self {
            path,
            backup_path,
            doc: Arc::new(RwLock::new(initial_doc)),
            last_write_mtime: Arc::new(Mutex::new(None)),
            dirty_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn mark_dirty(&self, path: String, op: Option<()>) {
        self.dirty_keys.lock().insert(path, op);
    }

    pub fn create_backup(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::copy(&self.path, &self.backup_path).map_err(TextStoreError::from)?;
        }
        Ok(())
    }

    pub fn load_or_empty(&self) -> Result<D> {
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path).map_err(TextStoreError::from)?;
            D::parse(&content)
        } else {
            Ok(D::empty())
        }
    }

    /// Merge-persist: reads the current on-disk state, applies only the dirty
    /// keys from the in-memory doc on top, then writes the result atomically.
    /// This ensures concurrent writers (e.g. confy) are not clobbered.
    pub fn persist(&self) -> Result<()> {
        let dirty = {
            let mut guard = self.dirty_keys.lock();
            if guard.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *guard)
        };

        // Read what is currently on disk (another writer may have changed it).
        let mut on_disk = if self.path.exists() {
            let content = std::fs::read_to_string(&self.path).map_err(TextStoreError::from)?;
            D::parse(&content)?
        } else {
            D::empty()
        };

        // Apply only our dirty keys on top of the on-disk document.
        {
            let doc_guard = self.doc.read();
            for (key, op) in &dirty {
                let parts = split_path(key);
                match op {
                    Some(()) => {
                        if let Some(node) = doc_guard.get(&parts) {
                            on_disk.set(&parts, node.clone())?;
                        }
                    }
                    None => {
                        on_disk.delete(&parts)?;
                    }
                }
            }
        }

        // Bring the in-memory doc up to date with the merged result so that the
        // next persist (or a watcher reload) sees a consistent picture.
        *self.doc.write() = on_disk.clone();

        let content = on_disk.serialize()?;
        persist_atomic(&self.path, &content).map_err(TextStoreError::from)?;

        if let Ok(meta) = std::fs::metadata(&self.path)
            && let Ok(mtime) = meta.modified()
        {
            *self.last_write_mtime.lock() = Some(mtime);
        }
        Ok(())
    }

    /// Full overwrite — used for the meta file which is exclusively owned by
    /// internal migrations and never shared with external writers.
    pub fn persist_full(&self) -> Result<()> {
        let content = self.doc.read().serialize()?;
        persist_atomic(&self.path, &content).map_err(TextStoreError::from)?;

        if let Ok(meta) = std::fs::metadata(&self.path)
            && let Ok(mtime) = meta.modified()
        {
            *self.last_write_mtime.lock() = Some(mtime);
        }
        Ok(())
    }

    pub fn restore_from_backup(&self, fallback_to_initial: &D) {
        *self.doc.write() = fallback_to_initial.clone();
        self.dirty_keys.lock().clear();

        if self.backup_path.exists() {
            let _ = std::fs::copy(&self.backup_path, &self.path);
            let _ = std::fs::remove_file(&self.backup_path);
        } else if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }

        if let Ok(meta) = std::fs::metadata(&self.path)
            && let Ok(mtime) = meta.modified()
        {
            *self.last_write_mtime.lock() = Some(mtime);
        } else {
            *self.last_write_mtime.lock() = None;
        }
    }

    pub fn clean_backup(&self) {
        if self.backup_path.exists() {
            let _ = std::fs::remove_file(&self.backup_path);
        }
    }
}

pub struct StoreFiles<D: TextDocument> {
    pub data: StoreFile<D>,
    pub meta: StoreFile<D>,
}

impl<D: TextDocument> Clone for StoreFiles<D> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            meta: self.meta.clone(),
        }
    }
}

impl<D: TextDocument> StoreFiles<D> {
    pub fn create_backups(&self) -> Result<()> {
        self.data.create_backup()?;
        self.meta.create_backup()?;
        Ok(())
    }

    pub fn persist(&self) -> Result<()> {
        self.data.persist()?;
        // Meta is exclusively owned by internal migrations — full overwrite is correct.
        self.meta.persist_full()?;
        Ok(())
    }

    pub fn clean_backups(&self) {
        self.data.clean_backup();
        self.meta.clean_backup();
    }

    pub fn restore_from_backups(&self, fallback_data: &D, fallback_meta: &D) {
        self.data.restore_from_backup(fallback_data);
        self.meta.restore_from_backup(fallback_meta);
    }
}

#[derive(Clone)]
pub struct TextStore<D: TextDocument> {
    pub(crate) files: StoreFiles<D>,
    pub(crate) subscriptions: Arc<RwLock<Vec<SubscriptionEntry>>>,
    pub(crate) next_id: Arc<AtomicU64>,
    pub(crate) debouncer: Arc<Debouncer>,
    pub(crate) watcher: Arc<Ticker>,
}

impl<D: TextDocument> Debug for TextStore<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextStore")
            .field("data_path", &self.files.data.path)
            .field("meta_path", &self.files.meta.path)
            .finish()
    }
}

impl<D: TextDocument + 'static> TextStore<D> {
    pub fn open(
        config: StoreConfig,
        migration_set: MigrationSet,
    ) -> Result<(Self, MigrationReport)> {
        let path = config.path.clone();
        let meta_path = config.path.with_extension("meta");

        let files = StoreFiles {
            data: StoreFile::new(path, D::empty()),
            meta: StoreFile::new(meta_path, D::empty()),
        };

        files.create_backups()?;

        let initial_data = files.data.load_or_empty()?;
        let initial_meta = files.meta.load_or_empty()?;

        *files.data.doc.write() = initial_data.clone();
        *files.meta.doc.write() = initial_meta.clone();

        let store = Self::new(config, files);

        match store.run_migrations(migration_set) {
            Ok(report) => {
                store.files.clean_backups();
                Ok((store, report))
            }
            Err(e) => {
                store
                    .files
                    .restore_from_backups(&initial_data, &initial_meta);
                Err(e)
            }
        }
    }

    fn new(config: StoreConfig, files: StoreFiles<D>) -> Self {
        info!(
            path = %config.path.display(),
            "initializing TextStore"
        );

        let subscriptions = Arc::new(RwLock::new(Vec::<SubscriptionEntry>::new()));

        let files_debounce = files.clone();
        let debouncer = Debouncer::new(config.save_debounce, move || {
            if let Err(e) = files_debounce.persist() {
                warn!("store persist failed: {e:#}");
            }
        });

        let files_watch = files.clone();
        let watch_subs = subscriptions.clone();

        let watcher = Ticker::new(config.watch_interval, move || {
            if let Some(on_disk) = check_reload_file(&files_watch.data) {
                let events = {
                    let mut guard = files_watch.data.doc.write();
                    let old = guard.clone();
                    *guard = on_disk;
                    info!("external store change detected");
                    diff_documents::<D>(&old, &*guard)
                };
                for event in events {
                    events::emit_event(&watch_subs, event);
                }
            }

            if let Some(on_disk) = check_reload_file(&files_watch.meta) {
                let matches = {
                    let guard = files_watch.meta.doc.read();
                    let current_str = guard.serialize().unwrap_or_default();
                    let on_disk_str = on_disk.serialize().unwrap_or_default();
                    current_str == on_disk_str
                };
                if !matches {
                    warn!(
                        "⚠️ External modification of metadata file detected! Metadata must only be mutated via internal migrations."
                    );
                }
            }
        });

        Self {
            files,
            subscriptions,
            next_id: Arc::new(AtomicU64::new(1)),
            debouncer: Arc::new(debouncer),
            watcher: Arc::new(watcher),
        }
    }

    pub(crate) fn emit(&self, event: StoreEvent) {
        let callbacks = self
            .subscriptions
            .read()
            .iter()
            .filter(|s| matches_kind(&s.kind, &event.path))
            .map(|s| s.callback.clone())
            .collect::<Vec<_>>();
        for cb in callbacks {
            cb(&event);
        }
    }

    fn check_debouncer(&self) -> Result<()> {
        if self.debouncer.is_poisoned() {
            panic!("debouncer thread is dead — store integrity cannot be guaranteed");
        }
        Ok(())
    }

    fn check_watcher(&self) {
        if self.watcher.is_poisoned() {
            panic!("watcher thread is dead — external change detection unavailable");
        }
    }
}

impl<D: TextDocument + 'static> SchemaAwareStore for TextStore<D> {
    fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport> {
        struct TextProvider<D: TextDocument> {
            data_doc: Arc<RwLock<D>>,
            meta_doc: Arc<RwLock<D>>,
        }

        impl<D: TextDocument> StorageProvider for TextProvider<D> {
            fn atomic<F, T>(&self, f: F) -> Result<T>
            where
                F: FnOnce(&mut dyn RawStorage) -> Result<T>,
            {
                let mut data_guard = self.data_doc.write();
                let mut meta_guard = self.meta_doc.write();

                let mut storage = TextRawStorage {
                    data_doc: &mut *data_guard,
                    meta_doc: &mut *meta_guard,
                };

                f(&mut storage)
            }
        }

        let provider = TextProvider {
            data_doc: self.files.data.doc.clone(),
            meta_doc: self.files.meta.doc.clone(),
        };
        let engine = MigrationEngine::new(&provider);
        engine.run(mset)
    }
}

impl<D: TextDocument> Store for TextStore<D> {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        self.check_watcher();
        let guard = self.files.data.doc.read();
        let parts = split_path(path);
        if let Some(node) = guard.get(&parts) {
            Ok(Some(D::deserialize_node(node)?))
        } else {
            Ok(None)
        }
    }

    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()> {
        self.check_debouncer()?;
        self.check_watcher();

        let path_str = normalize_path(path)?;
        let parts = split_path(&path_str);
        let node = D::serialize_node(value)?;

        let (old_bytes, new_bytes) = {
            let mut guard = self.files.data.doc.write();
            let old = guard
                .get(&parts)
                .map(|n| D::node_to_bytes(n))
                .transpose()?
                .map(Bytes::from);
            guard.set(&parts, node)?;
            let new_node = guard.get(&parts).unwrap();
            let new_bytes = Bytes::from(D::node_to_bytes(new_node)?);
            (old, new_bytes)
        };

        self.files.data.mark_dirty(path_str.clone(), Some(()));

        self.emit(StoreEvent {
            path: Arc::from(path_str),
            op: StoreOp::Set,
            old: old_bytes,
            new: Some(new_bytes),
        });

        self.debouncer.schedule();
        Ok(())
    }

    fn save_now(&self) -> Result<()> {
        self.files.persist()?;
        Ok(())
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Bytes)>> {
        let guard = self.files.data.doc.read();
        let parts = split_path(prefix);
        let mut raw_nodes = Vec::new();
        scan_prefix_recursive(&*guard, &parts, prefix, &mut raw_nodes);

        let mut results = Vec::new();
        for (k, node) in raw_nodes {
            let bytes = D::node_to_bytes(&node)?;
            results.push((k, Bytes::from(bytes)));
        }
        Ok(results)
    }

    fn delete(&self, path: &str) -> Result<()> {
        self.check_debouncer()?;
        self.check_watcher();

        let path_str = normalize_path(path)?;
        let parts = split_path(&path_str);

        let old_bytes = {
            let mut guard = self.files.data.doc.write();
            let old = guard
                .get(&parts)
                .map(|n| D::node_to_bytes(n))
                .transpose()?
                .map(Bytes::from);
            guard.delete(&parts)?;
            old
        };

        self.files.data.mark_dirty(path_str.clone(), None);

        self.emit(StoreEvent {
            path: Arc::from(path_str),
            op: StoreOp::Delete,
            old: old_bytes,
            new: None,
        });

        self.debouncer.schedule();
        Ok(())
    }

    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.subscriptions
            .write()
            .push(SubscriptionEntry { id, kind, callback });
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.subscriptions.write().retain(|s| s.id != id);
    }

    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T> {
        match D::bytes_to_node(bytes).and_then(|node| D::deserialize_node(&node)) {
            Ok(val) => Ok(val),
            Err(e) => {
                warn!(
                    target: "rpstate",
                    "Failed to decode text field. Data is corrupted or type changed. \
                    Using Default value. Error: {e}"
                );
                Ok(T::default())
            }
        }
    }

    fn flush_prefix(&self, _prefix: &str) -> Result<()> {
        // Unlike RedbStore, which can selectively commit pending updates for specific
        // prefixes to distinct tables, TextStore manages monolithic files (JSON/TOML).
        // Writing only a single prefix to disk is physically impossible—the entire
        // document must be serialized and written as a whole. Therefore, flushing
        // any prefix is semantically equivalent to a full synchronous save.
        self.save_now()
    }

    fn is_initialized(&self, namespace: &str) -> Result<bool> {
        let guard = self.files.meta.doc.read();
        let parts = vec!["__init", namespace];
        Ok(guard.get(&parts).is_some())
    }

    fn mark_initialized(&self, namespace: &str) -> Result<()> {
        {
            let mut guard = self.files.meta.doc.write();
            let parts = vec!["__init", namespace];
            let node = D::serialize_node(&true)?;
            guard.set(&parts, node)?;
        }

        self.files.meta.persist_full()?;
        Ok(())
    }
}

pub fn normalize_path(path: &str) -> Result<String> {
    let trimmed = path.trim();

    if trimmed == "." {
        return Ok(".".to_string());
    }

    let normalized = path
        .split('.')
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(".");

    if normalized.is_empty() {
        return Err(crate::error::Error::TextStore(TextStoreError::Codec(
            CodecError::Custom("path cannot be empty".into()),
        )));
    }
    Ok(normalized)
}

pub fn split_path(path: &str) -> Vec<&str> {
    if path == "." {
        return vec!["."];
    }
    if path.is_empty() {
        return vec![];
    }
    path.split('.').collect()
}

fn persist_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let dir = path.parent().unwrap_or(Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(content.as_bytes())?;
    tmp.persist(path).map_err(|e| e.error)?;

    Ok(())
}

fn check_reload_file<D: TextDocument>(file: &StoreFile<D>) -> Option<D> {
    if let Ok(meta) = std::fs::metadata(&file.path) {
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let should_reload = {
            let guard = file.last_write_mtime.lock();
            guard.map(|last| mtime > last).unwrap_or(true)
        };

        if should_reload && file.path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file.path) {
                if let Ok(on_disk) = D::parse(&content) {
                    *file.last_write_mtime.lock() = Some(mtime);
                    return Some(on_disk);
                }
            }
        }
    }
    None
}

pub(super) fn scan_prefix_recursive<D: TextDocument>(
    doc: &D,
    parts: &[&str],
    prefix_str: &str,
    results: &mut Vec<(String, D::Node)>,
) {
    let children = doc.scan(parts);
    if children.is_empty() {
        if !prefix_str.is_empty() {
            if let Some(node) = doc.get(parts) {
                results.push((prefix_str.to_string(), node.clone()));
            }
        }
    } else {
        for (full_key, _node) in children {
            let child_parts = split_path(&full_key);
            let grand_children = doc.scan(&child_parts);
            if grand_children.is_empty() {
                if let Some(child_node) = doc.get(&child_parts) {
                    results.push((full_key, child_node.clone()));
                }
            } else {
                scan_prefix_recursive(doc, &child_parts, prefix_str, results);
            }
        }
    }
}

fn diff_documents<D: TextDocument>(old: &D, new: &D) -> Vec<StoreEvent> {
    let mut old_nodes = Vec::new();
    scan_prefix_recursive(old, &[], "", &mut old_nodes);
    let old_map: std::collections::HashMap<String, D::Node> = old_nodes.into_iter().collect();

    let mut new_nodes = Vec::new();
    scan_prefix_recursive(new, &[], "", &mut new_nodes);
    let new_map: std::collections::HashMap<String, D::Node> = new_nodes.into_iter().collect();

    let mut events = Vec::new();

    let mut all_keys: std::collections::BTreeSet<String> = old_map.keys().cloned().collect();
    all_keys.extend(new_map.keys().cloned());

    for key in all_keys {
        let old_node = old_map.get(&key);
        let new_node = new_map.get(&key);

        match (old_node, new_node) {
            (Some(o), Some(n)) => {
                let old_bytes = D::node_to_bytes(o).ok();
                let new_bytes = D::node_to_bytes(n).ok();
                if old_bytes != new_bytes {
                    events.push(StoreEvent {
                        path: Arc::from(key),
                        op: StoreOp::Set,
                        old: old_bytes.map(Bytes::from),
                        new: new_bytes.map(Bytes::from),
                    });
                }
            }
            (Some(o), None) => {
                let old_bytes = D::node_to_bytes(o).ok();
                events.push(StoreEvent {
                    path: Arc::from(key),
                    op: StoreOp::Delete,
                    old: old_bytes.map(Bytes::from),
                    new: None,
                });
            }
            (None, Some(n)) => {
                let new_bytes = D::node_to_bytes(n).ok();
                events.push(StoreEvent {
                    path: Arc::from(key),
                    op: StoreOp::Set,
                    old: None,
                    new: new_bytes.map(Bytes::from),
                });
            }
            (None, None) => {}
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::string::string_regex;

    fn dirty_path_strategy() -> impl Strategy<Value = String> {
        string_regex("[a-zA-Z0-9_.-]{0,20}").unwrap()
    }

    proptest! {
        #[test]
        fn prop_normalize_path_is_clean(dirty_path in dirty_path_strategy()) {
            match normalize_path(&dirty_path) {
                Ok(norm) => {
                    assert!(!norm.is_empty(), "Normalized path cannot be empty");
                    if norm != "." {
                        assert!(!norm.starts_with('.'), "Should not start with a dot: {}", norm);
                    }
                    assert!(!norm.ends_with('.'), "Should not end with a dot: {}", norm);
                    assert!(!norm.contains(".."), "Should not contain double dots: {}", norm);
                }
                Err(_) => {
                    let only_dots = dirty_path.chars().all(|c| c == '.');
                    assert!(only_dots || dirty_path.trim().is_empty());
                }
            }
        }
    }
}
