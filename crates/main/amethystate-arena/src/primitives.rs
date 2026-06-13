use amethystate::{ReadOnlyMode, WritableMode};
use slotmap::DefaultKey;
use std::marker::PhantomData;

pub struct FieldHandle<T, M = ReadOnlyMode> {
    pub key: DefaultKey,
    pub _marker: PhantomData<(T, M)>,
}

impl<T, M> Copy for FieldHandle<T, M> {}
impl<T, M> Clone for FieldHandle<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}

pub type ReadOnlyHandle<T> = FieldHandle<T, ReadOnlyMode>;
pub type WritableHandle<T> = FieldHandle<T, WritableMode>;

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

pub struct MapHandle<K, V, M = ReadOnlyMode> {
    pub key: DefaultKey,
    pub _marker: PhantomData<(K, V, M)>,
}

pub type ReadOnlyMapHandle<K, V> = MapHandle<K, V, ReadOnlyMode>;
pub type WritableMapHandle<K, V> = MapHandle<K, V, WritableMode>;

impl<K, V, M> Copy for MapHandle<K, V, M> {}
impl<K, V, M> Clone for MapHandle<K, V, M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, M> PartialEq for FieldHandle<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K, V, M> PartialEq for MapHandle<K, V, M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> PartialEq for PipelineHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
