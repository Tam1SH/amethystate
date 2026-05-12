use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::migration::builder::MigrationBuilder;
use crate::migration::set::MigrationSet;
#[cfg(feature = "redb")]
use crate::store::backend::redb::RedbStore;
use crate::store::config::StoreConfig;
use crate::{DefaultStore, MigrationReport, Result};

pub struct NoMigrations;
pub struct WithMigrations;

pub struct StoreBuilder<M = NoMigrations> {
    config: StoreConfig,
    migration_set: MigrationSet,
    _state: PhantomData<M>,
}

impl StoreBuilder<NoMigrations> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            config: StoreConfig::new(path),
            migration_set: MigrationSet::default(),
            _state: PhantomData,
        }
    }
}

impl<M> StoreBuilder<M> {
    pub fn debounce(mut self, ms: u64) -> Self {
        self.config.save_debounce = Duration::from_millis(ms);
        self
    }

    pub fn watch_interval(mut self, ms: u64) -> Self {
        self.config.watch_interval = Duration::from_millis(ms);
        self
    }

    pub fn migrations(
        self,
        configure: impl FnOnce(&mut MigrationBuilder),
    ) -> StoreBuilder<WithMigrations> {
        let mut builder = MigrationBuilder::default();
        configure(&mut builder);
        StoreBuilder {
            config: self.config,
            migration_set: builder.into_set(),
            _state: PhantomData,
        }
    }

    pub fn collect_migrations(self) -> StoreBuilder<WithMigrations> {
        let mut builder = MigrationBuilder::default();
        builder.collect_codegen();
        StoreBuilder {
            config: self.config,
            migration_set: builder.into_set(),
            _state: PhantomData,
        }
    }
}

impl StoreBuilder<NoMigrations> {
    pub fn build(self) -> Result<Arc<DefaultStore>> {
        #[cfg(feature = "redb")]
        {
            let (store, _) = RedbStore::open(self.config, MigrationSet::default())?;
            Ok(Arc::new(store))
        }

        #[cfg(all(feature = "json", not(feature = "redb")))]
        {
            Ok(Arc::new(JsonStore::open(self.config))?)
        }
    }
}

impl StoreBuilder<WithMigrations> {
    pub fn build(self) -> Result<(Arc<DefaultStore>, MigrationReport)> {
        #[cfg(feature = "redb")]
        {
            let (store, report) = RedbStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((Arc::new(store), report))
        }

        #[cfg(all(feature = "json", not(feature = "redb")))]
        {
            let store = JsonStore::open(self.config)?;
            Ok((Arc::new(store), MigrationReport::default()))
        }
    }
}
