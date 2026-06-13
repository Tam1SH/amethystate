# Integrations

`amethystate` supports multiple GUI frameworks. Which mode to use — reactive or persistent-only — depends on the execution model of the framework.

## Execution models

### Retain-mode and TEA (egui, ratatui, iced, Xilem)

These frameworks own the update loop. Either they redraw every frame (immediate-mode: egui, ratatui), or all state changes flow through a message → update → render cycle (TEA/MVU: iced, Xilem). In both cases the framework decides when to read state, and there is no place to attach subscriptions in a natural way.

**Persistent-only mode** is the right fit. Fields are plain Rust types, you mutate them in `update` or `show`, and call `save_lazy()` when done.

One caveat: persistent-only state does not observe external changes. If another thread, another process, or a manually edited file changes the underlying store while the app is running, the loaded struct will not update. If you need that, use reactive mode and call `.get()` at the start of each frame or update cycle — the framework loop naturally polls the latest value.

- [egui](./egui.md)
- [iced](./iced.md)
- [ratatui](./ratatui.md)

### Property bindings (Slint)

Slint owns a set of typed properties on the UI side. The Rust side pushes values into them. There is no shared signal graph — the bridge is one-directional: subscribe to a `Field<T>`, and in the callback push the new value into the corresponding Slint property.

**Reactive mode** is required.

- [Slint](./slint.md)

### Signal-based (Dioxus, Leptos, Yew)

These frameworks use fine-grained signals: components subscribe to sources and re-render only when their inputs change. The integration pattern is the same across all three — subscribe to a `Field<T>` and write the new value into a framework signal. Components read the signal, not the `Field<T>` directly.

The signals differ in ownership model: Dioxus and Leptos use arena-allocated `Copy` handles; Yew uses RC-based handles that are passed by clone. The bridge looks slightly different in each case but the concept is identical.

**Reactive mode** is required.

- [Dioxus](./dioxus.md)
- [Leptos](./leptos.md)
- [Yew](./yew.md)

### Webview bridge (Tauri)

Tauri splits the application into a Rust backend and a frontend communicating over commands and events. `amethystate` provides a dedicated plugin that handles this boundary — state is loaded on the Rust side, and generated bindings expose it to the frontend. Both TypeScript and Rust frontend clients are supported.

- [Tauri](./tauri.md)

### GPUI

GPUI uses an entity model with deferred notification. Mutations are applied inside entity update closures, and the framework notifies dependents after the closure returns. This does not compose directly with synchronous `Field<T>` subscriptions, so the bridge goes through an async channel: a background task holds a `WeakEntity`, subscribes to reactive fields, and schedules updates via `AsyncApp`.

**Reactive mode** is required.

- [GPUI](./gpui.md)