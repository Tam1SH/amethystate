use crate::primitives::field_core::FieldValue;
use crate::{FieldCore, RpBackendAsync};
use std::sync::Arc;

pub async fn field_set_async<B, T>(
    backend: &B,
    core: &FieldCore<T>,
    path: Arc<str>,
    value: T,
    update_local_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackendAsync,
    T: FieldValue,
{
    let change = core
        .run_interceptors(path.clone(), value)
        .map_err(|_| backend.intercepted())?;

    backend.set(&path, &change.new_value).await?;

    if update_local_after_commit {
        core.signal.set(change.new_value);
    }

    Ok(())
}
