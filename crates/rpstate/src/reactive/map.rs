use crate::error::Error;
use crate::store::Store;
use crate::{
    AccessMode, DefaultStore, Field, ReadOnlyMode, Result, StoreSubscription, WritableMode,
};
use rpstate_core::{InterceptDisposer, MapChange, ReactiveMapCore, SignalSubscription};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

pub struct ReactiveMap<K, V, S: Store = DefaultStore, M: AccessMode = ReadOnlyMode> {
    pub core: ReactiveMapCore<K, V>,
    pub path: Arc<str>,
    pub store: S,
    pub(crate) store_sub: Arc<StoreSubscription<S>>,
    pub(crate) _mode: PhantomData<M>,
}

pub type ReadOnlyReactiveMap<TValue, S> = Field<TValue, S, ReadOnlyMode>;
pub type WritableReactiveMap<TValue, S> = Field<TValue, S, WritableMode>;

impl<K, V, S: Store, M: AccessMode> Clone for ReactiveMap<K, V, S, M> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            path: self.path.clone(),
            store: self.store.clone(),
            store_sub: self.store_sub.clone(),
            _mode: PhantomData,
        }
    }
}

impl<K, V, S, M> std::fmt::Debug for ReactiveMap<K, V, S, M>
where
    K: std::fmt::Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: std::fmt::Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
    S: Store,
    M: AccessMode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ReactiveMap");
        d.field("path", &self.path);

        match self.entries() {
            Ok(entries) => {
                d.field("entries", &entries);
            }
            Err(e) => {
                d.field("entries_error", &format_args!("{:?}", e));

                if let Ok(keys) = self.core.known_keys.try_lock() {
                    d.field("known_keys", &*keys);
                }
            }
        }

        d.field("core", &self.core).finish()
    }
}

impl<K, V, S, M> ReactiveMap<K, V, S, M>
where
    K: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Default,
    S: Store,
    M: AccessMode,
{
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let full_path = format!("{}.{}", self.path, key);
        self.store.get(&full_path)
    }

    pub fn contains_key(&self, key: &K) -> Result<bool> {
        self.get(key).map(|v| v.is_some())
    }

    pub fn entries(&self) -> Result<Vec<(K, V)>> {
        let prefix = format!("{}.", self.path);
        let kvs = self.store.scan_prefix(&prefix)?;
        let mut results = Vec::new();
        for (full_path, bytes) in kvs {
            if let Some(key_str) = full_path.strip_prefix(&prefix)
                && let Ok(k) = K::from_str(key_str)
                && let Ok(v) = self.store.decode::<V>(&bytes)
            {
                results.push((k, v));
            }
        }
        Ok(results)
    }

    pub fn len(&self) -> Result<usize> {
        let prefix = format!("{}.", self.path);
        let kvs = self.store.scan_prefix(&prefix)?;
        Ok(kvs.len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        self.len().map(|l| l == 0)
    }

    pub fn subscribe_any<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.core.subscribe_any(callback)
    }

    pub fn subscribe_key<F>(&self, key: K, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.core.subscribe_key(key, callback)
    }
}

impl<K, V, S> ReactiveMap<K, V, S, WritableMode>
where
    K: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
    S: Store,
{
    pub fn set(&self, key: K, value: &V) -> Result<()> {
        let old_value = self.get(&key)?;
        if let Some(old) = old_value {
            let change = MapChange::Update {
                key: key.clone(),
                old_value: old,
                new_value: value.clone(),
            };
            self.apply_change(change)
        } else {
            Err(Error::KeyNotFound(key.to_string()))
        }
    }

    pub fn set_or_create(&self, key: K, value: &V) -> Result<()> {
        let old_value = self.get(&key)?;
        let change = if let Some(old) = old_value {
            MapChange::Update {
                key: key.clone(),
                old_value: old,
                new_value: value.clone(),
            }
        } else {
            MapChange::Insert {
                key: key.clone(),
                value: value.clone(),
            }
        };
        self.apply_change(change)
    }

    pub fn remove(&self, key: K) -> Result<Option<V>> {
        let exists = {
            let keys = self.core.known_keys.lock().unwrap();
            keys.contains(&key)
        };

        if !exists {
            return Ok(None);
        }

        let old_value = self.get(&key)?;
        if let Some(old) = old_value {
            let change = MapChange::Remove {
                key: key.clone(),
                old_value: old.clone(),
            };
            self.apply_change(change)?;
            Ok(Some(old))
        } else {
            self.core.known_keys.lock().unwrap().remove(&key);
            Ok(None)
        }
    }

    pub fn clear(&self) -> Result<()> {
        self.apply_change(MapChange::Clear)
    }

    fn apply_change(&self, change: MapChange<K, V>) -> Result<()> {
        let context_path: Arc<str> = match change.key() {
            Some(key) => format!("{}.{}", self.path, key).into(),
            None => self.path.clone(),
        };

        let processed_change = self
            .core
            .run_interceptors(context_path, change)
            .map_err(|_| Error::Intercepted)?;

        match &processed_change {
            MapChange::Insert { key, value }
            | MapChange::Update {
                key,
                new_value: value,
                ..
            } => {
                let full_path = format!("{}.{}", self.path, key);
                self.store.set(&full_path, value)?;
            }
            MapChange::Remove { key, .. } => {
                let full_path = format!("{}.{}", self.path, key);
                self.store.delete(&full_path)?;
            }
            MapChange::Clear => {
                let prefix = format!("{}.", self.path);
                let kvs = self.store.scan_prefix(&prefix)?;
                for (full_path, _) in kvs {
                    self.store.delete(&full_path)?;
                }
                self.core.notify(&processed_change);
            }
        }

        Ok(())
    }

    pub fn intercept<F>(&self, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        self.core.intercept(self.path.clone(), callback)
    }

    pub fn intercept_key<F>(&self, key: K, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        self.core.intercept_key(key, callback)
    }
}

