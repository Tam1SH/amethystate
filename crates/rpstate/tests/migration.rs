use rpstate::store::builder::StoreBuilder;
use rpstate::store::migration::MigrateFrom;
use rpstate::{migrate, Store};
use rpstate_macros::rpstate;
use std::sync::Arc;

mod v1 {
    use super::*;

    #[rpstate(prefix = "app", version = 1)]
    pub struct Config {
        #[state(default = "localhost".to_string())]
        pub host: String,
    }
}

#[rpstate(prefix = "app", version = 2)]
pub struct Config {
    #[state(default = "localhost".to_string())]
    pub address: String,

    #[state(default = 8080)]
    pub port: u16,
}

migrate! {
    v1::Config_Data => Config_Data,
    rename: [host => address],
    |old| {
        Ok(Self {
            address: old.host,
            port: 9090,
        })
    }
}

#[test]
fn test_decentralized_codegen_migration() {
    let path = std::env::temp_dir().join("rpstate_integration_test.redb");
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }

    {
        let store = StoreBuilder::new(&path).build().unwrap();
        store.set("app.host", &"10.0.0.1".to_string()).unwrap();
    }

    let store = Arc::new(
        StoreBuilder::new(&path)
            .collect_migrations()
            .build()
            .unwrap(),
    );

    let config = Config::new(&store).expect("Failed to create Config");

    assert_eq!(config.address().get(), "10.0.0.1");

    assert_eq!(config.port().get(), 9090);

    let old_val: Option<String> = store.get("app.host").unwrap();
    assert!(
        old_val.is_none(),
        "Old key 'app.host' should be deleted after migration"
    );

    let new_val: Option<String> = store.get("app.address").unwrap();
    assert_eq!(new_val, Some("10.0.0.1".to_string()));
}
