use amethystate_macros::amethystate;

#[amethystate]
pub struct DatabaseConfig {
    #[amestate(default = "localhost".to_string())]
    pub host: String,
}

#[amethystate(prefix = "sys")]
pub struct SystemSettings {
    #[amestate(nested)]
    pub db: DatabaseConfig,
}

fn main() {}