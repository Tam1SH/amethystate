use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen as swb;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/core.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = "invoke_result")]
    async fn invoke_result_js(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn invoke_result<T, E>(command: &str, args: impl Serialize) -> Result<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    invoke_result_js(command, swb::to_value(&args).unwrap())
        .await
        .map(|val| swb::from_value(val).unwrap())
        .map_err(|err| swb::from_value(err).unwrap())
}
