use crate::MigrationError;
use crate::codec::CodecError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    //TODO: replace to store errs
    #[error(transparent)]
    Codec(#[from] CodecError),

    #[cfg(feature = "text")]
    #[error(transparent)]
    TextStore(#[from] crate::store::backend::text::error::TextStoreError),

    #[cfg(feature = "redb")]
    #[error(transparent)]
    RedbStore(#[from] crate::store::backend::redb::error::RedbStoreError),

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
