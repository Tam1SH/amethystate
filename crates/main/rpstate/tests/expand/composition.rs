use rpstate_macros::rpstate;
#[rpstate(prefix = "net")]
pub struct NetworkState {
    #[state(default = 8080, export_mut)]
    pub port: u16,

    #[state(default = "127.0.0.1".to_string())]
    pub host: String,
}

#[rpstate(prefix = "ui")]
pub struct UiState {
    #[state(lookup = "port", parent = NetworkState)]
    pub proxy_port: u16,

    #[state(lookup = "host", parent = NetworkState)]
    pub proxy_host: String,
}

fn main() {}