# Installation

Add `amethystate` to your `Cargo.toml`:

```toml
[dependencies]
amethystate = "*"
```

## Backends

`amethystate` requires a storage backend. The default is `redb`.

**redb** — the default. A fast embedded database.

```toml
amethystate = "*"
```

**Text** — human-readable files. Three formats are available: `json`, `toml`, and `ron`. Useful for debugging, when human-editable storage is required, or as a migration path from existing solutions like `confy` or custom file-based storage.

```toml
amethystate = { version = "*", default-features = false, features = ["json"] }
```

**SQLite** — via rusqlite. Use `sqlite-bundled` if you don't want a system SQLite dependency.

```toml
amethystate = { version = "*", default-features = false, features = ["sqlite-bundled"] }
```

## Tauri

Tauri integration includes a plugin, async backend, and Rust and TypeScript bindings generator. Enable it with the `tauri` feature:

```toml
amethystate = { version = "*", features = ["tauri"] }
```

See [Tauri integration](../integrations/tauri.md) for setup and usage.

## Migrating from an existing solution

See [Migrating from confy](../migrations/confy-compat.md) or [Migrating from a custom solution](../migrations/custom.md).

## Framework integrations

See [Integrations](../integrations/).