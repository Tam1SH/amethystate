use crate::DefaultStore;
use crate::migration::fields::RpStateFields;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub trait RpStateNode: Sized {
    fn new_node(store: &Arc<DefaultStore>, path: &str) -> crate::Result<Self>;
}

pub trait RpState {
    type Data: RpStateFields + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
}
