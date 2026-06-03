#[cfg(not(target_arch = "wasm32"))]
pub mod backend;

#[cfg(not(target_arch = "wasm32"))]
pub use backend::init;

#[cfg(target_arch = "wasm32")]
pub mod client;

#[cfg(target_arch = "wasm32")]
pub use client::*;

pub use serde_json;
