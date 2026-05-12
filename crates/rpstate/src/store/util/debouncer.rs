use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tracing::{debug, warn};

pub struct Debouncer {
    tx: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Debouncer {
    pub fn new<F>(interval: Duration, mut op: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<()>();

        let handle = thread::spawn(move || {
            while rx.recv().is_ok() {
                while let Ok(()) = rx.recv_timeout(interval) {
                    continue;
                }

                debug!("debouncer trigger: interval elapsed");
                op();
            }
            debug!("debouncer thread exiting (channel closed)");
        });

        Self {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    pub fn schedule(&self) {
        if let Some(ref tx) = self.tx
            && let Err(e) = tx.send(())
        {
            warn!("failed to schedule debounced operation: channel closed ({e})");
        }
    }

    fn shutdown(&mut self) {
        self.tx.take();

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Debouncer {
    fn drop(&mut self) {
        self.shutdown();
    }
}
