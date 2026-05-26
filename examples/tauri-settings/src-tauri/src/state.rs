use rpstate::rpstate;

#[rpstate(prefix = "settings", version = 1)]
pub struct AppSettings {
    #[state(default = "Guest".to_string())]
    pub username: String,

    #[state(default = 0)]
    pub counter: u32,

    #[state(default = "light".to_string())]
    pub theme: String,
}