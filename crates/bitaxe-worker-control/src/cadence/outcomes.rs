//! Phase-token completion counters independent of the timing owner's capture guard.
use super::CadenceSummary;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub(super) struct SendOutcomes {
    completed: AtomicU32,
    queue_failures: AtomicU32,
    send_failures: AtomicU32,
    overflow: AtomicBool,
    dropped: AtomicBool,
}

impl SendOutcomes {
    pub(super) const fn new() -> Self {
        Self {
            completed: AtomicU32::new(0),
            queue_failures: AtomicU32::new(0),
            send_failures: AtomicU32::new(0),
            overflow: AtomicBool::new(false),
            dropped: AtomicBool::new(false),
        }
    }

    fn increment(&self, counter: &AtomicU32) {
        if counter.fetch_add(1, Ordering::AcqRel) == u32::MAX {
            self.overflow.store(true, Ordering::Release);
        }
    }

    pub(super) fn queue_failed(&self) {
        self.increment(&self.queue_failures);
    }

    pub(super) fn completed(&self, successful: bool) {
        if !successful {
            self.increment(&self.send_failures);
        }
        // Completion publishes the failure outcome as well as the terminal count.
        self.increment(&self.completed);
    }

    pub(super) fn drop_observation(&self) {
        self.dropped.store(true, Ordering::Release);
    }

    pub(super) fn dropped(&self) -> u32 {
        u32::from(self.dropped.load(Ordering::Acquire))
    }

    pub(super) fn maybe_terminal_count(&self) -> Option<u32> {
        if self.overflow.load(Ordering::Acquire) {
            return None;
        }
        self.completed
            .load(Ordering::Acquire)
            .checked_add(self.queue_failures.load(Ordering::Acquire))
    }

    /// The timing owner stops creating tokens before freezing. A late completion may
    /// leave a conservative pending count in an export, but it cannot be discarded.
    pub(super) fn project(&self, summary: &mut CadenceSummary) {
        let completed = self.completed.load(Ordering::Acquire);
        let queue_failures = self.queue_failures.load(Ordering::Acquire);
        let send_failures = self.send_failures.load(Ordering::Acquire);
        let maybe_queued = summary.sends_queued.checked_sub(queue_failures);
        let maybe_pending = maybe_queued.and_then(|queued| queued.checked_sub(completed));
        summary.overflow |= self.overflow.load(Ordering::Acquire)
            || maybe_pending.is_none()
            || (maybe_pending == Some(0) && send_failures > completed);
        summary.sends_queued = maybe_queued.unwrap_or(0);
        summary.pending_sends = maybe_pending.unwrap_or(0);
        summary.sends_completed = completed;
        summary.queue_failures = queue_failures;
        summary.send_failures = send_failures;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_counter_overflow_blocks_further_phase_admission() {
        // Arrange
        let outcomes = SendOutcomes::new();
        outcomes.completed.store(u32::MAX, Ordering::Release);
        // Act
        outcomes.completed(true);
        // Assert
        assert!(outcomes.overflow.load(Ordering::Acquire));
        assert!(outcomes.maybe_terminal_count().is_none());
    }
}
