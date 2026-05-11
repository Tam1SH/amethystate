use crate::store::codec::CodecError;
use thiserror::Error;

pub(super) type RedbResult<T> = std::result::Result<T, RedbStoreError>;

#[derive(Error, Debug)]
pub enum RedbStoreError {
    #[error("redb commit error: {0}")]
    Commit(#[from] redb::CommitError),

    #[error("redb database error: {0}")]
    Database(#[from] redb::DatabaseError),

    #[error("redb storage error: {0}")]
    Storage(#[from] redb::StorageError),

    #[error("redb table error: {0}")]
    Table(#[from] redb::TableError),

    #[error("redb transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("redb store lock poisoned")]
    Poisoned,

    #[error(transparent)]
    Codec(#[from] CodecError),
}
