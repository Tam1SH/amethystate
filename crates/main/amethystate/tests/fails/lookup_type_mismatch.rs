use amethystate_macros::amethystate;

#[amethystate(prefix = "net")]
pub struct NetworkState {
    #[amestate(default = 8080)]
    pub port: u16,
}

#[amethystate(prefix = "ui")]
pub struct UiState {
    #[amestate(lookup = "port", parent = NetworkState)]
    pub proxy_port: String,
}

fn main() {}