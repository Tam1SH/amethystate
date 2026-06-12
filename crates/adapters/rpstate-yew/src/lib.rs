mod hooks;

use rpstate::{ReactiveMapKey, ReactiveMapValue};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

pub struct MapSignal<K, V>
where
    K: ReactiveMapKey,
    V: ReactiveMapValue,
{
    pub value: HashMap<K, V>,
    pub set: Callback<(K, V)>,
    pub remove: Callback<K>,
    pub clear: Callback<()>,
}


#[derive(Properties)]
pub struct RpStateProviderProps<B>
where
    B: Clone + PartialEq + 'static,
{
    pub backend: B,

    pub init: Rc<dyn Fn(B) -> Pin<Box<dyn Future<Output = ()>>>>,

    pub fallback: Html,

    pub children: Html,
}

impl<B> PartialEq for RpStateProviderProps<B>
where
    B: Clone + PartialEq + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.backend == other.backend && Rc::ptr_eq(&self.init, &other.init)
    }
}


#[function_component(RpStateProvider)]
pub fn rp_state_provider<B>(props: &RpStateProviderProps<B>) -> Html
where
    B: Clone + PartialEq + 'static,
{
    let ready = use_state(|| false);

    {
        let ready = ready.clone();
        let backend = props.backend.clone();
        let init = props.init.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                (init)(backend).await;
                ready.set(true);
            });
            || ()
        });
    }

    if !*ready {
        return props.fallback.clone();
    }

    props.children.clone()
}

#[macro_export]
macro_rules! provide_slices {
    ($($S:ty),+ $(,)?) => {
        ::std::rc::Rc::new(|backend: _| {
            Box::pin(async move {
                use ::rpstate::client::RpStateSliceAsync;
                $(
                    {
                        let state = <$S>::load_async(&backend)
                            .await
                            .unwrap_or_else(|e| panic!(
                                "rpstate-yew: failed to load {}: {e:?}",
                                ::std::any::type_name::<$S>()
                            ));
                        ::yew::functional::provide_context(state);
                    }
                )+
            }) as ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()>>>
        })
    };
}
