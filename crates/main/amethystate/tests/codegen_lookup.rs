use amethystate::store::builder::StoreBuilder;
use amethystate_core::test_utils::unique_path;
use amethystate_macros::amethystate;

#[amethystate]
pub struct InnerNode {
    #[amestate(default = 42, export_mut)]
    pub leaf: i32,
}

#[amethystate(prefix = "sys")]
pub struct RootNode {
    #[amestate(nested)]
    pub inner: InnerNode,
}

#[amethystate(prefix = "ui")]
pub struct Dashboard {
    #[amestate(lookup = "inner.leaf", parent = RootNode)]
    pub value: i32,

    #[amestate(lookup = "inner.leaf", parent = RootNode, export_mut)]
    pub writable_value: i32,
}

#[test]
fn test_deep_lookup_compilation_and_runtime() {
    let path = unique_path("deep_lookup.redb");

    let store = StoreBuilder::new(&path).build().unwrap();

    let root = RootNode::new_with(&store).unwrap();
    let ui = Dashboard::new_with(&store).unwrap();

    root.inner().leaf().set(100).unwrap();

    assert_eq!(ui.value().get(), 100);

    ui.writable_value().set(200).unwrap();
    assert_eq!(root.inner().leaf().get(), 200);
}
