# rpstate examples

Run them from the repository root:

```powershell
cargo run --manifest-path examples/egui-settings/Cargo.toml
cargo run --manifest-path examples/iced-settings/Cargo.toml
cargo run --manifest-path examples/slint-settings/Cargo.toml
cargo run --manifest-path examples/dioxus-settings/Cargo.toml
```

| Example           | GUI model                         | rpstate usage                                                                                                          |
|-------------------|-----------------------------------|------------------------------------------------------------------------------------------------------------------------|
| `egui-settings`   | immediate-mode UI                 | reactive fields read during `update`, writes from widgets, derived pipeline                                            |
| `iced-settings`   | TEA/MVU                           | persistent-only `State::load`, plain data mutation in `update`                                                         |
| `slint-settings`  | property bindings + event loop    | reactive fields, `ReactiveScope`, pipeline subscription updating Slint properties                                      |
| `dioxus-settings` | components + fine-grained signals | custom hooks bridging reactive fields and pipelines to Dioxus signals via async channels                               |
| `tauri-settings`  | Tauri v2 + vanilla TS frontend    | `AppSettings.load()`, sync `.value` reads/writes, `.subscribe()` for cross-component sync, theme + counter persistence |