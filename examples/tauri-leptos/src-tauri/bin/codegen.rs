#[allow(unused_imports)]
use tauri_leptos_lib as _;

amethystate_codegen::amethystate_codegen_main!(
    rs_out = "../src/bindings/amethystate.rs",
    framework = leptos
);