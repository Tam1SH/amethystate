use crate::MigrationError;
use crate::codec::CodecError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Codec(#[from] CodecError),

    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] crate::store::backend::json::error::JsonStoreError),

    #[cfg(feature = "redb")]
    #[error(transparent)]
    Redb(#[from] crate::store::backend::redb::error::RedbStoreError),

    #[error(transparent)]
    Migration(#[from] MigrationError),
}

pub type Result<T> = std::result::Result<T, Error>;
