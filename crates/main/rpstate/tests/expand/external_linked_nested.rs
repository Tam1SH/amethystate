use rpstate_macros::rpstate;
#[rpstate]
pub struct ConnectionPool {
    #[state(default = 10)]
    pub max_connections: u32,

    #[state(default = 30)]
    pub timeout_secs: u32,
}

#[rpstate(prefix = "sys.database")]
pub struct DatabaseState {
    #[state(nested)]
    pub pool: ConnectionPool,
}

#[rpstate(prefix = "ui.inspector")]
pub struct InspectorState {
    #[state(lookup_node = "pool", parent = DatabaseState)]
    pub db_pool_view: ConnectionPool,
}

fn main() {}