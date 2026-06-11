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
    type Storage = rpstate::tauri::TauriBackend;

    fn cb_call<T: Send + Sync + 'static>(cb: &Self::Callback<T>, val: T) {
        cb.call(val);
    }

    fn rs_get<T: Clone + Send + Sync + 'static>(rs: &Self::ReadSignal<T>) -> T {
        rs.read().clone()
    }
}

pub type MapSignal<K, V> = rpstate_arena::MapSignal<DioxusBackend, K, V>;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Props, PartialEq, Eq)]
pub struct RpStateProviderProps {
    pub store: DefaultStore,
    pub children: Element,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Props)]
pub struct RpStateProviderProps {
    pub backend: ::rpstate::tauri::TauriBackend,
    pub init: std::rc::Rc<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>>,
    pub children: Element,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(non_snake_case)]
pub fn RpStateProvider(props: RpStateProviderProps) -> Element {
    use_context_provider(DefaultArena::new);
    use_context_provider(|| props.store.clone());

    rsx! { {props.children} }
}

#[cfg(target_arch = "wasm32")]
#[allow(non_snake_case)]
pub fn RpStateProvider(props: RpStateProviderProps) -> Element {
    use_context_provider(DefaultArena::new);
    use_context_provider(|| props.backend.clone());

    let init = props.init.clone();
    let res = use_resource(move || {
        let f = init();
        async move {
            f.await;
        }
    });

    res.suspend()?;

    rsx! { {props.children} }
}

#[macro_export]
macro_rules! preload_slices {
    ($($S:ty),+ $(,)?) => {{
        std::rc::Rc::new(|| Box::pin(async move {
            use rpstate_arena::RpStateFrameworkNested;
            let backend = ::dioxus::prelude::use_context::<::rpstate_tauri::TauriBackend>();
            let arena = ::dioxus::prelude::use_context::<::rpstate_arena::DefaultArena>();
            $(
                {
                    use ::rpstate_core::RpStateSliceAsync;
                    let state = <$S as RpStateSliceAsync<::rpstate::tauri::TauriBackend>>
                        ::load_async(&backend)
                        .await
                        .unwrap_or_else(|e| panic!("rpstate: failed to load {}: {e:?}",
                            ::std::any::type_name::<$S>()));
                    let handle = state.register(&arena);
                    ::dioxus::prelude::provide_context(handle);
                }
            )+
        }) as ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()>>>)
    }};
}
