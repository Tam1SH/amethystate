use rpstate::{DefaultStore, ReadOnlyMode, WritableMode};
use slotmap::DefaultKey;
use std::marker::PhantomData;

pub struct FieldHandle<T, S, M> {
    pub key: DefaultKey,
    pub _marker: PhantomData<(T, S, M)>,
}

impl<T, S, M> Copy for FieldHandle<T, S, M> {}
impl<T, S, M> Clone for FieldHandle<T, S, M> {
    fn clone(&self) -> Self {
        *self
    }
}

pub type ReadOnlyHandle<T, S = DefaultStore> = FieldHandle<T, S, ReadOnlyMode>;
pub type WritableHandle<T, S = DefaultStore> = FieldHandle<T, S, WritableMode>;

pub struct PipelineHandle<T> {
    pub key: DefaultKey,
    pub _marker: PhantomData<T>,
}

impl<T> Copy for PipelineHandle<T> {}
impl<T> Clone for PipelineHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct MapHandle<K, V, S, M> {
    pub key: DefaultKey,
    pub _marker: PhantomData<(K, V, S, M)>,
}

pub type ReadOnlyMapHandle<K, V, S = DefaultStore> = MapHandle<K, V, S, ReadOnlyMode>;
pub type WritableMapHandle<K, V, S = DefaultStore> = MapHandle<K, V, S, WritableMode>;

impl<K, V, S, M> Copy for MapHandle<K, V, S, M> {}
impl<K, V, S, M> Clone for MapHandle<K, V, S, M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, S, M> PartialEq for FieldHandle<T, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K, V, S, M> PartialEq for MapHandle<K, V, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> PartialEq for PipelineHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
