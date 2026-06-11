pub mod backend;
pub mod builder;
pub mod config;
pub mod default;
pub mod meta;
mod primitives_factory;
pub(crate) mod sync_backend;
pub mod util;
pub use primitives_factory::*;

use crate::migration::AppliedStep;
use crate::migration::set::MigrationSet;
use crate::store::meta::{PrefixMeta, SchemaSnapshot};
use crate::{MigrationReport, Result};
use rpstate_core::ReactiveScope;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use uuid::Uuid;

pub type SubscriptionId = u64;
pub type StoreCallback = Arc<dyn Fn(&StoreEvent) + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Set,
    Delete,
}

#[derive(Debug, Clone)]
pub struct StoreEvent {
    pub path: Arc<str>,
    pub op: StoreOp,
    pub old: Option<Vec<u8>>,
    pub new: Option<Vec<u8>>,
    pub source: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionKind {
    Any,
    ExactPath(Arc<str>),
    Prefix(Arc<str>),
}

pub trait Store: Eq + Clone + Sized + Send + Sync + 'static {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()>;
    fn set_owned<T: Serialize>(&self, path: Arc<str>, value: &T) -> Result<()> {
        self.set(&path, value)
    }
    fn set_with_source<T: Serialize>(
        &self,
        path: &str,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<()>;
    fn set_owned_with_source<T: Serialize>(
        &self,
        path: Arc<str>,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<()>;

    fn save_now(&self) -> Result<()>;
    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>>;

    fn delete_with_source(&self, path: &str, source: Option<Uuid>) -> Result<()>;
    fn delete(&self, path: &str) -> Result<()>;
    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId);
    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T>;

    /// Flushes pending in-memory modifications under the specified prefix to disk.
    ///
    /// # Note
    /// Behavior is backend-specific: transactional engines (such as `RedbStore`) will
    /// selectively commit changes under the given prefix, while monolithic document engines
    /// (such as `TextStore`) will serialize and rewrite the entire file.
    fn flush_prefix(&self, prefix: &str) -> Result<()>;

    //TODO: I'm not sure if this makes sense
    fn is_initialized(&self, namespace: &str) -> Result<bool>;
    fn mark_initialized(&self, namespace: &str) -> Result<()>;
}

pub trait SchemaAwareStore: Store {
    fn run_migrations(&self, mset: MigrationSet) -> Result<MigrationReport>;
}

pub trait StateScope {
    const PREFIX: &'static str;
}

pub trait RpStateSlice<S: Store>: Sized {
    fn load_slice(store: &S) -> Result<Self>;

    fn subscribe_all<F>(&self, callback: F) -> ReactiveScope
    where
        F: Fn() + Send + Sync + 'static;
    fn subscribe_all_external<F>(&self, callback: F) -> ReactiveScope
    where
        F: Fn() + Send + Sync + 'static;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecFormat {
    #[cfg(test)]
    Default,

    #[cfg(feature = "redb")]
    MessagePack,

    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "sqlite")]
    SonicJson,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(feature = "ron")]
    Ron,
}

pub trait MigrationBackend {
    fn format(&self) -> CodecFormat;

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<()>;
    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>>;

    fn get_meta(&self, prefix: &str) -> Result<Option<PrefixMeta>>;
    fn set_meta(&mut self, prefix: &str, meta: &PrefixMeta) -> Result<()>;
    fn get_schema_snapshot(&self, prefix: &str) -> Result<Option<SchemaSnapshot>>;
    fn set_schema_snapshot(&mut self, prefix: &str, snapshot: &SchemaSnapshot) -> Result<()>;
    fn get_migration_log(&self, prefix: &str) -> Result<Option<Vec<AppliedStep>>>;
    fn set_migration_log(&mut self, prefix: &str, log: &[AppliedStep]) -> Result<()>;
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
