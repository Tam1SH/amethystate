mod pipeline_buidler;
mod primitives;

pub use pipeline_buidler::PipelineBuilder;
pub use primitives::*;

use parking_lot::RwLock;
use rpstate::{
    AccessMode, Field, MapChange, Pipeline, ReactiveMap, Result as RpResult, SignalSubscription,
    Store, WritableMode,
};
use serde::{Serialize, de::DeserializeOwned};
use slotmap::{DefaultKey, SlotMap};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

type ErasedItem = Box<dyn Any + Send + Sync>;

#[derive(Clone)]
pub struct Arena {
    storage: Arc<RwLock<SlotMap<DefaultKey, ErasedItem>>>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(SlotMap::new())),
        }
    }

    pub fn with_item<Item, R, F>(&self, key: DefaultKey, type_name: &str, f: F) -> R
    where
        Item: Any,
        F: FnOnce(&Item) -> R,
    {
        let storage = self.storage.read();
        let item = storage.get(key).unwrap_or_else(|| {
            panic!("rpstate-arena: Attempted to access a dropped {}", type_name)
        });
        let target = item
            .downcast_ref::<Item>()
            .unwrap_or_else(|| panic!("rpstate-arena: Type mismatch for {}", type_name));
        f(target)
    }

    pub fn register_field<T, S, M>(&self, field: Field<T, S, M>) -> FieldHandle<T, S, M>
    where
        T: Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        let key = self.storage.write().insert(Box::new(field));
        FieldHandle {
            key,
            _marker: PhantomData,
        }
    }

    pub fn get_field<T, S, M>(&self, handle: FieldHandle<T, S, M>) -> T
    where
        T: DeserializeOwned + Serialize + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        self.with_item::<Field<T, S, M>, _, _>(handle.key, "Field", |field| field.get())
    }

    pub fn set_field<T, S>(&self, handle: WritableHandle<T, S>, value: T) -> RpResult<()>
    where
        T: DeserializeOwned + Serialize + Clone + Send + Sync + 'static,
        S: Store,
    {
        self.with_item::<Field<T, S, WritableMode>, _, _>(handle.key, "Field", |field| {
            field.set(value)
        })
    }

    pub fn subscribe_field<T, S, M, F>(
        &self,
        handle: FieldHandle<T, S, M>,
        callback: F,
    ) -> SignalSubscription
    where
        T: DeserializeOwned + Serialize + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
        F: Fn(T) + Send + Sync + 'static,
    {
        self.with_item::<Field<T, S, M>, _, _>(handle.key, "Field", |field| {
            field.subscribe(callback)
        })
    }

    pub fn register_pipeline<T>(&self, pipeline: Pipeline<T>) -> PipelineHandle<T>
    where
        T: Send + Sync + 'static,
    {
        let key = self.storage.write().insert(Box::new(pipeline));
        PipelineHandle {
            key,
            _marker: PhantomData,
        }
    }

    pub fn get_pipeline<T>(&self, handle: PipelineHandle<T>) -> T
    where
        T: Clone + Send + Sync + 'static,
    {
        self.with_item::<Pipeline<T>, _, _>(handle.key, "Pipeline", |pipe| pipe.get())
    }

    pub fn subscribe_pipeline<T, F>(
        &self,
        handle: PipelineHandle<T>,
        callback: F,
    ) -> SignalSubscription
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        self.with_item::<Pipeline<T>, _, _>(handle.key, "Pipeline", |pipe| pipe.subscribe(callback))
    }

    pub fn register_map<S, M, K, V>(&self, map: ReactiveMap<K, V, S, M>) -> MapHandle<K, V, S, M>
    where
        K: Send + Sync + 'static,
        V: Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        let key = self.storage.write().insert(Box::new(map));
        MapHandle {
            key,
            _marker: PhantomData,
        }
    }

    pub fn get_map_entry<S, M, K, V>(
        &self,
        handle: MapHandle<K, V, S, M>,
        key: &K,
    ) -> RpResult<Option<V>>
    where
        K: std::fmt::Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: std::fmt::Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        self.with_item::<ReactiveMap<K, V, S, M>, _, _>(handle.key, "ReactiveMap", |map| {
            map.get(key)
        })
    }

    pub fn set_map_entry<S, K, V>(
        &self,
        handle: MapHandle<K, V, S, WritableMode>,
        key: K,
        value: V,
    ) -> RpResult<()>
    where
        K: std::fmt::Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: std::fmt::Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
    {
        self.with_item::<ReactiveMap<K, V, S, WritableMode>, _, _>(
            handle.key,
            "ReactiveMap",
            |map| map.set_or_create(key, &value),
        )
    }

    pub fn subscribe_map_any<S, M, K, V, F>(
        &self,
        handle: MapHandle<K, V, S, M>,
        callback: F,
    ) -> SignalSubscription
    where
        K: std::fmt::Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: std::fmt::Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.with_item::<ReactiveMap<K, V, S, M>, _, _>(handle.key, "ReactiveMap", |map| {
            map.subscribe_any(callback)
        })
    }

    pub fn subscribe_map_key<S, M, K, V, F>(
        &self,
        handle: MapHandle<K, V, S, M>,
        key: K,
        callback: F,
    ) -> SignalSubscription
    where
        K: std::fmt::Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: std::fmt::Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
        F: Fn(&MapChange<K, V>) + Send + Sync + 'static,
    {
        self.with_item::<ReactiveMap<K, V, S, M>, _, _>(handle.key, "ReactiveMap", |map| {
            map.subscribe_key(key, callback)
        })
    }

    pub fn remove_pipeline<T>(&self, handle: PipelineHandle<T>) {
        self.storage.write().remove(handle.key);
    }

    pub fn get_map_entries<K, V, S, M>(
        &self,
        handle: MapHandle<K, V, S, M>,
    ) -> rpstate::Result<Vec<(K, V)>>
    where
        K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        self.with_item::<ReactiveMap<K, V, S, M>, _, _>(handle.key, "ReactiveMap", |map| {
            map.entries().map(|vec| vec.into_iter().collect())
        })
    }
    pub fn remove_map_entry<K, V, S>(
        &self,
        handle: WritableMapHandle<K, V, S>,
        key: K,
    ) -> RpResult<Option<V>>
    where
        K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
    {
        self.with_item::<ReactiveMap<K, V, S, WritableMode>, _, _>(
            handle.key,
            "ReactiveMap",
            |map| map.remove(key),
        )
    }

    pub fn clear_map<K, V, S>(&self, handle: WritableMapHandle<K, V, S>) -> RpResult<()>
    where
        K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
    {
        self.with_item::<ReactiveMap<K, V, S, WritableMode>, _, _>(
            handle.key,
            "ReactiveMap",
            |map| map.clear(),
        )
    }
}

