mod hooks;
pub use hooks::*;
use leptos::prelude::*;
pub use rpstate::*;

use leptos::callback::Callback;
use rpstate_arena::{DefaultArena, ReactiveBackend};

pub struct LeptosBackend;

impl ReactiveBackend for LeptosBackend {
    type Callback<T: Send + Sync + 'static> = Callback<T>;
    type ReadSignal<T: Send + Sync + 'static> = ReadSignal<T>;

    #[cfg(not(target_arch = "wasm32"))]
    type Storage = DefaultStore;

    #[cfg(target_arch = "wasm32")]
    type Storage = rpstate_tauri::TauriBackend;

    fn cb_call<T: Send + Sync + 'static>(cb: &Self::Callback<T>, val: T) {
        cb.run(val);
    }

    fn rs_get<T: Clone + Send + Sync + 'static>(rs: &Self::ReadSignal<T>) -> T {
        rs.get()
    }
}

#[cfg(all(target_arch = "wasm32", not(feature = "tauri-backend")))]
compile_error!(
    "rpstate-leptos: feature 'tauri-backend' must be enabled when compiling for wasm32."
);

pub type MapSignal<K, V> = rpstate_arena::MapSignal<LeptosBackend, K, V>;

#[component]
pub fn RpStateProvider(store: DefaultStore, children: Children) -> impl IntoView {
    provide_context(DefaultArena::new());
    provide_context(store);

    children()
}
