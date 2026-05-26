use rpstate::{DefaultStore, StoreBuilder};
use std::sync::Arc;

pub mod state;
pub use state::AppSettings;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    {
        let _ = tauri_plugin_rpstate::codegen::export("../src/bindings/rpstate.ts");
    }

    let app_dir = std::env::current_dir().unwrap_or_default();
    let db_path = app_dir.join("rpstate_settings.redb");
    let store = StoreBuilder::new(&db_path)
        .build()
        .expect("Failed to initialize rpstate store");

    let _settings = AppSettings::new(&store).expect("Failed to load AppSettings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_rpstate::init())
        .manage(store)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}