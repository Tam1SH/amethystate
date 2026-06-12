use rpstate_macros::rpstate;

#[rpstate(prefix = "net")]
pub struct NetworkState {
    #[state(default = "127.0.0.1".to_string())]
    pub host: String,
}

#[rpstate(prefix = "ui")]
pub struct UiState {
    #[state(lookup = "host", parent = NetworkState, export_mut)]
    pub proxy_host: String,
}

fn main() {
    let dummy_field: rpstate::Field<u16, rpstate::DefaultStore, rpstate::ReadOnlyMode> =
        unsafe { std::mem::zeroed() };
    dummy_field.set(9090);
}
