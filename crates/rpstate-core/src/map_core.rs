use crate::SignalSubscription;
use crate::change::MapChange;
use crate::intercept::{InterceptDisposer, InterceptGuard};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub type InterceptorAny<K, V> =
    Arc<dyn Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static>;
pub type InterceptorKey<K, V> =
    Arc<dyn Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static>;
pub type SubscriberAny<K, V> = Arc<dyn Fn(&MapChange<K, V>) + Send + Sync + 'static>;
pub type SubscriberKey<K, V> = Arc<dyn Fn(&MapChange<K, V>) + Send + Sync + 'static>;

pub struct ReactiveMapCore<K, V> {
    pub interceptors_any: Arc<Mutex<Vec<(u64, InterceptorAny<K, V>)>>>,
    pub interceptors_key: Arc<Mutex<HashMap<K, Vec<(u64, InterceptorKey<K, V>)>>>>,
    pub subscribers_any: Arc<Mutex<Vec<(u64, SubscriberAny<K, V>)>>>,
    pub subscribers_key: Arc<Mutex<HashMap<K, Vec<(u64, SubscriberKey<K, V>)>>>>,
    pub next_id: Arc<AtomicU64>,
    pub intercept_depth: Arc<AtomicUsize>,
    pub known_keys: Arc<Mutex<HashSet<K>>>,
}

impl<K, V> Clone for ReactiveMapCore<K, V> {
    fn clone(&self) -> Self {
        Self {
            interceptors_any: self.interceptors_any.clone(),
            interceptors_key: self.interceptors_key.clone(),
            subscribers_any: self.subscribers_any.clone(),
            subscribers_key: self.subscribers_key.clone(),
            next_id: self.next_id.clone(),
            intercept_depth: self.intercept_depth.clone(),
            known_keys: self.known_keys.clone(),
        }
    }
}

impl<K: std::fmt::Debug, V> std::fmt::Debug for ReactiveMapCore<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ReactiveMapCore");

        if let Ok(keys) = self.known_keys.try_lock() {
            d.field("known_keys", &*keys);
        } else {
            d.field("known_keys", &"<locked>");
        }

        let interceptors_count = self
            .interceptors_any
            .try_lock()
            .map(|l| l.len())
            .unwrap_or(0);
        let subscribers_count = self
            .subscribers_any
            .try_lock()
            .map(|l| l.len())
            .unwrap_or(0);

        d.field("interceptors_any_count", &interceptors_count)
            .field("subscribers_any_count", &subscribers_count)
            .finish()
    }
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static> Default
    for ReactiveMapCore<K, V>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static>
    ReactiveMapCore<K, V>
{
    pub fn new() -> Self {
        Self {
            interceptors_any: Arc::new(Mutex::new(Vec::new())),
            interceptors_key: Arc::new(Mutex::new(HashMap::new())),
            subscribers_any: Arc::new(Mutex::new(Vec::new())),
            subscribers_key: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(0)),
            intercept_depth: Arc::new(AtomicUsize::new(0)),
            known_keys: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn subscribe_any<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.subscribers_any
            .lock()
            .unwrap()
            .push((id, Arc::new(callback)));
        let subs = self.subscribers_any.clone();
        SignalSubscription {
            id,
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs.lock() {
                    lock.retain(|(i, _)| *i != id);
                }
            }),
        }
    }

    pub fn subscribe_key<F>(&self, key: K, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.subscribers_key
            .lock()
            .unwrap()
            .entry(key.clone())
            .or_default()
            .push((id, Arc::new(callback)));
        let subs = self.subscribers_key.clone();
        SignalSubscription {
            id,
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs.lock()
                    && let Some(list) = lock.get_mut(&key)
                {
                    list.retain(|(i, _)| *i != id);
                }
            }),
        }
    }

    pub fn intercept<F>(&self, path: Arc<str>, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.interceptors_any
            .lock()
            .unwrap()
            .push((id, Arc::new(callback)));
        let subs = self.interceptors_any.clone();
        InterceptDisposer {
            id,
            path,
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs.lock() {
                    lock.retain(|(i, _)| *i != id);
                }
            }),
        }
    }

    pub fn intercept_key<F>(&self, key: K, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.interceptors_key
            .lock()
            .unwrap()
            .entry(key.clone())
            .or_default()
            .push((id, Arc::new(callback)));
        let subs = self.interceptors_key.clone();
        InterceptDisposer {
            id,
            path: Arc::from(""),
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs.lock()
                    && let Some(list) = lock.get_mut(&key)
                {
                    list.retain(|(i, _)| *i != id);
                }
            }),
        }
    }

    pub fn run_interceptors(
        &self,
        path: Arc<str>,
        mut change: MapChange<K, V>,
    ) -> Result<MapChange<K, V>, String> {
        if let Some(_guard) = InterceptGuard::enter(&self.intercept_depth, path) {
            let mut keys_to_intercept = Vec::new();
            if let Some(k) = change.key() {
                keys_to_intercept.push(k.clone());
            } else {
                let lock = self.interceptors_key.lock().unwrap();
                keys_to_intercept = lock.keys().cloned().collect();
            }

            for key in keys_to_intercept {
                let interceptors = {
                    let lock = self.interceptors_key.lock().unwrap();
                    lock.get(&key).cloned().unwrap_or_default()
                };
                for (_, interceptor) in interceptors {
                    if let Some(new_change) = interceptor(change.clone()) {
                        change = new_change;
                    } else {
                        return Err("Map change intercepted by key filter".to_string());
                    }
                }
            }

            let interceptors_any = {
                let lock = self.interceptors_any.lock().unwrap();
                lock.clone()
            };
            for (_, interceptor) in interceptors_any {
                if let Some(new_change) = interceptor(change.clone()) {
                    change = new_change;
                } else {
                    return Err("Map change intercepted by global filter".to_string());
                }
            }
        }
        Ok(change)
    }

    pub fn notify(&self, change: &MapChange<K, V>) {
        if let Some(k) = change.key()
            && let Ok(lock) = self.subscribers_key.lock()
            && let Some(cbs) = lock.get(k)
        {
            for (_, cb) in cbs {
                cb(change);
            }
        }
        if let Ok(lock) = self.subscribers_any.lock() {
            for (_, cb) in lock.iter() {
                cb(change);
            }
        }
    }
}
