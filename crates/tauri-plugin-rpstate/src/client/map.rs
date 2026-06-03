use futures::future::AbortHandle;
use futures::stream::StreamExt;
use rpstate_core::{InterceptDisposer, MapChange, ReactiveMapCore, SignalSubscription};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri_sys::event::Event;

pub struct ReactiveMap<K, V> {
    pub core: ReactiveMapCore<K, V>,
    pub prefix: Arc<str>,
    _unlisten: Arc<Mutex<Option<AbortHandle>>>,
}

impl<K, V> PartialEq for ReactiveMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix && Arc::ptr_eq(&self.core.next_id, &other.core.next_id)
    }
}

impl<K, V> std::fmt::Debug for ReactiveMap<K, V>
where
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ReactiveMap");
        d.field("prefix", &self.prefix);

        if let Ok(keys) = self.core.known_keys.try_lock() {
            d.field("known_keys", &*keys);
        } else {
            d.field("known_keys", &"<locked>");
        }

        d.field("core", &self.core).finish()
    }
}

impl<K, V> Eq for ReactiveMap<K, V> {}

impl<K, V> Clone for ReactiveMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            prefix: self.prefix.clone(),
            _unlisten: self._unlisten.clone(),
        }
    }
}

impl<K, V> ReactiveMap<K, V>
where
    K: FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static + for<'de> Deserialize<'de>,
    V: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
{
    pub fn new(prefix: impl Into<Arc<str>>, initial_values: HashMap<K, V>) -> Self {
        let prefix = prefix.into();
        let core = ReactiveMapCore::new();
        {
            let mut keys = core.known_keys.lock().unwrap();
            for k in initial_values.keys() {
                keys.insert(k.clone());
            }
        }

        let mut map = Self {
            core,
            prefix,
            _unlisten: Arc::new(Mutex::new(None)),
        };
        map.init_subscription();
        map
    }

    pub async fn get(&self, key: &K) -> Result<Option<V>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            #[derive(Serialize)]
            struct GetArgs {
                key: String,
            }

            let full_key = format!("{}.{}", self.prefix, key);
            let raw_res = tauri_sys::core::invoke_result::<Option<serde_json::Value>, String>(
                "plugin:rpstate|rpstate_get",
                &GetArgs { key: full_key },
            )
            .await
            .map_err(|e| e.to_string())?;

            let value: Option<V> = raw_res
                .map(|val| serde_json::from_value(val).map_err(|e| e.to_string()))
                .transpose()?;

            Ok(value)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Err("WASM target only".to_string())
        }
    }

    pub async fn remove(&self, key: K) -> Result<Option<V>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            let exists = self.core.known_keys.lock().unwrap().contains(&key);

            if !exists {
                return Ok(None);
            }

            let old_value = self.get(&key).await?;
            if let Some(old) = old_value {
                let change = MapChange::Remove {
                    key: key.clone(),
                    old_value: old.clone(),
                };

                let processed = self
                    .core
                    .run_interceptors(self.prefix.clone(), change)
                    .map_err(|e| e.to_string())?;

                #[derive(Serialize)]
                struct DeleteArgs {
                    key: String,
                }

                let full_key = format!("{}.{}", self.prefix, key);
                tauri_sys::core::invoke_result::<(), String>(
                    "plugin:rpstate|rpstate_delete",
                    &DeleteArgs { key: full_key },
                )
                .await
                .map_err(|e| e.to_string())?;

                self.core.notify(&processed);
                Ok(Some(old))
            } else {
                self.core.known_keys.lock().unwrap().remove(&key);
                Ok(None)
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Err("WASM target only".to_string())
        }
    }

    pub async fn entries(&self) -> Result<HashMap<K, V>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            #[derive(Serialize)]
            struct PrefixArgs<'a> {
                prefix: &'a str,
            }

            let raw: HashMap<String, serde_json::Value> =
                tauri_sys::core::invoke_result::<_, String>(
                    "plugin:rpstate|rpstate_get_prefix",
                    &PrefixArgs {
                        prefix: &format!("{}.", self.prefix),
                    },
                )
                .await?;

            let prefix_dot = format!("{}.", self.prefix);
            let mut result = HashMap::new();
            for (k, v) in raw {
                if let Some(sub_key) = k.strip_prefix(&prefix_dot)
                    && let Ok(parsed_k) = K::from_str(sub_key)
                    && let Ok(parsed_v) = serde_json::from_value::<V>(v)
                {
                    result.insert(parsed_k, parsed_v);
                }
            }
            Ok(result)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err("WASM target only".to_string())
        }
    }

    pub async fn set(&self, key: K, value: &V) -> Result<(), String> {
        let exists = self.core.known_keys.lock().unwrap().contains(&key);
        let change = if exists {
            MapChange::Update {
                key: key.clone(),
                old_value: V::default(),
                new_value: value.clone(),
            }
        } else {
            MapChange::Insert {
                key: key.clone(),
                value: value.clone(),
            }
        };

        let processed = self
            .core
            .run_interceptors(self.prefix.clone(), change)
            .map_err(|e| e.to_string())?;

        #[cfg(target_arch = "wasm32")]
        {
            #[derive(Serialize)]
            struct SetArgs<'a, Val> {
                key: String,
                value: &'a Val,
            }

            let full_key = format!("{}.{}", self.prefix, key);
            tauri_sys::core::invoke_result::<(), String>(
                "plugin:rpstate|rpstate_set",
                &SetArgs {
                    key: full_key,
                    value,
                },
            )
            .await
            .map_err(|e| e.to_string())?;
            self.core.known_keys.lock().unwrap().insert(key.clone());
            self.core.notify(&processed);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = processed;
            Err("WASM target only".to_string())
        }
    }

    pub fn subscribe_any<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.core.subscribe_any(callback)
    }

    pub fn subscribe_key<F>(&self, key: K, callback: F) -> SignalSubscription
    where
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.core.subscribe_key(key, callback)
    }

    pub fn intercept<F>(&self, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        self.core.intercept(self.prefix.clone(), callback)
    }

    pub fn intercept_key<F>(&self, key: K, callback: F) -> InterceptDisposer
    where
        F: Fn(MapChange<K, V>) -> Option<MapChange<K, V>> + Send + Sync + 'static,
    {
        self.core.intercept_key(key, callback)
    }

    fn init_subscription(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let core = self.core.clone();
            let prefix = self.prefix.clone();
            let event_channel = format!("rpstate://{}", prefix.replace('.', ":"));

            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            *self._unlisten.lock().unwrap() = Some(abort_handle);

            wasm_bindgen_futures::spawn_local(async move {
                #[derive(Serialize, Deserialize)]
                struct SubArgs<'a> {
                    key: &'a str,
                }

                let _: Result<(), String> = tauri_sys::core::invoke_result(
                    "plugin:rpstate|rpstate_subscribe",
                    &SubArgs { key: &prefix },
                )
                .await;

                if let Ok(stream) =
                    tauri_sys::event::listen::<serde_json::Value>(&event_channel).await
                {
                    let mut aborted_stream =
                        futures::stream::Abortable::new(stream, abort_registration);
                    while let Some(Event { payload, .. }) = aborted_stream.next().await {
                        if let Ok(change) = serde_json::from_value::<MapChangeHelper<K, V>>(payload)
                        {
                            let core_change = change.into_core();

                            {
                                let mut keys = core.known_keys.lock().unwrap();
                                match &core_change {
                                    MapChange::Insert { key, .. } => {
                                        keys.insert(key.clone());
                                    }
                                    MapChange::Remove { key, .. } => {
                                        keys.remove(key);
                                    }
                                    MapChange::Clear => {
                                        keys.clear();
                                    }
                                    _ => {}
                                }
                            }

                            core.notify(&core_change);
                        }
                    }
                }
            });
        }
    }
}

impl<K, V> Drop for ReactiveMap<K, V> {
    fn drop(&mut self) {
        if let Some(handle) = self._unlisten.lock().unwrap().take() {
            handle.abort();
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum MapChangeHelper<K, V> {
    Insert {
        key: K,
        value: V,
    },
    Update {
        key: K,
        #[serde(rename = "oldValue")]
        old_value: V,
        #[serde(rename = "newValue")]
        new_value: V,
    },
    Remove {
        key: K,
        #[serde(rename = "oldValue")]
        old_value: V,
    },
    Clear,
}

impl<K, V> MapChangeHelper<K, V> {
    fn into_core(self) -> MapChange<K, V> {
        match self {
            MapChangeHelper::Insert { key, value } => MapChange::Insert { key, value },
            MapChangeHelper::Update {
                key,
                old_value,
                new_value,
            } => MapChange::Update {
                key,
                old_value,
                new_value,
            },
            MapChangeHelper::Remove { key, old_value } => MapChange::Remove { key, old_value },
            MapChangeHelper::Clear => MapChange::Clear,
        }
    }
}
