use crate::primitives::field_core::FieldValue;
use crate::primitives::map_core::{ReactiveMapKey, ReactiveMapValue};
use crate::{FieldCore, ReactiveMapCore, RpBackendAsync};
use serde::Deserialize;
use std::sync::Arc;

pub struct SubscriptionHandle {
    cleanup: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl SubscriptionHandle {
    pub fn new(cleanup: impl FnOnce() + Send + 'static) -> Self {
        Self {
            cleanup: Some(Box::new(cleanup)),
        }
    }

    pub fn noop() -> Self {
        Self { cleanup: None }
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

pub trait AsyncSubscriptionBackend: RpBackendAsync + Clone + Send + Sync + 'static {
    fn subscribe_field<T>(&self, path: Arc<str>, core: FieldCore<T>) -> Option<SubscriptionHandle>
    where
        T: FieldValue;

    fn subscribe_map<K, V>(
        &self,
        path: Arc<str>,
        core: ReactiveMapCore<K, V>,
    ) -> Option<SubscriptionHandle>
    where
        K: ReactiveMapKey + for<'de> Deserialize<'de>,
        V: ReactiveMapValue;
}
