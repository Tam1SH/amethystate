<div align="center">

# rpstate

[![Crates.io](https://img.shields.io/crates/v/rpstate.svg)](https://crates.io/crates/rpstate)
[![Docs.rs](https://docs.rs/rpstate/badge.svg)](https://docs.rs/rpstate)
[![CI](https://github.com/Tam1SH/rpstate/actions/workflows/ci.yml/badge.svg)](https://github.com/Tam1SH/rpstate/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Type-safe reactive persistence with automated migrations and schema drift detection. Designed for GUI applications,
with a focus on vertical-slice/feature-based architectures and compile-time verified relations.*

</div>

## Why?

GUI apps in Rust almost always end up in the same place. It usually starts reasonably — a config struct, serde on top,
load on startup, save on exit. Then the project grows.
Persistent and ephemeral state start bleeding into each other. Business logic finds its way into serialization.
Reactivity gets added as an afterthought — a file watcher, a channel, a full reload on any change. Versioning, if it
appears at all, is a fragile enum that guesses at the data's shape rather than tracking it explicitly.

In other ecosystems this is a solved problem. SwiftUI's @AppStorage, Android's DataStore, and Qt's Settings
provide persistent, reactive state with minimal boilerplate. In Rust, there's no established equivalent for native GUI
apps.

`rpstate` is my attempt at something different. Each feature declares its own slice of state independently. References
between slices are explicit and verified by the compiler—mistype a field name or get the type wrong and it's a compile
error, not a runtime surprise.

Persistence is built in, not bolted on. `.set()` writes to the in-memory buffer, fires reactive subscriptions, and
schedules a debounced flush to disk—all in one call. There's no separate save layer to think about.

Migrations I built because… I can.

## Alternatives

|                                                | Persistence | Reactivity | Migrations | Typed fields |
|------------------------------------------------|:-----------:|:----------:|:----------:|:------------:|
| `Arc<Mutex<AppState>>`                         |      ❌      |     ❌      |     ❌      |      ✅       |
| `rustato`                                      |      ❌      |     ✅      |     ❌      |      ✅       |
| `reactive-state`                               |      ❌      |     ✅      |     ❌      |      ✅       |
| `Config managers (confy, figment, twelf, etc)` |      ✅      |     ❌      |     ❌      |      ✅       |
| `bevy_pkv`*                                    |      ✅      |  partial   |     ❌      |      ✅       |
| `KV stores (redb, sled, etc)`                  |      ✅      |  partial   |     ❌      |      ❌       |
| `tauri-plugin-store`                           |      ✅      |  partial   |     ❌      |      ❌       |
| `Traditional DBs (SQLite, etc)`                |      ✅      |   manual   |     ✅      |      ✅       |
| **rpstate**                                    |      ✅      |     ✅      |     ✅      |      ✅       |

* The Bevy dependency is optional, but there is no documentation or examples for non-Bevy usage.

Why not a traditional database (SQLite + ORM)? It comes down to **Collection-oriented vs. State-oriented** design.

Databases are built to store and query collections of records (users, messages, logs). `rpstate` is built for
application state—singleton feature slices with field-level reactive signals out of the box.

If you need to manage lists of entities, use a database. If you need reactive, persistent GUI state without the
boilerplate, use `rpstate`.

## A note on naming

rpstate stands for Reactive Persistent State. When I was checking for name availability, putting the R first was a very
conscious and deliberate choice. One search for the alternative anagram was enough to convince me that "managing your
internal state" should remain a strictly technical endeavor. 🥴

## Quick start

```rust
use rpstate::{rpstate, DefaultStore};
use rpstate::store::builder::StoreBuilder;
use std::sync::Arc;

#[rpstate(prefix = "network")]
pub struct NetworkState {
    #[state(default = "127.0.0.1".to_string())]
    pub host: String,

    #[state(default = 8080)]
    pub port: u16,
}

fn main() -> rpstate::Result<()> {
    let store: Arc<DefaultStore> = Arc::new(
        StoreBuilder::new("./app.redb").build()?
    );

    let state = NetworkState::new(&store)?;

    // Read
    println!("{}", state.host().get()); // "127.0.0.1"

    // Write — persists to pending buffer immediately, flushes to disk debounced
    state.port().set(9090)?;

    // Subscribe
    let _sub = state.port().subscribe(|p| {
        println!("port changed to {p}");
    });

    state.port().set(3000)?; // triggers callback

    Ok(())
}
```

## Backends

`rpstate` supports two storage backends, selected at compile time via Cargo features.

`redb` is the default. It is a fast embedded database and the only backend that supports migrations:

```toml
[dependencies]
rpstate = { version = "*" }  # redb is enabled by default
```

`json` is useful for human-readable storage or debugging. To use it, disable the default features:

```toml
[dependencies]
rpstate = { version = "*", default-features = false, features = ["json"] }
```

## Cross-struct references

Fields can share storage with another struct via `lookup`.

```rust
#[rpstate(prefix = "net")]
pub struct NetworkState {
    #[state(default = 8080, export_mut)]   // export_mut = writable by others
    pub port: u16,

    #[state(default = "127.0.0.1".to_string())]
    pub host: String,
}

#[rpstate(prefix = "ui")]
pub struct UiState {
    // Read-write link to a single field
    #[state(lookup = "port", parent = NetworkState, export_mut)]
    pub proxy_port: u16,

    // Read-only link
    #[state(lookup = "host", parent = NetworkState)]
    pub proxy_host: String,
}
```

`lookup_node` links an entire sub-struct, acting as a namespace for the reactive fields inside it:

```rust
#[rpstate]
pub struct ConnectionPool {
    #[state(default = 10)]
    pub max_connections: u32,
}

#[rpstate(prefix = "sys.database")]
pub struct DatabaseState {
    #[state(nested)]
    pub pool: ConnectionPool,
}

#[rpstate(prefix = "ui.inspector")]
pub struct InspectorState {
    #[state(lookup_node = "pool", parent = DatabaseState)]
    pub db_pool_view: ConnectionPool,
    // Accessed as `state.db_pool_view().max_connections().get()`
}
```

Wrong field name → `no associated item named '__schema_field_porrt'` at compile time.  
Wrong type → `TypeCheck<String>` is not implemented for `ReadOnly<u16>` at compile time.  
Writing a read-only link → `no method named 'perform_set' found for ReadOnly<T>` at compile time.

## Volatile fields

Fields marked `volatile` live in memory only and are never written to the store. They reset to their default on every
restart.

```rust
#[rpstate(prefix = "app")]
pub struct AppState {
    #[state(default = 8080)]
    pub port: u16,

    #[state(default = false, volatile)]
    pub loading: bool,   // always starts as false, never persisted
}
```

## Nested structs

```rust
#[rpstate]
pub struct DatabaseConfig {
    #[state(default = "localhost".to_string())]
    pub host: String,
}

#[rpstate(prefix = "sys")]
pub struct SystemSettings {
    #[state(nested)]
    pub db: DatabaseConfig,   // stored at "sys.db.host", etc.
}
```

## Migrations

The migration system manages persistent state evolution using a dependency graph between components. All transformations
are executed in the correct topological order.

### What migrates and what doesn't

The migrator works exclusively with persistent data (the generated `_Data` types).

- **Included:** Regular fields and `nested` structures.
- **Ignored:** `volatile`, `lookup`, and `lookup_node` fields—they are ephemeral or reactive links and don't exist in
  physical storage.

### Automatic steps (`migrate!`)

Define versioned structs in a `mod v1 { ... }` module, then describe the transformation with the `migrate!` macro.
It handles field mapping, key renaming in the database, and cleanup of removed keys automatically.

```rust
mod v1 {
    use super::*;

    #[rpstate(prefix = "app", version = 1)]
    pub struct Config {
        #[state(default = "localhost".to_string())]
        pub host: String,
    }
}

#[rpstate(prefix = "app", version = 2)]
pub struct Config {
    #[state(default = "localhost".to_string())]
    pub address: String,

    #[state(default = 8080)]
    pub port: u16,
}

migrate! {
    v1::Config_Data => Config_Data,
    rename: [host => address],
    |old| {
        Ok(Self {
            address: old.host,
            port: 9090,
        })
    }
}
```

For nested structs, use `migrate_field!` to delegate migration to a child node's own `migrate!` definition:

```rust
migrate! {
    v1::SystemConfig_Data => SystemConfig_Data,
    rename: [],
    |old, ctx| {
        Ok(Self {
            net: migrate_field!(ctx, old.net),
        })
    }
}
```

To run all auto-generated migrations on startup without any custom steps:

```rust
fn auto_migrations() -> rpstate::Result<()> {
    let store = Arc::new(
        StoreBuilder::new("./app.redb")
            .collect_migrations()
            .build()?
    );
    Ok(())
}
```

### Interleaving automatic and manual steps

Use `.migrations(|m| { ... })` to mix codegen migrations with custom logic. The migrator resolves execution order via
topological sort, so you can safely read data from another node that is guaranteed to have already migrated:

```rust
fn migrate() -> rpstate::Result<()> {
    let store = StoreBuilder::new("./app.redb")
        .migrations(|m| {
            // 1. Pull in all automatic migrations defined via migrate! macros
            m.collect_codegen();

            // 2. Interleave a manual custom step
            m.for_node::<Profile>()
                .depends_on::<Identity>() // Tell the migrator about the dependency
                .step(3, "Complex cross-node logic", |ctx| {
                    // Identity has already migrated at this point.
                    // We can safely pull its up-to-date global data.
                    let identity_plan = ctx.global_get::<String>("identity.plan")?.unwrap();

                    let name: String = ctx.get("display_name")?.unwrap();
                    ctx.set("initials", &name.chars().next().unwrap_or_default().to_string())?;
                    ctx.set("synced_plan", &identity_plan)?;
                    Ok(())
                });
        })
        .build()?;
    Ok(())
}
```

### Schema drift detection

`rpstate` records the schema hash and field types of all persistent fields on every run. If you change a field's type or add/remove fields without bumping the version, no migration runs—but the discrepancy is still noticed.

On startup, `rpstate` compares the stored schema against the current code. Any mismatch produces a warning in the log:

```
⚠️  Schema drift detected in prefix 'app_settings'
  + field 'timeout': Duration
  - field 'host' (exists in DB, missing in code)
  ~ field 'port': u16 -> u32
  Suggestion: increment version and write a migration if these changes are intentional.
```

Three kinds of drift are reported:

| Symbol | Meaning                                                |
|--------|--------------------------------------------------------|
| `+`    | field exists in code but is absent from the database   |
| `-`    | field exists in the database but was removed from code |
| `~`    | field exists in both, but its type changed             |

Drift **does not block startup**—it is a warning, not an error. The application continues running, and the full diff is
available in `MigrationReport` for programmatic inspection:

```rust,ignore
let (store, report) = StoreBuilder::new("./app.redb")
    .collect_migrations()
    .build()?;

if report.has_drift() {
    // report.components → comp.nagging contains the per-prefix diff
}
```

If the changes are intentional, increment the version and write a migration. If not, you may have accidentally dropped
or renamed a field.

### Guarantees and safety

1. **Component atomicity:** Nodes linked by dependencies are grouped into Weakly Connected Components (WCC). All
   migrations in a component run inside a single transaction. A failure in one node rolls back the entire group.
2. **Gap detection:** If the database is at `v1` but the code only provides logic for `v3` and above, the migrator
   fails immediately.
3. **Downgrade protection:** If the version in the database is higher than what the current binary supports, the
   migrator blocks execution to prevent data corruption.