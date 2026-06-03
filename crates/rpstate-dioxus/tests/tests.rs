use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use rpstate::store::field_with_path;
use rpstate::{DefaultStore, IntoPipeline, MapChange, rpstate};
use rpstate_arena::{Arena, PipelineBuilder, PipelineHandle, WritableHandle, WritableMapHandle};
use rpstate_dioxus::DioxusIntoPipeline;
use rpstate_dioxus::{
    MapSignal, RpStateProvider, use_field, use_map, use_map_subscribe_any, use_map_subscribe_key,
    use_pipeline, use_rpstate,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn unique_store(suffix: &str) -> Arc<DefaultStore> {
    use rpstate::store::config::StoreConfig;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rpstate-arena-test-{suffix}-{nanos}.json"));
    Arc::new(
        DefaultStore::open(StoreConfig::new(path), Default::default())
            .unwrap()
            .0,
    )
}

#[derive(Clone)]
struct Probe<T>(Arc<Mutex<Vec<T>>>);
impl<T> Probe<T> {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }
    fn push(&self, val: T) {
        self.0.lock().unwrap().push(val);
    }
    fn last(&self) -> Option<T>
    where
        T: Clone,
    {
        self.0.lock().unwrap().last().cloned()
    }
    fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

impl<T> PartialEq for Probe<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

struct DummyScope;
impl rpstate::StateScope for DummyScope {
    const PREFIX: &'static str = "test";
}

#[derive(Clone, Props)]
struct FieldTestProps {
    arena: Arena,
    handle: WritableHandle<i32, DefaultStore>,
    probe: Probe<i32>,
    setter_probe: Probe<Callback<i32>>,
}

impl PartialEq for FieldTestProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn FieldTestComponent(props: FieldTestProps) -> Element {
    let (val_signal, setter) = use_field(props.handle);
    props.probe.push(*val_signal.read());

    let setter1 = setter;
    let setter2 = setter;
    props.setter_probe.push(setter1);
    props.setter_probe.push(setter2);

    rsx! { div { "Field: {val_signal}" } }
}

#[tokio::test]
async fn test_use_field_requirements() {
    let store = unique_store("field");
    let arena = Arena::new();

    let field =
        rpstate::store::field_with_path(&store, std::sync::Arc::from("field_1"), 10).unwrap();
    let handle = arena.register_field(field);

    let probe = Probe::new();
    let setter_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        |props: FieldTestProps| {
            use_context_provider(|| props.arena.clone());
            rsx! { FieldTestComponent { ..props } }
        },
        FieldTestProps {
            arena: arena.clone(),
            handle,
            probe: probe.clone(),
            setter_probe: setter_probe.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last(), Some(10));

    let setter = setter_probe.last().unwrap();
    setter.call(42);

    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last(), Some(42));
    assert_eq!(arena.get_field(handle), 42);

    let _ = arena.set_field(handle, 100);

    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last(), Some(100));
}

#[derive(Clone, Props)]
struct MapTestProps {
    arena: Arena,
    handle: WritableMapHandle<String, String, DefaultStore>,
    probe: Probe<HashMap<String, String>>,
    methods_probe: Probe<MapSignal<String, String>>,
}

impl PartialEq for MapTestProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn MapTestComponent(props: MapTestProps) -> Element {
    let map_signal = use_map(props.handle);
    props.probe.push(map_signal.entries.read().clone());
    props.methods_probe.push(map_signal);

    rsx! { div {} }
}

#[tokio::test]
async fn test_use_map_requirements() {
    let store = unique_store("map");
    let arena = Arena::new();

    let map = rpstate::store::reactive_map_with_path::<DummyScope, String, String, _, _>(
        &store,
        std::sync::Arc::from("map_1"),
        HashMap::new(),
    )
    .unwrap();
    let handle = arena.register_map(map);

    let _ = arena.set_map_entry(handle, "key1".to_string(), "val1".to_string());

    let probe = Probe::new();
    let methods_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        |props: MapTestProps| {
            use_context_provider(|| props.arena.clone());
            rsx! { MapTestComponent { ..props } }
        },
        MapTestProps {
            arena: arena.clone(),
            handle,
            probe: probe.clone(),
            methods_probe: methods_probe.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    let initial = probe.last().unwrap();
    assert_eq!(initial.get("key1").unwrap(), "val1");

    let methods = methods_probe.last().unwrap();

    methods.set_or_create("key2".to_string(), "val2".to_string());
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last().unwrap().get("key2").unwrap(), "val2");

    methods.remove("key1".to_string());
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert!(!probe.last().unwrap().contains_key("key1"));

    methods.clear();
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert!(probe.last().unwrap().is_empty());

    let _ = arena.set_map_entry(handle, "external".to_string(), "value".to_string());
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last().unwrap().get("external").unwrap(), "value");
}

