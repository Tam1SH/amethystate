use crate::DefaultStore;

pub fn unique_store(suffix: &str) -> DefaultStore {
    use crate::store::config::StoreConfig;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rpstate-test-{suffix}-{nanos}.db"));

    DefaultStore::open(StoreConfig::new(path), Default::default())
        .unwrap()
        .0
}
