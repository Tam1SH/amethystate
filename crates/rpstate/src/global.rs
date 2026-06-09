use crate::store::builder::WithMigrations;
use crate::{DefaultStore, MigrationReport, StoreBuilder};
use std::path::Path;
use std::sync::OnceLock;

static GLOBAL_STORE: OnceLock<DefaultStore> = OnceLock::new();

pub trait IntoGlobalStore {
    type Output;

    fn init_global(self) -> Self::Output;
}

impl IntoGlobalStore for StoreBuilder {
    type Output = ();

    fn init_global(self) -> Self::Output {
        let store = self.build().unwrap_or_else(|err| {
            panic!(
                "rpstate: Failed to build global Store.\n\
                     Ensure the database path is writable and not locked by another process.\n\
                     Details: {err}"
            );
        });

        GLOBAL_STORE.set(store).unwrap_or_else(|_| {
            panic!(
                "rpstate: Global store is already initialized.\n\
                     Ensure `init_global` is called exactly once during application startup."
            );
        });
    }
}

impl IntoGlobalStore for StoreBuilder<WithMigrations> {
    type Output = MigrationReport;

    fn init_global(self) -> Self::Output {
        let (store, report) = self
            .build()
            .unwrap_or_else(|err| {
                panic!(
                    "rpstate: Failed to build global Store and run migrations.\n\
                     Please verify that the database file is not corrupted and your migration logic is correct.\n\
                     Details: {err}"
                );
            });

        GLOBAL_STORE.set(store).unwrap_or_else(|_| {
            panic!(
                "rpstate: Global store is already initialized.\n\
                     Ensure `init_global` is called exactly once during application startup."
            );
        });

        report
    }
}

impl IntoGlobalStore for &str {
    type Output = ();

    fn init_global(self) -> Self::Output {
        StoreBuilder::new(self).init_global()
    }
}

impl IntoGlobalStore for &Path {
    type Output = ();

    fn init_global(self) -> Self::Output {
        StoreBuilder::new(self).init_global()
    }
}

pub fn init_global<T: IntoGlobalStore>(source: T) -> T::Output {
    source.init_global()
}

pub fn global_store() -> DefaultStore {
    GLOBAL_STORE.get().unwrap().clone()
}
