#[cfg(feature = "async")]
use crate::RpBackendAsync;
#[cfg(feature = "async")]
pub trait RpStateSliceAsync<B: RpBackendAsync>: Sized {
    type Error;
    fn load_async(store: &B) -> impl Future<Output = Result<Self, Self::Error>>;
}
