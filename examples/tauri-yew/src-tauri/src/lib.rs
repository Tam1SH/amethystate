use rpstate::StoreBuilder;
use rpstate::{rpstate, ReactiveMap};
use shared::ProxyProfile;

#[rpstate(prefix = "settings")]
pub struct AppSettings {
    #[state(default = "Guest".to_string())]
    pub username: String,

    #[state(default = 0)]
    pub counter: i32,

    #[state(nested)]
    pub theme: Theme,

    #[state(default = Default::default())]
    pub proxy: ProxyProfile,

    #[state(default = {
        "HTTP_PROXY": "http://127.0.0.1:8080".to_string(),
        "NO_PROXY": "localhost".to_string()
    })]
    pub env: ReactiveMap<String, String>,
}

#[rpstate]
pub struct Theme {
    #[state(default = "light".to_string())]
    pub mode: String,

    #[state(default = "#ffffff".to_string())]
    pub background: String,

    #[state(default = "#000000".to_string())]
    pub foreground: String,
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = StoreBuilder::new("./rpstate_settings.redb")
        .build()
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_rpstate::init())
        .manage(store)
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}