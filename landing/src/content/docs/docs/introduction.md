---
title: Introduction
---

`amethystate` is a persistent reactive state library for Rust GUI applications.

## What it does

### Persistence

State is stored automatically. Changing a field writes to an in-memory buffer and schedules a debounced flush to disk — there is no separate save layer to think about.

Two modes are available depending on what your framework expects:

- **Reactive mode** — fields are `Field<T>` handles with `.get()`, `.set()`, and `.subscribe()`. Writing a field fires subscriptions immediately and persists in the background.
- **Persistent-only mode** — fields are plain Rust types. Useful for frameworks that own their update loop, like iced. Persistence happens via explicit `.save()` or `.save_lazy()` calls.

### Reactivity

Reactive fields can be composed into derived pipelines — synchronous transformations that recompute automatically when any upstream field changes. Pipelines support `.map()`, `.filter_map()`, `.dedupe()`, and `.inspect()`.

Dynamic collections are handled by `ReactiveMap<K, V>`, where each entry is stored as an individual key in the database and changes can be observed per-key or for the entire map.

### Migrations

Schema evolution is built in. Structs are versioned, and migrations run automatically on startup.

If a field's type or name changes without a version bump, `amethystate` detects the discrepancy on startup and logs a warning with a diff of what changed. Startup is not blocked — the drift is reported, not enforced.