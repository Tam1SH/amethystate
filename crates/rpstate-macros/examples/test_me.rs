use rpstate_macros::rpstate;

#[rpstate(prefix = "app")]
pub struct AppConfig {
    #[state(default = 8080)]
    pub port: u16,

    #[state(default = "localhost".to_string(), volatile)]
    pub session_id: String,
}

fn main() {}
