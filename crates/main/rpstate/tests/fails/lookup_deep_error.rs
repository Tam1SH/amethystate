use rpstate_macros::rpstate;

#[rpstate]
pub struct Inner {
    #[state(default = 1)]
    pub value: i32,
}

#[rpstate(prefix = "root")]
pub struct Root {
    #[state(nested)]
    pub inner: Inner,
}

#[rpstate(prefix = "ui")]
pub struct UiState {
    #[state(lookup = "inner.wrong", parent = Root)]
    pub val: i32,
}

fn main() {}