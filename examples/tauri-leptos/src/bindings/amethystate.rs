// GENERATED AUTOMATICALLY. DO NOT EDIT.
use amethystate_arena::amethystate_framework_arena;

#[amethystate_framework_arena]
#[::amethystate::amethystate(target = "tauri-wasm")]
pub struct Theme {
    pub mode: String,
    pub background: String,
    pub foreground: String,
}

#[amethystate_framework_arena]
#[::amethystate::amethystate(prefix = "settings", target = "tauri-wasm")]
pub struct AppSettings {
    pub username: String,
    pub counter: i32,
    #[amestate(nested)]
    pub theme: Theme,
    pub proxy: ProxyProfile,
    pub env: ReactiveMap<String, String>,
}
