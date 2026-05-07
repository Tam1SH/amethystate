use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Path error: {0}")]
    InvalidPath(String),

    #[error("Concurrency error: lock poisoned")]
    Poisoned,

    #[error("Migration required for [{prefix}]: DB v{db_version}, Code v{code_version}")]
    MigrationRequired {
        prefix: String,
        db_version: u32,
        code_version: u32,
    },

    #[error("Downgrade detected for [{prefix}]: DB v{db_version}, Code v{code_version}")]
    Downgrade {
        prefix: String,
        db_version: u32,
        code_version: u32,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
