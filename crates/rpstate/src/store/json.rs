use super::{Result, error::Error};
use crate::store::config::StoreConfig;
use crate::store::debouncer::Debouncer;
use crate::store::shared::{SubscriptionEntry, matches_kind};
use crate::store::{Store, StoreCallback, StoreEvent, StoreOp, SubscriptionId, SubscriptionKind};
use anyhow::{Context, anyhow, bail};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{collections::BTreeSet, thread};
use tracing::{debug, info, instrument, trace, warn};

pub struct JsonStore {
    path: PathBuf,
    map: Arc<RwLock<Map<String, Value>>>,
    subscriptions: Arc<RwLock<Vec<SubscriptionEntry>>>,
    next_id: AtomicU64,
    debouncer: Debouncer,
}

impl Debug for JsonStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonStore")
            .field("path", &self.path)
            .finish()
    }
}

impl Store for JsonStore {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        let guard = self.map.read().map_err(|_| Error::Poisoned)?;

        match get_at_path(&guard, split_path(path)) {
            Some(v) => Ok(Some(
                serde_json::from_value(v.clone())
                    .map_err(|e| Error::Serialization(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()> {
        let json_value =
            serde_json::to_value(value).map_err(|e| Error::Serialization(e.to_string()))?;

        let path_str = normalize_path(path)?;

        let (old_bytes, new_bytes) = {
            let mut map = self.map.write().map_err(|_| Error::Poisoned)?;

            let old = set_at_path(&mut map, &path_str, json_value.clone())?;

            let old_bytes = old
                .map(|v| serde_json::to_vec(&v))
                .transpose()
                .map_err(|e| Error::Serialization(e.to_string()))?;

            let new_bytes =
                serde_json::to_vec(&json_value).map_err(|e| Error::Serialization(e.to_string()))?;

            (old_bytes, new_bytes)
        };

        self.emit(StoreEvent {
            path: path_str,
            op: StoreOp::Set,
            old: old_bytes,
            new: Some(new_bytes),
        });

        self.debouncer.schedule();
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<()> {
        let path_str = normalize_path(path)?;

        let old_bytes = {
            let mut guard = self.map.write().map_err(|_| Error::Poisoned)?;

            let old = delete_at_path(&mut guard, &path_str)?;

            old.map(|v| serde_json::to_vec(&v))
                .transpose()
                .map_err(|e| Error::Serialization(e.to_string()))?
        };

        self.emit(StoreEvent {
            path: path_str,
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
            .unwrap()
            .push(SubscriptionEntry { id, kind, callback });

        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        let mut subs = self.subscriptions.write().unwrap();
        let before = subs.len();
        subs.retain(|s| s.id != id);
        if subs.len() == before {
            warn!(subscription_id = id, "unsubscribe called for unknown id");
        }
    }

    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        Ok(serde_json::from_slice(bytes).map_err(|e| Error::Serialization(e.to_string()))?)
    }

    fn evolve_prefix(&self, prefix: &str, version: u32, hash: u64) -> Result<()> {
        todo!()
    }
}

impl JsonStore {
    pub fn open(config: StoreConfig) -> Result<Self> {
        let initial = if config.path.exists() {
            Self::load_map(&config.path)?
        } else {
            Map::new()
        };

        Ok(Self::new(config, initial))
    }

    fn new(config: StoreConfig, initial: Map<String, Value>) -> Self {
        info!(
            path = %config.path.display(),
            initial_keys = initial.len(),
            "initializing JsonStore"
        );

        let inner = Arc::new(RwLock::new(initial));
        let subscriptions = Arc::new(RwLock::new(Vec::<SubscriptionEntry>::new()));

        let last_write_mtime: Arc<RwLock<Option<std::time::SystemTime>>> =
            Arc::new(RwLock::new(None));

        let persist_path = config.path.clone();
        let persist_inner = inner.clone();
        let lw_capture = last_write_mtime.clone();

        let debouncer = Debouncer::new(config.save_debounce, move || {
            let snapshot = persist_inner.read().unwrap().clone();
            if let Err(e) = persist_atomic(&persist_path, &snapshot) {
                warn!("store save failed: {e:#}");
            } else if let Ok(meta) = std::fs::metadata(&persist_path) {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(mut lw) = lw_capture.write() {
                        *lw = Some(mtime);
                    }
                }
            }
        });

        let watch_path = config.path.clone();
        let watch_inner = inner.clone();
        let watch_subs = subscriptions.clone();
        let watch_mtime = last_write_mtime.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(config.watch_interval);

                if let Ok(meta) = std::fs::metadata(&watch_path) {
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if let Ok(lw) = watch_mtime.read() {
                        if let Some(last) = *lw {
                            if mtime <= last {
                                continue;
                            }
                        }
                    }
                }

                let on_disk = if watch_path.exists() {
                    match Self::load_map(&watch_path) {
                        Ok(map) => map,
                        Err(e) => {
                            warn!("watch reload failed: {e:#}");
                            continue;
                        }
                    }
                } else {
                    Map::new()
                };

                let events = {
                    let mut guard = watch_inner.write().unwrap();
                    if *guard == on_disk {
                        vec![]
                    } else {
                        let old = guard.clone();
                        *guard = on_disk.clone();
                        info!("external store change detected");
                        diff_maps(&old, &on_disk)
                    }
                };

                for event in events {
                    emit_event(&watch_subs, event);
                }
            }
        });

        Self {
            path: config.path,
            map: inner,
            subscriptions,
            next_id: AtomicU64::new(1),
            debouncer,
        }
    }

    pub fn patch(&self, path: &str, patch: Value) -> Result<()> {
        let patch_obj = patch
            .as_object()
            .cloned()
            .ok_or_else(|| Error::InvalidPath("patch must be an object".to_string()))?;

        let path_str = normalize_path(path)?;

        let (old_bytes, new_bytes) = {
            let mut guard = self.map.write().map_err(|_| Error::Poisoned)?;

            let old = get_at_path(&guard, split_path(&path_str)).cloned();

            let target = ensure_object_at_path(&mut guard, &path_str)?;

            merge_objects(target, &patch_obj);

            let new = get_at_path(&guard, split_path(&path_str)).cloned();

            let old_bytes = old
                .map(|v| serde_json::to_vec(&v))
                .transpose()
                .map_err(|e| Error::Serialization(e.to_string()))?;

            let new_bytes = new
                .map(|v| serde_json::to_vec(&v))
                .transpose()
                .map_err(|e| Error::Serialization(e.to_string()))?;

            (old_bytes, new_bytes)
        };

        self.emit(StoreEvent {
            path: path_str,
            op: StoreOp::Patch,
            old: old_bytes,
            new: new_bytes,
        });

        self.debouncer.schedule();
        Ok(())
    }

    pub fn snapshot(&self) -> Map<String, Value> {
        self.map.read().unwrap().clone()
    }

    pub fn save_now(&self) -> Result<()> {
        info!(path = %self.path.display(), "saving store synchronously");
        persist_atomic(&self.path, &self.snapshot())?;
        Ok(())
    }

    pub fn on_any<F: Fn(&StoreEvent) + Send + Sync + 'static>(&self, cb: F) -> SubscriptionId {
        self.subscribe(SubscriptionKind::Any, Arc::new(cb))
    }

    pub fn on_path<F: Fn(&StoreEvent) + Send + Sync + 'static>(
        &self,
        path: Arc<str>,
        cb: F,
    ) -> SubscriptionId {
        self.subscribe(SubscriptionKind::ExactPath(path), Arc::new(cb))
    }

    pub fn on_prefix<F: Fn(&StoreEvent) + Send + Sync + 'static>(
        &self,
        prefix: Arc<str>,
        cb: F,
    ) -> SubscriptionId {
        self.subscribe(SubscriptionKind::Prefix(prefix), Arc::new(cb))
    }

