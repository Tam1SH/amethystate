use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tracing::{debug, warn};

pub struct Debouncer {
    tx: mpsc::Sender<()>,
}

impl Debouncer {
    pub fn new<F>(interval: Duration, mut op: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<()>();

        thread::spawn(move || {
            while rx.recv().is_ok() {
                while let Ok(()) = rx.recv_timeout(interval) {
                    continue;
                }

                debug!("debouncer trigger: interval elapsed");
                op();
            }
            debug!("debouncer thread exiting (channel closed)");
        });

        Self { tx }
    }

    pub fn schedule(&self) {
        if let Err(e) = self.tx.send(()) {
            warn!("failed to schedule debounced operation: channel closed ({e})");
        }
    }
}
