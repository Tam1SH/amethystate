use crate::RpBackendAsync as RpBackend;
use crate::primitives::map_core::{ReactiveMapKey, ReactiveMapValue};
use crate::{MapChange, ReactiveMapCore, map_apply_remote_change};

use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

pub async fn map_get_async<B, K, V>(backend: &B, path: &str, key: &K) -> Result<Option<V>, B::Error>
where
    B: RpBackend,
    K: Display,
    V: DeserializeOwned,
{
    backend.get(&format!("{}.{}", path, key)).await
}

pub async fn map_contains_key_async<B, K, V>(
    backend: &B,
    path: &str,
    key: &K,
) -> Result<bool, B::Error>
where
    B: RpBackend,
    K: Display,
    V: DeserializeOwned,
{
    map_get_async::<B, K, V>(backend, path, key)
        .await
        .map(|v| v.is_some())
}

pub async fn map_entries_async<B, K, V>(backend: &B, path: &str) -> Result<Vec<(K, V)>, B::Error>
where
    B: RpBackend,
    K: FromStr,
    V: DeserializeOwned + Default,
{
    let prefix = format!("{}.", path);
    let kvs = backend.scan_prefix(&prefix).await?;
    let mut results = Vec::new();

    for (full_path, raw) in kvs {
        if let Some(key_str) = full_path.strip_prefix(&prefix)
            && let Ok(k) = K::from_str(key_str)
            && let Ok(v) = backend.decode::<V>(&raw)
        {
            results.push((k, v));
        }
    }

    Ok(results)
}

pub async fn map_len_async<B>(backend: &B, path: &str) -> Result<usize, B::Error>
where
    B: RpBackend,
{
    backend
        .scan_prefix(&format!("{}.", path))
        .await
        .map(|kvs| kvs.len())
}

pub async fn map_set_existing_async<B, K, V>(
    backend: &B,
    core: &ReactiveMapCore<K, V>,
    path: Arc<str>,
    key: K,
    value: &V,
    notify_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackend,
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    let full_path = format!("{}.{}", path, key);
    let old_value = match backend.get::<V>(&full_path).await? {
        Some(old_value) => old_value,
        None => return Err(backend.key_not_found(key.to_string())),
    };

    let change = MapChange::Update {
        key,
        old_value,
        new_value: value.clone(),
    };

    map_apply_change_async(backend, core, path, change, notify_after_commit).await
}

pub async fn map_set_or_create_async<B, K, V>(
    backend: &B,
    core: &ReactiveMapCore<K, V>,
    path: Arc<str>,
    key: K,
    value: &V,
    notify_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackend,
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    let full_path = format!("{}.{}", path, key);
    let old_value = backend.get::<V>(&full_path).await?;
    let change = if let Some(old_value) = old_value {
        MapChange::Update {
            key,
            old_value,
            new_value: value.clone(),
        }
    } else {
        MapChange::Insert {
            key,
            value: value.clone(),
        }
    };

    map_apply_change_async(backend, core, path, change, notify_after_commit).await
}

pub async fn map_remove_async<B, K, V>(
    backend: &B,
    core: &ReactiveMapCore<K, V>,
    path: Arc<str>,
    key: K,
    notify_after_commit: bool,
) -> Result<Option<V>, B::Error>
where
    B: RpBackend,
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    let exists = core.known_keys.lock().unwrap().contains(&key);
    if !exists {
        return Ok(None);
    }

    let full_path = format!("{}.{}", path, key);
    let old_value = backend.get::<V>(&full_path).await?;
    if let Some(old_value) = old_value {
        let change = MapChange::Remove {
            key,
            old_value: old_value.clone(),
        };
        map_apply_change_async(backend, core, path, change, notify_after_commit).await?;
        Ok(Some(old_value))
    } else {
        core.known_keys.lock().unwrap().remove(&key);
        Ok(None)
    }
}

pub async fn map_clear_async<B, K, V>(
    backend: &B,
    core: &ReactiveMapCore<K, V>,
    path: Arc<str>,
    notify_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackend,
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    map_apply_change_async(backend, core, path, MapChange::Clear, notify_after_commit).await
}

pub async fn map_apply_change_async<B, K, V>(
    backend: &B,
    core: &ReactiveMapCore<K, V>,
    path: Arc<str>,
    change: MapChange<K, V>,
    notify_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackend,
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    let context_path: Arc<str> = match change.key() {
        Some(key) => format!("{}.{}", path, key).into(),
        None => path.clone(),
    };

    let processed = core
        .run_interceptors(context_path, change)
        .map_err(|_| backend.intercepted())?;

    match &processed {
        MapChange::Insert { key, value }
        | MapChange::Update {
            key,
            new_value: value,
            ..
        } => {
            backend.set(&format!("{}.{}", path, key), value).await?;
        }
        MapChange::Remove { key, .. } => {
            backend.delete(&format!("{}.{}", path, key)).await?;
        }
        MapChange::Clear => {
            let prefix = format!("{}.", path);
            let kvs = backend.scan_prefix(&prefix).await?;
            for (full_path, _) in kvs {
                backend.delete(&full_path).await?;
            }
        }
    }

    map_apply_remote_change(core, &processed);
    if notify_after_commit {
        core.notify(&processed);
    }

    Ok(())
}
