use crate::{LeptosBackend, MapSignal};
use leptos::callback::Callback;
use leptos::prelude::*;
use rpstate::{AccessMode, MapChange, Pipeline, ReactiveMapKey, ReactiveMapValue};
use rpstate_arena::{
    DefaultArena, FieldHandle, MapHandle, PIPELINE_ARENA, RpStateFramework, RpStateFrameworkNested,
    WritableHandle, WritableMapHandle,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub type Handle<S> = <S as RpStateFrameworkNested>::Handle;

pub fn use_rpstate<S>() -> S::Handle
where
    S: RpStateFramework<LeptosBackend> + 'static,
{
    if let Some(handle) = use_context::<S::Handle>() {
        return handle;
    }

    #[cfg(target_arch = "wasm32")]
    {
        panic!(
            "rpstate-leptos: State slice '{}' was not initialized! \
             Make sure to call `use_init_rpstate` at the root/parent component before accessing it.",
            std::any::type_name::<S>()
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let store = use_context::<rpstate::DefaultStore>()
            .expect("rpstate-leptos: Store context not found");
        let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena context not found");

        let state = S::load_slice(&store).unwrap_or_else(|err| {
            panic!(
                "rpstate-leptos: Failed to load state slice. \
                 Ensure that the store path is writable, \
                 and the database is not locked by another process.\n\
                 Error details: {err}",
            );
        });
        let handle = state.register(&arena);

        provide_context(handle);

        handle
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn use_init_rpstate<S>() -> Option<S::Handle>
where
    S: RpStateFramework<LeptosBackend> + 'static,
{
    if let Some(handle) = use_context::<S::Handle>() {
        return Some(handle);
    }

    let store =
        use_context::<rpstate::DefaultStore>().expect("rpstate-leptos: Store context not found");
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena context not found");

    let state = S::load_slice(&store).unwrap_or_else(|err| {
        panic!("rpstate-leptos: Failed to load state slice: {err}");
    });
    let handle = state.register(&arena);

    provide_context(handle);

    Some(handle)
}

#[cfg(target_arch = "wasm32")]
pub fn use_init_rpstate<S>() -> Option<S::Handle>
where
    S: RpStateFramework<LeptosBackend>
        + rpstate::client::RpStateSliceAsync<rpstate_tauri::TauriBackend>,
    <S as rpstate::client::RpStateSliceAsync<rpstate_tauri::TauriBackend>>::Error: std::fmt::Debug,
{
    if let Some(handle) = use_context::<S::Handle>() {
        return Some(handle);
    }

    let backend = use_context::<rpstate_tauri::TauriBackend>()
        .expect("rpstate-leptos: Store context not found");

    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena context not found");

    let resource = LocalResource::new(move || {
        let backend = backend.clone();
        let arena = arena.clone();
        async move {
            let state = S::load_async(&backend).await.unwrap_or_else(|err| {
                panic!("rpstate-leptos: Failed to load state slice asynchronously: {err:?}");
            });
            state.register(&arena)
        }
    });

    let handle = resource.get();
    if let Some(h) = handle {
        provide_context(h);
    }
    handle
}

pub fn use_field<T>(handle: WritableHandle<T>) -> (ReadSignal<T>, SignalSetter<T>)
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let (signal, set_signal) = signal(arena.get_field(handle));

    let sub = arena.subscribe_field(handle, move |val| {
        set_signal.set(val);
    });
    on_cleanup(move || drop(sub));

    let arena_clone = arena.clone();
    let setter = SignalSetter::map(move |val: T| {
        let _ = arena_clone.set_field(handle, val);
    });

    (signal, setter)
}

pub fn use_read_only_field<T, M>(handle: FieldHandle<T, M>) -> ReadSignal<T>
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
    M: AccessMode,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let (signal, set_signal) = signal(arena.get_field(handle));

    let sub = arena.subscribe_field(handle, move |val| {
        set_signal.set(val);
    });
    on_cleanup(move || drop(sub));

    signal
}

pub fn use_pipeline<T, F>(f: F) -> ReadSignal<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
    F: FnOnce() -> Pipeline<T> + 'static,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");

    let handle = PIPELINE_ARENA.with(|a| {
        *a.borrow_mut() = Some(arena.clone());
        let pipeline = f();
        *a.borrow_mut() = None;

        arena.register_pipeline(pipeline)
    });

    let arena_clone = arena.clone();
    on_cleanup(move || {
        arena_clone.remove_pipeline(handle);
    });

    let (signal, set_signal) = signal(arena.get_pipeline(handle));

    let sub = arena.subscribe_pipeline(handle, move |val| {
        set_signal.set(val);
    });
    on_cleanup(move || drop(sub));

    signal
}

pub fn use_map<K, V>(handle: WritableMapHandle<K, V>) -> MapSignal<K, V>
where
    K: ReactiveMapKey + for<'de> serde::Deserialize<'de>,
    V: ReactiveMapValue,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let (signal, set_signal) = signal(
        arena
            .get_map_entries(handle)
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<K, V>>(),
    );

    let arena_sub = arena.clone();
    let sub = arena.subscribe_map_any(handle, move |_| {
        let entries = arena_sub
            .get_map_entries(handle)
            .unwrap_or_default()
            .into_iter()
            .collect();
        set_signal.set(entries);
    });
    on_cleanup(move || drop(sub));

    let arena_set = arena.clone();
    let _set = Callback::new(move |(key, val)| {
        let _ = arena_set.set_map_entry(handle, key, val);
    });

    let arena_set_or_create = arena.clone();
    let _set_or_create = Callback::new(move |(key, val)| {
        let _ = arena_set_or_create.set_map_entry(handle, key, val);
    });

    let arena_remove = arena.clone();
    let _remove = Callback::new(move |key| {
        let _ = arena_remove.remove_map_entry(handle, key);
    });

    let arena_clear = arena.clone();
    let _clear = Callback::new(move |_| {
        let _ = arena_clear.clear_map(handle);
    });

    MapSignal::new(signal, _set, _set_or_create, _remove, _clear)
}

pub fn use_map_entry<K, V, M>(handle: MapHandle<K, V, M>, key: K) -> ReadSignal<Option<V>>
where
    K: ReactiveMapKey + for<'de> serde::Deserialize<'de>,
    V: ReactiveMapValue,
    M: AccessMode,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let (signal, set_signal) = signal(arena.get_map_entry(handle, &key).ok().flatten());

    let key_clone = key.clone();
    let sub = arena.subscribe_map_key(handle, key_clone, move |change| match change {
        MapChange::Insert { value, .. }
        | MapChange::Update {
            new_value: value, ..
        } => {
            set_signal.set(Some(value.clone()));
        }
        MapChange::Remove { .. } | MapChange::Clear => {
            set_signal.set(None);
        }
    });
    on_cleanup(move || drop(sub));

    signal
}

pub fn use_map_subscribe_any<K, V, M, F>(handle: MapHandle<K, V, M>, callback: F)
where
    K: ReactiveMapKey,
    V: ReactiveMapValue,
    M: AccessMode,
    F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let sub = arena.subscribe_map_any(handle, callback);
    on_cleanup(move || drop(sub));
}

pub fn use_map_subscribe_key<K, V, M, F>(handle: MapHandle<K, V, M>, key: K, callback: F)
where
    K: ReactiveMapKey,
    V: ReactiveMapValue,
    M: AccessMode,
    F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
{
    let arena = use_context::<DefaultArena>().expect("rpstate-leptos: Arena not found");
    let sub = arena.subscribe_map_key(handle, key, callback);
    on_cleanup(move || drop(sub));
}
