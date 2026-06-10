use crate::MigrationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "text")]
    #[error(transparent)]
    TextStore(#[from] crate::store::backend::text::error::TextStoreError),

    #[cfg(feature = "redb")]
    #[error(transparent)]
    RedbStore(#[from] crate::store::backend::redb::error::RedbStoreError),

    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Sqlite(#[from] crate::store::backend::sqlite::error::SqliteStoreError),

    #[error(transparent)]
    Migration(#[from] MigrationError),

    //TODO: remove
    #[error("Change intercepted")]
    Intercepted,

    //TODO: remove
    #[error("Key not found in ReactiveMap: {0}")]
    KeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
