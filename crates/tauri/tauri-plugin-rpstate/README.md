<div align="center">

# tauri-plugin-rpstate

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-rpstate.svg)](https://crates.io/crates/tauri-plugin-rpstate)
[![Docs.rs](https://docs.rs/tauri-plugin-rpstate/badge.svg)](https://docs.rs/tauri-plugin-rpstate)
[![CI](https://github.com/Tam1SH/rpstate/actions/workflows/ci.yml/badge.svg)](https://github.com/Tam1SH/rpstate/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Tauri v2 plugin that exposes [rpstate](https://github.com/Tam1SH/rpstate) reactive persistent state to the frontend, with TypeScript codegen.*

</div>

> **⚠️ Experimental**  
> This plugin is provided as-is. The API may change without notice and has not been hardened for production use. Feedback and contributions are welcome.

## Overview

`tauri-plugin-rpstate` bridges your `rpstate` state slices to the Tauri frontend. It exposes six IPC commands for reading, writing, and subscribing to state, and ships a runtime code generator that produces fully-typed TypeScript bindings from your Rust struct definitions — no manual type duplication.

## Installation

Add the plugin to your Tauri app's Rust crate:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-rpstate = "*"
```

`rpstate` is re-exported as `tauri_plugin_rpstate::rpstate`, so no separate dependency is needed.

Register the plugin and your store in `main.rs`:

```rust
use std::sync::Arc;
use tauri_plugin_rpstate::rpstate::StoreBuilder;

fn main() {
    let store = Arc::new(
        StoreBuilder::new("./app").build().unwrap()
    );

    tauri::Builder::default()
        .manage(store)
        .plugin(tauri_plugin_rpstate::init())
        .run(tauri::generate_context!())
        .unwrap();
}
```

## TypeScript codegen

The plugin ships `tauri_plugin_rpstate::backend::codegen::CodegenRegistry`, which walks all `#[rpstate]` structs registered via `inventory` and writes a fully-typed `.ts` file. Because collection happens inside a running process, it cannot be called from `build.rs` — use a test instead:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        use tauri_plugin_rpstate::backend::codegen::CodegenRegistry;
        let reg = CodegenRegistry::new();

        reg.export_ts("../src/bindings/rpstate.ts")
            .expect("TS codegen failed");
    }
}
```

For a complete usage example see [`examples/tauri-settings`](../../../examples/tauri-settings).


## Rust codegen (WASM client)

For frontends written in Rust targeting `wasm32` (e.g. Leptos, Yew, Dioxus), the registry can emit typed Rust bindings instead of TypeScript:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        use tauri_plugin_rpstate::backend::codegen::CodegenRegistry;
        let reg = CodegenRegistry::new();

        reg.export_rust("../src/bindings/rpstate.rs")
            .expect("Rust codegen failed");
    }
}
```

The generated file compiles on `wasm32` only. For a complete usage example see [`examples/tauri-leptos`](../../../examples/tauri-leptos).

## Mental model

There are three layers of state, each with its own representation:

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (TypeScript or Rust/WASM)                         │
│  in-memory snapshot populated by load() and kept in         │
│  Field objects / structs                                    │
└────────────────────────┬────────────────────────────────────┘
                         │ IPC (Tauri commands)
┌────────────────────────▼────────────────────────────────────┐
│  rpstate (Rust)                                             │
│  in-memory write buffer, reactive subscriptions             │
└────────────────────────┬────────────────────────────────────┘
                         │ debounced flush / explicit save
┌────────────────────────▼────────────────────────────────────┐
│  Disk (redb / json)                                         │
└─────────────────────────────────────────────────────────────┘
```

## Permissions

Add the default permission set to `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "rpstate:default"
  ]
}
```

`rpstate:default` includes the following permissions:

| Identifier                          | Description                       |
|-------------------------------------|-----------------------------------|
| `rpstate:allow-rpstate-get`         | Read a single key                 |
| `rpstate:allow-rpstate-set`         | Write a single key                |
| `rpstate:allow-rpstate-delete`      | Delete a single key               |
| `rpstate:allow-rpstate-subscribe`   | Subscribe to key changes          |
| `rpstate:allow-rpstate-unsubscribe` | Unsubscribe from a key            |
| `rpstate:allow-rpstate-get-prefix`  | Bulk-read all keys under a prefix |
| `rpstate:allow-rpstate-flush`       | Flush pending writes to disk      |

Every permission has a corresponding `deny-*` variant that takes priority over `allow-*`.

## Requirements

- Tauri **v2**
- rpstate **v0.x** (see [Cargo.toml](../../../Cargo.toml) for the exact workspace version)

## License

MIT — see [LICENSE](../../../LICENSE).