use std::sync::{Arc, Condvar, Mutex};

pub mod debouncer;
pub mod ticker;

struct DeadNotifier(Arc<(Mutex<bool>, Condvar)>);

impl Drop for DeadNotifier {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.0;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    }
}
