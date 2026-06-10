use crate::store::backend;

#[cfg(backend = "json")]
pub use backend::text::JsonStore;

#[cfg(backend = "sqlite")]
pub use backend::sqlite::SqliteStore;

#[cfg(backend = "redb")]
pub use backend::redb::RedbStore;

#[cfg(backend = "toml")]
pub use backend::text::TomlStore;

#[cfg(backend = "ron")]
pub use backend::text::RonStore;

#[cfg(backend = "redb")]
pub type DefaultStore = RedbStore;

#[cfg(backend = "sqlite")]
pub type DefaultStore = SqliteStore;

#[cfg(backend = "json")]
pub type DefaultStore = JsonStore;

#[cfg(backend = "toml")]
pub type DefaultStore = TomlStore;

#[cfg(backend = "ron")]
pub type DefaultStore = RonStore;
