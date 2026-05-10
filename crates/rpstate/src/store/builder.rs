use super::Result;
use crate::store::config::StoreConfig;
#[cfg(feature = "json")]
use crate::store::json::JsonStore;
use crate::store::migration::registry::MigrationStepEntry;
use crate::store::migration::set::MigrationSet;
use crate::store::migration::Migrator;
#[cfg(feature = "redb")]
use crate::store::redb::RedbStore;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

pub struct StoreBuilder {
    config: StoreConfig,
    migration_set: MigrationSet,
}

impl StoreBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            config: StoreConfig::new(path),
            migration_set: MigrationSet::default(),
        }
    }

    pub fn debounce(mut self, ms: u64) -> Self {
        self.config.save_debounce = Duration::from_millis(ms);
        self
    }

    #[cfg(feature = "json")]
    pub fn build_json(self) -> Result<JsonStore> {
        JsonStore::open(self.config)
    }

    #[cfg(feature = "redb")]
    pub fn build_redb(self) -> Result<RedbStore> {
        RedbStore::open(self.config, self.migration_set)
    }

    pub fn collect_migrations(mut self) -> Self {
        let mut groups: HashMap<&'static str, Vec<&'static MigrationStepEntry>> = HashMap::new();

        for entry in inventory::iter::<MigrationStepEntry> {
            groups.entry(entry.prefix).or_default().push(entry);
        }

        for (prefix, steps) in groups {
            let mut migrator = Migrator::new();

            let mut merged_deps: Vec<&'static str> = steps
                .iter()
                .flat_map(|s| s.dependencies.iter().copied())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            merged_deps.sort();

            #[cfg(debug_assertions)]
            {
                let first_deps = steps.first().map(|s| s.dependencies).unwrap_or(&[]);
                if steps.iter().any(|s| s.dependencies != first_deps) {
                    tracing::warn!(
                        prefix,
                        "Migration steps for prefix '{}' have inconsistent dependencies — \
                     using union. Consider aligning deps across all versions.",
                        prefix
                    );
                }
            }

            for step in &steps {
                migrator = migrator.step(step.target_version, step.description, step.run);
            }

            self.migration_set = self.migration_set.add(prefix, migrator, &merged_deps);
        }

        self
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
