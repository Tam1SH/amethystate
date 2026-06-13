use amethystate_macros::amethystate;
#[amethystate(prefix = "net")]
pub struct NetworkState {
    #[amestate(default = 8080, export_mut)]
    pub port: u16,

    #[amestate(default = "127.0.0.1".to_string())]
    pub host: String,
}

#[amethystate(prefix = "ui")]
pub struct UiState {
    #[amestate(lookup = "port", parent = NetworkState)]
    pub proxy_port: u16,

    #[amestate(lookup = "host", parent = NetworkState)]
    pub proxy_host: String,
}

fn main() {}