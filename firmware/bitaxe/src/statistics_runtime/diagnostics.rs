//! Closed, boot-retained statistics allocation facts; recording never allocates or waits.
use std::fmt;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

const EMPTY: u32 = 0;
const PREPARING: u32 = 1;
const PREPARED: u32 = 2;
const ACTIVE: u32 = 3;
const CANCELLED: u32 = 4;
const SPAWN_FAILED: u32 = 5;
const CONFIG_FAILED: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HeapObservation {
    pub free_bytes: u32,
    pub largest_block_bytes: u32,
}

struct DiagnosticNumber<T>(Option<T>);
impl<T: fmt::Display> fmt::Display for DiagnosticNumber<T> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_ref() {
            Some(value) => value.fmt(output),
            None => output.write_str("unavailable"),
        }
    }
}

pub(crate) static STARTUP: StartupDiagnostic = StartupDiagnostic::new();

pub(crate) struct StartupDiagnostic {
    state: AtomicU32,
    stack_bytes: AtomicU32,
    stack_caps: AtomicU32,
    errno: AtomicI32,
    errno_available: AtomicU32,
    before_free: AtomicU32,
    before_largest: AtomicU32,
    after_free: AtomicU32,
    after_largest: AtomicU32,
}

impl StartupDiagnostic {
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU32::new(EMPTY),
            stack_bytes: AtomicU32::new(0),
            stack_caps: AtomicU32::new(0),
            errno: AtomicI32::new(0),
            errno_available: AtomicU32::new(0),
            before_free: AtomicU32::new(0),
            before_largest: AtomicU32::new(0),
            after_free: AtomicU32::new(0),
            after_largest: AtomicU32::new(0),
        }
    }
    pub(super) fn begin(&self, stack_bytes: u32) -> bool {
        if self
            .state
            .compare_exchange(EMPTY, PREPARING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        self.stack_bytes.store(stack_bytes, Ordering::Relaxed);
        true
    }
    pub(super) fn observe_before(&self, stack_caps: u32, before: HeapObservation) {
        self.stack_caps.store(stack_caps, Ordering::Relaxed);
        self.before_free.store(before.free_bytes, Ordering::Relaxed);
        self.before_largest
            .store(before.largest_block_bytes, Ordering::Relaxed);
    }
    pub(super) fn config_failed(&self, stack_bytes: u32) {
        self.stack_bytes.store(stack_bytes, Ordering::Relaxed);
        self.state.store(CONFIG_FAILED, Ordering::Release);
    }
    pub(super) fn finish(&self, after: HeapObservation, outcome: Result<(), Option<i32>>) {
        self.after_free.store(after.free_bytes, Ordering::Relaxed);
        self.after_largest
            .store(after.largest_block_bytes, Ordering::Relaxed);
        let state = match outcome {
            Ok(()) => PREPARED,
            Err(maybe_errno) => {
                if let Some(errno) = maybe_errno {
                    self.errno.store(errno, Ordering::Relaxed);
                    self.errno_available.store(1, Ordering::Relaxed);
                }
                SPAWN_FAILED
            }
        };
        self.state.store(state, Ordering::Release);
    }
    pub(super) fn active(&self) {
        self.state.store(ACTIVE, Ordering::Release);
    }
    pub(super) fn cancelled(&self) {
        self.state.store(CANCELLED, Ordering::Release);
    }

    /// Formatting belongs only to the independent USB writer, never the recorder.
    pub(crate) fn maybe_marker(&self) -> Option<String> {
        let state = self.state.load(Ordering::Acquire);
        let label = match state {
            PREPARED => "prepared",
            ACTIVE => "active",
            CANCELLED => "cancelled",
            SPAWN_FAILED => "spawn_failed",
            CONFIG_FAILED => "config_failed",
            _ => return None,
        };
        let stack_bytes = self.stack_bytes.load(Ordering::Relaxed);
        let errno = DiagnosticNumber(
            (self.errno_available.load(Ordering::Relaxed) != 0)
                .then(|| self.errno.load(Ordering::Relaxed)),
        );
        let measured = |field: &AtomicU32| {
            DiagnosticNumber((state != CONFIG_FAILED).then(|| field.load(Ordering::Relaxed)))
        };
        let marker = format!("statistics_startup schema=v1 state={label} errno={errno} stack_bytes={stack_bytes} stack_caps={} before_free_bytes={} before_largest_block_bytes={} after_free_bytes={} after_largest_block_bytes={} redacted=true",
            measured(&self.stack_caps), measured(&self.before_free), measured(&self.before_largest), measured(&self.after_free), measured(&self.after_largest));
        (self.state.load(Ordering::Acquire) == state).then_some(marker)
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
