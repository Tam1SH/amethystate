pub mod backend;
pub mod builder;
pub mod config;
pub mod util;

#[cfg(feature = "json")]
pub use backend::json::JsonStore;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

#[cfg(feature = "redb")]
pub use backend::redb::RedbStore;

use crate::migration::set::MigrationSet;
use crate::reactive::{MapChange, ReactiveMap, SubscriberAny, SubscriberKey};
use crate::{AccessMode, Field, MigrationReport, Result, Signal, StoreSubscription, WritableMode};
use bytes::Bytes;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

pub type SubscriptionId = u64;
pub type StoreCallback = Arc<dyn Fn(&StoreEvent) + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Set,
    Delete,
    Patch,
}

#[derive(Debug, Clone)]
pub struct StoreEvent {
    pub path: Arc<str>,
    pub op: StoreOp,
    pub old: Option<Bytes>,
    pub new: Option<Bytes>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionKind {
    Any,
    ExactPath(Arc<str>),
    Prefix(Arc<str>),
}

pub trait Store: Send + Sync + 'static {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()>;
    fn set_owned<T: Serialize>(&self, path: Arc<str>, value: &T) -> Result<()> {
        self.set(&path, value)
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Bytes)>>;
    fn delete(&self, path: &str) -> Result<()>;
    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId);
    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T>;
    fn flush_prefix(&self, prefix: &str) -> Result<()>;
}

pub trait SchemaAwareStore: Store {
    fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport>;
}

pub trait StateScope {
    const PREFIX: &'static str;
}

pub fn scoped_path<T: StateScope>(key: &str) -> String {
    if key.trim().is_empty() {
        return T::PREFIX.to_string();
    }
    format!(
        "{}.{}",
        T::PREFIX.trim_end_matches('.'),
        key.trim_start_matches('.')
    )
}

pub fn field<TScope, TValue, S>(
    store: &Arc<S>,
    key: &str,
    default: TValue,
) -> Result<Field<TValue, S, WritableMode>>
where
    TScope: StateScope,
    S: Store,
    TValue: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
{
    let path: Arc<str> = scoped_path::<TScope>(key).into();
    field_with_path(store, path, default)
}

pub fn field_with_path<TValue, S, M>(
    store: &Arc<S>,
    path: Arc<str>,
    default: TValue,
) -> Result<Field<TValue, S, M>>
where
    S: Store,
    TValue: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
    M: AccessMode,
{
    if store.get::<TValue>(&path)?.is_none() {
        store.set(&path, &default)?;
    }

    let current = store
        .get::<TValue>(&path)?
        .unwrap_or_else(|| default.clone());
    let signal = Arc::new(Signal::new(current));

    let sig_clone = Arc::clone(&signal);
    let store_clone = Arc::clone(store);
    let path_log = Arc::clone(&path);

    let id = store.subscribe(
        SubscriptionKind::ExactPath(path.clone()),
        Arc::new(move |event| {
            if let Some(raw) = &event.new {
                match store_clone.decode::<TValue>(raw) {
                    Ok(parsed) => sig_clone.set(parsed),
                    Err(e) => tracing::error!(path = %path_log, error = %e, "decode failed"),
                }
            }
        }),
    );

    Ok(Field {
        signal,
        path,
        store_sub: Some(Arc::new(StoreSubscription {
            store: Arc::clone(store),
            id,
        })),
        _mode: std::marker::PhantomData,
        interceptors: Arc::new(Mutex::new(Vec::new())),
        next_interceptor_id: Arc::new(AtomicUsize::new(0)),
        intercept_depth: Arc::new(AtomicUsize::new(0)),
    })
}

pub fn reactive_map<TScope, K, V, S>(
    store: &Arc<S>,
    key: &str,
    default: HashMap<K, V>,
) -> Result<ReactiveMap<K, V, S, WritableMode>>
where
    TScope: StateScope,
    S: Store,
    K: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
{
    let path: Arc<str> = scoped_path::<TScope>(key).into();
    reactive_map_with_path(store, path, default)
}

