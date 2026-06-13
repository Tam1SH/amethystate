# amethystate-codegen

Copies your backend state types to the frontend crate, adding the necessary macros for your Rust WASM framework.

## Supported Frameworks

| Feature flag | Framework    |
|--------------|--------------|
| `dioxus`     | Dioxus       |
| `leptos`     | Leptos       |
| *(none)*     | Vanilla WASM |

## Setup

The `codegen` binary must live in the same crate where your `#[amethystate]` types are declared.

**1. Add the binary target to `Cargo.toml`**

```toml
[[bin]]
name = "codegen"
path = "src/bin/codegen.rs"

[dependencies]
amethystate-codegen = { version = "*", features = ["leptos"] }
```

**2. Create `src/bin/codegen.rs`**

```rust,ignore
#[allow(unused_imports)]
use your_crate_with_amethystate_types as _; // pulls inventory registrations into the binary

amethystate_codegen::amethystate_codegen_main!(
    rs_out = "../src/bindings/amethystate.rs",
    framework = leptos
);
```

**3. Run**

```sh
cargo run --bin codegen
```