impl<K, V, S: Store, M: AccessMode> PartialEq for ReactiveMap<K, V, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && Arc::ptr_eq(&self.core.next_id, &other.core.next_id)
    }
}

impl<K, V, S: Store, M: AccessMode> Eq for ReactiveMap<K, V, S, M> {}

#[cfg(test)]
mod tests {
    struct TestScope;
    impl crate::StateScope for TestScope {
        const PREFIX: &'static str = "test";
    }

    use super::*;
    use crate::DefaultStore;
    use crate::store::builder::StoreBuilder;
    use rpstate_core::WritableMode;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tracing_test::traced_test;

    fn setup_store(name: &str) -> DefaultStore {
        let suffix = rand::random::<u32>();
        let path = std::env::temp_dir().join(format!("rpstate-map-unit-{}-{}.db", name, suffix));
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        StoreBuilder::new(path)
            .debounce(50)
            .build()
            .expect("Failed to build DefaultStore")
    }

    #[test]
    fn test_map_crud_logic() {
        let store = setup_store("crud");
        let path: Arc<str> = Arc::from("test_map.data");

        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                path,
                HashMap::new(),
            )
            .unwrap();

        map.set_or_create("a".into(), &10).unwrap();
        assert_eq!(map.get(&"a".into()).unwrap(), Some(10));
        assert_eq!(map.len().unwrap(), 1);

        map.set("a".into(), &20).unwrap();
        assert_eq!(map.get(&"a".into()).unwrap(), Some(20));

        let res = map.set("missing".into(), &30);
        assert!(matches!(res, Err(Error::KeyNotFound(_))));

        map.set_or_create("b".into(), &100).unwrap();
        let entries = map.entries().unwrap();
        assert_eq!(entries.len(), 2);

        let removed = map.remove("a".into()).unwrap();
        assert_eq!(removed, Some(20));
        assert_eq!(map.len().unwrap(), 1);

