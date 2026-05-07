use rpstate_macros::rpstate;

#[rpstate]
pub struct DatabaseConfig {
    #[state(default = "localhost".to_string())]
    pub host: String,
}

#[rpstate(prefix = "sys")]
pub struct SystemSettings {
    #[state(nested)]
    pub db: DatabaseConfig,
}

fn main() {}