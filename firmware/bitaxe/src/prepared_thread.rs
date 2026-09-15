//! Fixed runtime threads reserve their stack early and cannot run before activation.
use std::io;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

const PREPARED: u8 = 0;
const ACTIVE: u8 = 1;
const CANCELLED: u8 = 2;

pub(crate) struct ActivationGate(AtomicU8);
impl ActivationGate {
    pub(crate) fn wait(&self) -> bool {
        loop {
            match self.0.load(Ordering::Acquire) {
                ACTIVE => return true,
                CANCELLED => return false,
                _ => thread::park(),
            }
        }
    }
}

pub(crate) struct Prepared {
    gate: Arc<ActivationGate>,
    maybe_thread: Option<JoinHandle<()>>,
}

/// Uses one compiled spawn path for all prepared owners; callback allocation is startup-only.
#[inline(never)]
pub(crate) fn spawn(
    name: &str,
    stack_bytes: usize,
    operation: Box<dyn FnOnce() + Send>,
) -> io::Result<Prepared> {
    prepare(|gate| {
        thread::Builder::new()
            .name(name.to_owned())
            .stack_size(stack_bytes)
            .spawn(move || {
                if gate.wait() {
                    operation();
                }
            })
    })
}

pub(crate) fn prepare(
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
    pub(crate) fn is_prepared(&self) -> bool {
        self.maybe_thread.is_some()
    }

    pub(crate) fn activate(&mut self) -> bool {
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
#[path = "prepared_thread/tests.rs"]
mod tests;
