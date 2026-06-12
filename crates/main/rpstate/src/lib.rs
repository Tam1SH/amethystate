#![allow(clippy::complexity)]
mod codec;
mod error;
mod global;
mod macros;

pub mod migration;
pub mod reactive;
pub mod store;

pub type RpData<T> = <T as RpState>::Data;

pub use inventory;
pub use serde;
pub use uuid;

pub use error::Error;
pub use error::Result;
pub use reactive::{
    AccessMode, Change, Field, InterceptDisposer, IntoPipeline, MapChange, Pipeline, Reactive,
    ReactiveMap, ReactiveMapKey, ReactiveMapValue, ReactiveScope, ReadOnly, ReadOnlyField,
    ReadOnlyMode, RpState, RpStateNode, Signal, SignalSubscription, StoreSubscription, Writable,
    WritableField, WritableMode,
};

pub mod stores {
    pub use crate::store::default::*;
}

pub use store::{
    builder::StoreBuilder, config::StoreConfig, default::DefaultStore, join_path, RpStateSlice, StateScope, Store,
    StoreEvent, StoreOp, SubscriptionKind,
};

pub use migration::{MigrationContext, MigrationError, MigrationPlan, MigrationReport};

pub use global::*;
pub use rpstate_macros::{migrate, rpstate, RpType};

#[cfg(any(feature = "tauri", feature = "json"))]
pub use serde_json;

#[cfg(any(feature = "confy-compat", feature = "confy-compat-0-6"))]
pub mod confy;

#[cfg(any(feature = "test-utils", test))]
pub mod test_utils;

#[cfg(feature = "tauri")]
pub mod tauri {
    pub use rpstate_core::scheme::*;
    pub use rpstate_tauri::*;
}

pub mod core {
    pub use rpstate_core::*;
}

#[cfg(any(feature = "async", feature = "tauri"))]
pub mod client {
    pub use rpstate_core::async_impl::*;
    pub use rpstate_core::RpBackendAsync;
    pub use rpstate_core::RpStateSliceAsync;
}
