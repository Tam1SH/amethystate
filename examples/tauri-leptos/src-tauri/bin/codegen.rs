#[allow(unused_imports)]
use tauri_leptos_lib as _;

rpstate_codegen::rpstate_codegen_main!(
    rs_out = "../src/bindings/rpstate.rs",
    framework = leptos
);