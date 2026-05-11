use crate::store::migration::MigrationContext;

pub trait MigrateFrom<TOld>: Sized {
    const RENAMES: &'static [(&'static str, &'static str)] = &[];
    const CONVERTS: &'static [(&'static str, u64, u64)] = &[];

    fn migrate(old: TOld, ctx: &mut MigrationContext) -> crate::store::Result<Self>;
}
