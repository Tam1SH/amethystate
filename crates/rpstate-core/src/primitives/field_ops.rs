use crate::FieldCore;
use crate::RpBackend;
use crate::primitives::field_core::FieldValue;

use std::sync::Arc;

pub fn field_set<B, T>(
    backend: &B,
    core: &FieldCore<T>,
    path: Arc<str>,
    value: T,
    update_local_after_commit: bool,
) -> Result<(), B::Error>
where
    B: RpBackend,
    T: FieldValue,
{
    let change = core
        .run_interceptors(path.clone(), value)
        .map_err(|_| backend.intercepted())?;

    backend.set(&path, &change.new_value)?;

    if update_local_after_commit {
        core.signal.set(change.new_value);
    }

    Ok(())
}

pub fn field_apply_remote_value<T>(core: &FieldCore<T>, value: T)
where
    T: Clone + 'static,
{
    core.signal.set(value);
}