    fn load_map(path: &Path) -> Result<Map<String, Value>> {
        let raw = std::fs::read(path)?;

        let value: Value =
            serde_json::from_slice(&raw).map_err(|e| Error::Serialization(e.to_string()))?;

        match value {
            Value::Object(map) => Ok(map),
            _ => Err(Error::Backend("root must be an object".to_string())),
        }
    }

    fn emit(&self, event: StoreEvent) {
        let callbacks = self
            .subscriptions
            .read()
            .map(|subs| {
                subs.iter()
                    .filter(|s| matches_kind(&s.kind, &event.path))
                    .map(|s| s.callback.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap();
        for cb in callbacks {
            cb(&event);
        }
    }
}

fn emit_event(subs: &Arc<RwLock<Vec<SubscriptionEntry>>>, event: StoreEvent) {
    let callbacks = subs
        .read()
        .map(|s| {
            s.iter()
                .filter(|e| matches_kind(&e.kind, &event.path))
                .map(|e| e.callback.clone())
                .collect::<Vec<_>>()
        })
        .unwrap();
    for cb in callbacks {
        cb(&event);
    }
}

fn diff_maps(old: &Map<String, Value>, new: &Map<String, Value>) -> Vec<StoreEvent> {
    let mut events = Vec::new();
    let keys = old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in keys {
        collect_diff(old.get(&key), new.get(&key), &key, &mut events);
    }
    events
}

fn collect_diff(
    old: Option<&Value>,
    new: Option<&Value>,
    path: &str,
    events: &mut Vec<StoreEvent>,
) {
    match (old, new) {
        (None, None) | (Some(_), Some(_)) if old == new => {}
        (Some(Value::Object(om)), Some(Value::Object(nm))) => {
            let keys = om.keys().chain(nm.keys()).cloned().collect::<BTreeSet<_>>();
            for key in keys {
                collect_diff(om.get(&key), nm.get(&key), &format!("{path}.{key}"), events);
            }
        }
        (None, Some(nv)) => events.push(StoreEvent {
            path: path.to_string(),
            op: StoreOp::Set,
            old: None,
            new: serde_json::to_vec(nv).ok(),
        }),
        (Some(ov), None) => events.push(StoreEvent {
            path: path.to_string(),
            op: StoreOp::Delete,
            old: serde_json::to_vec(ov).ok(),
            new: None,
        }),
        (Some(ov), Some(nv)) => events.push(StoreEvent {
            path: path.to_string(),
            op: StoreOp::Set,
            old: serde_json::to_vec(ov).ok(),
            new: serde_json::to_vec(nv).ok(),
        }),
        _ => {}
    }
}

fn normalize_path(path: &str) -> Result<String> {
    let normalized = path
        .split('.')
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(".");
    if normalized.is_empty() {
        return Err(Error::InvalidPath("empty path".to_string()));
    }
    Ok(normalized)
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('.').filter(|s| !s.is_empty()).collect()
}

fn get_at_path<'a>(map: &'a Map<String, Value>, parts: Vec<&str>) -> Option<&'a Value> {
    let mut iter = parts.into_iter();
    let mut current = map.get(iter.next()?)?;
    for key in iter {
        current = current.as_object()?.get(key)?;
    }
    Some(current)
}

fn set_at_path(map: &mut Map<String, Value>, path: &str, value: Value) -> Result<Option<Value>> {
    let (parent, key) = walk_mut(map, path, true)?;
    Ok(parent.insert(key, value))
}

fn ensure_object_at_path<'a>(
    map: &'a mut Map<String, Value>,
    path: &str,
) -> Result<&'a mut Map<String, Value>> {
    let (parent, key) = walk_mut(map, path, true)?;
    let target = parent
        .entry(key)
        .or_insert_with(|| Value::Object(Map::new()));

    target
        .as_object_mut()
        .ok_or_else(|| Error::InvalidPath("Target is not an object".into()))
}

