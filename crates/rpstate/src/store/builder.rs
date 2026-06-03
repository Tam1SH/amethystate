use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::migration::builder::MigrationBuilder;
use crate::migration::set::MigrationSet;

use crate::store::config::StoreConfig;
use crate::{DefaultStore, MigrationReport, Result};

pub struct NoMigrations;
pub struct WithMigrations;

#[cfg(feature = "redb")]
const FILE_EXTENSION: &str = "redb";

#[cfg(all(feature = "json", not(feature = "redb")))]
const FILE_EXTENSION: &str = "json";

pub struct StoreBuilder<M = NoMigrations> {
    config: StoreConfig,
    migration_set: MigrationSet,
    _state: PhantomData<M>,
}

impl StoreBuilder<NoMigrations> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let mut path: PathBuf = path.into();

        if path.extension().is_none() {
            path.set_extension(FILE_EXTENSION);
        }

        Self {
            config: StoreConfig::new(path),
            migration_set: MigrationSet::default(),
            _state: PhantomData,
        }
    }

    pub fn for_app(app_name: impl AsRef<str>) -> std::io::Result<Self> {
        let app_name = app_name.as_ref();

        let proj_dirs = directories::ProjectDirs::from("", "", app_name).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to resolve system application directories",
            )
        })?;

        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;

        let path = data_dir.join(app_name);

        Ok(Self::new(path))
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

    #[cfg(not(target_arch = "wasm32"))]
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
    pub fn build(self) -> Result<DefaultStore> {
        #[cfg(feature = "redb")]
        {
            let (store, _) =
                crate::store::backend::redb::RedbStore::open(self.config, MigrationSet::default())?;
            Ok(store)
        }

        #[cfg(all(feature = "json", not(feature = "redb")))]
        {
            Ok(crate::store::backend::json::JsonStore::open(self.config, Default::default())?.0)
        }
    }
}

impl StoreBuilder<WithMigrations> {
    pub fn build(self) -> Result<(DefaultStore, MigrationReport)> {
        #[cfg(feature = "redb")]
        {
            let (store, report) =
                crate::store::backend::redb::RedbStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }

        #[cfg(all(feature = "json", not(feature = "redb")))]
        {
            let (store, report) =
                crate::store::backend::json::JsonStore::open(self.config, Default::default())?;
            Ok((store, report))
        }
    }
}
