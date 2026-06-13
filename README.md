<div align="center">

# amethystate

[![Crates.io](https://img.shields.io/crates/v/amethystate.svg)](https://crates.io/crates/amethystate)
[![Docs.rs](https://docs.rs/amethystate/badge.svg)](https://docs.rs/amethystate)
[![CI](https://github.com/Tam1SH/amethystate/actions/workflows/ci.yml/badge.svg)](https://github.com/Tam1SH/amethystate/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Type-safe reactive persistence with automated migrations, schema drift detection, persistent-only data access, and
derived reactive pipelines. Designed for GUI applications, with a focus on vertical-slice/feature-based architectures
and compile-time verified relations.*

</div>

## Features

`amethystate` is built around feature-local state slices:

- persistent reactive fields with `.get()`, `.set()`, and `.subscribe()`;
- persistent-only loading with plain mutable data for frameworks that own their state model;
- derived reactive pipelines for small synchronous transformations;
- feature-local state that can be composed, shared, versioned, and checked for schema drift.

Use `State::new(&store)` for reactive fields and `State::load(&store)` when you only want persistence.

## Status

`amethystate` is pre-1.0, but the current API is meant to be usable as-is. I do not plan to break it without a strong
reason, though minor releases may still contain breaking changes if real usage exposes a design issue.

## Examples

| Example            | GUI model                         | amethystate usage                                                                                                                             |
|--------------------|-----------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| `egui-settings`    | immediate-mode UI                 | reactive fields read during `update`, writes from widgets, derived pipeline                                                               |
| `iced-settings`    | TEA/MVU                           | persistent-only `State::load`, plain data mutation in `update`                                                                            |
| `slint-settings`   | property bindings + event loop    | reactive fields, `ReactiveScope`, pipeline subscription updating Slint properties                                                         |
| `dioxus-settings`  | components + fine-grained signals | custom hooks bridging reactive fields and pipelines to Dioxus signals via async channels                                                  |
| `ratatui-settings` | TUI poll loop                     | persistent-only `State::load`, direct struct reads in loop, `mutate_lazy()` for deferred writes                                           |
| `tauri-settings`   | Tauri v2 + vanilla TS frontend    | `AppSettings.load()`, sync `.value` reads/writes, `.subscribe()`, generated bindings                                                      |
| `tauri-leptos`     | Tauri v2 + Leptos frontend        | `use_amethystate_field` hooks bridging `Field<T>` to Leptos signals, nested state, `ReactiveMap`, pipelines via `.pipe()`, generated bindings |

## Quick start

By default, `amethystate` structures are compiled in **`reactive`** mode. This exposes reactive `Field<T>` handles with automatic change propagation.

```rust
use amethystate::{StoreBuilder, IntoPipeline, ReactiveScope, amethystate};

#[amethystate(prefix = "network")]
pub struct NetworkState {
    #[amestate(default = "127.0.0.1".to_string())]
    pub host: String,

    #[amestate(default = 8080)]
    pub port: u16,
}

fn main() -> amethystate::Result<()> {
    let store = StoreBuilder::new("./app").build()?;

    let state = NetworkState::new_with(&store)?;

    // Read
    println!("{}", state.host().get()); // "127.0.0.1"

    // Write — persists to pending buffer immediately, flushes to disk debounced
    state.port().set(9090)?;

    // Subscribe
    let _sub = state.port().subscribe(|p| {
        println!("port changed to {p}");
    });

    state.port().set(3000)?; // triggers callback

    // Derive values with a synchronous reactive pipeline
    let address = (state.host(), state.port()).pipe()
        .map(|(host, port)| format!("{host}:{port}"))
        .dedupe();

    let mut scope = ReactiveScope::new();
    scope.watch(address.subscribe(|addr| {
        println!("address is now {addr}");
    }));

    Ok(())
}
```

## Persistent-only mode

Some frameworks already own the render/update loop and have no use for reactive subscriptions. For example, iced uses The Elm Architecture: you mutate plain model data in `update`, then let the framework render from that model.

For those cases, you can declare your struct with `mode = "persistent"`. This removes the overhead of reactive `Field` wrappers. Fields are exposed as plain Rust types and are accessed and mutated directly.

```rust
#[amethystate(prefix = "network", mode = "persistent")]
pub struct NetworkState {
    #[amestate(default = "127.0.0.1".to_string())]
    pub host: String,

    #[amestate(default = 8080)]
    pub port: u16,
}
```

Usage:

```rust,ignore
let mut state = NetworkState::load(&store)?;

println!("{}", state.port);

// --- Scenario 1: Direct Field Mutation ---
state.port = 9090;

state.save_lazy()?; // RAM-buffer write (debounced/background)
state.save()?;      // Synchronous/immediate flush to disk

// --- Scenario 2: Block Mutation (Immediate Flush) ---
state.mutate(|d| {
    d.host = "127.0.0.1".to_string();
    d.port = 4040;
})?; // Mutates and immediately flushes changes to disk synchronously

// --- Scenario 3: Block Mutation (Debounced Background Flush) ---
state.mutate_lazy(|d| {
    d.host = "192.168.1.1".to_string();
    d.port = 3000;
})?; // Mutates and schedules a debounced background write to disk
```

For edge cases where you want both reactive fields and a flat persistent-only wrapper generated for the exact same struct, you can use `mode = "both"`.

## Why?

GUI apps in Rust almost always end up in the same place. It usually starts reasonably — a config struct, serde on top,
load on startup, save on exit. Then the project grows.
Persistent and ephemeral state start bleeding into each other. Business logic finds its way into serialization.
Reactivity gets added as an afterthought — a file watcher, a channel, a full reload on any change. Versioning, if it
appears at all, is a fragile enum that guesses at the data's shape rather than tracking it explicitly.

In other ecosystems this is a solved problem. SwiftUI's @AppStorage, Android's DataStore, Qt's Settings, and Flutter's Hive provide persistent, reactive state with minimal boilerplate. In Rust, there's no established equivalent for native GUI apps.

`amethystate` is my attempt at something different. Each feature declares its own slice of state independently. References
between slices are explicit and verified by the compiler—mistype a field name or get the type wrong and it's a compile
error, not a runtime surprise.

Persistence is built in, not bolted on. Changing state writes to the in-memory buffer, fires reactive subscriptions, and
schedules a debounced flush to disk—all in one call. There's no separate save layer to think about.

Migrations I built because… I can.

If your data is naturally a collection of records, use SQLite, redb/sled directly, or an ORM. If you need
human-editable config collected from multiple sources, `confy`, `figment`, or `twelf` are a better fit.

## Backends

`amethystate` supports two storage backends, selected at compile time via Cargo features.

`redb` is the default. It is a fast embedded database and the only backend that supports migrations:

```toml
[dependencies]
amethystate = { version = "*" }  # redb is enabled by default
```

`json` is useful for human-readable storage or debugging. To use it, disable the default features:

```toml
[dependencies]
amethystate = { version = "*", default-features = false, features = ["json"] }
```

## Cross-struct references

Fields can share storage with another struct via `lookup`.

```rust
#[amethystate(prefix = "net")]
pub struct NetworkState {
    #[amestate(default = 8080, export_mut)]   // export_mut = writable by others
    pub port: u16,

    #[amestate(default = "127.0.0.1".to_string())]
    pub host: String,
}

#[amethystate(prefix = "ui")]
pub struct UiState {
    // Read-write link to a single field
    #[amestate(lookup = "port", parent = NetworkState, export_mut)]
    pub proxy_port: u16,

    // Read-only link
    #[amestate(lookup = "host", parent = NetworkState)]
    pub proxy_host: String,
}
```

`lookup_node` links an entire sub-struct, acting as a namespace for the reactive fields inside it:

```rust
#[amethystate]
pub struct ConnectionPool {
    #[amestate(default = 10)]
    pub max_connections: u32,
}

#[amethystate(prefix = "sys.database")]
pub struct DatabaseState {
    #[amestate(nested)]
    pub pool: ConnectionPool,
}

#[amethystate(prefix = "ui.inspector")]
pub struct InspectorState {
    #[amestate(lookup_node = "pool", parent = DatabaseState)]
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
#[amethystate(prefix = "app")]
pub struct AppState {
    #[amestate(default = 8080)]
    pub port: u16,

    #[amestate(default = false, volatile)]
    pub loading: bool,   // always starts as false, never persisted
}
```

## Nested structs

```rust
#[amethystate]
pub struct DatabaseConfig {
    #[amestate(default = "localhost".to_string())]
    pub host: String,
}

#[amethystate(prefix = "sys")]
pub struct SystemSettings {
    #[amestate(nested)]
    pub db: DatabaseConfig,   // stored at "sys.db.host", etc.
}
```

## Reactive pipelines

Pipelines are synchronous derived reactive values. They are useful when one or more fields should produce a formatted
value, validation result, log event, or side effect without nesting subscriptions by hand.

```rust
use amethystate::IntoPipeline;

let display_port = state.port().pipe()
.map(|p| format!(":{p}"))
.dedupe();

let _sub = display_port.subscribe(|port| {
println!("display port changed: {port}");
});
```

Pipelines are readable. A GUI can call `.get()` during its own render cycle without subscribing:

```rust
let display_port = state.port().pipe()
.map(|p| format!(":{p}"));

assert_eq!(display_port.get(), ":8080");
```

Tuple pipelines derive a value from the latest values of all inputs. When any input changes, the pipeline reads every
input and recomputes from that full tuple.

```rust
let address = (state.host(), state.port()).pipe()
.map(|(host, port)| format!("{host}:{port}"));
```

Pipelines compose because `Pipeline<T>` is itself reactive:

```rust
let display_port = state.port().pipe()
.map(|p| format!(":{p}"));

let address = (state.host(), display_port).pipe()
.map(|(host, port)| format!("{host}{port}"));
```

Subscriptions are RAII handles. Store them directly or put them in a `ReactiveScope`:

```rust
use amethystate::ReactiveScope;

let mut scope = ReactiveScope::new();

scope.watch(address.subscribe(|addr| {
println!("address changed: {addr}");
}));

scope.clear(); // drops all watched subscriptions
```

Available operators:

| Operator         | Behavior                                                                                                |
|------------------|---------------------------------------------------------------------------------------------------------|
| `.map(f)`        | Transform every value.                                                                                  |
| `.filter_map(f)` | Accept `Some(value)` and suppress `None`; if the initial value is `None`, `Default::default()` is used. |
| `.inspect(f)`    | Observe values without changing them.                                                                   |
| `.dedupe()`      | Suppress consecutive duplicate values.                                                                  |

Propagation is synchronous. There is no runtime, scheduler, batching, or dependency tracker. If two upstream fields are
changed one after another, a tuple pipeline fires once for each change.

## Reactive Maps

`ReactiveMap<K, V>` manages dynamic collections where each entry is stored as an individual key in the database.

### Declaration

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, AmeType)]
pub struct AlertThresholds {
    pub warning: u64,
    pub critical: u64,
}

#[amethystate(prefix = "sys")]
pub struct SystemSettings {
    #[amestate(default = {
        "cpu": AlertThresholds { warning: 70, critical: 90 },
        "mem": AlertThresholds { warning: 80, critical: 95 }
    })]
    pub limits: ReactiveMap<String, AlertThresholds>,
}
```

### Usage

```rust,ignore
let state = SystemSettings::new_with(&store)?;

// Upsert (Insert or Update)
state.limits().set_or_create("gpu".into(), &AlertThresholds { warning: 60, critical: 85 })?;

// Lookup
let cpu_limit = state.limits().get(&"cpu".into())?; // Result<Option<AlertThresholds>>

// Scan
for (key, val) in state.limits().entries()? {
    println!("{key}: {val:?}");
}
```

### Reactivity and Interceptors

You can subscribe to changes for the entire map or a specific key.

```rust,ignore
// Subscribe to any change in the map
let _any_sub = state.limits().subscribe_any(|change| {
    match change {
        MapChange::Insert { key, .. } => println!("Added {key}"),
        MapChange::Update { key, .. } => println!("Updated {key}"),
        MapChange::Remove { key, .. } => println!("Removed {key}"),
        MapChange::Clear => println!("Cleared"),
    }
});

// Subscribe to a specific key only
let _key_sub = state.limits().subscribe_key("cpu".into(), |change| {
    println!("CPU limit changed");
});
```

#### Interceptors and Cycle Protection
Interceptors allow you to validate or transform changes before they are committed.

*   **Rejection:** If an interceptor returns `None`, the operation is cancelled and returns `Err(Error::Intercepted)`.
*   **Cycle Protection:** The system tracks recursion depth (max depth = 10). If an interceptor triggers a change to the same path, execution is aborted with a `warning` in the log to prevent deadlocks.

```rust,ignore
state.limits().intercept(|change| {
    if let MapChange::Update { new_value, .. } = &change {
        // Validation: critical threshold cannot be lower than warning
        if new_value.critical < new_value.warning {
            return None; // Will cause the .set() call to return Err(Error::Intercepted)
        }
    }
    Some(change)
});
```



### Default values

`#[amestate(default = ...)]` is applied only on first initialization. To add new default entries in a later version, use a migration step.

### Storage
Data is persisted using the path format `{prefix}.{field_name}.{key}` (e.g., `sys.limits.cpu`).


## Migrations

The migration system manages persistent state evolution using a dependency graph between components. All transformations
are executed in the correct topological order.

### What migrates and what doesn't

The migrator works exclusively with persistent data (`AmeData<T>` types).

- **Included:** Regular fields and `nested` structures.
- **Ignored:** `volatile`, `lookup`, and `lookup_node` fields — they are ephemeral or reactive links and don't exist in
  physical storage.

### Defining a migration step

Define versioned structs in a `mod v1 { ... }` module, then describe the transformation with the `#[migrate]` attribute
on a plain function. Field renames are declared with `#[rename]` and generate a compile-time check that both fields exist.

```rust
mod v1 {
    use super::*;

    #[amethystate(prefix = "app", version = 1)]
    pub struct Config {
        #[amestate(default = "localhost".to_string())]
        pub host: String,
    }
}

#[amethystate(prefix = "app", version = 2)]
pub struct Config {
    #[amestate(default = "localhost".to_string())]
    pub address: String,

    #[amestate(default = 8080)]
    pub port: u16,
}

#[migrate]
#[rename(host => address)]
fn migrate_config_v1_to_v2(old: AmeData<v1::Config>) -> amethystate::Result<AmeData<Config>> {
    Ok(AmeData::<Config> {
        address: old.host,
        port: 9090,
    })
}
```

To run all auto-generated migrations on startup without any custom steps:

```rust
fn auto_migrations() -> amethystate::Result<()> {
    let store = StoreBuilder::new("./app.redb")
        .collect_migrations()
        .build()?;
    Ok(())
}
```

### Interleaving automatic and manual steps

Use `.migrations(|m| { ... })` to mix codegen migrations with custom logic. The migrator resolves execution order via
topological sort, so you can safely read data from another node that is guaranteed to have already migrated:

```rust
fn migrate() -> amethystate::Result<()> {
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

`amethystate` records the schema hash and field types of all persistent fields on every run. If you change a field's type or add/remove fields without bumping the version, no migration runs—but the discrepancy is still noticed.

On startup, `amethystate` compares the stored schema against the current code. Any mismatch produces a warning in the log:

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