fn delete_at_path(map: &mut Map<String, Value>, path: &str) -> Result<Option<Value>> {
    let (parent, key) = walk_mut(map, path, false)?;
    Ok(parent.remove(&key))
}

fn walk_mut<'a>(
    root: &'a mut Map<String, Value>,
    path: &str,
    create_missing: bool,
) -> Result<(&'a mut Map<String, Value>, String)> {
    let parts = split_path(path);
    let (last, heads) = parts
        .split_last()
        .ok_or_else(|| Error::InvalidPath("Path cannot be empty".to_string()))?;

    let mut current = root;
    for &key in heads {
        let entry = if create_missing {
            current
                .entry(key)
                .or_insert_with(|| Value::Object(Map::new()))
        } else {
            current
                .get_mut(key)
                .ok_or_else(|| Error::InvalidPath(format!("Path segment '{}' not found", key)))?
        };

        current = entry
            .as_object_mut()
            .ok_or_else(|| Error::InvalidPath(format!("Segment '{}' is not an object", key)))?;
    }

    Ok((current, last.to_string()))
}

fn merge_objects(target: &mut Map<String, Value>, patch: &Map<String, Value>) {
    for (k, v) in patch {
        match (target.get_mut(k), v) {
            (Some(Value::Object(t)), Value::Object(p)) => merge_objects(t, p),
            _ => {
                target.insert(k.clone(), v.clone());
            }
        }
    }
}

