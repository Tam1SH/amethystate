use crate::ReactiveScope;
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

type SignalCallback<T> = Arc<dyn Fn(&T, Option<Uuid>) + Send + Sync + 'static>;
type SubscriberEntry<T> = (u64, SignalCallback<T>, SubscriptionMeta);
type SignalSubscribers<T> = Arc<Mutex<Vec<SubscriberEntry<T>>>>;

#[derive(Clone, Copy)]
pub struct SubscriptionMeta {
    pub id: u64,
    pub location: &'static std::panic::Location<'static>,
    pub name: Option<&'static str>,
}

pub struct Signal<T> {
    pub value: Arc<ArcSwap<T>>,
    pub subscribers: SignalSubscribers<T>,
    pub next_id: Arc<AtomicU64>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            next_id: self.next_id.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SignalSubscription {
    pub id: u64,
    pub location: &'static std::panic::Location<'static>,
    pub name: Option<&'static str>,
    pub set_name: Arc<dyn Fn(&'static str) + Send + Sync + 'static>,
    pub cleanup: Arc<dyn Fn(u64) + Send + Sync + 'static>,
}

impl SignalSubscription {
    pub fn named(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        (self.set_name)(name);
        self
    }

    pub fn watch(self, scope: &mut ReactiveScope) {
        scope.watch(self);
    }
}

impl Drop for SignalSubscription {
    fn drop(&mut self) {
        (self.cleanup)(self.id);
    }
}

impl<T: 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(ArcSwap::from_pointee(initial)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set(&self, new_value: T, source: Option<Uuid>) {
        self.value.store(Arc::new(new_value));
        self.emit(source);
    }

    fn emit(&self, _source: Option<Uuid>) {
        let val = self.value.load_full();
        let callbacks: Vec<_> = {
            let subs = self.subscribers.lock().unwrap();
            subs.iter().map(|(_, cb, meta)| (cb.clone(), *meta)).collect()
        };
        for (cb, meta) in callbacks {
            tracing::trace!(
                target: "amethystate",
                subscription_id = meta.id,
                name = meta.name,
                location = format!("{}:{}", meta.location.file(), meta.location.line()),
                "signal emit → subscription fire",
            );
            cb(&val, _source);
        }
    }

    #[track_caller]
    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.subscribe_with_source(move |val, _src| callback(val))
    }

    #[track_caller]
    pub fn subscribe_with_source<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&T, Option<Uuid>) + Send + Sync + 'static,
    {
        let location = std::panic::Location::caller();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let meta = SubscriptionMeta { id, location, name: None };
        {
            let mut subs = self.subscribers.lock().unwrap();
            subs.push((id, Arc::new(callback), meta));
        }

        let subscribers_for_name = self.subscribers.clone();
        let set_name = Arc::new(move |name: &'static str| {
            if let Ok(mut subs) = subscribers_for_name.lock() {
                if let Some(entry) = subs.iter_mut().find(|(sid, _, _)| *sid == id) {
                    entry.2.name = Some(name);
                }
            }
        });

        let subscribers_for_cleanup = self.subscribers.clone();
        SignalSubscription {
            id,
            location,
            name: None,
            set_name,
            cleanup: Arc::new(move |id| {
                if let Ok(mut subs) = subscribers_for_cleanup.lock() {
                    subs.retain(|(sid, _, _)| *sid != id);
                }
            }),
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn get(&self) -> T {
        self.value.load().as_ref().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn signal_subscription_cleanup_on_drop() {
        let signal = Signal::new("a".to_string());
        let counter = Arc::new(Mutex::new(0usize));
        {
            let cap = counter.clone();
            let _sub = signal.subscribe(move |_: &String| {
                *cap.lock().unwrap() += 1;
            });
            signal.set("b".to_string(), None);
            assert_eq!(*counter.lock().unwrap(), 1);
        }
        signal.set("c".to_string(), None);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn named_updates_meta_in_subscribers() {
        let signal = Signal::new(0i32);
        let _sub = signal.subscribe(|_| {}).named("MyWatcher");

        let subs = signal.subscribers.lock().unwrap();
        assert_eq!(subs[0].2.name, Some("MyWatcher"));
    }

    #[test]
    fn subscription_location_captured() {
        let signal = Signal::new(0i32);
        let sub = signal.subscribe(|_| {});
        assert!(!sub.location.file().is_empty());
    }
}
