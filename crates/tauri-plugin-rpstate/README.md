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
use tauri_plugin_rpstate::rpstate::store::builder::StoreBuilder;

fn main() {
    let store = Arc::new(
        StoreBuilder::new("./app.redb").build().unwrap()
    );

    tauri::Builder::default()
        .manage(store)
        .plugin(tauri_plugin_rpstate::init())
        .run(tauri::generate_context!())
        .unwrap();
}
```

## TypeScript codegen

The plugin ships `tauri_plugin_rpstate::codegen::export(path)`, a runtime function that walks all `#[rpstate]` structs registered via `inventory` and writes a fully-typed `.ts` file to `path`. Because collection happens inside a running process, `export` cannot be called from a `build.rs` script — it must run inside a real binary.

The simplest approach is to call it once on startup in dev mode:

```rust
fn main() {
    #[cfg(debug_assertions)]
    tauri_plugin_rpstate::codegen::export("../src/bindings/rpstate.ts")
        .expect("codegen failed");

    // ... rest of builder
}
```

The file regenerates on every dev launch. Commit it to version control so the frontend always has up-to-date types.

### What gets generated

For every `#[rpstate(prefix = "...")]` struct a root class and a typed schema are emitted. For every `#[rpstate]` struct without a prefix (nested structs) a helper `*Fields` class is emitted. Additionally, a global `StateSchema` type is generated that maps every persistent key to its TypeScript type.

Given these Rust definitions:

```rust,ignore
use rpstate::rpstate;

#[rpstate]
pub struct DatabaseConfig {
    #[state(default = "localhost".to_string())]
    pub host: String,
}

#[rpstate(prefix = "app")]
pub struct AppSettings {
    #[state(default = 8080)]
    pub port: u16,

    #[state(default = false, volatile)]
    pub loading: bool,

    #[state(nested)]
    pub db: DatabaseConfig,
}
```

The generator produces:

```typescript
export type StateSchema = {
    "app.port": number;
    /** volatile */
    "app.loading": boolean;
    "app.db.host": string;
};

class DatabaseConfigFields {
    readonly host: Field<string>;
    constructor(prefix: string, initialValues?: Record<string, any>) { ... }
}

export type AppSettingsSchema = {
    "app.port": number;
    /** volatile */
    "app.loading": boolean;
    "app.db.host": string;
};

export class AppSettings {
    readonly port: Field<number>;
    readonly loading: Field<boolean>;
    readonly db: DatabaseConfigFields;

    constructor(initialValues?: Partial<AppSettingsSchema>) { ... }
    static async load(): Promise<AppSettings> { ... }
    async save(): Promise<void> { ... }
}
```

## Mental model

There are three layers of state, each with its own representation:

```
┌─────────────────────────────────────┐
│  Frontend (TypeScript)              │
│  in-memory snapshot populated by    │
│  load() and kept in Field objects   │
└────────────────┬────────────────────┘
                 │ IPC (Tauri commands)
┌────────────────▼────────────────────┐
│  rpstate (Rust)                     │
│  in-memory write buffer,            │
│  reactive subscriptions             │
└────────────────┬────────────────────┘
                 │ debounced flush / explicit save
┌────────────────▼────────────────────┐
│  Disk (redb / json)                 │
└─────────────────────────────────────┘
```

Reading `.value` or calling `.setSync()` touches only the frontend snapshot — no IPC, no Rust, no disk. The snapshot drifts from the Rust layer until a subscription event or an explicit `.get()` reconciles it.

Calling `.get()` or `.set()` crosses the IPC boundary into rpstate's write buffer. rpstate then propagates the change to its own subscribers and schedules a debounced flush to disk. The frontend snapshot is updated when rpstate emits the corresponding subscription event back.

`save()` forces an immediate flush of the entire slice from rpstate's buffer to disk, bypassing the debounce. It does not affect the frontend snapshot.

This means there is no single moment where all three layers are guaranteed to be in sync — that is a deliberate tradeoff for responsiveness. If you need a read that is guaranteed to reflect disk state, use `.get()` after `save()`.

## Frontend usage

