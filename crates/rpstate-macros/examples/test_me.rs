use rpstate_macros::rpstate;

#[rpstate]
pub struct NetworkSettings {
    #[state(default = 8080)]
    pub port: u16,
}

#[rpstate(prefix = "system", version = 1)]
pub struct SystemConfig {
    #[state(nested)]
    pub net: NetworkSettings,
}

#[rpstate(prefix = "ui")]
pub struct Dashboard {
    #[state(lookup = "net.port", parent = SystemConfig)]
    pub sys_port: u16,

    #[state(lookup_node = "net", parent = SystemConfig)]
    pub net_node: NetworkSettings,

    #[state(default = false, volatile)]
    pub is_loading: bool,
}

fn main() {}
