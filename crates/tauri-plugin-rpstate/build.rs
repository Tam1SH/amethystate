const COMMANDS: &[&str] = &[
    "rpstate_get",
    "rpstate_set",
    "rpstate_subscribe",
    "rpstate_unsubscribe",
    "rpstate_get_prefix",
    "rpstate_flush",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
