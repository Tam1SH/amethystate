use crate::codec::CodecError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqliteStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Codec(#[from] CodecError),
}
