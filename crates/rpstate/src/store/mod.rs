pub mod backend;
pub mod builder;
pub mod config;
pub mod util;

#[cfg(feature = "json")]
pub use backend::json::JsonStore;

#[cfg(feature = "redb")]
pub use backend::redb::RedbStore;

use bytes::Bytes;
use std::sync::Arc;

use crate::migration::set::MigrationSet;
use crate::{AccessMode, Field, MigrationReport, Result, Signal, StoreSubscription, WritableMode};
use serde::Serialize;
use serde::de::DeserializeOwned;

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

    fn delete(&self, path: &str) -> Result<()>;
    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId);
    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T>;
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
