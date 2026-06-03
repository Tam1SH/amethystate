use futures::StreamExt;
use futures::future::AbortHandle;
use rpstate_core::{Change, FieldCore, InterceptDisposer, SignalSubscription};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};
use tauri_sys::event::Event;

pub struct Field<T> {
    pub core: FieldCore<T>,
    pub path: Arc<str>,
    _unlisten: Arc<Mutex<Option<AbortHandle>>>,
}

impl<TValue> PartialEq for Field<TValue> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && Arc::ptr_eq(&self.core.signal.value, &other.core.signal.value)
    }
}

impl<T> std::fmt::Debug for Field<T>
where
    T: std::fmt::Debug + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("path", &self.path)
            .field("value", &self.core.get())
            .finish()
    }
}

impl<TValue> Eq for Field<TValue> {}

impl<T> Clone for Field<T> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            path: self.path.clone(),
            _unlisten: self._unlisten.clone(),
        }
    }
}

impl<T> Field<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(key: impl Into<Arc<str>>, initial_value: T) -> Self {
        let key_arc = key.into();
        let core = FieldCore::new(initial_value);

        let mut field = Self {
            core,
            path: key_arc,
            _unlisten: Arc::new(Mutex::new(None)),
        };
        field.init_subscription();
        field
    }

    pub fn value(&self) -> T {
        self.core.get()
    }

    pub async fn get(&self) -> Result<T, String> {
        #[cfg(target_arch = "wasm32")]
        {
            #[derive(Serialize)]
            struct GetArgs<'a> {
                key: &'a str,
            }

            tauri_sys::core::invoke_result(
                "plugin:rpstate|rpstate_get",
                &GetArgs { key: &self.path },
            )
            .await
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err("WASM target only".to_string())
        }
    }

    pub async fn set(&self, value: T) -> Result<(), String> {
        let change = self
            .core
            .run_interceptors(self.path.clone(), value)
            .map_err(|e| e.to_string())?;

        #[cfg(target_arch = "wasm32")]
        {
            #[derive(Serialize)]
            struct SetArgs<'a, Val> {
                key: &'a str,
                value: &'a Val,
            }

            tauri_sys::core::invoke_result::<(), String>(
                "plugin:rpstate|rpstate_set",
                &SetArgs {
                    key: &self.path,
                    value: &change.new_value,
                },
            )
            .await?;

            self.core.signal.set(change.new_value);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = change;
            Err("WASM target only".to_string())
        }
    }

    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.core.subscribe(callback)
    }

    pub fn intercept<F>(&self, callback: F) -> InterceptDisposer
    where
        F: Fn(Change<T>) -> Option<Change<T>> + Send + Sync + 'static,
    {
        self.core.intercept(self.path.clone(), callback)
    }

    fn init_subscription(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let core = self.core.clone();
            let key = self.path.clone();
            let event_channel = format!("rpstate://{}", key.replace('.', ":"));

            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            *self._unlisten.lock().unwrap() = Some(abort_handle);

            wasm_bindgen_futures::spawn_local(async move {
                #[derive(Serialize)]
                struct SubArgs<'a> {
                    key: &'a str,
                }

                let _ = tauri_sys::core::invoke_result::<(), String>(
                    "plugin:rpstate|rpstate_subscribe",
                    &SubArgs { key: &key },
                )
                .await;

                if let Ok(stream) = tauri_sys::event::listen::<T>(&event_channel).await {
                    let mut aborted_stream =
                        futures::stream::Abortable::new(stream, abort_registration);

                    while let Some(Event { payload, .. }) = aborted_stream.next().await {
                        core.signal.set(payload);
                    }
                }
            });
        }
    }
}

impl<T> rpstate_core::pipeline::Reactive<T> for Field<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn get(&self) -> T {
        self.core.get()
    }

    fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.core.subscribe(callback)
    }
}

impl<T> Drop for Field<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self._unlisten) == 1 {
            log::debug!("Field::drop LAST clone, aborting: {}", self.path);
            if let Some(handle) = self._unlisten.lock().unwrap().take() {
                handle.abort();
            }
        } else {
            log::debug!(
                "Field::drop clone, count={}, path={}",
                Arc::strong_count(&self._unlisten),
                self.path
            );
        }
    }
}
