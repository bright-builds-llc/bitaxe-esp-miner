//! One statistics owner is allocated early and cannot sample before activation.
use std::io;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

const PREPARED: u8 = 0;
const ACTIVE: u8 = 1;
const CANCELLED: u8 = 2;

pub(super) struct ActivationGate(AtomicU8);
impl ActivationGate {
    pub(super) fn wait(&self) -> bool {
        loop {
            match self.0.load(Ordering::Acquire) {
                ACTIVE => return true,
                CANCELLED => return false,
                _ => thread::park(),
            }
        }
    }
}

pub(super) struct Prepared {
    gate: Arc<ActivationGate>,
    maybe_thread: Option<JoinHandle<()>>,
}

pub(super) fn prepare(
    spawn: impl FnOnce(Arc<ActivationGate>) -> io::Result<JoinHandle<()>>,
) -> io::Result<Prepared> {
    let gate = Arc::new(ActivationGate(AtomicU8::new(PREPARED)));
    let handle = spawn(Arc::clone(&gate))?;
    Ok(Prepared {
        gate,
        maybe_thread: Some(handle),
    })
}

impl Prepared {
    pub(super) fn is_prepared(&self) -> bool {
        self.maybe_thread.is_some()
    }

    pub(super) fn activate(&mut self) -> bool {
        let Some(handle) = self.maybe_thread.as_ref() else {
            return false;
        };
        if handle.is_finished()
            || self
                .gate
                .0
                .compare_exchange(PREPARED, ACTIVE, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return false;
        }
        handle.thread().unpark();
        // Dropping the JoinHandle detaches this same boot-lifetime producer.
        self.maybe_thread.take();
        true
    }
}

impl Drop for Prepared {
    fn drop(&mut self) {
        if let Some(handle) = self.maybe_thread.take() {
            let _cancelled = self.gate.0.compare_exchange(
                PREPARED,
                CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            handle.thread().unpark();
        }
    }
}

#[cfg(test)]
#[path = "lifecycle/tests.rs"]
mod tests;
