use super::error::RedbResult;
use super::tables::{TABLE_DATA, TABLE_LOG};
use crate::store::shared::{SubscriptionEntry, matches_kind};
use crate::store::{StoreEvent, StoreOp};
use redb::{Database, ReadableTable};
use std::sync::RwLock;

pub(super) fn process_inbox(
    db: &Database,
    subs: &RwLock<Vec<SubscriptionEntry>>,
) -> RedbResult<()> {
    let write_txn = db.begin_write()?;
    let mut events = Vec::new();

    {
        let mut log_table = write_txn.open_table(TABLE_LOG)?;
        let data_table = write_txn.open_table(TABLE_DATA)?;

        let mut to_delete = Vec::new();
        for result in log_table.iter()? {
            let (id, path_guard) = result?;
            let path = path_guard.value();

            let current_val = data_table.get(path)?;

            events.push(StoreEvent {
                path: path.to_string(),
                op: if current_val.is_some() {
                    StoreOp::Set
                } else {
                    StoreOp::Delete
                },
                old: None,
                new: current_val.map(|v| v.value().to_vec()),
            });

            to_delete.push(id.value());
        }

        for id in to_delete {
            log_table.remove(id)?;
        }
    }
    write_txn.commit()?;

    for event in events {
        emit_local(subs, event);
    }

    Ok(())
}

pub(super) fn emit_local(subs_lock: &RwLock<Vec<SubscriptionEntry>>, event: StoreEvent) {
    let callbacks = {
        let guard = subs_lock.read().unwrap();
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
