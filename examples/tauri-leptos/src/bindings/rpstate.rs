// GENERATED AUTOMATICALLY. DO NOT EDIT.
use rpstate_arena::rpstate_framework_arena;

#[rpstate_framework_arena]
#[::rpstate::rpstate(target = "tauri-wasm")]
pub struct Theme {
    pub mode: String,
    pub background: String,
    pub foreground: String,
} 

#[rpstate_framework_arena]
#[::rpstate::rpstate(prefix = "settings", target = "tauri-wasm")]
pub struct AppSettings {
    pub username: String,
    pub counter: i32,
    #[state(nested)]
    pub theme: Theme,
    pub proxy: ProxyProfile,
    pub env: ReactiveMap<String, String>,
}
