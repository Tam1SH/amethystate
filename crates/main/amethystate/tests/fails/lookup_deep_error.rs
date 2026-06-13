use amethystate_macros::amethystate;

#[amethystate]
pub struct Inner {
    #[amestate(default = 1)]
    pub value: i32,
}

#[amethystate(prefix = "root")]
pub struct Root {
    #[amestate(nested)]
    pub inner: Inner,
}

#[amethystate(prefix = "ui")]
pub struct UiState {
    #[amestate(lookup = "inner.wrong", parent = Root)]
    pub val: i32,
}

fn main() {}