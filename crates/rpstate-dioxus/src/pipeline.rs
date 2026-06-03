use crate::Arena;
use rpstate::{AccessMode, Signal, Store};
use rpstate::{Pipeline, SignalSubscription};
use rpstate_arena::FieldHandle;
use serde::{Serialize, de::DeserializeOwned};
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    pub(crate) static PIPELINE_ARENA: RefCell<Option<Arena>> = const { RefCell::new(None) };
}

pub fn pipeline_arena() -> Arena {
    PIPELINE_ARENA.with(|a| {
        a.borrow()
            .clone()
            .expect("called outside use_rpstate_pipeline")
    })
}

pub trait DioxusReactive<T>: Clone + Send + Sync + 'static
where
    T: Clone + Send + Sync + 'static,
{
    fn dx_get_with(&self, arena: &Arena) -> T;

    fn dx_get(&self) -> T {
        self.dx_get_with(&pipeline_arena())
    }

    fn dx_subscribe<F>(&self, arena: &Arena, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static;
}

impl<T, S, M> DioxusReactive<T> for FieldHandle<T, S, M>
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
    S: Store,
    M: AccessMode,
{
    fn dx_get_with(&self, arena: &Arena) -> T {
        arena.get_field(*self)
    }

    fn dx_subscribe<F>(&self, arena: &Arena, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        arena.subscribe_field(*self, callback)
    }
}

pub trait DioxusIntoPipeline<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn pipe(self) -> Pipeline<T>;
}

impl<R, T> DioxusIntoPipeline<T> for R
where
    R: DioxusReactive<T>,
    T: Clone + Send + Sync + 'static,
{
    fn pipe(self) -> Pipeline<T> {
        let initial = self.dx_get();
        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let sub = self.dx_subscribe(&pipeline_arena(), move |val| target.set(val));
        Pipeline::from_signal(signal, vec![sub], vec![])
    }
}

macro_rules! impl_dx_tuple_pipeline {
    ($(($source_ty:ident, $source:ident, $value_ty:ident, $value:ident)),+ $(,)?) => {
        impl<$($source_ty, $value_ty),+> DioxusIntoPipeline<($($value_ty,)+)> for ($($source_ty,)+)
        where
            $(
                $source_ty: DioxusReactive<$value_ty>,
                $value_ty: Clone + Send + Sync + 'static,
            )+
        {
            #[allow(non_snake_case)]
            fn pipe(self) -> Pipeline<($($value_ty,)+)> {
                let ($($source,)+) = self;
                let initial = ($($source.dx_get(),)+);
                let signal = Arc::new(Signal::new(initial));
                let mut source_subs = Vec::new();

                let refresh: Arc<dyn Fn() -> ($($value_ty,)+) + Send + Sync> = {
                    let arena = pipeline_arena();
                    $(let $value = $source.clone();)+
                    Arc::new(move || ($($value.dx_get_with(&arena),)+))
                };

                $(
                    let target = Arc::clone(&signal);
                    let refresh_cb = Arc::clone(&refresh);
                    source_subs.push($source.dx_subscribe(&pipeline_arena(), move |_| {
                        target.set(refresh_cb());
                    }));
                )+

                Pipeline::from_signal(signal, source_subs, vec![])
            }
        }
    };
}

impl_dx_tuple_pipeline!((RA, ra, A, a), (RB, rb, B, b));
impl_dx_tuple_pipeline!((RA, ra, A, a), (RB, rb, B, b), (RC, rc, C, c));
impl_dx_tuple_pipeline!(
    (RA, ra, A, a),
    (RB, rb, B, b),
    (RC, rc, C, c),
    (RD, rd, D, d)
);
