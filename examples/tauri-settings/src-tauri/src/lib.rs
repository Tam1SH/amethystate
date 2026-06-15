use amethystate::{amethystate, AmeType, ReactiveMap, StoreBuilder};
use serde::{Deserialize, Serialize};

pub mod state;


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, AmeType)]
pub struct ProxyProfile {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for ProxyProfile {
    fn default() -> Self {
        Self {
            name: "Default Proxy".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            enabled: false,
        }
    }
}

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

    let app_dir = std::env::current_dir().unwrap_or_default();
    let db_path = app_dir.join("amethystate_settings.redb");
    let store = StoreBuilder::new(&db_path)
        .build()
        .expect("Failed to initialize amethystate store");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_amethystate::init(store))
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}