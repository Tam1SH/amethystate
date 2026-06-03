pub mod field;
pub mod map;

pub use field::Field;
pub use map::ReactiveMap;

pub use rpstate_core::{
    Change, IntoPipeline, MapChange, Pipeline, Reactive, ReactiveScope, SignalSubscription,
};

use serde::Serialize;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct TauriEventPayload<T> {
    pub payload: T,
}

pub async fn invoke_get_prefix(prefix: &str) -> Result<HashMap<String, serde_json::Value>, String> {
    #[derive(Serialize)]
    struct PrefixArgs<'a> {
        prefix: &'a str,
    }

    tauri_sys::core::invoke_result("plugin:rpstate|rpstate_get_prefix", &PrefixArgs { prefix })
        .await
}

pub async fn invoke_flush(prefix: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct PrefixArgs<'a> {
        prefix: &'a str,
    }

    tauri_sys::core::invoke_result("plugin:rpstate|rpstate_flush", &PrefixArgs { prefix }).await
}