#[derive(Clone, Props)]
struct PipelineProps {
    arena: Arena,
    dep_handle: WritableHandle<i32, DefaultStore>,
    probe: Probe<i32>,
    handle_probe: Probe<PipelineHandle<i32>>,
}

impl PartialEq for PipelineProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone)]
struct PipelineCleanup {
    arena: Arena,
    handle: PipelineHandle<i32>,
}

impl Drop for PipelineCleanup {
    fn drop(&mut self) {
        self.arena.remove_pipeline(self.handle);
    }
}
#[component]
fn PipelineTestComponent(props: PipelineProps) -> Element {
    let dep = props.dep_handle;
    let val = use_pipeline(move || dep.pipe().map(|v| v * 2));

    props.probe.push(*val.read());

    let arena = use_context::<Arena>();
    let cleanup = use_hook(|| {
        let builder = PipelineBuilder::new(&arena);
        let pipeline = builder.field(dep).pipe().map(|v| v * 2);
        let handle = arena.register_pipeline(pipeline);
        Arc::new(PipelineCleanup {
            arena: arena.clone(),
            handle,
        })
    });

    props.handle_probe.push(cleanup.handle);

    rsx! { div {} }
}

#[tokio::test]
async fn test_use_pipeline_requirements() {
    let store = unique_store("pipeline");
    let arena = Arena::new();

    let field =
        rpstate::store::field_with_path(&store, std::sync::Arc::from("field_2"), 5).unwrap();
    let dep_handle = arena.register_field(field);

    let probe = Probe::new();
    let handle_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        |props: PipelineProps| {
            use_context_provider(|| props.arena.clone());
            rsx! { PipelineTestComponent { ..props } }
        },
        PipelineProps {
            arena: arena.clone(),
            dep_handle,
            probe: probe.clone(),
            handle_probe: handle_probe.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last(), Some(10));

    let _ = arena.set_field(dep_handle, 20);

    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    assert_eq!(probe.last(), Some(40));

    let handle = handle_probe.last().unwrap();

    drop(vdom);
    handle_probe.clear();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        arena.get_pipeline(handle);
    }));
    assert!(result.is_err());
}

#[rpstate_macros_dioxus::rpstate_dioxus]
#[rpstate(prefix = "test")]
pub struct MyTestState {
    #[state(default = 0)]
    pub id: usize,
}

#[derive(Clone, Props)]
struct RpStateProps {
    parent_probe: Probe<MyTestStateHandle>,
    child_probe: Probe<MyTestStateHandle>,
}

impl PartialEq for RpStateProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn RpStateParent(props: RpStateProps) -> Element {
    let handle = use_rpstate::<MyTestState>();
    props.parent_probe.push(handle);

    rsx! {
        RpStateChild {
            parent_probe: props.parent_probe.clone(),
            child_probe: props.child_probe.clone(),
        }
    }
}

#[component]
fn RpStateChild(props: RpStateProps) -> Element {
    let handle = use_rpstate::<MyTestState>();
    props.child_probe.push(handle);

    rsx! { div {} }
}

#[derive(Clone, Props)]
struct RpStateTestWrapperProps {
    store: Arc<DefaultStore>,
    parent_probe: Probe<MyTestStateHandle>,
    child_probe: Probe<MyTestStateHandle>,
}

impl PartialEq for RpStateTestWrapperProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.store, &other.store)
            && self.parent_probe == other.parent_probe
            && self.child_probe == other.child_probe
    }
}

#[component]
fn RpStateTestWrapper(props: RpStateTestWrapperProps) -> Element {
    rsx! {
        RpStateProvider {
            store: props.store.clone(),
            RpStateParent {
                parent_probe: props.parent_probe.clone(),
                child_probe: props.child_probe.clone(),
            }
        }
    }
}

