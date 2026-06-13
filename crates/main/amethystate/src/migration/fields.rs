use crate::{MigrationContext, Result};

pub struct FieldDescriptor {
    pub name: &'static str,
    pub type_hash: u32,
    pub type_name: &'static str,
}

pub trait AmeStateFields: Sized {
    const FIELDS: &'static [FieldDescriptor];
    const VERSION: u32;
    const SCHEMA_HASH: u32;
    const PARENT_PREFIX: &'static str;
    const MIGRATION_DEPS: &'static [&'static str];

    fn load_struct(ctx: &mut MigrationContext) -> Result<Self>;

    fn save_struct(&self, ctx: &mut MigrationContext) -> Result<()>;
}
