use crate::DefaultStore;
use crate::migration::fields::RpStateFields;
use serde::{Deserialize, Serialize};

pub trait RpStateNode: Sized {
    fn new_node(store: &DefaultStore, path: &str) -> crate::Result<Self>;
}

pub trait RpState {
    type Data: RpStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}
