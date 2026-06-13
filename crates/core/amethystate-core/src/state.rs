#[cfg(feature = "async")]
use crate::AmeBackendAsync;
#[cfg(feature = "async")]
pub trait AmeStateSliceAsync<B: AmeBackendAsync>: Sized {
    type Error;
    fn load_async(store: &B) -> impl Future<Output = Result<Self, Self::Error>>;
}
