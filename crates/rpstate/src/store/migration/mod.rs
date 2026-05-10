use crate::store::Result;

pub mod context;
pub mod fields;
pub mod migrate_from;
pub mod registry;
pub mod set;
pub mod types;

use crate::store::error::Error;
use crate::store::migration::fields::RpStateFields;
pub use crate::store::migration::migrate_from::MigrateFrom;
pub use context::MigrationContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppliedStep {
    pub prefix: String,
    pub target_version: u32,
    pub description: Option<String>,
    pub applied_at: u64,
}

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub components: Vec<ComponentResult>,
}

#[derive(Debug)]
pub struct ComponentResult {
    pub prefixes: Vec<String>,
    pub outcome: ComponentOutcome,
    pub nagging: Vec<NaggingRecord>,
}

#[derive(Debug, Clone)]
pub struct NaggingRecord {
    pub prefix: String,
    pub old_hash: u64,
    pub new_hash: u64,
}

#[derive(Debug)]
pub enum ComponentOutcome {
    Committed { steps: Vec<AppliedStep> },
    Skipped,
    Failed { error: Error },
}

impl MigrationReport {
    pub fn has_failures(&self) -> bool {
        self.components
            .iter()
            .any(|c| matches!(c.outcome, ComponentOutcome::Failed { .. }))
    }
}

pub trait Migration: Send + Sync {
    fn target_version(&self) -> u32;
    fn description(&self) -> Option<&str> {
        None
    }
    fn run(&self, ctx: &mut MigrationContext) -> Result<()>;
}

pub trait RawStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<()>;
}

pub struct Migrator {
    pub(crate) steps: Vec<Box<dyn Migration>>,
}

impl Migrator {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn codegen_step<TOld, TNew>(self, version: u32) -> Self
    where
        TOld: RpStateFields + Send + Sync + 'static,
        TNew: RpStateFields + MigrateFrom<TOld> + Send + Sync + 'static,
    {
        let mut desc = format!("v{} codegen:", version);
        for (o, n) in TNew::RENAMES {
            desc.push_str(&format!(" {}->{};", o, n));
        }
        for d in TNew::DROPPED {
            desc.push_str(&format!(" dropped {};", d));
        }

        self.step(version, &desc, move |ctx| {
            let old = TOld::load_struct(ctx)?;

            let new_val = TNew::migrate(old)?;

            for (old_name, _) in TNew::RENAMES {
                ctx.delete(old_name)?;
            }
            for dropped_name in TNew::DROPPED {
                ctx.delete(dropped_name)?;
            }

            new_val.save_struct(ctx)?;

            Ok(())
        })
    }

    pub fn step<F>(mut self, version: u32, description: &str, f: F) -> Self
    where
        F: Fn(&mut MigrationContext) -> Result<()> + Send + Sync + 'static,
    {
        struct ClosureMigration<F> {
            v: u32,
            d: String,
            f: F,
        }
        impl<F> Migration for ClosureMigration<F>
        where
            F: Fn(&mut MigrationContext) -> Result<()> + Send + Sync + 'static,
        {
            fn target_version(&self) -> u32 {
                self.v
            }
            fn description(&self) -> Option<&str> {
                Some(&self.d)
            }
            fn run(&self, ctx: &mut MigrationContext) -> Result<()> {
                (self.f)(ctx)
            }
        }

        self.steps.push(Box::new(ClosureMigration {
            v: version,
            d: description.to_string(),
            f,
        }));
        self.steps.sort_by_key(|s| s.target_version());
        self
    }
}
