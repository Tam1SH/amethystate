mod hooks;

use dioxus::prelude::*;
pub use hooks::*;
pub use rpstate::*;
pub use rpstate_arena::*;

pub struct DioxusBackend;

impl ReactiveBackend for DioxusBackend {
    type Callback<T: Send + Sync + 'static> = Callback<T>;
    type ReadSignal<T: Send + Sync + 'static> = ReadSignal<T>;

    #[cfg(not(target_arch = "wasm32"))]
    type Storage = rpstate::DefaultStore;

    #[cfg(target_arch = "wasm32")]
    #[cfg(feature = "tauri-backend")]
    type Storage = rpstate_tauri::TauriBackend;

    fn cb_call<T: Send + Sync + 'static>(cb: &Self::Callback<T>, val: T) {
        cb.call(val);
    }

    fn rs_get<T: Clone + Send + Sync + 'static>(rs: &Self::ReadSignal<T>) -> T {
        rs.read().clone()
    }
}

pub type MapSignal<K, V> = rpstate_arena::MapSignal<DioxusBackend, K, V>;

#[derive(Clone, Props)]
pub struct RpStateProviderProps {
    pub store: DefaultStore,
    pub children: Element,
}

impl PartialEq for RpStateProviderProps {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.store, &other.store) && self.children == other.children
    }
}

#[allow(non_snake_case)]
pub fn RpStateProvider(props: RpStateProviderProps) -> Element {
    use_context_provider(DefaultArena::new);
    use_context_provider(|| props.store.clone());

    rsx! { {props.children} }
}
