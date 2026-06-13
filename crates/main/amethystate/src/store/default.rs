use crate::store::backend;

#[cfg(feature = "json")]
pub use backend::text::JsonStore;

#[cfg(feature = "sqlite")]
pub use backend::sqlite::SqliteStore;

#[cfg(feature = "redb")]
pub use backend::redb::RedbStore;

#[cfg(feature = "toml")]
pub use backend::text::TomlStore;

#[cfg(feature = "ron")]
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
