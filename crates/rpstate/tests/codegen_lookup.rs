use rpstate::store::builder::StoreBuilder;
use rpstate_macros::rpstate;

#[rpstate]
pub struct InnerNode {
    #[state(default = 42, export_mut)]
    pub leaf: i32,
}

#[rpstate(prefix = "sys")]
pub struct RootNode {
    #[state(nested)]
    pub inner: InnerNode,
}

#[rpstate(prefix = "ui")]
pub struct Dashboard {
    #[state(lookup = "inner.leaf", parent = RootNode)]
    pub value: i32,

    #[state(lookup = "inner.leaf", parent = RootNode, export_mut)]
    pub writable_value: i32,
}

#[test]
fn test_deep_lookup_compilation_and_runtime() {
    let path = std::env::temp_dir().join("deep_lookup.redb");
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }

    let store = StoreBuilder::new(&path).build().unwrap();

    let root = RootNode::new_with(&store).unwrap();
    let ui = Dashboard::new_with(&store).unwrap();

    root.inner().leaf().set(100).unwrap();

    assert_eq!(ui.value().get(), 100);

    ui.writable_value().set(200).unwrap();
    assert_eq!(root.inner().leaf().get(), 200);
}
