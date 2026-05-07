use crate::store::Result;
pub mod context;
pub use context::MigrationContext;

pub trait Migration: Send + Sync {
    fn target_version(&self) -> u32;
    fn run(&self, ctx: &MigrationContext) -> Result<()>;
}

pub trait RawStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<()>;
}

// pub struct Migrator {
//     prefix: String,
//     steps: Vec<Box<dyn Migration>>,
// }
//
// impl Migrator {
//     pub fn new(prefix: impl Into<String>) -> Self {
//         Self {
//             prefix: prefix.into(),
//             steps: Vec::new(),
//         }
//     }
//
//     pub fn add_step(mut self, step: Box<dyn Migration>) -> Self {
//         self.steps.push(step);
//         self.steps.sort_by_key(|m| m.target_version());
//         self
//     }
//
//     pub fn apply(&self, store: &dyn crate::store::Store, target_version: u32, target_hash: u64) -> Result<()> {
//         let current_meta = match store.get_prefix_meta(&self.prefix)? {
//             Some(meta) => meta,
//             None => {
//                 return store.evolve_prefix(&self.prefix, target_version, target_hash);
//             }
//         };
//
//         if target_version <= current_meta.version {
//             return store.evolve_prefix(&self.prefix, target_version, target_hash);
//         }
//
//         let relevant_steps: Vec<&Box<dyn Migration>> = self.steps.iter()
//             .filter(|m| m.target_version() > current_meta.version && m.target_version() <= target_version)
//             .collect();
//
//         if relevant_steps.is_empty() {
//             return store.evolve_prefix(&self.prefix, target_version, target_hash);
//         }
//
//         store.apply_migrations(&self.prefix, target_version, target_hash, &relevant_steps)
//     }
// }
