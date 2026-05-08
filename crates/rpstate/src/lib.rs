pub mod signal;
pub mod store;

use serde::de::DeserializeOwned;
use serde::Serialize;
pub use signal::{Signal, SignalSubscription};
use std::sync::Arc;
pub use store::Result;
pub use store::{StateScope, Store, StoreEvent, StoreOp, SubscriptionKind};

#[cfg(feature = "json")]
pub use store::JsonStore;

#[cfg(feature = "redb")]
pub use store::RedbStore;

#[cfg(all(feature = "json", not(feature = "redb")))]
pub type DefaultStore = JsonStore;

#[cfg(feature = "redb")]
pub type DefaultStore = RedbStore;

pub type StoreSubscription = store::field::StoreSubscription<DefaultStore>;

pub use crate::store::shared::{ReadOnlyMode, WritableMode};
pub use store::scoped_path;

pub type Field<T, M = ReadOnlyMode> = store::field::Field<T, DefaultStore, M>;

pub fn field<TScope, TValue>(
    store: &Arc<DefaultStore>,
    key: &str,
    default: TValue,
) -> Result<Field<TValue, WritableMode>>
where
    TScope: StateScope,
    TValue: Serialize + Default + DeserializeOwned + Clone + Send + Sync + 'static,
{
    store::field::<TScope, TValue, DefaultStore>(store, key, default)
}
