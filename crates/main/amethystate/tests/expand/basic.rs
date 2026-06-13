use amethystate_macros::amethystate;

#[amethystate(prefix = "app")]
pub struct AppConfig {
    #[amestate(default = 8080)]
    pub port: u16,

    #[amestate(default = "localhost".to_string(), volatile)]
    pub session_id: String,
}

fn main() {}