use crate::StoreEvent;
use crate::store::{SubscriptionEntry, matches_kind};
use parking_lot::RwLock;
use std::sync::Arc;

pub(super) fn emit_event(subs: &Arc<RwLock<Vec<SubscriptionEntry>>>, event: StoreEvent) {
    let callbacks = subs
        .read()
        .iter()
        .filter(|e| matches_kind(&e.kind, &event.path))
        .map(|e| e.callback.clone())
        .collect::<Vec<_>>();
    for cb in callbacks {
        cb(&event);
    }
}
