mod hooks;
mod pipeline;
use dioxus::prelude::*;
pub use hooks::*;
pub use pipeline::*;
pub use rpstate::*;
pub use rpstate_arena::*;
pub use rpstate_macros_dioxus::rpstate_dioxus;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub trait RpStateDioxusNested {
    type Handle: Copy + 'static;
    fn register_dioxus(&self, arena: &Arena) -> Self::Handle;
}

pub trait RpStateDioxus: RpStateSlice + RpStateDioxusNested {}

impl<T: RpStateSlice + RpStateDioxusNested> RpStateDioxus for T {}

pub struct MapSignal<K: 'static, V: 'static> {
    pub entries: ReadSignal<HashMap<K, V>>,
    _set: Callback<(K, V)>,
    _set_or_create: Callback<(K, V)>,
    _remove: Callback<K>,
    _clear: Callback<()>,
}

impl<K: 'static, V: 'static> Copy for MapSignal<K, V> {}
impl<K: 'static, V: 'static> Clone for MapSignal<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> MapSignal<K, V>
where
    K: Clone + Hash + Eq + 'static,
    V: Clone + 'static,
{
    pub fn set(&self, key: K, val: V) {
        self._set.call((key, val));
    }

    pub fn set_or_create(&self, key: K, val: V) {
        self._set_or_create.call((key, val));
    }

    pub fn remove(&self, key: K) {
        self._remove.call(key);
    }

    pub fn clear(&self) {
        self._clear.call(());
    }

    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }
}

#[derive(Clone, Props)]
pub struct RpStateProviderProps {
    pub store: Arc<DefaultStore>,
    pub children: Element,
}

impl PartialEq for RpStateProviderProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.store, &other.store) && self.children == other.children
    }
}

#[allow(non_snake_case)]
pub fn RpStateProvider(props: RpStateProviderProps) -> Element {
    use_context_provider(Arena::new);
    use_context_provider(|| props.store.clone());

    rsx! { {props.children} }
}
