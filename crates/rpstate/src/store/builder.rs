use super::Result;
use crate::store::config::StoreConfig;
#[cfg(feature = "json")]
use crate::store::json::JsonStore;
use crate::store::migration::MigrationContext;
use crate::store::migration::Migrator;
use crate::store::migration::registry::MigrationStepEntry;
use crate::store::migration::set::MigrationSet;
#[cfg(feature = "redb")]
use crate::store::redb::RedbStore;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

pub struct StoreBuilder {
    config: StoreConfig,
    migration_set: MigrationSet,
}

#[derive(Default)]
pub struct MigrationBuilder {
    prefixes: HashMap<String, PrefixPlan>,
}

#[derive(Default)]
struct PrefixPlan {
    migrator: Migrator,
    dependencies: BTreeSet<String>,
}

pub struct PrefixMigrationBuilder<'a> {
    builder: &'a mut MigrationBuilder,
    prefix: String,
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

    pub fn migrations(mut self, configure: impl FnOnce(&mut MigrationBuilder)) -> Self {
        let mut builder = MigrationBuilder::default();
        configure(&mut builder);
        self.migration_set = builder.into_set();
        self
    }

    pub fn collect_migrations(self) -> Self {
        self.migrations(|m| {
            m.collect_codegen();
        })
    }

    pub fn build(self) -> Result<crate::DefaultStore> {
        #[cfg(feature = "redb")]
        return self.build_redb();

        #[cfg(all(feature = "json", not(feature = "redb")))]
        return self.build_json();

        #[cfg(not(any(feature = "json", feature = "redb")))]
        compile_error!("No storage backend enabled. Enable 'json' or 'redb' feature.");
    }
}

impl MigrationBuilder {
    pub fn collect_codegen(&mut self) -> &mut Self {
        let mut groups: HashMap<&'static str, Vec<&'static MigrationStepEntry>> = HashMap::new();

        for entry in inventory::iter::<MigrationStepEntry> {
            groups.entry(entry.prefix).or_default().push(entry);
        }

        for (prefix, steps) in groups {
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
                self.for_prefix(prefix)
                    .step(step.target_version, step.description, step.run);
            }

            for dep in merged_deps {
                self.for_prefix(prefix).depends_on(dep);
            }
        }

        self
    }

    pub fn for_prefix(&mut self, prefix: impl Into<String>) -> PrefixMigrationBuilder<'_> {
        PrefixMigrationBuilder {
            builder: self,
            prefix: prefix.into(),
        }
    }

    fn prefix_plan(&mut self, prefix: &str) -> &mut PrefixPlan {
        self.prefixes.entry(prefix.to_string()).or_default()
    }

    fn into_set(self) -> MigrationSet {
        let mut set = MigrationSet::default();

        let mut prefixes = self.prefixes.into_iter().collect::<Vec<_>>();
        prefixes.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (prefix, plan) in prefixes {
            let dependencies = plan.dependencies.into_iter().collect::<Vec<_>>();
            let dependency_refs = dependencies.iter().map(String::as_str).collect::<Vec<_>>();
            set = set.add(prefix, plan.migrator, &dependency_refs);
        }

        set
    }
}

impl PrefixMigrationBuilder<'_> {
    pub fn depends_on(&mut self, dependency: impl Into<String>) -> &mut Self {
        let dependency = dependency.into();
        self.builder
            .prefix_plan(&self.prefix)
            .dependencies
            .insert(dependency);
        self
    }

    pub fn step<F>(&mut self, target_version: u32, description: &str, run: F) -> &mut Self
    where
        F: Fn(&mut MigrationContext) -> Result<()> + Send + Sync + 'static,
    {
        let plan = self.builder.prefix_plan(&self.prefix);
        let migrator = std::mem::take(&mut plan.migrator);
        plan.migrator = migrator.step(target_version, description, run);
        self
    }
}