Each field exposes two read/write strategies. **Cached** operations are synchronous and work against the in-memory snapshot populated by `load()` — fast, but may lag behind a pending background write. **Persistent** operations are async and go through the IPC layer directly to the store, reflecting what is actually on disk. The tradeoff mirrors the one described for reactive fields in the [rpstate docs](https://github.com/Tam1SH/rpstate).

### Loading a slice

`load()` fetches all keys under the slice's prefix in a single IPC call and pre-populates every field:

```typescript
import { AppSettings } from "./bindings/rpstate";

const settings = await AppSettings.load();
```

### Reading values

```typescript
// Cached — synchronous, reads from the in-memory snapshot.
console.log(settings.port.value); // number | null

// Persistent — async, queries the store directly.
const port = await settings.port.get(); // Promise<number>
```

### Writing values

```typescript
// Cached — updates local memory immediately, queues a debounced background write.
settings.port.value = 9090;

// Persistent — queues a write in the store's write buffer.
await settings.port.set(9090);

// Flush the entire slice to disk right now.
await settings.save();
```

### Subscribing to changes

`subscribe` registers a backend subscription and fires the callback on every change emitted by rpstate. The returned `unlisten` function unregisters the backend subscription and drops the Tauri event listener. Call it when the component that owns the subscription is torn down to avoid leaking listeners.

```typescript
const unlisten = settings.port.subscribe((newPort) => {
    console.log("port changed to", newPort);
});

// Unsubscribe when done.
unlisten();

// Or destroy the field/slice to clean up everything at once.
settings.port.destroy();
```

### Reactive maps

Fields backed by `ReactiveMap<K, V>` on the Rust side are exposed as `ReactiveMapField<K, V>`:

```typescript
// Persistent get / set
const threshold = await settings.limits.get("cpu");
await settings.limits.set("gpu", { warning: 60, critical: 85 });

// Cached get / set
const cached = settings.limits.getSync("cpu");
settings.limits.setSync("gpu", { warning: 60, critical: 85 });

// Subscribe to any change in the map
const unlisten = settings.limits.subscribeAny((change) => {
    if (change.type === "Insert") console.log("added", change.key);
    if (change.type === "Update") console.log("updated", change.key);
    if (change.type === "Remove") console.log("removed", change.key);
});

// Subscribe to a single key
const unlistenCpu = settings.limits.subscribeKey("cpu", (val) => {
    console.log("cpu threshold:", val);
});

// Read the entire in-memory snapshot
for (const [key, val] of settings.limits.entries) {
    console.log(key, val);
}
```

### `ReadonlyField`

Fields declared with `lookup` and no `export_mut` on the Rust side are exposed as `ReadonlyField<T>` — they have `.value` and `.get()` but no `.set()` or `.value` setter.

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

| Identifier                          | Description                             |
|-------------------------------------|-----------------------------------------|
| `rpstate:allow-rpstate-get`         | Read a single key                       |
| `rpstate:allow-rpstate-set`         | Write a single key                      |
| `rpstate:allow-rpstate-subscribe`   | Subscribe to key changes                |
| `rpstate:allow-rpstate-unsubscribe` | Unsubscribe from a key                  |
| `rpstate:allow-rpstate-get-prefix`  | Bulk-read all keys under a prefix       |
| `rpstate:allow-rpstate-flush`       | Flush pending writes to disk            |

Every permission has a corresponding `deny-*` variant that takes priority over `allow-*`.

## Commands reference

| Command               | Parameters             | Returns                                   |
|-----------------------|------------------------|-------------------------------------------|
| `rpstate_get`         | `key: string`          | `Option<Value>`                           |
| `rpstate_set`         | `key: string`, `value` | `()`                                      |
| `rpstate_get_prefix`  | `prefix: string`       | `HashMap<string, Value>`                  |
| `rpstate_flush`       | `prefix: string`       | `()`                                      |
| `rpstate_subscribe`   | `key: string`          | `()` — emits events on `rpstate://<key>`  |
| `rpstate_unsubscribe` | `key: string`          | `()`                                      |

Subscription events are emitted on the Tauri event channel `rpstate://<key>` with dots replaced by colons (e.g. `rpstate://app:port`). For `ReactiveMapField`, changes to individual entries are emitted as `MapChange` payloads on the map prefix channel.

## Requirements

- Tauri **v2**
- rpstate **v0.x** (see [Cargo.toml](../../Cargo.toml) for the exact workspace version)

## License

MIT — see [LICENSE](../../LICENSE).