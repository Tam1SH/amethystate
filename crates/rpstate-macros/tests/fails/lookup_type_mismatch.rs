use rpstate_macros::rpstate;

#[rpstate(prefix = "net")]
pub struct NetworkState {
    #[state(default = 8080)]
    pub port: u16,
}

#[rpstate(prefix = "ui")]
pub struct UiState {
    #[state(lookup = "port", parent = NetworkState)]
    pub proxy_port: String,
}

fn main() {}