use crate::store::sync_backend::StoreBackend;
use crate::store::{Store, SubscriptionId};
use crate::{AccessMode, ReadOnlyMode, Result, WritableMode};
use rpstate_core::{Change, FieldCore, InterceptDisposer, Signal, SignalSubscription};
use std::sync::Arc;

pub struct StoreSubscription<S: Store> {
    pub store: S,
    pub id: SubscriptionId,
}

impl<S: Store> Drop for StoreSubscription<S> {
    fn drop(&mut self) {
        self.store.unsubscribe(self.id);
    }
}
pub use rpstate_core::primitives::field_core::FieldValue;

pub struct Field<TValue, S: Store, M: AccessMode = ReadOnlyMode> {
    pub(crate) core: FieldCore<TValue>,
    pub path: Arc<str>,
    pub(crate) store_sub: Option<Arc<StoreSubscription<S>>>,
    pub(crate) _mode: std::marker::PhantomData<M>,
}

pub type ReadOnlyField<TValue, S> = Field<TValue, S, ReadOnlyMode>;
pub type WritableField<TValue, S> = Field<TValue, S, WritableMode>;

impl<TValue, S: Store, M: AccessMode> Clone for Field<TValue, S, M> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            path: Arc::clone(&self.path),
            store_sub: self.store_sub.clone(),
            _mode: std::marker::PhantomData,
        }
    }
}

impl<TValue, S, M> Field<TValue, S, M>
where
    TValue: FieldValue,
    S: Store,
    M: AccessMode,
{
    pub fn get(&self) -> TValue {
        self.core.get()
    }

    pub fn get_arc(&self) -> Arc<TValue> {
        self.core.get_arc()
    }

    pub fn path(&self) -> Arc<str> {
        self.path.clone()
    }

    pub fn as_signal(&self) -> Signal<TValue> {
        self.core.signal.clone()
    }

    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(TValue) + Send + Sync + 'static,
    {
        self.core.subscribe(callback)
    }
}

impl<TValue, S> Field<TValue, S, WritableMode>
where
    TValue: FieldValue,
    S: Store,
{
    pub fn set(&self, value: TValue) -> Result<()> {
        if let Some(sub) = &self.store_sub {
            let backend = StoreBackend::new(sub.store.clone());
            rpstate_core::field_set(&backend, &self.core, self.path.clone(), value, false)?;
        } else {
            let change = self
                .core
                .run_interceptors(self.path.clone(), value)
                .map_err(|_| crate::error::Error::Intercepted)?;
            self.core.signal.set(change.new_value);
        }
        Ok(())
    }

    pub fn intercept<F>(&self, callback: F) -> InterceptDisposer
    where
        F: Fn(Change<TValue>) -> Option<Change<TValue>> + Send + Sync + 'static,
    {
        self.core.intercept(self.path.clone(), callback)
    }

    pub fn new_volatile(path: Arc<str>, default: TValue) -> Self {
        Self {
            core: FieldCore::new(default),
            path,
            store_sub: None,
            _mode: std::marker::PhantomData,
        }
    }
}

impl<TValue, S: Store, M: AccessMode> PartialEq for Field<TValue, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && Arc::ptr_eq(&self.core.signal.value, &other.core.signal.value)
    }
}

impl<TValue, S: Store, M: AccessMode> Eq for Field<TValue, S, M> {}

impl<TValue, S, M> rpstate_core::pipeline::Reactive<TValue> for Field<TValue, S, M>
where
    TValue: FieldValue,
    S: Store,
    M: AccessMode,
{
    fn get(&self) -> TValue {
        Field::get(self)
    }

    fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(TValue) + Send + Sync + 'static,
    {
        Field::subscribe(self, callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::config::StoreConfig;
    use crate::store::{StateScope, Store};
    use crate::{DefaultStore, SubscriptionKind};
    use std::sync::Mutex;
    use std::sync::atomic::Ordering;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_test::traced_test;

    fn unique_store(suffix: &str) -> DefaultStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path = std::env::temp_dir().join(format!("rpstate-reactive-{suffix}-{nanos}.json"));
        DefaultStore::open(StoreConfig::new(path), Default::default())
            .unwrap()
            .0
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

        field.core.signal.set(22);
        assert_eq!(*callback_val.lock().unwrap(), 22);
    }

    #[test]
    fn store_subscription_drop_unsubscribes() {
        let store = unique_store("drop-unsub");
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let core = FieldCore::new("test_val".to_string());

        let cap = calls.clone();

        {
            let sub_id = store.subscribe(
                SubscriptionKind::Prefix(Arc::from("test.field")),
                Arc::new(move |_| {
                    cap.fetch_add(1, Ordering::SeqCst);
                }),
            );

            let field: Field<String, DefaultStore, WritableMode> = Field {
                core,
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
        let core = FieldCore::new(100i32);
        let sub_id = store.subscribe(crate::store::SubscriptionKind::Any, Arc::new(|_| {}));
        let field: Field<i32, DefaultStore> = Field {
            core: core.clone(),
            path: Arc::from("some.path"),
            store_sub: Some(Arc::new(StoreSubscription {
                store: store.clone(),
                id: sub_id,
            })),
            _mode: Default::default(),
        };

        let extracted = field.as_signal();
        assert!(Arc::ptr_eq(&core.signal.value, &extracted.value));
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

    #[test]
    fn test_field_additional_coverage() {
        let field = Field::<i32, DefaultStore, WritableMode>::new_volatile(Arc::from("test"), 42);

        let disp = field.intercept(|mut change| {
            change.new_value *= 2;
            Some(change)
        });

        field.set(10).unwrap();
        assert_eq!(field.get(), 20);

        drop(disp);

        field.set(10).unwrap();
        assert_eq!(field.get(), 20, "Interceptor should survive manual drop");

        let disp2 = field.intercept(|mut change| {
            change.new_value += 1;
            Some(change)
        });

        field.set(5).unwrap();
        assert_eq!(field.get(), 11);

        disp2.remove();

        field.set(5).unwrap();
        assert_eq!(field.get(), 10);
    }

    #[test]
    fn test_field_depth_guard() {
        let field = Field::<i32, DefaultStore, WritableMode>::new_volatile(Arc::from("test"), 1);

        field.core.intercept_depth.store(100, Ordering::SeqCst);

        let _disp = field.intercept(|mut c| {
            c.new_value = 999;
            Some(c)
        });

        field.set(10).unwrap();
        assert_eq!(field.get(), 10);
    }

    #[test]
    #[traced_test]
    fn test_field_recursion_warning() {
        let field = Field::<i32, DefaultStore, WritableMode>::new_volatile(
            Arc::from("test.recursive_field"),
            0,
        );

        let field_clone = field.clone();

        field.intercept(move |change| {
            let _ = field_clone.set(change.new_value + 1);
            Some(change)
        });

        let _ = field.set(1);

        assert!(logs_contain("maximum intercept depth reached"));
        assert!(logs_contain("path=test.recursive_field"));
    }
}
