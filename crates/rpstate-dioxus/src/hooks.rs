use crate::{MapSignal, RpStateDioxus};
use crate::{PIPELINE_ARENA, RpStateDioxusNested};
use dioxus::core::{Callback, spawn, use_hook};
use dioxus::hooks::{try_use_context, use_callback, use_context, use_context_provider, use_signal};
use dioxus::prelude::{ReadSignal, WritableExt};
use rpstate::{AccessMode, DefaultStore, MapChange, Pipeline, Store};
use rpstate_arena::{
    Arena, FieldHandle, MapHandle, PipelineHandle, WritableHandle, WritableMapHandle,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type Handle<S> = <S as RpStateDioxusNested>::Handle;

pub fn use_rpstate<S>() -> S::Handle
where
    S: RpStateDioxus + 'static,
{
    if let Some(handle) = try_use_context::<S::Handle>() {
        return handle;
    }

    let store = use_context::<Arc<DefaultStore>>();
    let arena = use_context::<Arena>();

    let handle = use_hook(|| {
        let state = S::load_slice(&store).expect(
            "rpstate-dioxus: Failed to load state slice. \
             Ensure that the store path is writable, \
             and the database is not locked by another process.",
        );
        state.register_dioxus(&arena)
    });

    use_context_provider(|| handle);

    handle
}

pub fn use_field<T, S>(handle: WritableHandle<T, S>) -> (ReadSignal<T>, Callback<T>)
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
    S: Store,
{
    let arena = use_context::<Arena>();
    let mut signal = use_signal(|| arena.get_field(handle));

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();

        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });

        tx
    });

    let arena_clone = arena.clone();

    use_hook(move || {
        let sub = arena.subscribe_field(handle, move |val| {
            let _ = tx.send(val);
        });
        Arc::new(sub)
    });

    let setter = use_callback(move |val: T| {
        let _ = arena_clone.set_field(handle, val);
    });

    (signal.into(), setter)
}

pub fn use_read_only_field<T, S, M>(handle: FieldHandle<T, S, M>) -> ReadSignal<T>
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
    S: Store,
    M: AccessMode,
{
    let arena = use_context::<Arena>();
    let mut signal = use_signal(|| arena.get_field(handle));

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();

        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });

        tx
    });

    use_hook(move || {
        let sub = arena.subscribe_field(handle, move |val| {
            let _ = tx.send(val);
        });
        Arc::new(sub)
    });

    signal.into()
}

pub fn use_pipeline<T, F>(f: F) -> ReadSignal<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
    F: FnOnce() -> Pipeline<T> + 'static,
{
    let arena = use_context::<Arena>();

    let handle = use_hook(|| {
        PIPELINE_ARENA.with(|a| *a.borrow_mut() = Some(arena.clone()));
        let pipeline = f();
        PIPELINE_ARENA.with(|a| *a.borrow_mut() = None);

        arena.register_pipeline(pipeline)
    });

    let arena_clone = arena.clone();
    use_hook(move || {
        struct Guard<T: 'static> {
            arena: Arena,
            handle: PipelineHandle<T>,
        }
        impl<T: 'static> Drop for Guard<T> {
            fn drop(&mut self) {
                self.arena.remove_pipeline(self.handle);
            }
        }
        Arc::new(Guard {
            arena: arena_clone,
            handle,
        })
    });

    let mut signal = use_signal(|| arena.get_pipeline(handle));

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();
        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });
        tx
    });

    use_hook(move || {
        let sub = arena.subscribe_pipeline(handle, move |val| {
            let _ = tx.send(val);
        });
        Arc::new(sub)
    });

    signal.into()
}

pub fn use_map<K, V, S>(handle: WritableMapHandle<K, V, S>) -> MapSignal<K, V>
where
    K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + PartialEq + 'static,
    V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
    S: Store,
{
    let arena = use_context::<Arena>();
    let mut signal = use_signal(|| {
        arena
            .get_map_entries(handle)
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<K, V>>()
    });

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<HashMap<K, V>>();
        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });
        tx
    });

    let arena_sub = arena.clone();
    use_hook(move || {
        let arena_sub_sub = arena_sub.clone();
        let sub = arena_sub.subscribe_map_any(handle, move |_| {
            let entries = arena_sub_sub
                .get_map_entries(handle)
                .unwrap_or_default()
                .into_iter()
                .collect();
            let _ = tx.send(entries);
        });
        Arc::new(sub)
    });

    let arena_set = arena.clone();
    let _set = use_callback(move |(key, val)| {
        let _ = arena_set.set_map_entry(handle, key, val);
    });

    let arena_set_or_create = arena.clone();
    let _set_or_create = use_callback(move |(key, val)| {
        let _ = arena_set_or_create.set_map_entry(handle, key, val);
    });

    let arena_remove = arena.clone();
    let _remove = use_callback(move |key| {
        let _ = arena_remove.remove_map_entry(handle, key);
    });

    let arena_clear = arena.clone();
    let _clear = use_callback(move |_| {
        let _ = arena_clear.clear_map(handle);
    });

    MapSignal {
        entries: signal.into(),
        _set,
        _set_or_create,
        _remove,
        _clear,
    }
}

pub fn use_map_entry<K, V, S, M>(handle: MapHandle<K, V, S, M>, key: K) -> ReadSignal<Option<V>>
where
    K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + PartialEq + 'static,
    V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
    S: Store,
    M: AccessMode,
{
    let arena = use_context::<Arena>();
    let mut signal = use_signal(|| arena.get_map_entry(handle, &key).ok().flatten());

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<Option<V>>();

        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });

        tx
    });

    let key_clone = key.clone();
    use_hook(move || {
        let sub = arena.subscribe_map_key(handle, key_clone, move |change| match change {
            MapChange::Insert { value, .. }
            | MapChange::Update {
                new_value: value, ..
            } => {
                let _ = tx.send(Some(value.clone()));
            }
            MapChange::Remove { .. } | MapChange::Clear => {
                let _ = tx.send(None);
            }
        });
        Arc::new(sub)
    });

    signal.into()
}

pub fn use_map_subscribe_any<K, V, S, M, F>(handle: MapHandle<K, V, S, M>, callback: F)
where
    K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
    S: Store,
    M: AccessMode,
    F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
{
    let arena = use_context::<Arena>();
    use_hook(move || {
        let sub = arena.subscribe_map_any(handle, callback);
        Arc::new(sub)
    });
}

pub fn use_map_subscribe_key<K, V, S, M, F>(handle: MapHandle<K, V, S, M>, key: K, callback: F)
where
    K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
    V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
    S: Store,
    M: AccessMode,
    F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
{
    let arena = use_context::<Arena>();
    use_hook(move || {
        let sub = arena.subscribe_map_key(handle, key, callback);
        Arc::new(sub)
    });
}
