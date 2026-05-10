use rpstate_macros::rpstate;

#[rpstate(prefix = "net", migrations)]
pub struct NetworkConfig {
    #[state(default = "127.0.0.1".to_string())]
    pub host: String,

    #[state(default = 8080)]
    pub port: u16,

    #[state(volatile, default = false)]
    pub connected: bool,
}

fn build_migrations() -> rpstate::store::migration::Migrator {
    rpstate::store::migration::Migrator::new()
}

fn main() {}
