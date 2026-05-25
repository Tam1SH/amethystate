use crate::AccessMode;
use crate::reactive::{Field, Signal, SignalSubscription};
use crate::store::Store;
use std::sync::{Arc, Mutex};

pub trait Reactive<T>: Clone + Send + Sync + 'static
where
    T: Clone + Send + Sync + 'static,
{
    fn get(&self) -> T;
    fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static;

    fn keepalive(&self) -> Option<Arc<dyn Send + Sync>> {
        None
    }
}

pub trait IntoPipeline<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn pipe(self) -> Pipeline<T>;
}

struct PipelineInner<T> {
    signal: Arc<Signal<T>>,
    _source_subs: Vec<SignalSubscription>,
    _keepalive: Vec<Arc<dyn Send + Sync>>,
}

pub struct Pipeline<T> {
    inner: Arc<PipelineInner<T>>,
}

#[derive(Default)]
pub struct ReactiveScope {
    subs: Vec<SignalSubscription>,
}

impl ReactiveScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn watch(&mut self, sub: SignalSubscription) {
        self.subs.push(sub);
    }

    pub fn clear(&mut self) {
        self.subs.clear();
    }

    pub fn len(&self) -> usize {
        self.subs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }
}

impl<T> Clone for Pipeline<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Pipeline<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from_signal(
        signal: Arc<Signal<T>>,
        source_subs: Vec<SignalSubscription>,
        keepalive: Vec<Arc<dyn Send + Sync>>,
    ) -> Self {
        Self {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: source_subs,
                _keepalive: keepalive,
            }),
        }
    }

    pub fn get(&self) -> T {
        self.inner.signal.get()
    }

    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.inner
            .signal
            .subscribe(move |val| callback(val.clone()))
    }

    pub fn map<U, F>(self, f: F) -> Pipeline<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(T) -> U + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let initial = f(self.get());
        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let mapper = Arc::clone(&f);

        let sub = self.subscribe(move |value| {
            target.set(mapper(value));
        });

        Pipeline {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: vec![sub],
                _keepalive: vec![self.inner],
            }),
        }
    }

    pub fn filter_map<U, F>(self, f: F) -> Pipeline<U>
    where
        U: Default + Clone + Send + Sync + 'static,
        F: Fn(T) -> Option<U> + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let initial = f(self.get()).unwrap_or_default();
        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let mapper = Arc::clone(&f);

        let sub = self.subscribe(move |value| {
            if let Some(mapped) = mapper(value) {
                target.set(mapped);
            }
        });

        Pipeline {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: vec![sub],
                _keepalive: vec![self.inner],
            }),
        }
    }

    pub fn inspect<F>(self, f: F) -> Pipeline<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let initial = self.get();
        f(&initial);

        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let inspector = Arc::clone(&f);

        let sub = self.subscribe(move |value| {
            inspector(&value);
            target.set(value);
        });

        Pipeline {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: vec![sub],
                _keepalive: vec![self.inner],
            }),
        }
    }

    pub fn dedupe(self) -> Pipeline<T>
    where
        T: PartialEq,
    {
        let initial = self.get();
        let last = Arc::new(Mutex::new(initial.clone()));
        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let last_seen = Arc::clone(&last);

        let sub = self.subscribe(move |value| {
            let mut last = last_seen.lock().unwrap();
            if *last != value {
                *last = value.clone();
                target.set(value);
            }
        });

        Pipeline {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: vec![sub],
                _keepalive: vec![self.inner],
            }),
        }
    }
}

impl<T> Reactive<T> for Pipeline<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn get(&self) -> T {
        Pipeline::get(self)
    }

    fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Pipeline::subscribe(self, callback)
    }

    fn keepalive(&self) -> Option<Arc<dyn Send + Sync>> {
        Some(self.inner.clone())
    }
}

