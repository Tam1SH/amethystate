use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    #[error(
        "Migration chain gap for [{prefix}]: reached v{reached_version}, expected v{expected_version}"
    )]
    Gap {
        prefix: String,
        reached_version: u32,
        expected_version: u32,
    },

    #[error("Migration cycle detected at prefix: {0}")]
    Cycle(String),

    #[error("Migration error: {0}")]
    Custom(String),

    #[error("Downgrade detected for [{prefix}]: DB v{db_version}, Code v{code_version}")]
    Downgrade {
        prefix: String,
        db_version: u32,
        code_version: u32,
    },
}
