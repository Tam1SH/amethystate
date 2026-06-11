## Using with a custom GPUI version

If your project depends on a git version of GPUI, add a `[patch]` to your
workspace `Cargo.toml` to ensure a single copy of the crate is used:

```toml
[patch.crates-io]
gpui = { git = "https://github.com/zed-industries/zed", rev = "abc123" }
```

Without this, Cargo will treat the crates.io and git versions as separate
crates and you'll get type mismatch errors at compile time.