impl<TValue, S, M> Reactive<TValue> for Field<TValue, S, M>
where
    TValue: serde::de::DeserializeOwned + serde::Serialize + Clone + Send + Sync + 'static,
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

impl<R, T> IntoPipeline<T> for R
where
    R: Reactive<T>,
    T: Clone + Send + Sync + 'static,
{
    fn pipe(self) -> Pipeline<T> {
        let initial = self.get();
        let signal = Arc::new(Signal::new(initial));
        let target = Arc::clone(&signal);
        let sub = self.subscribe(move |value| {
            target.set(value);
        });

        Pipeline {
            inner: Arc::new(PipelineInner {
                signal,
                _source_subs: vec![sub],
                _keepalive: self.keepalive().into_iter().collect(),
            }),
        }
    }
}

macro_rules! impl_tuple_pipeline {
    ($(($source_ty:ident, $source:ident, $value_ty:ident, $value:ident)),+ $(,)?) => {
        impl<$($source_ty, $value_ty),+> IntoPipeline<($($value_ty,)+)> for ($($source_ty,)+)
        where
            $(
                $source_ty: Reactive<$value_ty>,
                $value_ty: Clone + Send + Sync + 'static,
            )+
        {
            #[allow(non_snake_case)]
            fn pipe(self) -> Pipeline<($($value_ty,)+)> {
                let ($($source,)+) = self;
                let initial = ($($source.get(),)+);
                let signal = Arc::new(Signal::new(initial));
                let mut source_subs = Vec::new();
                let mut keepalive = Vec::new();

                let refresh: Arc<dyn Fn() -> ($($value_ty,)+) + Send + Sync> = {
                    $(let $value = $source.clone();)+
                    Arc::new(move || ($($value.get(),)+))
                };

                $(
                    if let Some(inner) = $source.keepalive() {
                        keepalive.push(inner);
                    }

                    let target = Arc::clone(&signal);
                    let refresh_cb = Arc::clone(&refresh);
                    source_subs.push($source.subscribe(move |_| {
                        target.set(refresh_cb());
                    }));
                )+

                Pipeline::from_signal(signal, source_subs, keepalive)
            }
        }
    };
}

