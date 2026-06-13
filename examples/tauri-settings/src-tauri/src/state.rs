use amethystate::amethystate;

#[amethystate(prefix = "settings", version = 1)]
pub struct AppSettings {
    #[amestate(default = "Guest".to_string())]
    pub username: String,

    #[amestate(default = 0)]
    pub counter: u32,

    #[amestate(default = "light".to_string())]
    pub theme: String,
}