fn persist_atomic(path: &Path, map: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_vec_pretty(&Value::Object(map.clone()))
        .map_err(|e| Error::Serialization(e.to_string()))?;

    std::fs::write(&tmp, data)?;

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    std::fs::rename(&tmp, path)?;

    debug!("store persisted atomically");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn unique_test_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("rpstate-{suffix}-{nanos}.json"))
    }

    fn make_store(suffix: &str) -> JsonStore {
        JsonStore::open(StoreConfig::new(unique_test_path(suffix))).unwrap()
    }

    fn decode_event_value(bytes: Option<&Vec<u8>>) -> Option<Value> {
        bytes.map(|b| serde_json::from_slice(b).expect("event bytes must be valid JSON"))
    }

    #[test]
    fn set_get_delete_roundtrip() {
        let store = make_store("roundtrip");

        store
            .set("ui.theme.dark", &true)
            .expect("set should succeed");
        assert_eq!(store.get::<bool>("ui.theme.dark").unwrap(), Some(true));

        store
            .delete("ui.theme.dark")
            .expect("delete should succeed");
        assert_eq!(store.get::<bool>("ui.theme.dark").unwrap(), None);
    }

    #[test]
    fn patch_merges_objects() {
        let store = make_store("patch");

        store
            .set(
                "process.columns",
                &json!({"cpu": {"width": 70}, "memory": {"width": 120}}),
            )
            .expect("set should succeed");

        store
            .patch("process.columns", json!({"memory": {"width": 140}}))
            .expect("patch should succeed");

        assert_eq!(
            store.get::<u64>("process.columns.cpu.width").unwrap(),
            Some(70)
        );
        assert_eq!(
            store.get::<u64>("process.columns.memory.width").unwrap(),
            Some(140)
        );
    }

    #[test]
    fn subscriptions_any_exact_prefix_fire() {
        let store = make_store("subscriptions");

        let any_hits: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let exact_hits: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let prefix_hits: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let cap = any_hits.clone();
        store.on_any(move |evt| {
            cap.lock().unwrap().push(evt.path.clone());
        });

        let cap = exact_hits.clone();
        store.on_path(Arc::from("ui.theme.dark"), move |evt| {
            cap.lock().unwrap().push(evt.path.clone());
        });

        let cap = prefix_hits.clone();
        store.on_prefix(Arc::from("ui.theme"), move |evt| {
            cap.lock().unwrap().push(evt.path.clone());
        });

        store
            .set("ui.theme.dark", &true)
            .expect("set should succeed");
        store
            .set("ui.layout.sidebar_width", &260u64)
            .expect("set should succeed");

        assert_eq!(any_hits.lock().unwrap().len(), 2);
        assert_eq!(exact_hits.lock().unwrap().as_slice(), ["ui.theme.dark"]);
        assert_eq!(prefix_hits.lock().unwrap().as_slice(), ["ui.theme.dark"]);
    }

    #[test]
    fn unsubscribe_stops_callbacks() {
        let store = make_store("unsubscribe");

        let hit_count = Arc::new(Mutex::new(0usize));
        let cap = hit_count.clone();
        let id = store.on_any(move |_| {
            *cap.lock().unwrap() += 1;
        });

        store
            .set("ui.theme.dark", &true)
            .expect("set should succeed");
        store.unsubscribe(id);
        store
            .set("ui.theme.dark", &false)
            .expect("set should succeed");

        assert_eq!(*hit_count.lock().unwrap(), 1);
    }

    // #[test]
    // fn file_watch_emits_set_for_external_change() {
    //     let path = unique_test_path("watch-set");
    //     std::fs::write(
    //         &path,
    //         serde_json::to_vec_pretty(&json!({
    //             "rpstate": { "watch_interval_ms": 50 },
    //             "ui": { "theme": { "dark": false } }
    //         }))
    //         .unwrap(),
    //     )
    //     .expect("seed file should be written");
    //
    //     let store = make_store(path.to_str().unwrap());
    //     let (tx, rx) = std::sync::mpsc::channel::<StoreEvent>();
    //
    //     store.on_path(Arc::from("ui.theme.dark"), move |evt| {
    //         let _ = tx.send(evt.clone());
    //     });
    //
    //     std::fs::write(
    //         &path,
    //         serde_json::to_vec_pretty(&json!({
    //             "rpstate": { "watch_interval_ms": 50 },
    //             "ui": { "theme": { "dark": true } }
    //         }))
    //         .unwrap(),
    //     )
    //     .expect("updated file should be written");
    //
    //     let event = rx
    //         .recv_timeout(Duration::from_secs(3))
    //         .expect("watcher should emit set event");
    //
    //     assert_eq!(event.path, "ui.theme.dark");
    //     assert_eq!(event.op, StoreOp::Set);
    //     assert_eq!(decode_event_value(event.old.as_ref()), Some(json!(false)));
    //     assert_eq!(decode_event_value(event.new.as_ref()), Some(json!(true)));
    // }
    //
    // #[test]
    // fn file_watch_emits_delete_for_external_removal() {
    //     let path = unique_test_path("watch-delete");
    //     std::fs::write(
    //         &path,
    //         serde_json::to_vec_pretty(&json!({
    //             "rpstate": { "watch_interval_ms": 50 },
    //             "ui": { "theme": { "dark": true } }
    //         }))
    //         .unwrap(),
    //     )
    //     .expect("seed file should be written");
    //
    //     let store = make_store(path.to_str().unwrap());
    //     let (tx, rx) = std::sync::mpsc::channel::<StoreEvent>();
    //
    //     store.on_path(Arc::from("ui.theme.dark"), move |evt| {
    //         let _ = tx.send(evt.clone());
    //     });
    //
    //     std::fs::write(
    //         &path,
    //         serde_json::to_vec_pretty(&json!({
    //             "rpstate": { "watch_interval_ms": 50 },
    //             "ui": { "theme": {} }
    //         }))
    //         .unwrap(),
    //     )
    //     .expect("updated file should be written");
    //
    //     let event = rx
    //         .recv_timeout(Duration::from_secs(3))
    //         .expect("watcher should emit delete event");
    //
    //     assert_eq!(event.path, "ui.theme.dark");
    //     assert_eq!(event.op, StoreOp::Delete);
    //     assert_eq!(decode_event_value(event.old.as_ref()), Some(json!(true)));
    //     assert_eq!(event.new, None);
    // }
    //
    // #[test]
    // fn snapshot_and_save_now() {
    //     let path = unique_test_path("snapshot");
    //     let store = make_store(path.to_str().unwrap());
    //
    //     store.set("app.version", &json!("1.0.0")).unwrap();
    //     store.set("app.debug", &true).unwrap();
    //
    //     let snap = store.snapshot();
    //     assert_eq!(snap.len(), 1);
    //     assert_eq!(snap["app"]["version"], "1.0.0");
    //
    //     if path.exists() {
    //         std::fs::remove_file(&path).unwrap();
    //     }
    //
    //     store.save_now().unwrap();
    //
    //     assert!(path.exists());
    //     let content = std::fs::read_to_string(&path).unwrap();
    //     let disk: Value = serde_json::from_str(&content).unwrap();
    //     assert_eq!(disk["app"]["version"], "1.0.0");
    // }

    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::string::string_regex;

    fn json_scalar_strategy() -> impl Strategy<Value = Value> {
        prop_oneof![
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(|n| Value::Number(n.into())),
            ".*".prop_map(Value::String),
            Just(Value::Null),
        ]
    }

    fn path_strategy() -> impl Strategy<Value = String> {
        vec(string_regex("[a-zA-Z0-9_-]{1,10}").unwrap(), 1..5).prop_map(|parts| parts.join("."))
    }

    fn dirty_path_strategy() -> impl Strategy<Value = String> {
        string_regex("[a-zA-Z0-9_.-]{0,20}").unwrap()
    }

    proptest! {
        #[test]
        fn prop_normalize_path_is_clean(dirty_path in dirty_path_strategy()) {
            match normalize_path(&dirty_path) {
                Ok(norm) => {
                    assert!(!norm.is_empty(), "Normalized path cannot be empty");
                    assert!(!norm.starts_with('.'), "Should not start with a dot: {}", norm);
                    assert!(!norm.ends_with('.'), "Should not end with a dot: {}", norm);
                    assert!(!norm.contains(".."), "Should not contain double dots: {}", norm);
                }
                Err(Error::InvalidPath(_)) => {
                    let only_dots = dirty_path.chars().all(|c| c == '.');
                    assert!(only_dots || dirty_path.trim().is_empty());
                }
                Err(_) => panic!("Expected InvalidPath error only"),
            }
        }

        #[test]
        fn prop_set_and_get_roundtrip(path in path_strategy(), val in json_scalar_strategy()) {
            let mut map = Map::new();

            set_at_path(&mut map, &path, val.clone())
                .expect("set_at_path must not fail on valid paths");

            let parts = split_path(&path);
            let retrieved = get_at_path(&map, parts)
                .expect("Value must exist after being set");

            assert_eq!(retrieved, &val, "Retrieved value must match the inserted value");
        }

        #[test]
        fn prop_delete_removes_value(path in path_strategy(), val in json_scalar_strategy()) {
            let mut map = Map::new();

            set_at_path(&mut map, &path, val).unwrap();
            let deleted = delete_at_path(&mut map, &path).unwrap();

            assert!(deleted.is_some(), "delete_at_path must return the removed value");

            let parts = split_path(&path);
            assert!(get_at_path(&map, parts).is_none(), "Value must not be found after deletion");
        }

        #[test]
        fn prop_merge_objects_into_empty(key in "[a-z]{1,5}", val in json_scalar_strategy()) {
            let mut target = Map::new();
            let mut patch = Map::new();
            patch.insert(key.clone(), val.clone());

            merge_objects(&mut target, &patch);

            assert_eq!(target.get(&key), Some(&val));
        }
    }
}
