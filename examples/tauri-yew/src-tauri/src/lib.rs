use amethystate::StoreBuilder;
use amethystate::{amethystate, ReactiveMap};
use shared::ProxyProfile;

#[amethystate(prefix = "settings")]
pub struct AppSettings {
    #[amestate(default = "Guest".to_string())]
    pub username: String,

    #[amestate(default = 0)]
    pub counter: i32,

    #[amestate(nested)]
    pub theme: Theme,

    #[amestate(default = Default::default())]
    pub proxy: ProxyProfile,

    #[amestate(default = {
        "HTTP_PROXY": "http://127.0.0.1:8080".to_string(),
        "NO_PROXY": "localhost".to_string()
    })]
    pub env: ReactiveMap<String, String>,
}

#[amethystate]
pub struct Theme {
    #[amestate(default = "light".to_string())]
    pub mode: String,

    #[amestate(default = "#ffffff".to_string())]
    pub background: String,

    #[amestate(default = "#000000".to_string())]
    pub foreground: String,
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = StoreBuilder::new("./amethystate_settings.redb")
        .build()
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_amethystate::init())
        .manage(store)
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}