// GENERATED AUTOMATICALLY. DO NOT EDIT.
use tauri_plugin_rpstate::client::field::Field;
use tauri_plugin_rpstate::client::map::ReactiveMap;
use tauri_plugin_rpstate::client::{invoke_get_prefix, invoke_flush};
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeFields {
    pub mode: Field<String>,
    pub background: Field<String>,
    pub foreground: Field<String>,
}

#[allow(dead_code)]
impl ThemeFields {
    pub fn new(prefix: &str, initial: &std::collections::HashMap<String, tauri_plugin_rpstate::serde_json::Value>) -> Self {
        Self {
            mode: Field::new(format!("{}.mode", prefix), initial.get(&format!("{}.mode", prefix)).and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<String>((*v).clone()).ok()).unwrap_or_default()),
            background: Field::new(format!("{}.background", prefix), initial.get(&format!("{}.background", prefix)).and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<String>((*v).clone()).ok()).unwrap_or_default()),
            foreground: Field::new(format!("{}.foreground", prefix), initial.get(&format!("{}.foreground", prefix)).and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<String>((*v).clone()).ok()).unwrap_or_default()),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub username: Field<String>,
    pub counter: Field<i32>,
    pub theme: ThemeFields,
    pub proxy: Field<ProxyProfile>,
    pub env: ReactiveMap<String, String>,
}

#[allow(dead_code)]
impl AppSettings {
    pub async fn load() -> Result<Self, String> {
        let initial = invoke_get_prefix("settings").await?;
        Ok(Self {
            username: Field::new("settings.username", initial.get("settings.username").and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<String>((*v).clone()).ok()).unwrap_or_default()),
            counter: Field::new("settings.counter", initial.get("settings.counter").and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<i32>((*v).clone()).ok()).unwrap_or_default()),
            theme: ThemeFields::new("settings.theme", &initial),
            proxy: Field::new("settings.proxy", initial.get("settings.proxy").and_then(|v| tauri_plugin_rpstate::serde_json::from_value::<ProxyProfile>((*v).clone()).ok()).unwrap_or_default()),
            env: {
                let mut map_init = std::collections::HashMap::new();
                let map_prefix = "settings.env.";
                for (k, v) in &initial {
                    if let Some(sub_key) = k.strip_prefix(map_prefix) {
                        if let Ok(parsed_k) = <String as std::str::FromStr>::from_str(sub_key) {
                            if let Ok(parsed_v) = tauri_plugin_rpstate::serde_json::from_value::<String>((*v).clone()) {
                                map_init.insert(parsed_k, parsed_v);
                            }
                        }
                    }
                }
                ReactiveMap::new("settings.env", map_init)
            },
        })
    }

    pub async fn save(&self) -> Result<(), String> {
        invoke_flush("settings").await
    }
}

