pub mod signal;
pub mod store;

use serde::de::DeserializeOwned;
use serde::Serialize;
pub use signal::{Signal, SignalSubscription};
use std::sync::Arc;
pub use store::Result;
pub use store::{StateScope, Store, StoreEvent, StoreOp, SubscriptionKind};

pub use inventory;
pub use serde;

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

#[macro_export]
macro_rules! register_migrations {
    ($T:ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        ::inventory::submit! {
            $crate::store::migration::registry::MigrationEntry {
                prefix:       <$T as $crate::StateScope>::PREFIX,
                dependencies: <$T as $crate::store::migration::registry::HasMigrations>::MIGRATION_DEPS,
                build:        <$T as $crate::store::migration::registry::HasMigrations>::migrations,
            }
        }
    };
}

#[macro_export]
macro_rules! migrate {
    (
        $old:path => $new:path,
        rename: [$($old_f:ident => $new_f:ident),* $(,)?]
        $(, drop: [$($drop_f:ident),* $(,)?])?
        $(, convert: [$($conv_f:ident : $conv_old:ty => $conv_new:ty),* $(,)?])?
        , |$old_val:ident| $logic_block:block
    ) => {
        impl $crate::store::migration::migrate_from::MigrateFrom<$old> for $new {
            const RENAMES: &'static [(&'static str, &'static str)] = &[
                $((stringify!($old_f), stringify!($new_f))),*
            ];
            const DROPPED: &'static [&'static str] = &[
                $($(stringify!($drop_f)),*)?
            ];
            const CONVERTS: &'static [(&'static str, u64, u64)] = &[
                $($( (
                    stringify!($conv_f),
                    <$conv_old as $crate::store::migration::types::RpType>::TYPE_HASH,
                    <$conv_new as $crate::store::migration::types::RpType>::TYPE_HASH,
                ) ),*)?
            ];

            fn migrate($old_val: $old) -> $crate::store::Result<Self> {
                $logic_block
            }
        }

        $crate::inventory::submit! {
            $crate::store::migration::registry::MigrationStepEntry {
                prefix: <$old as $crate::store::migration::fields::RpStateFields>::PARENT_PREFIX,
                target_version: <$old as $crate::store::migration::fields::RpStateFields>::VERSION,
                dependencies: <$old as $crate::store::migration::fields::RpStateFields>::MIGRATION_DEPS,
                description: concat!(
                    "baseline v",
                    stringify!(<$old as $crate::store::migration::fields::RpStateFields>::VERSION)
                ),
                run: |_ctx| Ok(())
            }
        }

        $crate::inventory::submit! {
            $crate::store::migration::registry::MigrationStepEntry {
                prefix: <$new as $crate::store::migration::fields::RpStateFields>::PARENT_PREFIX,
                target_version: <$new as $crate::store::migration::fields::RpStateFields>::VERSION,
                dependencies: <$new as $crate::store::migration::fields::RpStateFields>::MIGRATION_DEPS,
                description: concat!("Migration to v", stringify!(<$new as ::rpstate::store::migration::fields::RpStateFields>::VERSION)),
                run: |ctx| {
                    use $crate::store::migration::fields::RpStateFields;
                    use $crate::store::migration::migrate_from::MigrateFrom;

                    let old_data = <$old as RpStateFields>::load_struct(ctx)?;
                    let new_data = <$new as MigrateFrom<$old>>::migrate(old_data)?;

                    for (old_k, _) in <$new as MigrateFrom<$old>>::RENAMES { ctx.delete(old_k)?; }
                    for drop_k in <$new as MigrateFrom<$old>>::DROPPED { ctx.delete(drop_k)?; }

                    new_data.save_struct(ctx)?;
                    Ok(())
                }
            }
        }
    };
}
