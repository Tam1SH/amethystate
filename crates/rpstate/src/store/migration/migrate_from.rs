pub trait MigrateFrom<TOld>: Sized {
    const RENAMES: &'static [(&'static str, &'static str)] = &[];
    const CONVERTS: &'static [(&'static str, u64, u64)] = &[];

    fn migrate(old: TOld) -> crate::store::Result<Self>;
}
