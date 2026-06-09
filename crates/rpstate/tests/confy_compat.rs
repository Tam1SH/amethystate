#![cfg(any(feature = "confy-compat", feature = "confy-compat-0-6"))]
use rpstate::confy;
use rpstate_macros::rpstate;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TestConfig {
    name: String,
    comfy: bool,
    foo: i64,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            name: "Unknown".to_string(),
            comfy: true,
            foo: 42,
        }
    }
}

#[test]
fn test_confy_compat_lifecycle() {
    let app_name = "confy_compat_integration_test_app";

    let file_path =
        confy::get_configuration_file_path(app_name, None).expect("Failed to get config path");

    if file_path.exists() {
        let _ = fs::remove_file(&file_path);
    }

    let cfg: TestConfig =
        confy::load(app_name, None).expect("Failed to load default configuration");

    assert_eq!(cfg, TestConfig::default());
    assert!(
        file_path.exists(),
        "Configuration file must be created on disk"
    );

    let updated_cfg = TestConfig {
        name: "TestUser".to_string(),
        comfy: false,
        foo: 99,
    };
    confy::store(app_name, None, &updated_cfg).expect("Failed to store updated configuration");

    let loaded_cfg: TestConfig =
        confy::load(app_name, None).expect("Failed to reload configuration");

    assert_eq!(loaded_cfg, updated_cfg);

    if let Some(parent) = file_path.parent()
        && parent.exists()
    {
        fs::remove_dir_all(parent).expect("Failed to clean up test configuration directory");
    }
}

#[rpstate(prefix = "network", mode = "persistent")]
pub struct NetworkState {
    #[state(default = 8080u16)]
    pub port: u16,
}

#[test]
#[cfg(not(backend = "redb"))]
fn test_confy_rpstate_coexistence() {
    use rpstate::{IntoGlobalStore, StoreBuilder};

    let app_name = "confy_rpstate_coexistence_test";

    let file_path =
        confy::get_configuration_file_path(app_name, None).expect("Failed to get config path");

    if file_path.exists() {
        let _ = fs::remove_file(&file_path);
    }

    StoreBuilder::new(&file_path).init_global();

    let legacy = TestConfig {
        name: "legacy".to_string(),
        comfy: true,
        foo: 1,
    };

    confy::store(app_name, None, &legacy).expect("confy store failed");

    let mut network = NetworkState::load().expect("rpstate init failed");
    network.mutate(|n| n.port = 9090).expect("mutate failed");

    let loaded_legacy: TestConfig = confy::load(app_name, None).expect("confy load failed");
    assert_eq!(loaded_legacy, legacy);

    assert_eq!(network.port, 9090);

    let contents = fs::read_to_string(&file_path).expect("read failed");
    assert!(contents.contains(
        r#"name = "legacy"
comfy = true
foo = 1

[network]
port = 9090
"#
    ));

    if let Some(parent) = file_path.parent()
        && parent.exists()
    {
        fs::remove_dir_all(parent).expect("cleanup failed");
    }
}

#[test]
fn test_compare_real_confy_and_rpstate() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");

    let legacy = TestConfig {
        name: "legacy".to_string(),
        comfy: true,
        foo: 1,
    };

    let real_path = dir.path().join("real_confy.toml");
    real_confy::store_path(&real_path, &legacy).expect("real confy store failed");
    let real_contents = std::fs::read_to_string(&real_path).expect("read failed");

    let rp_path = dir.path().join("rpstate_confy.toml");
    confy::store_path(&rp_path, &legacy).expect("rpstate confy store failed");
    let rp_contents = std::fs::read_to_string(&rp_path).expect("read failed");

    println!("=== REAL CONFY ===");
    println!("{}", real_contents);

    println!("=== RPSTATE CONFY ===");
    println!("{}", rp_contents);

    assert_eq!(real_contents.trim(), rp_contents.trim());
}
