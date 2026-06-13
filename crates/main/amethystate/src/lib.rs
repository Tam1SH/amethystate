#![allow(clippy::complexity)]
mod codec;
mod error;
mod global;
mod macros;

pub mod migration;
pub mod reactive;
pub mod store;

pub type AmeData<T> = <T as AmeState>::Data;

pub use inventory;
pub use serde;
pub use uuid;

pub use error::Error;
pub use error::Result;
pub use reactive::{
    AccessMode, Change, Field, InterceptDisposer, IntoPipeline, MapChange, Pipeline, Reactive,
    ReactiveMap, ReactiveMapKey, ReactiveMapValue, ReactiveScope, ReadOnly, ReadOnlyField,
    ReadOnlyMode, AmeState, AmeStateNode, Signal, SignalSubscription, StoreSubscription, Writable,
    WritableField, WritableMode,
};

pub mod stores {
    pub use crate::store::default::*;
}

pub use store::{
    builder::StoreBuilder, config::StoreConfig, default::DefaultStore, join_path, AmeStateSlice, StateScope, Store,
    StoreEvent, StoreOp, SubscriptionKind,
};

pub use migration::{MigrationContext, MigrationError, MigrationPlan, MigrationReport};

pub use global::*;
pub use amethystate_macros::{migrate, amethystate, AmeType};

#[cfg(any(feature = "tauri", feature = "json"))]
pub use serde_json;

#[cfg(any(feature = "confy-compat", feature = "confy-compat-0-6"))]
pub mod confy;

#[cfg(any(feature = "test-utils", test))]
pub mod test_utils;

#[cfg(feature = "tauri")]
pub mod tauri {
    pub use amethystate_core::scheme::*;
    pub use amethystate_tauri::*;
}

pub mod core {
    pub use amethystate_core::*;
}

#[cfg(any(feature = "async", feature = "tauri"))]
pub mod client {
    pub use amethystate_core::async_impl::*;
    pub use amethystate_core::AmeBackendAsync;
    pub use amethystate_core::AmeStateSliceAsync;
}