#[tokio::test]
async fn test_use_rpstate_requirements() {
    let store = unique_store("rpstate");

    let parent_probe = Probe::new();
    let child_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        RpStateTestWrapper,
        RpStateTestWrapperProps {
            store,
            parent_probe: parent_probe.clone(),
            child_probe: child_probe.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    let parent_handle = parent_probe.last().unwrap();
    let child_handle = child_probe.last().unwrap();

    assert!(parent_handle == child_handle);
}

#[derive(Clone, Props)]
struct MapSubProps {
    arena: Arena,
    handle: WritableMapHandle<String, String, DefaultStore>,
    any_changes: Arc<Mutex<Vec<MapChange<String, String>>>>,
    key_changes: Arc<Mutex<Vec<MapChange<String, String>>>>,
}

impl PartialEq for MapSubProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn MapSubComponent(props: MapSubProps) -> Element {
    let any_changes = props.any_changes.clone();
    use_map_subscribe_any(props.handle, move |change| {
        any_changes.lock().unwrap().push(change.clone());
    });

    let key_changes = props.key_changes.clone();
    use_map_subscribe_key(props.handle, "target".to_string(), move |change| {
        key_changes.lock().unwrap().push(change.clone());
    });

    rsx! { div {} }
}

#[tokio::test]
async fn test_map_sub_requirements() {
    let store = unique_store("sub");
    let arena = Arena::new();

    let map = rpstate::store::reactive_map_with_path::<DummyScope, String, String, _, _>(
        &store,
        std::sync::Arc::from("map_2"),
        HashMap::new(),
    )
    .unwrap();
    let write_handle = arena.register_map(map);

    let any_changes = Arc::new(Mutex::new(Vec::new()));
    let key_changes = Arc::new(Mutex::new(Vec::new()));

    let mut vdom = VirtualDom::new_with_props(
        |props: MapSubProps| {
            use_context_provider(|| props.arena.clone());
            rsx! { MapSubComponent { ..props } }
        },
        MapSubProps {
            arena: arena.clone(),
            handle: write_handle,
            any_changes: any_changes.clone(),
            key_changes: key_changes.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    let _ = arena.set_map_entry(write_handle, "target".to_string(), "hello".to_string());
    tokio::task::yield_now().await;

    assert_eq!(any_changes.lock().unwrap().len(), 1);
    assert_eq!(key_changes.lock().unwrap().len(), 1);

    let _ = arena.set_map_entry(write_handle, "other".to_string(), "world".to_string());
    tokio::task::yield_now().await;

    assert_eq!(any_changes.lock().unwrap().len(), 2);
    assert_eq!(key_changes.lock().unwrap().len(), 1);

    drop(vdom);

    let _ = arena.set_map_entry(write_handle, "target".to_string(), "dropped".to_string());
    tokio::task::yield_now().await;

    assert_eq!(any_changes.lock().unwrap().len(), 2);
    assert_eq!(key_changes.lock().unwrap().len(), 1);
}

#[derive(Clone, Props)]
struct AllPrimitivesProps {
    arena: Arena,
    field_handle: WritableHandle<i32, DefaultStore>,
    map_handle: WritableMapHandle<String, String, DefaultStore>,

    field_probe: Probe<i32>,
    map_probe: Probe<HashMap<String, String>>,
    pipeline_probe: Probe<i32>,
    map_sub_any_probe: Probe<MapChange<String, String>>,
    map_sub_key_probe: Probe<MapChange<String, String>>,
    pipeline_handle_probe: Probe<PipelineHandle<i32>>,
}

impl PartialEq for AllPrimitivesProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn AllPrimitivesComponent(props: AllPrimitivesProps) -> Element {
    let (field_val, _field_setter) = use_field(props.field_handle);
    props.field_probe.push(*field_val.read());

    let map_signal = use_map(props.map_handle);
    props.map_probe.push(map_signal.entries.read().clone());

    let field_handle_copy = props.field_handle;
    let pipeline_val = use_pipeline(move || field_handle_copy.pipe().map(|v| v * 2));

    props.pipeline_probe.push(*pipeline_val.read());

    let map_sub_any_probe = props.map_sub_any_probe.clone();
    use_map_subscribe_any(props.map_handle, move |change| {
        map_sub_any_probe.push(change.clone());
    });

    let map_sub_key_probe = props.map_sub_key_probe.clone();
    use_map_subscribe_key(props.map_handle, "target".to_string(), move |change| {
        map_sub_key_probe.push(change.clone());
    });

    let arena = use_context::<Arena>();
    let cleanup = use_hook(|| {
        let builder = PipelineBuilder::new(&arena);
        let pipeline = builder.field(field_handle_copy).pipe().map(|v| v * 3);
        let handle = arena.register_pipeline(pipeline);
        Arc::new(PipelineCleanup {
            arena: arena.clone(),
            handle,
        })
    });
    props.pipeline_handle_probe.push(cleanup.handle);

    rsx! { div {} }
}

#[derive(Clone, Props)]
struct AllPrimitivesToggleProps {
    arena: Arena,
    field_handle: WritableHandle<i32, DefaultStore>,
    map_handle: WritableMapHandle<String, String, DefaultStore>,

    field_probe: Probe<i32>,
    map_probe: Probe<HashMap<String, String>>,
    pipeline_probe: Probe<i32>,
    map_sub_any_probe: Probe<MapChange<String, String>>,
    map_sub_key_probe: Probe<MapChange<String, String>>,
    pipeline_handle_probe: Probe<PipelineHandle<i32>>,

    signal_probe: Probe<Signal<bool>>,
}

impl PartialEq for AllPrimitivesToggleProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn AllPrimitivesToggleComponent(props: AllPrimitivesToggleProps) -> Element {
    use_context_provider(|| props.arena.clone());
    let toggle = use_signal(|| false);

    use_hook(|| {
        props.signal_probe.push(toggle);
    });

    rsx! {
        if *toggle.read() {
            AllPrimitivesComponent {
                arena: props.arena.clone(),
                field_handle: props.field_handle,
                map_handle: props.map_handle,
                field_probe: props.field_probe.clone(),
                map_probe: props.map_probe.clone(),
                pipeline_probe: props.pipeline_probe.clone(),
                map_sub_any_probe: props.map_sub_any_probe.clone(),
                map_sub_key_probe: props.map_sub_key_probe.clone(),
                pipeline_handle_probe: props.pipeline_handle_probe.clone(),
            }
        }
    }
}

#[tokio::test]
async fn test_all_primitives_simultaneous_lifecycle() {
    let store = unique_store("all_primitives");
    let arena = Arena::new();

    let field =
        rpstate::store::field_with_path(&store, std::sync::Arc::from("field_all"), 10).unwrap();
    let field_handle = arena.register_field(field);

    let map = rpstate::store::reactive_map_with_path::<DummyScope, String, String, _, _>(
        &store,
        std::sync::Arc::from("map_all"),
        HashMap::new(),
    )
    .unwrap();
    let map_handle = arena.register_map(map);

    let field_probe = Probe::new();
    let map_probe = Probe::new();
    let pipeline_probe = Probe::new();
    let map_sub_any_probe = Probe::new();
    let map_sub_key_probe = Probe::new();
    let pipeline_handle_probe = Probe::new();
    let signal_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        AllPrimitivesToggleComponent,
        AllPrimitivesToggleProps {
            arena: arena.clone(),
            field_handle,
            map_handle,
            field_probe: field_probe.clone(),
            map_probe: map_probe.clone(),
            pipeline_probe: pipeline_probe.clone(),
            map_sub_any_probe: map_sub_any_probe.clone(),
            map_sub_key_probe: map_sub_key_probe.clone(),
            pipeline_handle_probe: pipeline_handle_probe.clone(),
            signal_probe: signal_probe.clone(),
        },
    );

    vdom.rebuild(&mut dioxus::core::NoOpMutations);

    let mut toggle_signal = signal_probe.last().expect("Exposed toggle signal missing");

    toggle_signal.set(true);
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;

    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(field_probe.last(), Some(10));
    assert!(map_probe.last().unwrap().is_empty());
    assert_eq!(pipeline_probe.last(), Some(20));

    let pipe_handle1 = pipeline_handle_probe
        .last()
        .expect("Manual pipeline missing");
    assert_eq!(arena.get_pipeline(pipe_handle1), 30);

    let _ = arena.set_field(field_handle, 100);
    let _ = arena.set_map_entry(map_handle, "target".to_string(), "hello".to_string());
    let _ = arena.set_map_entry(map_handle, "other".to_string(), "world".to_string());

    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(field_probe.last(), Some(100));
    assert_eq!(pipeline_probe.last(), Some(200));
    assert_eq!(arena.get_pipeline(pipe_handle1), 300);

    assert_eq!(map_sub_any_probe.count(), 2);
    assert_eq!(map_sub_key_probe.count(), 1);

    toggle_signal.set(false);
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    let result1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        arena.get_pipeline(pipe_handle1);
    }));
    assert!(result1.is_err());

    let _ = arena.set_map_entry(
        map_handle,
        "target".to_string(),
        "dropped_value".to_string(),
    );
    tokio::task::yield_now().await;

    assert_eq!(map_sub_any_probe.count(), 2);
    assert_eq!(map_sub_key_probe.count(), 1);

    toggle_signal.set(true);
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    let pipe_handle2 = pipeline_handle_probe
        .last()
        .expect("Second manual pipeline missing");
    assert!(pipe_handle1 != pipe_handle2);
    assert_eq!(arena.get_pipeline(pipe_handle2), 300);

    let _ = arena.set_map_entry(map_handle, "target".to_string(), "new_value".to_string());
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(map_sub_any_probe.count(), 3);
    assert_eq!(map_sub_key_probe.count(), 2);

    toggle_signal.set(false);
    tokio::task::yield_now().await;
    let _ = vdom.wait_for_work().await;
    vdom.render_immediate(&mut dioxus::core::NoOpMutations);

    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        arena.get_pipeline(pipe_handle2);
    }));
    assert!(result2.is_err());
}

