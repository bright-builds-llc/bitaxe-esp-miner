//! Raw receive progress grants byte capacity only, never application authority.

use std::sync::atomic::{AtomicU32, Ordering};

/// Counts bytes after the Hello delimiter, including incomplete records.
#[derive(Default)]
pub struct SerialReceiveProgress(u32);

impl SerialReceiveProgress {
    #[must_use]
    pub const fn received_bytes(&self) -> u32 {
        self.0
    }
    /// Advances without wrapping. Exhaustion requires a fresh logical session.
    pub fn advance(&mut self, bytes: u32) -> Option<u32> {
        self.0 = self.0.checked_add(bytes)?;
        Some(self.0)
    }
}

/// One receive owner publishes coalesced counts; the independent writer samples them.
/// No worker operation, allocation, mutex, or output completion is involved.
pub struct ReceiveCreditMailbox {
    epoch: AtomicU32,
    received_bytes: AtomicU32,
    latest_epoch: AtomicU32,
    closing_epoch: AtomicU32,
    closing_bytes: AtomicU32,
    closing_started: AtomicU32,
}

impl Default for ReceiveCreditMailbox {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiveCreditMailbox {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            epoch: AtomicU32::new(0),
            received_bytes: AtomicU32::new(0),
            latest_epoch: AtomicU32::new(0),
            closing_epoch: AtomicU32::new(0),
            closing_bytes: AtomicU32::new(0),
            closing_started: AtomicU32::new(0),
        }
    }

    /// Called only by the receive owner when it creates a fresh increasing epoch.
    pub fn begin(&self, epoch: u32) {
        self.closing_epoch.store(0, Ordering::Release);
        self.latest_epoch.store(epoch, Ordering::Release);
        self.epoch.store(0, Ordering::Release);
        self.received_bytes.store(0, Ordering::Release);
        self.epoch.store(epoch, Ordering::Release);
    }

    /// Called only by the receive owner after native bytes have been drained.
    pub fn publish(&self, received_bytes: u32) {
        self.received_bytes.store(received_bytes, Ordering::Release);
    }

    /// Returns no progress when closure or a newer session invalidates the sample.
    #[must_use]
    pub fn maybe_received_bytes(&self, epoch: u32) -> Option<u32> {
        if epoch == 0 || self.epoch.load(Ordering::Acquire) != epoch {
            return None;
        }
        let bytes = self.received_bytes.load(Ordering::Acquire);
        (self.epoch.load(Ordering::Acquire) == epoch).then_some(bytes)
    }

    /// Revocation discards an old epoch without disturbing its successor.
    pub fn close(&self, epoch: u32) {
        self.finish_terminal(epoch);
        let _ = self
            .epoch
            .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire);
    }

    /// A validated Close revokes authority first, retaining only its final byte receipt.
    pub fn close_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce()) {
        let maybe_previous = self.maybe_received_bytes(epoch);
        revoke();
        let Some(previous) = maybe_previous else {
            return;
        };
        if epoch == 0
            || previous.checked_add(1) != Some(received)
            || self.latest_epoch.load(Ordering::Acquire) != epoch
        {
            return;
        }
        self.closing_bytes.store(received, Ordering::Release);
        self.closing_started.store(now_ms as u32, Ordering::Release);
        self.closing_epoch.store(epoch, Ordering::Release);
    }

    /// Closing receipts expire within the unchanged record bound and never cross Hello.
    #[must_use]
    pub fn maybe_terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        if epoch == 0 || self.closing_epoch.load(Ordering::Acquire) != epoch {
            return None;
        }
        let started = self.closing_started.load(Ordering::Acquire);
        let bytes = self.closing_bytes.load(Ordering::Acquire);
        (self.latest_epoch.load(Ordering::Acquire) == epoch
            && self.closing_epoch.load(Ordering::Acquire) == epoch
            && (now_ms as u32).wrapping_sub(started) < 2000)
            .then_some(bytes)
    }

    /// Completion or failure consumes a terminal receipt; no retry or authority survives.
    pub fn finish_terminal(&self, epoch: u32) {
        let _ = self
            .closing_epoch
            .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_partial_byte_counts_before_a_frame_is_complete() {
        // Arrange
        let mut progress = SerialReceiveProgress::default();
        let mailbox = ReceiveCreditMailbox::new();
        mailbox.begin(1);
        // Act
        for _ in 0..1024 {
            mailbox.publish(progress.advance(1).expect("bounded progress"));
        }
        // Assert
        assert_eq!(mailbox.maybe_received_bytes(1), Some(1024));
    }

    #[test]
    fn new_hello_restarts_counts_and_discards_old_epoch_credit() {
        // Arrange
        let mailbox = ReceiveCreditMailbox::new();
        mailbox.begin(1);
        mailbox.publish(1024);
        // Act
        mailbox.begin(2);
        mailbox.close(1);
        // Assert
        assert_eq!(mailbox.maybe_received_bytes(1), None);
        assert_eq!(mailbox.maybe_received_bytes(2), Some(0));
    }

    #[test]
    fn exhaustion_never_wraps_or_refunds_received_bytes() {
        // Arrange
        let mut progress = SerialReceiveProgress(u32::MAX - 1);
        // Act / Assert
        assert_eq!(progress.advance(1), Some(u32::MAX));
        assert_eq!(progress.advance(1), None);
        assert_eq!(progress.0, u32::MAX);
    }

    #[test]
    fn terminal_receipt_follows_revocation_and_expires_without_authority() {
        // Arrange
        let mailbox = ReceiveCreditMailbox::new();
        mailbox.begin(1);
        mailbox.publish(255);
        // Act
        mailbox.close_record(1, 256, 100, || {
            assert_eq!(mailbox.maybe_terminal_bytes(1, 100), None);
            mailbox.close(1);
        });
        // Assert
        assert_eq!(mailbox.maybe_received_bytes(1), None);
        assert_eq!(mailbox.maybe_terminal_bytes(1, 2099), Some(256));
        assert_eq!(mailbox.maybe_terminal_bytes(1, 2100), None);
    }

    #[test]
    fn newer_hello_invalidates_an_unwritten_terminal_receipt() {
        // Arrange
        let mailbox = ReceiveCreditMailbox::new();
        mailbox.begin(1);
        mailbox.publish(255);
        mailbox.close_record(1, 256, 100, || mailbox.close(1));
        // Act
        mailbox.begin(2);
        // Assert
        assert_eq!(mailbox.maybe_terminal_bytes(1, 101), None);
        assert_eq!(mailbox.maybe_received_bytes(2), Some(0));
    }
}
