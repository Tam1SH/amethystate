use crate::async_impl::{AsyncSubscriptionBackend, SubscriptionHandle};
use crate::primitives::field_core::FieldValue;
use crate::{Change, FieldCore, InterceptDisposer, SignalSubscription};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

pub struct Field<T, B> {
    pub core: FieldCore<T>,
    pub path: Arc<str>,
    _subscription: Arc<Mutex<Option<SubscriptionHandle>>>,
    backend: B,
}

impl<T, B> Clone for Field<T, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            path: self.path.clone(),
            _subscription: self._subscription.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<T, B> PartialEq for Field<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && Arc::ptr_eq(&self.core.signal.value, &other.core.signal.value)
    }
}

impl<T, B> Eq for Field<T, B> {}

impl<T, B> std::fmt::Debug for Field<T, B>
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

impl<T, B> Field<T, B>
where
    T: FieldValue,
    B: AsyncSubscriptionBackend,
{
    pub fn new(key: impl Into<Arc<str>>, initial_value: T) -> Self
    where
        B: Default,
    {
        Self::new_with_backend(key, initial_value, B::default())
    }

    pub fn new_with_backend(key: impl Into<Arc<str>>, initial_value: T, backend: B) -> Self {
        let path = key.into();
        let core = FieldCore::new(initial_value);
        let subscription = backend.subscribe_field(path.clone(), core.clone());

        Self {
            core,
            path,
            _subscription: Arc::new(Mutex::new(subscription)),
            backend,
        }
    }

    pub fn value(&self) -> T {
        self.core.get()
    }

    pub async fn get(&self) -> Result<T, B::Error> {
        self.backend
            .get(&self.path)
            .await?
            .ok_or_else(|| self.backend.key_not_found(self.path.to_string()))
    }

    pub async fn set(&self, value: T) -> Result<(), B::Error> {
        crate::field_set_async(&self.backend, &self.core, self.path.clone(), value, true).await
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
}

impl<T, B> crate::pipeline::Reactive<T> for Field<T, B>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    B: AsyncSubscriptionBackend,
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

impl<T, B> Drop for Field<T, B> {
    fn drop(&mut self) {
        if Arc::strong_count(&self._subscription) == 1 {
            let _ = self._subscription.lock().unwrap().take();
        }
    }
}
