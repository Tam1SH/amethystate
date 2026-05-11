use rpstate::store::builder::StoreBuilder;
use rpstate::{migrate, migrate_field, Store};
use rpstate_macros::rpstate;
use std::sync::Arc;

mod v1 {
    use super::*;

    #[rpstate]
    pub struct NetworkSettings {
        #[state(default = 8080)]
        pub port: u16,
    }

    #[rpstate(prefix = "system", version = 1)]
    pub struct SystemConfig {
        #[state(nested)]
        pub net: NetworkSettings,
    }

    #[rpstate(prefix = "ui")]
    pub struct Dashboard {
        #[state(lookup = "net.port", parent = SystemConfig)]
        pub sys_port: u16,

        #[state(lookup_node = "net", parent = SystemConfig)]
        pub net_node: NetworkSettings,

        #[state(default = false, volatile)]
        pub is_loading: bool,
    }
}

#[rpstate]
pub struct NetworkSettings {
    #[state(default = 8080)]
    pub listen_port: u16,
}

#[rpstate(prefix = "system", version = 2)]
pub struct SystemConfig {
    #[state(nested)]
    pub net: NetworkSettings,
}

#[rpstate(prefix = "ui")]
pub struct Dashboard {
    #[state(lookup = "net.listen_port", parent = SystemConfig)]
    pub sys_port: u16,

    #[state(lookup_node = "net", parent = SystemConfig)]
    pub net_node: NetworkSettings,

    #[state(default = false, volatile)]
    pub is_loading: bool,
}

migrate! {
    v1::NetworkSettings_Data => NetworkSettings_Data,
    rename: [port => listen_port],
    |old, _ctx| {
        Ok(Self { listen_port: old.port })
    }
}

migrate! {
    v1::SystemConfig_Data => SystemConfig_Data,
    rename: [],
    |old, ctx| {
        Ok(Self {
            net: migrate_field!(ctx, old.net),
        })
    }
}

#[test]
fn test_nested_and_ephemeral_integration() {
    let path = std::env::temp_dir().join("rpstate_ephemeral_test.redb");
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }

    {
        let store = Arc::new(StoreBuilder::new(&path).build().unwrap());

        let sys = v1::SystemConfig::new(&store).unwrap();
        let ui = v1::Dashboard::new(&store).unwrap();

        sys.net().port().set(9999).unwrap();
        ui.is_loading().set(true).unwrap();

        assert_eq!(ui.sys_port().get(), 9999);
        assert_eq!(ui.net_node().port().get(), 9999);
        assert!(ui.is_loading().get());

        store.save_now().unwrap();
    }

    {
        let store = Arc::new(
            StoreBuilder::new(&path)
                .collect_migrations()
                .build()
                .unwrap(),
        );

        let sys = SystemConfig::new(&store).expect("Failed to load v2 system");
        let ui = Dashboard::new(&store).expect("Failed to load dashboard");

        assert_eq!(sys.net().listen_port().get(), 9999);

        assert_eq!(ui.sys_port().get(), 9999);

        assert_eq!(ui.net_node().listen_port().get(), 9999);

        assert!(!ui.is_loading().get());

        let old_raw: Option<u16> = store.get("system.net.port").unwrap();
        assert!(old_raw.is_none(), "Old nested key should be gone");

        let new_raw: Option<u16> = store.get("system.net.listen_port").unwrap();
        assert_eq!(new_raw, Some(9999));
    }
}
