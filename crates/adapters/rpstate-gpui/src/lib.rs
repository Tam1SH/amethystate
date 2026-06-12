use futures::StreamExt as _;
use gpui::{App, AppContext, Entity};
use rpstate::{DefaultStore, ReactiveScope, RpStateSlice, Store};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Clone)]
pub struct RpView<T, S> {
    inner: T,
    _scope: ReactiveScope,
    _phantom: PhantomData<S>
}

impl<T, S> Deref for RpView<T, S> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: Store, T: RpStateSlice<S>> RpView<T, S> {
    pub fn new(inner: T, tx: futures::channel::mpsc::UnboundedSender<()>) -> Self {
        let _scope = inner.subscribe_all_external(move || {
            let _ = tx.unbounded_send(());
        });

        Self {
            inner,
            _scope,
            _phantom: PhantomData
        }
    }
}

pub type RpEntity<T, S = DefaultStore> = Entity<RpView<T, S>>;

pub trait RpStateExt {
    fn new_rpstate<S: Store, T: RpStateSlice<S> + 'static, E>(
        &mut self,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<RpEntity<T, S>, E>;
}

impl RpStateExt for App {
    fn new_rpstate<S: Store, T: RpStateSlice<S> + 'static, E>(
        &mut self,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<RpEntity<T, S>, E> {
        let reservation = self.reserve_entity();
        let inner = f()?;

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<()>();

        let entity = self.insert_entity(reservation, move |ctx| {

            ctx.spawn(async move |this, cx| {
                while let Some(()) = rx.next().await {
                    let _ = this.update(cx, |_, entity_cx| {
                        entity_cx.notify();
                    });
                }
            }).detach();

            RpView::new(inner, tx)
        });

        Ok(entity)
    }
}