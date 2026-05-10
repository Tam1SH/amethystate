use crate::store::migration::{MigrationContext, Migrator};
use crate::store::StateScope;

pub trait HasMigrations: StateScope {
    const MIGRATION_DEPS: &'static [&'static str];
    fn migrations() -> Migrator;
}

#[derive(Clone)]
pub struct MigrationStepEntry {
    pub prefix: &'static str,
    pub target_version: u32,
    pub description: &'static str,
    pub dependencies: &'static [&'static str],
    pub run: fn(&mut MigrationContext) -> crate::store::Result<()>,
}

#[cfg(not(target_arch = "wasm32"))]
inventory::collect!(MigrationStepEntry);
