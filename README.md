# rpstate

Type-safe reactive persistence with automated migrations and schema drift detection. Designed for GUI applications and oriented towards vertical-slice/feature-based architectures with compile-time verified relations.

## Backends

| Feature flag | Backend | Format |
|---|---|---|
| `redb` (default) | [redb](https://github.com/cberner/redb) | MessagePack |
| `json` | JSON file with file-watcher | JSON |

## A note on naming

rpstate stands for Reactive Persistent State. When I was checking for name availability, putting the R first was a very conscious and deliberate choice. One search for the alternative anagram was enough to convince me that "managing your internal state" should remain a strictly technical endeavor. 🥴


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


## Status

Early development. The storage layer, reactive fields, proc-macro are working. Migration runner, CLI tooling are not yet implemented.