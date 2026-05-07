use super::Result;
use crate::store::config::StoreConfig;
#[cfg(feature = "json")]
use crate::store::json::JsonStore;
#[cfg(feature = "redb")]
use crate::store::redb::RedbStore;

pub struct StoreBuilder {
    config: StoreConfig,
}

impl StoreBuilder {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            config: StoreConfig::new(path),
        }
    }

    pub fn debounce(mut self, ms: u64) -> Self {
        self.config.save_debounce = std::time::Duration::from_millis(ms);
        self
    }

    #[cfg(feature = "json")]
    pub fn build_json(self) -> Result<JsonStore> {
        JsonStore::open(self.config)
    }

    #[cfg(feature = "redb")]
    pub fn build_redb(self) -> Result<RedbStore> {
        RedbStore::open(self.config)
    }

    pub fn build(self) -> Result<crate::DefaultStore> {
        #[cfg(all(feature = "redb"))]
        return self.build_redb();

        #[cfg(all(feature = "json", not(feature = "redb")))]
        return self.build_json();

        #[cfg(not(any(feature = "json", feature = "redb")))]
        compile_error!("No storage backend enabled. Enable 'json' or 'redb' feature.");
    }
}
