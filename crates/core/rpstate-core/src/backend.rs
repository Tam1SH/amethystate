use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

//TODO: bullshit
pub trait RpBackend {
    type Error;
    type Raw;

    fn get<T>(&self, path: &str) -> Result<Option<T>, Self::Error>
    where
        T: DeserializeOwned;

    fn set_with_source<T: Serialize>(
        &self,
        path: &str,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error>;
    fn set_owned_with_source<T: Serialize>(
        &self,
        path: Arc<str>,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error>;

    fn set<T>(&self, path: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize;

    fn delete(&self, path: &str) -> Result<(), Self::Error>;

    fn delete_with_source(&self, path: &str, source: Option<Uuid>) -> Result<(), Self::Error>;

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Self::Raw)>, Self::Error>;

    fn decode<T>(&self, raw: &Self::Raw) -> Result<T, Self::Error>
    where
        T: DeserializeOwned + Default;

    fn intercepted(&self) -> Self::Error;

    fn key_not_found(&self, key: String) -> Self::Error;
}
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait RpBackendAsync {
    type Error;
    type Raw;

    async fn get<T>(&self, path: &str) -> Result<Option<T>, Self::Error>
    where
        T: DeserializeOwned;

    async fn set<T>(&self, path: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize;
    async fn set_with_source<T: Serialize>(
        &self,
        path: &str,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error>;
    async fn set_owned_with_source<T: Serialize>(
        &self,
        path: Arc<str>,
        value: &T,
        source: Option<Uuid>,
    ) -> Result<(), Self::Error>;

    async fn delete(&self, path: &str) -> Result<(), Self::Error>;
    async fn delete_with_source(&self, path: &str, source: Option<Uuid>) -> Result<(), Self::Error>;

    async fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Self::Raw)>, Self::Error>;

    fn decode<T>(&self, raw: &Self::Raw) -> Result<T, Self::Error>
    where
        T: DeserializeOwned + Default;

    fn intercepted(&self) -> Self::Error;

    fn key_not_found(&self, key: String) -> Self::Error;
}
