use crate::codec::CodecError;
use thiserror::Error;

pub(super) type JsonResult<T> = Result<T, JsonStoreError>;

#[derive(Error, Debug)]
pub enum JsonStoreError {
    #[error("JSON store IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Codec(#[from] CodecError),

    #[error("JSON store lock poisoned")]
    Poisoned,

    #[error("JSON root must be an object")]
    RootMustBeObject,

    #[error("JSON patch must be an object")]
    PatchMustBeObject,

    #[error("JSON path cannot be empty")]
    EmptyPath,

    #[error("JSON path target is not an object")]
    TargetNotObject,

    #[error("JSON path segment '{0}' not found")]
    PathSegmentMissing(String),

    #[error("JSON path segment '{0}' is not an object")]
    PathSegmentNotObject(String),
}