impl PartialEq for Arena {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }
}

impl Eq for Arena {}

#[cfg(test)]
mod tests {
    use super::*;
    use rpstate::StoreBuilder;

    fn unique_temp_dir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rpstate_arena_panic_test_{nanos}"))
    }

    #[test]
    #[should_panic(expected = "Attempted to access a dropped Field")]
    fn test_dropped_field_panic() {
        let arena = Arena::new();
        let fake_handle: FieldHandle<i32, rpstate::DefaultStore, rpstate::WritableMode> =
            FieldHandle {
                key: DefaultKey::default(),
                _marker: PhantomData,
            };
        arena.get_field(fake_handle);
    }

    #[test]
    #[should_panic(expected = "Type mismatch for Field")]
    fn test_field_type_mismatch_panic() {
        let temp_dir = unique_temp_dir();
        let store = StoreBuilder::new(&temp_dir).build().unwrap();
        let field: Field<i32, rpstate::DefaultStore, rpstate::WritableMode> =
            rpstate::store::field_with_path(&store, std::sync::Arc::from("test.int_field"), 42)
                .unwrap();

        let arena = Arena::new();
        let handle = arena.register_field(field);

        let bad_handle: FieldHandle<String, rpstate::DefaultStore, rpstate::WritableMode> =
            FieldHandle {
                key: handle.key,
                _marker: PhantomData,
            };

        arena.get_field(bad_handle);
    }
}
