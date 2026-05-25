use rpstate::store::builder::StoreBuilder;
use rpstate::{Store, rpstate};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[rpstate(prefix = "network")]
pub struct NetworkState {
    #[state(default = "localhost".to_string())]
    pub host: String,

    #[state(default = 8080)]
    pub port: u16,
}

fn unique_path(suffix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time is after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("rpstate-{suffix}-{nanos}.redb"))
}

#[cfg(feature = "redb")]
#[test]
fn persistent_only_load_save_and_mutate() {
    let path = unique_path("persistent-only");
    let store = Arc::new(StoreBuilder::new(&path).build().unwrap());

    let state = NetworkState::new(&store).unwrap();
    state.host().set("10.0.0.1".to_string()).unwrap();
    state.port().set(3030).unwrap();

    let mut data = NetworkState::load(&store).unwrap();
    assert_eq!(data.host, "10.0.0.1");
    assert_eq!(data.port, 3030);

    data.port = 9090;
    data.save().unwrap();
    assert_eq!(store.get::<u16>("network.port").unwrap(), Some(9090));

    data.mutate(|d| {
        d.host = "127.0.0.1".to_string();
        d.port = 4040;
    })
    .unwrap();

    assert_eq!(
        store.get::<String>("network.host").unwrap(),
        Some("127.0.0.1".to_string())
    );
    assert_eq!(store.get::<u16>("network.port").unwrap(), Some(4040));
}
