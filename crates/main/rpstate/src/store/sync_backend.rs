use crate::Store;
use crate::error::Error;
use rpstate_core::RpBackend;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) struct StoreBackend<S> {
    pub(crate) store: S,
}

impl<S> StoreBackend<S> {
    pub(crate) fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> RpBackend for StoreBackend<S>
where
    S: Store,
{
    type Error = Error;
    type Raw = Vec<u8>;

    fn get<T>(&self, path: &str) -> Result<Option<T>, Self::Error>
    where
        T: DeserializeOwned,
    {
        self.store.get(path)
    }

    fn set_with_source<T: Serialize>(
        &self,
        path: &str,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error> {
        self.store.set_with_source(path, value, source)
    }

    fn set_owned_with_source<T: Serialize>(
        &self,
        path: Arc<str>,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error> {
        self.store.set_owned_with_source(path, value, source)
    }

    fn set<T>(&self, path: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.store.set(path, value)
    }

    fn delete(&self, path: &str) -> Result<(), Self::Error> {
        self.store.delete(path)
    }

    fn delete_with_source(&self, path: &str, source: Option<Uuid>) -> Result<(), Self::Error> {
        self.store.delete_with_source(path, source)
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Self::Raw)>, Self::Error> {
        self.store.scan_prefix(prefix)
    }

    fn decode<T>(&self, raw: &Self::Raw) -> Result<T, Self::Error>
    where
        T: DeserializeOwned + Default,
    {
        self.store.decode(raw)
    }

    fn intercepted(&self) -> Self::Error {
        Error::Intercepted
    }

    fn key_not_found(&self, key: String) -> Self::Error {
        Error::KeyNotFound(key)
    }
}
