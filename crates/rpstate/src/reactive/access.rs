pub trait AccessMode: Send + Sync + 'static {}

pub struct ReadOnlyMode;

impl AccessMode for ReadOnlyMode {}

pub struct WritableMode;

impl AccessMode for WritableMode {}

pub struct ReadOnly<T>(std::marker::PhantomData<T>);

pub struct Writable<T>(std::marker::PhantomData<T>);

impl<T> std::ops::Deref for ReadOnly<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unreachable!("Type-level token only")
    }
}

impl<T> std::ops::Deref for Writable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unreachable!("Type-level token only")
    }
}