        store.save_now().unwrap();
        assert_eq!(map.get(&"a".into()).unwrap(), None);
    }

    #[test]
    fn test_map_intercept_and_reject() {
        let store = setup_store("reject");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.intercept"),
                HashMap::new(),
            )
            .unwrap();

        map.intercept(|change| match change {
            MapChange::Insert { value, .. }
            | MapChange::Update {
                new_value: value, ..
            } if value < 0 => None,
            _ => Some(change),
        });

        let res = map.set_or_create("val".into(), &-1);
        assert!(matches!(res, Err(Error::Intercepted)));

        store.save_now().unwrap();
        assert_eq!(map.get(&"val".into()).unwrap(), None);

        map.set_or_create("val".into(), &10).unwrap();
        assert_eq!(map.get(&"val".into()).unwrap(), Some(10));
    }

    #[test]
    fn test_map_intercept_transform() {
        let store = setup_store("transform");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.transform"),
                HashMap::new(),
            )
            .unwrap();

        map.intercept(|change| match change {
            MapChange::Insert { key, value } => Some(MapChange::Insert {
                key,
                value: value * 2,
            }),
            MapChange::Update {
                key,
                old_value,
                new_value,
            } => Some(MapChange::Update {
                key,
                old_value,
                new_value: new_value * 2,
            }),
            _ => Some(change),
        });

        map.set_or_create("x".into(), &5).unwrap();
        assert_eq!(map.get(&"x".into()).unwrap(), Some(10));
    }

    #[test]
    fn test_map_subscriptions() {
        let store = setup_store("subs");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.subs"),
                HashMap::new(),
            )
            .unwrap();

        let events = Arc::new(Mutex::new(Vec::new()));
        let e_clone = events.clone();

        let _sub = map.subscribe_any(move |change| {
            e_clone.lock().unwrap().push(change.clone());
        });

        map.set_or_create("key1".into(), &1).unwrap();
        map.set("key1".into(), &2).unwrap();
        map.remove("key1".into()).unwrap();

        std::thread::sleep(Duration::from_millis(100));

        let res = events.lock().unwrap();

        assert!(res.len() >= 3);
        assert!(matches!(res[0], MapChange::Insert { .. }));
        assert!(matches!(res[1], MapChange::Update { .. }));
        assert!(matches!(res[2], MapChange::Remove { .. }));
    }

    #[test]
    fn test_reentrancy_guard() {
        let store = setup_store("reentrancy");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.reentrancy"),
                HashMap::new(),
            )
            .unwrap();

        let map_clone = map.clone();
        map.intercept(move |change| {
            if let MapChange::Update { key, .. } = &change
                && key == "a"
            {
                let _ = map_clone.set("a".into(), &999);
            }
            Some(change)
        });

        map.set_or_create("a".into(), &1).unwrap();
        map.set("a".into(), &2).unwrap();

        assert_eq!(map.get(&"a".into()).unwrap(), Some(2));
    }

    #[test]
    fn test_map_clear() {
        let store = setup_store("clear");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.clear"),
                HashMap::new(),
            )
            .unwrap();

        map.set_or_create("k1".into(), &1).unwrap();
        map.set_or_create("k2".into(), &2).unwrap();

        assert_eq!(map.len().unwrap(), 2);

        let clear_events_count = Arc::new(AtomicUsize::new(0));
        let clear_events_count_clone = clear_events_count.clone();

        let _sub = map.subscribe_any(move |change| {
            if let MapChange::Clear = change {
                clear_events_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        map.clear().unwrap();
        store.save_now().unwrap();

        assert_eq!(map.len().unwrap(), 0);
        assert!(map.is_empty().unwrap());

        assert_eq!(clear_events_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_contains_key_and_cleanup() {
        let store = setup_store("contains");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.contains"),
                HashMap::new(),
            )
            .unwrap();

        map.set_or_create("key1".into(), &1).unwrap();

        assert!(map.contains_key(&"key1".into()).unwrap());
        assert!(!map.contains_key(&"key2".into()).unwrap());

        let call_count = Arc::new(AtomicUsize::new(0));
        let c_clone = call_count.clone();
        {
            let _sub = map.subscribe_any(move |_| {
                c_clone.fetch_add(1, Ordering::SeqCst);
            });
            map.set("key1".into(), &2).unwrap();
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
        }

        map.set("key1".into(), &3).unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_key_specific_logic() {
        let store = setup_store("key_spec");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.keyspec"),
                HashMap::new(),
            )
            .unwrap();

        map.set_or_create("target".into(), &10).unwrap();
        map.set_or_create("other".into(), &20).unwrap();

        let target_calls = Arc::new(AtomicUsize::new(0));
        let t_clone = target_calls.clone();
        let _sub = map.subscribe_key("target".into(), move |_| {
            t_clone.fetch_add(1, Ordering::SeqCst);
        });

        map.set("target".into(), &11).unwrap();
        map.set("other".into(), &21).unwrap();
        assert_eq!(target_calls.load(Ordering::SeqCst), 1);

        map.intercept_key("target".into(), |change| {
            if let MapChange::Update { new_value, .. } = change
                && new_value > 100
            {
                return None;
            }
            Some(change)
        });

        map.set("target".into(), &50).unwrap();
        let res = map.set("target".into(), &150);
        assert!(matches!(res, Err(Error::Intercepted)));

        map.set("other".into(), &150).unwrap();
    }

    #[test]
    fn test_entries_parsing_failures() {
        let store = setup_store("parsing");
        let path: Arc<str> = Arc::from("test.parse");

        {
            let map_str: ReactiveMap<String, String, DefaultStore, WritableMode> =
                crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                    &store,
                    path.clone(),
                    HashMap::new(),
                )
                .unwrap();

            map_str
                .set_or_create("not_int_key".into(), &"1".into())
                .unwrap();
            map_str
                .set_or_create("123".into(), &"invalid_value".into())
                .unwrap();
        }

        let map_int: ReactiveMap<i32, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                path,
                HashMap::new(),
            )
            .unwrap();

        let entries = map_int.entries().unwrap();

        // i32::from_str("123") succeed, but decoder falls back to Default (0) for invalid bytes
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], (123, 0));
    }

    #[test]
    fn test_remove_edge_cases() {
        let store = setup_store("remove_edge");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.remove"),
                HashMap::new(),
            )
            .unwrap();

        let res = map.remove("none".into()).unwrap();
        assert!(res.is_none());

        map.set_or_create("ghost".into(), &1).unwrap();
        store.delete("test.remove.ghost").unwrap();

        let res = map.remove("ghost".into()).unwrap();
        assert!(res.is_none());
        assert!(!map.contains_key(&"ghost".into()).unwrap());
    }

    #[test]
    #[traced_test]
    fn test_map_recursion_warning() {
        let store = setup_store("map_trace");
        let map: ReactiveMap<String, i32, DefaultStore, WritableMode> =
            crate::store::reactive_map_with_path::<TestScope, _, _, _, _>(
                &store,
                Arc::from("test.recursive_map"),
                HashMap::new(),
            )
            .unwrap();

        let map_clone = map.clone();

        map.intercept(move |change| {
            if let Some(key) = change.key() {
                let _ = map_clone.set_or_create(key.clone(), &999);
            }
            Some(change)
        });

        let _ = map.set_or_create("key_a".into(), &1);

        assert!(logs_contain("maximum intercept depth reached"));
        assert!(logs_contain("test.recursive_map.key_a"));
    }
}
