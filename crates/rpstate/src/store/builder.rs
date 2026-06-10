use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

use crate::migration::builder::MigrationBuilder;
use crate::migration::set::MigrationSet;

use crate::store::config::StoreConfig;
use crate::{DefaultStore, MigrationReport, Result};

pub struct NoMigrations;
pub struct WithMigrations;

#[cfg(backend = "redb")]
const FILE_EXTENSION: &str = "redb";

#[cfg(backend = "json")]
const FILE_EXTENSION: &str = "json";

#[cfg(backend = "toml")]
const FILE_EXTENSION: &str = "toml";

#[cfg(backend = "ron")]
const FILE_EXTENSION: &str = "ron";

#[cfg(backend = "sqlite")]
const FILE_EXTENSION: &str = "db";



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

    /// Returns a [`StoreBuilder`] configured to use the platform-appropriate configuration
    /// directory for the given application name.
    ///
    /// The directory strategy depends on the active feature flag:
    /// - `confy-compat-0-6`: uses the [`directories`] crate (legacy `confy` v0.6 behavior)
    /// - default: uses the [`etcetera`] crate (XDG / native OS strategy)
    pub fn for_app(
        app_name: impl AsRef<str>,
        config_name: impl AsRef<str>,
    ) -> std::io::Result<Self> {
        #[cfg(feature = "confy-compat-0-6")]
        {
            Self::for_app_v06(app_name, config_name)
        }
        #[cfg(not(feature = "confy-compat-0-6"))]
        {
            Self::for_app_v2(app_name, config_name)
        }
    }

    pub fn for_app_v2(
        app_name: impl AsRef<str>,
        config_name: impl AsRef<str>,
    ) -> std::io::Result<Self> {
        use etcetera::{choose_app_strategy, AppStrategy, AppStrategyArgs};

        let project = choose_app_strategy(AppStrategyArgs {
            top_level_domain: "rs".to_string(),
            author: "".to_string(),
            app_name: app_name.as_ref().to_string(),
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;

        let mut path = project.config_dir();
        path.push(config_name.as_ref());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self::new(path))
    }

    #[cfg(feature = "confy-compat-0-6")]
    pub fn for_app_v06(
        app_name: impl AsRef<str>,
        config_name: impl AsRef<str>,
    ) -> std::io::Result<Self> {
        use directories::ProjectDirs;

        let project = ProjectDirs::from("rs", "", app_name.as_ref()).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to resolve system application directories",
            )
        })?;

        let mut path = project.config_dir().to_path_buf();
        path.push(config_name.as_ref());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

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
        #[cfg(backend = "redb")]
        {
            let (store, _) =
                crate::store::backend::redb::RedbStore::open(self.config, Default::default())?;
            Ok(store)
        }
        #[cfg(backend = "sqlite")]
        {
            let (store, _) =
                crate::store::backend::sqlite::SqliteStore::open(self.config, Default::default())?;
            Ok(store)
        }

        #[cfg(backend = "ron")]
        {
            let (store, _) =
                crate::store::backend::text::RonStore::open(self.config, Default::default())?;
            Ok(store)
        }

        #[cfg(backend = "json")]
        {
            let (store, _) =
                crate::store::backend::text::JsonStore::open(self.config, Default::default())?;
            Ok(store)
        }

        #[cfg(backend = "toml")]
        {
            let (store, _) =
                crate::store::backend::text::TomlStore::open(self.config, Default::default())?;
            Ok(store)
        }
    }
}

impl StoreBuilder<WithMigrations> {
    pub fn build(self) -> Result<(DefaultStore, MigrationReport)> {
        #[cfg(backend = "redb")]
        {
            let (store, report) =
                crate::store::backend::redb::RedbStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }

        #[cfg(backend = "sqlite")]
        {
            let (store, report) =
                crate::store::backend::sqlite::SqliteStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }

        #[cfg(backend = "json")]
        {
            let (store, report) =
                crate::store::backend::text::JsonStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }

        #[cfg(backend = "ron")]
        {
            let (store, report) =
                crate::store::backend::text::RonStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }

        #[cfg(backend = "toml")]
        {
            let (store, report) =
                crate::store::backend::text::TomlStore::open(self.config, self.migration_set)?;
            report.log_to_tracing();
            Ok((store, report))
        }
    }
}
