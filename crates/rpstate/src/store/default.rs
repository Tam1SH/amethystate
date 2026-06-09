use crate::store::backend;

#[cfg(backend = "json")]
pub use backend::text::JsonStore;

#[cfg(backend = "redb")]
pub use backend::redb::RedbStore;

#[cfg(backend = "toml")]
pub use backend::text::TomlStore;

#[cfg(backend = "redb")]
pub type DefaultStore = RedbStore;

#[cfg(backend = "json")]
pub type DefaultStore = JsonStore;

#[cfg(backend = "toml")]
pub type DefaultStore = TomlStore;
