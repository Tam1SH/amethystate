use crate::{Arena, FieldHandle, MapHandle};
use rpstate::{AccessMode, Field, ReactiveMap, Store};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

pub struct PipelineBuilder<'a> {
    arena: &'a Arena,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        Self { arena }
    }

    pub fn field<T, S, M>(&self, handle: FieldHandle<T, S, M>) -> Field<T, S, M>
    where
        T: Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        self.arena
            .with_item::<Field<T, S, M>, _, _>(handle.key, "Field", |f| f.clone())
    }

    pub fn map<K, V, S, M>(&self, handle: MapHandle<K, V, S, M>) -> ReactiveMap<K, V, S, M>
    where
        K: Debug + FromStr + Display + Clone + Hash + Eq + Send + Sync + 'static,
        V: Debug + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
        S: Store,
        M: AccessMode,
    {
        self.arena
            .with_item::<ReactiveMap<K, V, S, M>, _, _>(handle.key, "ReactiveMap", |m| m.clone())
    }
}
