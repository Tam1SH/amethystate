#![allow(clippy::complexity)]
mod codec;
mod error;
mod global;
mod macros;

pub mod migration;
pub mod observability;
pub mod reactive;
pub mod store;

pub type AmeData<T> = <T as AmeState>::Data;

pub use inventory;
pub use serde;
pub use uuid;

pub use error::Error;
pub use error::Result;
pub use reactive::{
    AccessMode, AmeState, AmeStateNode, Change, Field, InterceptDisposer, IntoPipeline, MapChange,
    Pipeline, Reactive, ReactiveMap, ReactiveMapKey, ReactiveMapValue, ReactiveScope, ReadOnly,
    ReadOnlyField, ReadOnlyMode, Signal, SignalSubscription, StoreSubscription, Writable,
    WritableField, WritableMode,
};

pub mod stores {
    pub use crate::store::default::*;
}

pub use store::{
    AmeStateSlice, StateScope, Store, StoreEvent, StoreOp, SubscriptionKind, builder::StoreBuilder,
    config::StoreConfig, default::DefaultStore, join_path,
};

pub use migration::{MigrationContext, MigrationError, MigrationPlan, MigrationReport};

pub use amethystate_macros::{AmeType, amethystate, migrate};
pub use global::*;

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
    pub use amethystate_core::AmeBackendAsync;
    pub use amethystate_core::AmeStateSliceAsync;
    pub use amethystate_core::async_impl::*;
}
