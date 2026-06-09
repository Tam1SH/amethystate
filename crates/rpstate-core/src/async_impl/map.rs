use crate::async_impl::{AsyncSubscriptionBackend, SubscriptionHandle};
use crate::primitives::map_core::{ReactiveMapKey, ReactiveMapValue};
use crate::{InterceptDisposer, MapChange, ReactiveMapCore, SignalSubscription};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ReactiveMap<K, V, B> {
    pub core: ReactiveMapCore<K, V>,
    pub prefix: Arc<str>,
    _subscription: Arc<Mutex<Option<SubscriptionHandle>>>,
    backend: B,
}

impl<K, V, B> Clone for ReactiveMap<K, V, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            prefix: self.prefix.clone(),
            _subscription: self._subscription.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<K, V, B> PartialEq for ReactiveMap<K, V, B> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix && Arc::ptr_eq(&self.core.next_id, &other.core.next_id)
    }
}

impl<K, V, B> Eq for ReactiveMap<K, V, B> {}

impl<K, V, B> std::fmt::Debug for ReactiveMap<K, V, B>
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

impl<K, V, B> ReactiveMap<K, V, B>
where
    K: ReactiveMapKey + for<'de> Deserialize<'de>,
    V: ReactiveMapValue,
    B: AsyncSubscriptionBackend,
{
    pub fn new(prefix: impl Into<Arc<str>>, initial_values: HashMap<K, V>) -> Self
    where
        B: Default,
    {
        Self::new_with_backend(prefix, initial_values, B::default())
    }

    pub fn new_with_backend(
        prefix: impl Into<Arc<str>>,
        initial_values: HashMap<K, V>,
        backend: B,
    ) -> Self {
        let prefix = prefix.into();
        let core = ReactiveMapCore::new();
        {
            let mut keys = core.known_keys.lock().unwrap();
            for k in initial_values.keys() {
                keys.insert(k.clone());
            }
        }

        let subscription = backend.subscribe_map(prefix.clone(), core.clone());

        Self {
            core,
            prefix,
            _subscription: Arc::new(Mutex::new(subscription)),
            backend,
        }
    }

    pub async fn get(&self, key: &K) -> Result<Option<V>, B::Error> {
        crate::map_get_async(&self.backend, &self.prefix, key).await
    }

    pub async fn remove(&self, key: K) -> Result<Option<V>, B::Error> {
        crate::map_remove_async(&self.backend, &self.core, self.prefix.clone(), key, true).await
    }

    pub async fn entries(&self) -> Result<HashMap<K, V>, B::Error> {
        Ok(crate::map_entries_async(&self.backend, &self.prefix)
            .await?
            .into_iter()
            .collect())
    }

    pub async fn set(&self, key: K, value: &V) -> Result<(), B::Error> {
        crate::map_set_or_create_async(
            &self.backend,
            &self.core,
            self.prefix.clone(),
            key,
            value,
            true,
        )
        .await
    }

    pub async fn clear(&self) -> Result<(), B::Error> {
        crate::map_clear_async(&self.backend, &self.core, self.prefix.clone(), true).await
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
}
