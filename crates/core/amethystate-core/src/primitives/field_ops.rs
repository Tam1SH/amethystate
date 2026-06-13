use crate::primitives::field_core::FieldValue;
use crate::FieldCore;
use crate::AmeBackend;
use std::sync::Arc;
use uuid::Uuid;

pub fn field_set<B, T>(
    backend: &B,
    core: &FieldCore<T>,
    path: Arc<str>,
    value: T,
    source: Option<Uuid>,
) -> Result<(), B::Error>
where
    B: AmeBackend,
    T: FieldValue,
{
    let change = core
        .run_interceptors(path.clone(), value, source)
        .map_err(|_| backend.intercepted())?;

    backend.set_owned_with_source(path, &change.new_value, change.source)?;

    Ok(())
}

pub fn field_apply_remote_value<T>(core: &FieldCore<T>, value: T, source: Option<Uuid>)
where
    T: Clone + 'static,
{
    core.signal.set(value, source);
}
