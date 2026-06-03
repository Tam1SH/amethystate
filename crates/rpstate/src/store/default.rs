use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::{Result, Store, StoreConfig, SubscriptionKind};

#[cfg(feature = "redb")]
use crate::store::{MigrationReport, MigrationSet, SchemaAwareStore};
use crate::store::{StoreCallback, SubscriptionId, backend};

#[cfg(feature = "json")]
pub use backend::json::JsonStore;

#[cfg(feature = "redb")]
pub use backend::redb::RedbStore;

#[cfg(feature = "redb")]
pub type DefaultStore = RedbStore;
#[cfg(feature = "json")]
type DefaultStore = JsonStore;
#[cfg(all(feature = "json", not(feature = "redb")))]
type DefaultStoreInner = JsonStore;
