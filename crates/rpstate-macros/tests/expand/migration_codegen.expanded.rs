use rpstate_macros::rpstate;
fn build_migrations() -> rpstate::store::migration::Migrator {
    rpstate::store::migration::Migrator::new()
}
fn main() {}