#[derive(Clone, Props)]
struct PipelineLifecycleProps {
    arena: Arena,
    field_a: WritableHandle<i32, DefaultStore>,
    field_b: WritableHandle<i32, DefaultStore>,
    probe: Probe<String>,
    signal_probe: Probe<Signal<bool>>,
}

impl PartialEq for PipelineLifecycleProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[component]
fn PipelineLifecycleInner(
    arena: Arena,
    field_a: WritableHandle<i32, DefaultStore>,
    field_b: WritableHandle<i32, DefaultStore>,
    probe: Probe<String>,
) -> Element {
    let val = use_pipeline(move || (field_a, field_b).pipe().map(|(a, b)| format!("{a}:{b}")));
    probe.push(val.read().clone());
    rsx! { div {} }
}

#[component]
fn PipelineLifecycleRoot(props: PipelineLifecycleProps) -> Element {
    use_context_provider(|| props.arena.clone());
    let toggle = use_signal(|| false);
    use_hook(|| {
        props.signal_probe.push(toggle);
    });

    rsx! {
        if *toggle.read() {
            PipelineLifecycleInner {
                arena: props.arena.clone(),
                field_a: props.field_a,
                field_b: props.field_b,
                probe: props.probe.clone(),
            }
        }
    }
}

#[tokio::test]
async fn test_pipeline_lifecycle_and_tuple_pipe() {
    let store = unique_store("pipeline_lifecycle");
    let arena = Arena::new();

    let fa = arena.register_field(field_with_path(&store, Arc::from("fa"), 1).unwrap());
    let fb = arena.register_field(field_with_path(&store, Arc::from("fb"), 2).unwrap());

    let probe = Probe::new();
    let signal_probe = Probe::new();

    let mut vdom = VirtualDom::new_with_props(
        PipelineLifecycleRoot,
        PipelineLifecycleProps {
            arena: arena.clone(),
            field_a: fa,
            field_b: fb,
            probe: probe.clone(),
            signal_probe: signal_probe.clone(),
        },
    );
    vdom.rebuild(&mut NoOpMutations);

    let mut toggle = signal_probe.last().unwrap();

    toggle.set(true);
    tokio::task::yield_now().await;
    vdom.wait_for_work().await;
    vdom.render_immediate(&mut NoOpMutations);
    assert_eq!(probe.last().as_deref(), Some("1:2"));

    arena.set_field(fa, 10).unwrap();
    arena.set_field(fb, 20).unwrap();
    tokio::task::yield_now().await;
    vdom.wait_for_work().await;
    vdom.render_immediate(&mut NoOpMutations);
    assert_eq!(probe.last().as_deref(), Some("10:20"));

    toggle.set(false);
    tokio::task::yield_now().await;
    vdom.wait_for_work().await;
    vdom.render_immediate(&mut NoOpMutations);

    let count_before = probe.count();

    arena.set_field(fa, 99).unwrap();
    tokio::task::yield_now().await;
    assert_eq!(probe.count(), count_before);

    toggle.set(true);
    tokio::task::yield_now().await;
    vdom.wait_for_work().await;
    vdom.render_immediate(&mut NoOpMutations);
    assert_eq!(probe.last().as_deref(), Some("99:20"));

    arena.set_field(fb, 42).unwrap();
    tokio::task::yield_now().await;
    vdom.wait_for_work().await;
    vdom.render_immediate(&mut NoOpMutations);
    assert_eq!(probe.last().as_deref(), Some("99:42"));
}
