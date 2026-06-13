use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn unique_path(suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("amethystate-{suffix}-{nanos}.db"))
}
