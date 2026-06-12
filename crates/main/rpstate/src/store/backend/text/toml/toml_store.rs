use super::toml_doc::TomlDocument;
use crate::migration::set::MigrationSet;
use crate::store::backend::text::store::TextStore;
use crate::store::config::StoreConfig;
use crate::store::{Store, StoreCallback, SubscriptionId, SubscriptionKind};
use crate::{MigrationReport, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TomlStore(pub TextStore<TomlDocument>);

impl TomlStore {
    pub fn open(
        config: StoreConfig,
        migration_set: MigrationSet,
    ) -> Result<(Self, MigrationReport)> {
        let (store, report) = TextStore::open(config, migration_set)?;
        Ok((TomlStore(store), report))
    }
}

impl Store for TomlStore {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        self.0.get(path)
    }

    fn set<T: Serialize>(&self, path: &str, value: &T) -> Result<()> {
        self.0.set(path, value)
    }

    fn set_with_source<T: Serialize>(&self, path: &str, value: &T, source: Option<Uuid>) -> Result<()> {
        self.0.set_with_source(path, value, source)
    }

    fn set_owned_with_source<T: Serialize>(&self, path: Arc<str>, value: &T, source: Option<Uuid>) -> Result<()> {
        self.0.set_owned_with_source(path, value, source)
    }

    fn save_now(&self) -> Result<()> {
        self.0.save_now()
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        self.0.scan_prefix(prefix)
    }

    fn delete_with_source(&self, path: &str, source: Option<Uuid>) -> Result<()> {
        self.0.delete_with_source(path, source)
    }

    fn delete(&self, path: &str) -> Result<()> {
        self.0.delete(path)
    }

    fn subscribe(&self, kind: SubscriptionKind, callback: StoreCallback) -> SubscriptionId {
        self.0.subscribe(kind, callback)
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.0.unsubscribe(id)
    }

    fn decode<T: DeserializeOwned + Default>(&self, bytes: &[u8]) -> Result<T> {
        self.0.decode(bytes)
    }

    fn flush_prefix(&self, prefix: &str) -> Result<()> {
        self.0.flush_prefix(prefix)
    }

    fn is_initialized(&self, namespace: &str) -> Result<bool> {
        self.0.is_initialized(namespace)
    }

    fn mark_initialized(&self, namespace: &str) -> Result<()> {
        self.0.mark_initialized(namespace)
    }
}

crate::define_store_test_suite!(
    TomlStore,
    "toml",
    "[rpstate]\nwatch_interval_ms = 50\n\n[ui.theme]\ndark = false",
    "[rpstate]\nwatch_interval_ms = 50\n\n[ui.theme]\ndark = true",
    "[rpstate]\nwatch_interval_ms = 50\n\n[ui]\ntheme = {}"
);
