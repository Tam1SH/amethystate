pub mod commands;

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("rpstate")
        .invoke_handler(tauri::generate_handler![
            commands::rpstate_get,
            commands::rpstate_set,
            commands::rpstate_subscribe,
            commands::rpstate_unsubscribe,
            commands::rpstate_get_prefix,
            commands::rpstate_flush,
            commands::rpstate_delete,
        ])
        .setup(|app, _api| {
            app.manage(commands::PluginState {
                subscriptions: std::sync::Mutex::new(std::collections::HashMap::new()),
            });
            Ok(())
        })
        .build()
}