pub fn reactive_map_with_path<K, V, S, M>(
    store: &Arc<S>,
    path: Arc<str>,
    default: HashMap<K, V>,
) -> Result<ReactiveMap<K, V, S, M>>
where
    S: Store,
    K: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Serialize + Default + DeserializeOwned + Clone + Send + Sync + 'static,
    M: AccessMode,
{
    let mut known_keys = HashSet::new();

    let prefix = format!("{}.", path);
    let existing = store.scan_prefix(&prefix)?;
    for (fpath, _) in existing {
        if let Some(k_str) = fpath.strip_prefix(&prefix)
            && let Ok(k) = K::from_str(k_str)
        {
            known_keys.insert(k);
        }
    }

    for (k, v) in default {
        let full_path = format!("{}.{}", path, k);
        if store.get::<V>(&full_path)?.is_none() {
            store.set(&full_path, &v)?;
        }
    }

    let known_keys = Arc::new(std::sync::Mutex::new(known_keys));
    let subscribers_any = Arc::new(Mutex::new(Vec::<(u64, SubscriberAny<K, V>)>::new()));
    let subscribers_key = Arc::new(Mutex::new(
        HashMap::<K, Vec<(u64, SubscriberKey<K, V>)>>::new(),
    ));

    let subs_any = Arc::clone(&subscribers_any);
    let subs_key = Arc::clone(&subscribers_key);
    let keys_tracker = Arc::clone(&known_keys);

    let path_for_sub = path.clone();
    let prefix_for_strip = format!("{}.", path);
    let store_clone = Arc::clone(store);

    let id = store.subscribe(
        SubscriptionKind::Prefix(path_for_sub),
        Arc::new(move |event| {
            if let Some(key_str) = event.path.strip_prefix(&prefix_for_strip)
                && let Ok(k) = K::from_str(key_str)
            {
                let mut keys = keys_tracker.lock().unwrap();

                let new_val = event
                    .new
                    .as_ref()
                    .and_then(|b| store_clone.decode::<V>(b).ok());
                let old_val = event
                    .old
                    .as_ref()
                    .and_then(|b| store_clone.decode::<V>(b).ok());

                let change = match event.op {
                    StoreOp::Set | StoreOp::Patch => {
                        if keys.contains(&k) {
                            MapChange::Update {
                                key: k.clone(),
                                old_value: old_val.unwrap_or_default(),
                                new_value: new_val.unwrap_or_default(),
                            }
                        } else {
                            keys.insert(k.clone());
                            MapChange::Insert {
                                key: k.clone(),
                                value: new_val.unwrap_or_default(),
                            }
                        }
                    }
                    StoreOp::Delete => {
                        keys.remove(&k);
                        MapChange::Remove {
                            key: k.clone(),
                            old_value: old_val.unwrap_or_default(),
                        }
                    }
                };

                if let Ok(lock) = subs_key.lock()
                    && let Some(cbs) = lock.get(&k)
                {
                    for (_, cb) in cbs {
                        cb(&change);
                    }
                }
                if let Ok(lock) = subs_any.lock() {
                    for (_, cb) in lock.iter() {
                        cb(&change);
                    }
                }
            }
        }),
    );

    Ok(ReactiveMap {
        path,
        store: Arc::clone(store),
        _mode: std::marker::PhantomData,
        _key: std::marker::PhantomData,
        _value: std::marker::PhantomData,
        interceptors_any: Arc::new(std::sync::Mutex::new(Vec::new())),
        interceptors_key: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        subscribers_any,
        subscribers_key,
        next_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        intercept_depth: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        store_sub: Arc::new(StoreSubscription {
            store: Arc::clone(store),
            id,
        }),
        known_keys,
    })
}

#[derive(Clone)]
pub struct SubscriptionEntry {
    pub id: SubscriptionId,
    pub kind: SubscriptionKind,
    pub callback: StoreCallback,
}

pub fn matches_kind(kind: &SubscriptionKind, path: &str) -> bool {
    match kind {
        SubscriptionKind::Any => true,
        SubscriptionKind::ExactPath(p) => **p == *path,
        SubscriptionKind::Prefix(prefix) => {
            *path == **prefix
                || path
                    .strip_prefix(&**prefix)
                    .is_some_and(|t| t.starts_with('.'))
        }
    }
}
