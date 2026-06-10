use crate::store::{matches_kind, StoreEvent, SubscriptionEntry};
use parking_lot::RwLock;

pub fn emit_events(subs_lock: &RwLock<Vec<SubscriptionEntry>>, event: StoreEvent) {
    let callbacks = {
        let guard = subs_lock.read();
        guard
            .iter()
            .filter(|s| matches_kind(&s.kind, &event.path))
            .map(|s| s.callback.clone())
            .collect::<Vec<_>>()
    };
    for cb in callbacks {
        cb(&event);
    }
}

#[cfg(any(feature = "sqlite", feature = "redb"))]
pub fn drain_pending_prefix(
    pending: &mut std::collections::HashMap<std::sync::Arc<str>, Option<Vec<u8>>>,
    prefix: &str,
) -> std::collections::HashMap<std::sync::Arc<str>, Option<Vec<u8>>> {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    if pending.is_empty() {
        return HashMap::new();
    }

    if prefix.is_empty() {
        std::mem::take(pending)
    } else {
        let prefix_dot = format!("{}.", prefix);
        let keys_to_remove: Vec<Arc<str>> = pending
            .keys()
            .filter(|k| k.starts_with(&prefix_dot) || &***k == prefix)
            .cloned()
            .collect();

        let mut matched = HashMap::with_capacity(keys_to_remove.len());
        for k in keys_to_remove {
            if let Some(v) = pending.remove(&k) {
                matched.insert(k, v);
            }
        }
        matched
    }
}
