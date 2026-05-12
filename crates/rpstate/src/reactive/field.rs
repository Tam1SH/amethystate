use crate::store::{Store, SubscriptionId};
use crate::{AccessMode, ReadOnlyMode, Result, Signal, SignalSubscription, WritableMode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::Arc;

pub struct StoreSubscription<S: Store> {
    pub store: Arc<S>,
    pub id: SubscriptionId,
}

impl<S: Store> Drop for StoreSubscription<S> {
    fn drop(&mut self) {
        self.store.unsubscribe(self.id);
    }
}

pub struct FieldSubscription {
    #[allow(unused)]
    pub signal_sub: SignalSubscription,
}

pub struct Field<TValue, S: Store, M: AccessMode = ReadOnlyMode> {
    pub signal: Arc<Signal<TValue>>,
    pub path: Arc<str>,
    pub store_sub: Option<Arc<StoreSubscription<S>>>,
    pub(crate) _mode: std::marker::PhantomData<M>,
}

impl<TValue, S: Store, M: AccessMode> Clone for Field<TValue, S, M> {
    fn clone(&self) -> Self {
        Self {
            signal: Arc::clone(&self.signal),
            path: Arc::clone(&self.path),
            store_sub: self.store_sub.clone(),
            _mode: std::marker::PhantomData,
        }
    }
}

impl<TValue, S, M> Field<TValue, S, M>
where
    TValue: DeserializeOwned + Serialize + Send + Sync + Clone + 'static,
    S: Store,
    M: AccessMode,
{
    pub fn get(&self) -> TValue {
        self.signal.get()
    }

    pub fn get_arc(&self) -> Arc<TValue> {
        self.signal.get_arc()
    }

    pub fn path(&self) -> Arc<str> {
        self.path.clone()
    }

    pub fn as_signal(&self) -> Arc<Signal<TValue>> {
        self.signal.clone()
    }

    pub fn subscribe<F>(&self, callback: F) -> FieldSubscription
    where
        F: Fn(TValue) + Send + Sync + 'static,
    {
        let signal_sub = self.signal.subscribe(move |val: &TValue| {
            callback(val.clone());
        });
        FieldSubscription { signal_sub }
    }
}

impl<TValue, S> Field<TValue, S, WritableMode>
where
    TValue: DeserializeOwned + Serialize + Send + Sync + Clone + 'static,
    S: Store,
{
    pub fn set(&self, value: TValue) -> Result<()> {
        if let Some(sub) = &self.store_sub {
            sub.store.set_owned(self.path.clone(), &value)
        } else {
            self.signal.set(value);
            Ok(())
        }
    }
}

impl<TValue, S> Field<TValue, S, WritableMode>
where
    TValue: DeserializeOwned + Serialize + Send + Sync + Clone + 'static,
    S: Store,
{
    pub fn new_volatile(path: Arc<str>, default: TValue) -> Self {
        Self {
            signal: Arc::new(Signal::new(default)),
            path,
            store_sub: None,
            _mode: std::marker::PhantomData,
        }
    }
}

impl<TValue: Debug + 'static, S: Store> Debug for Field<TValue, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("path", &self.path)
            .field("value", self.signal.get_arc().as_ref())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::config::StoreConfig;
    use crate::store::{StateScope, Store};
    use crate::{DefaultStore, SubscriptionKind};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_store(suffix: &str) -> Arc<DefaultStore> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rpstate-reactive-{suffix}-{nanos}.json"));
        Arc::new(
            DefaultStore::open(StoreConfig::new(path), Default::default())
                .unwrap()
                .0,
        )
    }

    struct UiScope;
    impl StateScope for UiScope {
        const PREFIX: &'static str = "ui";
    }

    #[test]
    fn field_get_set_and_subscribe() {
        let store = unique_store("field-int");
        let field = crate::store::field::<UiScope, i32, DefaultStore>(&store, "font_size", 14)
            .expect("field should be created");

        assert_eq!(field.get(), 14);
        assert_eq!(field.path().as_ref(), "ui.font_size");

        field.set(18).expect("set should succeed");
        assert_eq!(store.get::<i32>("ui.font_size").unwrap(), Some(18));

        let callback_val = Arc::new(Mutex::new(0i32));
        let cap = callback_val.clone();
        let _sub = field.subscribe(move |v| {
            *cap.lock().unwrap() = v;
        });

        field.signal.set(22);
        assert_eq!(*callback_val.lock().unwrap(), 22);
    }

    #[test]
    fn field_debug_output_contains_type_and_value() {
        let store = unique_store("field-debug");
        let signal = Arc::new(Signal::new("test_val".to_string()));
        let sub_id = store.subscribe(crate::store::SubscriptionKind::Any, Arc::new(|_| {}));
        let field: Field<String, DefaultStore> = Field {
            signal,
            path: Arc::from("debug.path"),
            store_sub: Some(Arc::new(StoreSubscription {
                store: store.clone(),
                id: sub_id,
            })),
            _mode: Default::default(),
        };

        let debug_str = format!("{:?}", field);
        assert!(debug_str.contains("Field"), "debug should show type name");
        assert!(
            debug_str.contains("test_val"),
            "debug should show current value"
        );
    }

    #[test]
    fn store_subscription_drop_unsubscribes() {
        let store = unique_store("drop-unsub");
        let calls = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new(Signal::new("test_val".to_string()));

        let cap = calls.clone();

        {
            let sub_id = store.subscribe(
                SubscriptionKind::Prefix(Arc::from("test.field")),
                Arc::new(move |_| {
                    cap.fetch_add(1, Ordering::SeqCst);
                }),
            );

            let field: Field<String, DefaultStore, WritableMode> = Field {
                signal,
                path: Arc::from("test.field"),
                store_sub: Some(Arc::new(StoreSubscription {
                    store: store.clone(),
                    id: sub_id,
                })),
                _mode: Default::default(),
            };

            field.set("hello".to_string()).unwrap();
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            store.set("test.field", &"world").unwrap();
            assert_eq!(calls.load(Ordering::SeqCst), 2);
        }

        store.set("test.field", &"world").unwrap();
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "callback must not fire after drop"
        );
    }

    #[test]
    fn field_as_signal_returns_same_arc() {
        let store = unique_store("as-signal");
        let signal = Arc::new(Signal::new(100i32));
        let sub_id = store.subscribe(crate::store::SubscriptionKind::Any, Arc::new(|_| {}));
        let field: Field<i32, DefaultStore> = Field {
            signal: signal.clone(),
            path: Arc::from("some.path"),
            store_sub: Some(Arc::new(StoreSubscription {
                store: store.clone(),
                id: sub_id,
            })),
            _mode: Default::default(),
        };

        let extracted = field.as_signal();
        assert!(Arc::ptr_eq(&signal, &extracted));
    }

    #[test]
    fn test_volatile_field_behavior() {
        let store = unique_store("test_volatile_field_behavior");

        let field_path: Arc<str> = Arc::from("ui.temp_spinner");

        let field =
            Field::<bool, DefaultStore, WritableMode>::new_volatile(field_path.clone(), false);

        let call_count = Arc::new(Mutex::new(0));
        let last_val = Arc::new(Mutex::new(false));

        let c_count = call_count.clone();
        let l_val = last_val.clone();

        let _sub = field.subscribe(move |val| {
            *c_count.lock().unwrap() += 1;
            *l_val.lock().unwrap() = val;
        });

        field.set(true).expect("Volatile set should work");

        assert!(field.get());

        assert!(*call_count.lock().unwrap() >= 1);
        assert!(*last_val.lock().unwrap());

        let in_store: Option<bool> = store.get(&field_path).unwrap();
        assert!(
            in_store.is_none(),
            "Volatile data must NOT be persisted to store"
        );
    }
}
