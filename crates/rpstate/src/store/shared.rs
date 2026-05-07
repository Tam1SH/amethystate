use crate::SubscriptionKind;
use crate::store::{StoreCallback, SubscriptionId};
use serde::{Deserialize, Serialize};

pub struct ReadOnly<T>(std::marker::PhantomData<T>);
pub struct Writable<T>(std::marker::PhantomData<T>);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
