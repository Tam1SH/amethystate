use crate::store::migration::fields::RpStateFields;
use crate::store::{StoreCallback, SubscriptionId};
use crate::{DefaultStore, SubscriptionKind};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub trait AccessMode: Send + Sync + 'static {}
pub struct ReadOnlyMode;
impl AccessMode for ReadOnlyMode {}
pub struct WritableMode;
impl AccessMode for WritableMode {}

pub struct ReadOnly<T>(std::marker::PhantomData<T>);
pub struct Writable<T>(std::marker::PhantomData<T>);

pub trait RpStateNode: Sized {
    fn new_node(store: &Arc<DefaultStore>, path: &str) -> crate::store::Result<Self>;
}

pub trait RpState {
    type Data: RpStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PrefixMeta {
    pub version: u32,
    pub hash: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffEntry {
    pub timestamp: u64,
    pub old_hash: u64,
    pub new_hash: u64,
}

pub struct SubscriptionEntry {
    pub id: SubscriptionId,
    pub kind: SubscriptionKind,
    pub callback: StoreCallback,
}

pub fn matches_kind(kind: &SubscriptionKind, path: &str) -> bool {
    match kind {
        SubscriptionKind::Any => true,
        SubscriptionKind::ExactPath(p) => **p == *path,
        SubscriptionKind::Prefix(prefix) => {
            *path == **prefix
                || path
                    .strip_prefix(&**prefix)
                    .is_some_and(|t| t.starts_with('.'))
        }
    }
}