impl_tuple_pipeline!((RA, ra, A, a), (RB, rb, B, b));
impl_tuple_pipeline!((RA, ra, A, a), (RB, rb, B, b), (RC, rc, C, c));
impl_tuple_pipeline!(
    (RA, ra, A, a),
    (RB, rb, B, b),
    (RC, rc, C, c),
    (RD, rd, D, d)
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DefaultStore, WritableMode};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn field<T>(value: T) -> Field<T, DefaultStore, WritableMode>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Clone + Send + Sync + 'static,
    {
        Field::new_volatile(Arc::from("pipeline.test"), value)
    }

    #[test]
    fn single_source_pipeline_maps_changes() {
        let source = field(1);
        let doubled = source.clone().pipe().map(|v| v * 2);

        assert_eq!(doubled.get(), 2);

        source.set(3).unwrap();
        assert_eq!(doubled.get(), 6);
    }

    #[test]
    fn filter_map_uses_default_initially_and_suppresses_none() {
        let source = field(1);
        let even = source
            .clone()
            .pipe()
            .filter_map(|v| (v % 2 == 0).then_some(v * 10));

        assert_eq!(even.get(), 0);

        source.set(2).unwrap();
        assert_eq!(even.get(), 20);

        source.set(3).unwrap();
        assert_eq!(even.get(), 20);
    }

    #[test]
    fn inspect_observes_initial_and_changed_values() {
        let source = field(5);
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&calls);

        let inspected = source.clone().pipe().inspect(move |v| {
            seen.lock().unwrap().push(*v);
        });

        source.set(8).unwrap();
        assert_eq!(inspected.get(), 8);
        assert_eq!(*calls.lock().unwrap(), vec![5, 8]);
    }

    #[test]
    fn dedupe_skips_repeated_values() {
        let source = field(1);
        let deduped = source.clone().pipe().dedupe();
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&calls);

        let _sub = deduped.subscribe(move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        });

        source.set(1).unwrap();
        source.set(2).unwrap();
        source.set(2).unwrap();
        source.set(3).unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(deduped.get(), 3);
    }

    #[test]
    fn tuple_pipeline_combines_latest_values() {
        let host = field("localhost".to_string());
        let port = field(8080u16);
        let address = (host.clone(), port.clone())
            .pipe()
            .map(|(host, port)| format!("{host}:{port}"));

        assert_eq!(address.get(), "localhost:8080");

        port.set(9090).unwrap();
        assert_eq!(address.get(), "localhost:9090");

        host.set("127.0.0.1".to_string()).unwrap();
        assert_eq!(address.get(), "127.0.0.1:9090");
    }

    #[test]
    fn pipelines_compose_as_sources() {
        let port = field(8080u16);
        let display_port = port.clone().pipe().map(|p| format!(":{p}"));
        let host = field("localhost".to_string());
        let address = (host.clone(), display_port)
            .pipe()
            .map(|(host, port)| format!("{host}{port}"));

        assert_eq!(address.get(), "localhost:8080");

        port.set(9090).unwrap();
        assert_eq!(address.get(), "localhost:9090");
    }

    #[test]
    fn dropping_pipeline_releases_upstream_subscription() {
        let source = field(1);
        let calls = Arc::new(AtomicUsize::new(0));

        let sub = {
            let calls = Arc::clone(&calls);
            source.clone().pipe().map(|v| v * 2).subscribe(move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
            })
        };

        source.set(2).unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        drop(sub);
    }

    #[test]
    fn complex_pipeline_graph_propagates_predictably() {
        let host = field("localhost".to_string());
        let port = field(8080u16);
        let enabled = field(false);

        let display_port = port
            .clone()
            .pipe()
            .filter_map(|p| (p >= 1024).then_some(format!(":{p}")))
            .dedupe();

        let address = (host.clone(), display_port.clone(), enabled.clone())
            .pipe()
            .filter_map(|(host, port, enabled)| enabled.then_some(format!("{host}{port}")))
            .dedupe();

        let events = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&events);
        let _sub = address.subscribe(move |value| {
            seen.lock().unwrap().push(value);
        });

        assert_eq!(display_port.get(), ":8080");
        assert_eq!(address.get(), "");

        port.set(80).unwrap();
        assert_eq!(display_port.get(), ":8080");
        assert!(events.lock().unwrap().is_empty());

        enabled.set(true).unwrap();
        assert_eq!(address.get(), "localhost:8080");
        assert_eq!(*events.lock().unwrap(), vec!["localhost:8080".to_string()]);

        port.set(9090).unwrap();
        assert_eq!(display_port.get(), ":9090");
        assert_eq!(address.get(), "localhost:9090");

        host.set("127.0.0.1".to_string()).unwrap();
        assert_eq!(address.get(), "127.0.0.1:9090");

        enabled.set(false).unwrap();
        assert_eq!(address.get(), "127.0.0.1:9090");
        assert_eq!(events.lock().unwrap().len(), 3);

        enabled.set(true).unwrap();
        assert_eq!(address.get(), "127.0.0.1:9090");
        assert_eq!(events.lock().unwrap().len(), 3);

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                "localhost:8080".to_string(),
                "localhost:9090".to_string(),
                "127.0.0.1:9090".to_string(),
            ]
        );
    }

    #[test]
    fn reactive_scope_owns_and_clears_subscriptions() {
        let source = field(1);
        let doubled = source.clone().pipe().map(|v| v * 2);
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&calls);
        let mut scope = ReactiveScope::new();

        scope.watch(doubled.subscribe(move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        }));

        assert_eq!(scope.len(), 1);

        source.set(2).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        scope.clear();
        assert!(scope.is_empty());

        source.set(3).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
