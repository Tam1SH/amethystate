use crate::migration::fields::AmeStateFields;
use crate::Store;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait AmeStateNode<S: Store>: Sized {
    fn new_node(store: &S, path: &str) -> crate::Result<Self>;
    fn new_node_with_id(store: &S, path: &str, instance_id: Uuid) -> crate::Result<Self>;
}

pub trait AmeState {
    type Data: AmeStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}
