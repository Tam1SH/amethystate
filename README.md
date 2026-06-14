<div align="center">

<img src="logo.svg" alt="amethystate" width="384" />

# amethystate

[![Crates.io](https://img.shields.io/crates/v/amethystate.svg)](https://crates.io/crates/amethystate)
[![Docs.rs](https://docs.rs/amethystate/badge.svg)](https://docs.rs/amethystate)
[![CI](https://github.com/Tam1SH/amethystate/actions/workflows/ci.yml/badge.svg)](https://github.com/Tam1SH/amethystate/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Persistent reactive state for Rust GUI apps.*

</div>

Every Rust GUI project builds the same persistence layer from scratch. It starts with a struct, `serde`, and `confy` —
or just the same boilerplate written by hand. Then the app grows: schema changes get mixed into validation logic,
a file watcher gets bolted on so settings reload without a restart, versioning becomes a fragile enum that guesses at
the data's shape.

`amethystate` is that layer, built once. Fields persist automatically, fire subscriptions on change, and flush to disk
in the background. Schema versions are explicit, migrations run on startup, and drift is detected and logged.

```rust
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

    state.port().set(9090)?;

    let _sub = state.port().subscribe(|p| println!("port → {p}"));

    let address = (state.host(), state.port())
        .pipe()
        .map(|(host, port)| format!("{host}:{port}"));

    Ok(())
}
```

egui, iced, ratatui, and other retain-mode frameworks are supported too — see [Integrations](./book/integrations/overview.md).

---

See the **[book](./book/)** for full documentation — concepts, migrations, and per-framework integration guides.
