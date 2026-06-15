use crate::SignalSubscription;
use crate::change::MapChange;
use crate::primitives::intercept::{InterceptDisposer, InterceptGuard};
use crate::primitives::signal::SubscriptionMeta;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub type InterceptorAny<K, V> =
    Arc<dyn Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static>;
pub type InterceptorKey<K, V> =
    Arc<dyn Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static>;
pub type SubscriberAny<K, V> = Arc<dyn Fn(&MapChange<K, V>) + Send + Sync + 'static>;
pub type SubscriberKey<K, V> = Arc<dyn Fn(&MapChange<K, V>) + Send + Sync + 'static>;

pub trait ReactiveMapKey: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static {}
impl<T: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static> ReactiveMapKey for T {}

pub trait ReactiveMapValue:
    Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Default
{
}
impl<T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Default> ReactiveMapValue
    for T
{
}

pub struct ReactiveMapCore<K, V> {
    pub interceptors_any: Arc<Mutex<Vec<(u64, InterceptorAny<K, V>)>>>,
    pub interceptors_key: Arc<Mutex<HashMap<K, Vec<(u64, InterceptorKey<K, V>)>>>>,
    pub subscribers_any: Arc<Mutex<Vec<(u64, SubscriberAny<K, V>, SubscriptionMeta)>>>,
    pub subscribers_key: Arc<Mutex<HashMap<K, Vec<(u64, SubscriberKey<K, V>, SubscriptionMeta)>>>>,
    pub next_id: Arc<AtomicU64>,
    pub intercept_depth: Arc<AtomicUsize>,
    pub cache: Arc<Mutex<HashMap<K, V>>>,
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
            cache: self.cache.clone(),
        }
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for ReactiveMapCore<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ReactiveMapCore");
        if let Ok(cache) = self.cache.try_lock() {
            d.field("cache", &*cache);
        } else {
            d.field("cache", &"<locked>");
        }
        let interceptors_count = self.interceptors_any.try_lock().map(|l| l.len()).unwrap_or(0);
        let subscribers_count = self.subscribers_any.try_lock().map(|l| l.len()).unwrap_or(0);
        d.field("interceptors_any_count", &interceptors_count)
            .field("subscribers_any_count", &subscribers_count)
            .finish()
    }
}

impl<K: ReactiveMapKey, V: ReactiveMapValue> Default for ReactiveMapCore<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: ReactiveMapKey, V: ReactiveMapValue> ReactiveMapCore<K, V> {
    pub fn new() -> Self {
        Self {
            interceptors_any: Arc::new(Mutex::new(Vec::new())),
            interceptors_key: Arc::new(Mutex::new(HashMap::new())),
            subscribers_any: Arc::new(Mutex::new(Vec::new())),
            subscribers_key: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(0)),
            intercept_depth: Arc::new(AtomicUsize::new(0)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[track_caller]
    pub fn subscribe_any<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        let location = std::panic::Location::caller();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let meta = SubscriptionMeta { id, location, name: None };
        self.subscribers_any.lock().unwrap().push((id, Arc::new(callback), meta));

        let subs_for_name = self.subscribers_any.clone();
        let set_name = Arc::new(move |name: &'static str| {
            if let Ok(mut lock) = subs_for_name.lock() {
                if let Some(entry) = lock.iter_mut().find(|(i, _, _)| *i == id) {
                    entry.2.name = Some(name);
                }
            }
        });
        let subs_for_cleanup = self.subscribers_any.clone();
        SignalSubscription {
            id,
            location,
            name: None,
            set_name,
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs_for_cleanup.lock() {
                    lock.retain(|(i, _, _)| *i != id);
                }
            }),
        }
    }

    #[track_caller]
    pub fn subscribe_key<F>(&self, key: K, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        let location = std::panic::Location::caller();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let meta = SubscriptionMeta { id, location, name: None };
        self.subscribers_key.lock().unwrap()
            .entry(key.clone())
            .or_default()
            .push((id, Arc::new(callback), meta));

        let subs_for_name = self.subscribers_key.clone();
        let key_for_name = key.clone();
        let set_name = Arc::new(move |name: &'static str| {
            if let Ok(mut lock) = subs_for_name.lock() {
                if let Some(list) = lock.get_mut(&key_for_name) {
                    if let Some(entry) = list.iter_mut().find(|(i, _, _)| *i == id) {
                        entry.2.name = Some(name);
                    }
                }
            }
        });
        let subs_for_cleanup = self.subscribers_key.clone();
        SignalSubscription {
            id,
            location,
            name: None,
            set_name,
            cleanup: Arc::new(move |id| {
                if let Ok(mut lock) = subs_for_cleanup.lock()
                    && let Some(list) = lock.get_mut(&key)
                {
                    list.retain(|(i, _, _)| *i != id);
                }
            }),
        }
    }

    pub fn intercept<F>(&self, path: Arc<str>, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.interceptors_any.lock().unwrap().push((id, Arc::new(callback)));
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
        self.interceptors_key.lock().unwrap()
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
            let interceptors_any = self.interceptors_any.lock().unwrap().clone();
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
            && let Some(entries) = lock.get(k)
        {
            for (_, cb, meta) in entries {
                tracing::trace!(
                    target: "amethystate",
                    subscription_id = meta.id,
                    name = meta.name,
                    location = format!("{}:{}", meta.location.file(), meta.location.line()),
                    "map signal emit → key subscription fire",
                );
                cb(change);
            }
        }

        if let Ok(lock) = self.subscribers_any.lock() {
            for (_, cb, meta) in lock.iter() {
                tracing::trace!(
                    target: "amethystate",
                    subscription_id = meta.id,
                    name = meta.name,
                    location = format!("{}:{}", meta.location.file(), meta.location.line()),
                    "map signal emit → any subscription fire",
                );
                cb(change);
            }
        }
    }
}
