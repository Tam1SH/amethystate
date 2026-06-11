use rpstate::store::builder::StoreBuilder;
use rpstate::{RpData, Store, migrate};
use rpstate_core::test_utils::unique_path;
use rpstate_macros::rpstate;

mod v1 {
    use super::*;

    #[rpstate(prefix = "app", version = 1)]
    pub struct Config {
        #[state(default = "localhostv1".to_string())]
        pub host: String,
    }
}

#[rpstate(prefix = "app", version = 2)]
pub struct Config {
    #[state(default = "localhostv2".to_string())]
    pub address: String,

    #[state(default = 8080)]
    pub port: u16,
}

#[migrate]
#[rename(host => address)]
fn migrate_config_v1_to_v2(old: RpData<v1::Config>) -> rpstate::Result<RpData<Config>> {
    Ok(RpData::<Config> {
        address: old.host,
        port: 9090,
    })
}

#[test]
fn test_decentralized_codegen_migration() {
    let path = unique_path("rpstate_integration_test.redb");

    {
        let store = StoreBuilder::new(&path).build().unwrap();
        let config = v1::Config::new_with(&store).unwrap();
        config.host().set("10.0.0.1".to_string()).unwrap();
    }

    let (store, reports) = StoreBuilder::new(&path)
        .collect_migrations()
        .build()
        .unwrap();

    assert!(!reports.has_failures());

    let config = Config::new_with(&store).expect("Failed to create Config");

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
