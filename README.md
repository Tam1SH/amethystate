# rpstate

Type-safe reactive persistence with automated migrations and schema drift detection. Designed for GUI applications and
oriented towards vertical-slice/feature-based architectures with compile-time verified relations.

## Backends

| Feature flag     | Backend                                 | Format      |
|------------------|-----------------------------------------|-------------|
| `redb` (default) | [redb](https://github.com/cberner/redb) | MessagePack |
| `json`           | JSON file with file-watcher             | JSON        |

## A note on naming

rpstate stands for Reactive Persistent State. When I was checking for name availability, putting the R first was a very
conscious and deliberate choice. One search for the alternative anagram was enough to convince me that "managing your
internal state" should remain a strictly technical endeavor. 🥴

## Quick start

```rust
use rpstate::{rpstate, DefaultStore};
use rpstate::store::builder::StoreBuilder;
use std::sync::Arc;

// Define a state slice
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

    // Write (persists immediately to pending buffer, flushes to disk debounced)
    state.set_port(9090)?;

    // Subscribe
    let _sub = state.port().subscribe(|p| {
        println!("port changed to {p}");
    });

    state.set_port(3000)?; // triggers callback

    Ok(())
}
```

## Cross-struct references

Fields can share storage with another struct.

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
    // Read-write link
    #[state(lookup = "port", parent = NetworkState, export_mut)]
    pub proxy_port: u16,

    // Read-only link
    #[state(lookup = "host", parent = NetworkState)]
    pub proxy_host: String,
}
```

or

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
    // Links the entire sub-struct as a reactive node
    #[state(lookup_node = "pool", parent = DatabaseState)]
    pub db_pool_view: ConnectionPool,
}
```

Wrong field name → `no associated item named '__schema_field_porrt'` at compile time.  
Wrong type → `TypeCheck<String>` is not implemented for `ReadOnly<u16>` at compile time.  
Writing a read-only link → `no method named 'perform_set' found for ReadOnly<T>` at compile time.

## Volatile fields

Fields marked `volatile` live in memory only and are never written to the store.

```rust
#[rpstate(prefix = "app")]
pub struct AppState {
    #[state(default = 8080)]
    pub port: u16,

    #[state(default = false, volatile)]
    pub loading: bool,   // in-memory only
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
    pub db: DatabaseConfig,   // keys stored at "sys.db.host", etc.
}
```

---

## Migrations

The `rpstate` migration system manages persistent state evolution using a dependency graph between components. The
migrator ensures that all transformations across different nodes are executed in the correct topological order.

### What Migrates and What Doesn't

The migrator works exclusively with persistent data structures (the generated `_Data` types).

* **Included:** Regular fields and `nested` structures.
* **Ignored:** Fields marked as `volatile`, `lookup`, or `lookup_node`. These are ephemeral or reactive links; they do
  not exist in the node's physical storage and therefore do not affect the schema or migration process.

### Automatic Steps (`migrate!`)

For standard `vN -> vN+1` transitions, the `migrate!` macro handles the heavy lifting:

* **Mapping:** It transforms the old data structure into the new one.
* **Renaming:** It automatically renames keys in the database if specified in the `rename` block.
* **Cleanup:** It purges storage of old keys that were either renamed or removed in the new version, ensuring no "dead"
  data remains.

```rust
rpstate::migrate! {
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

### Interleaving and Topological Sorting

One of the most powerful features is the ability to **interleave** automatic migrations (from macros) with manual steps
containing custom logic. The migrator automatically resolves the execution order using topological sorting.

This allows you to insert logic that, for example, reads data from another node that is guaranteed to have already been
migrated to its latest version:

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
}
```

### Guarantees and Safety

1. **Component Atomicity:** Nodes linked by dependencies are grouped into "Weakly Connected Components" (WCC). All
   migrations within a component are executed inside a single database transaction. A failure in one node rolls back the
   entire group.
2. **Gap Detection:** The migrator will fail to start if there is a gap in the version chain (e.g., the database is at
   `v1`, but the code only provides logic for `v3` and above).
3. **Downgrade Protection:** If the version stored in the database is higher than the version supported by the current
   binary, the migrator will block execution to prevent data corruption.
4. **Schema Drift Detection:** `rpstate` tracks type hashes for persistent fields. If you change a field type without
   incrementing the version, the system can detect this "nagging" discrepancy.

## Status

Early development. The storage layer, reactive fields, migration runner, proc-macro are working. CLI tooling are not yet
implemented.