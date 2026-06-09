use crate::Store;
use crate::migration::fields::RpStateFields;
use serde::{Deserialize, Serialize};

pub trait RpStateNode<S: Store>: Sized {
    fn new_node(store: &S, path: &str) -> crate::Result<Self>;
}

pub trait RpState {
    type Data: RpStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}
