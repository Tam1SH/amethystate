use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::DefaultStore;
use crate::store::migration::fields::RpStateFields;

pub trait RpStateNode: Sized {
    fn new_node(store: &Arc<DefaultStore>, path: &str) -> crate::store::Result<Self>;
}

pub trait RpState {
    type Data: RpStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}