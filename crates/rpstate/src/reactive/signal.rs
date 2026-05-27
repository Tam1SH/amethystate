use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

type SignalCallback<T> = Arc<dyn Fn(&T) + Send + Sync + 'static>;
type SignalSubscribers<T> = Arc<Mutex<Vec<(u64, SignalCallback<T>)>>>;

pub struct Signal<T> {
    pub(crate) value: Arc<ArcSwap<T>>,
    pub(crate) subscribers: SignalSubscribers<T>,
    pub(crate) next_id: Arc<AtomicU64>,
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
    pub(crate) id: u64,
    pub(crate) cleanup: Arc<dyn Fn(u64) + Send + Sync + 'static>,
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

    pub fn get_arc(&self) -> Arc<T> {
        self.value.load_full()
    }

    pub fn store_arc(&self, arc: Arc<T>) {
        self.value.store(arc);
        self.emit();
    }

    pub fn set(&self, new_value: T) {
        self.value.store(Arc::new(new_value));
        self.emit();
    }

    fn emit(&self) {
        let val = self.value.load_full();

        let callbacks: Vec<_> = {
            let subs = self.subscribers.lock().unwrap();
            subs.iter().map(|(_, cb)| cb.clone()).collect()
        };

        for cb in callbacks {
            cb(&val);
        }
    }

    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut subs = self.subscribers.lock().unwrap();
        subs.push((id, Arc::new(callback)));

        let subscribers = self.subscribers.clone();
        SignalSubscription {
            id,
            cleanup: Arc::new(move |id| {
                if let Ok(mut subs) = subscribers.lock() {
                    subs.retain(|(sid, _)| *sid != id);
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
    fn signal_basic_set_get_and_subscribe() {
        let signal = Signal::new(10i32);
        assert_eq!(signal.get(), 10);

        let received = Arc::new(Mutex::new(0i32));
        let cap = received.clone();
        let _sub = signal.subscribe(move |val: &i32| {
            *cap.lock().unwrap() = *val;
        });

        signal.set(20);
        assert_eq!(signal.get(), 20);
        assert_eq!(*received.lock().unwrap(), 20);

        signal.store_arc(Arc::new(30));
        assert_eq!(*signal.get_arc(), 30);
        assert_eq!(*received.lock().unwrap(), 30);
    }

    #[test]
    fn signal_subscription_cleanup_on_drop() {
        let signal = Signal::new("a".to_string());
        let counter = Arc::new(Mutex::new(0usize));

        {
            let cap = counter.clone();
            let _sub = signal.subscribe(move |_: &String| {
                *cap.lock().unwrap() += 1;
            });
            signal.set("b".to_string());
            assert_eq!(*counter.lock().unwrap(), 1);
        }

        signal.set("c".to_string());
        assert_eq!(*counter.lock().unwrap(), 1);
    }
}
