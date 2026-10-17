//! Whether a Gate link holds local mining off, for the handover (ADR-0038 step 4,
//! owner decision 2026-10-17). Local mining stops when a Gate link appears and
//! stays off while the link is active, so one Gate session can run several leases;
//! it resumes once the link has been quiet for [`LINK_QUIET_MS`].
//!
//! The Gate sends one Hello per Connect and never retries it. A Hello refused
//! because local mining held the gate therefore keeps local mining off for up to
//! [`HANDOVER_HOLD_MS`], until a Hello is admitted, so the operator's next Connect
//! finds the ASIC stopped rather than a freshly restarted run.

/// How long after its last Hello or heartbeat a Gate link still counts as active.
pub const LINK_QUIET_MS: u64 = 15_000;
/// How long a refused Hello keeps local mining off while waiting for the Gate to
/// connect again: the ordered safe-stop may take two minutes to cool the chip.
pub const HANDOVER_HOLD_MS: u64 = 5 * 60_000;

/// What the serial link thread has seen of the Gate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkStamps {
    maybe_last_activity_ms: Option<u64>,
    maybe_refused_hello_ms: Option<u64>,
}

impl LinkStamps {
    /// Nothing seen yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_last_activity_ms: None,
            maybe_refused_hello_ms: None,
        }
    }

    /// An authenticated heartbeat arrived.
    pub fn note_activity(&mut self, now_ms: u64) {
        self.maybe_last_activity_ms = Some(now_ms);
    }

    /// A valid Hello arrived, and the gate admitted or refused it.
    pub fn note_hello(&mut self, now_ms: u64, admitted: bool) {
        self.note_activity(now_ms);
        self.maybe_refused_hello_ms = (!admitted).then_some(now_ms);
    }

    /// Whether the Gate holds local mining off at `now_ms`. A stamp taken after
    /// `now_ms`, by a thread that read the clock later, counts as recent.
    #[must_use]
    pub fn holds_local_mining(&self, now_ms: u64) -> bool {
        let within = |maybe_ms: Option<u64>, window_ms: u64| {
            maybe_ms.is_some_and(|stamp| now_ms.saturating_sub(stamp) < window_ms)
        };
        within(self.maybe_last_activity_ms, LINK_QUIET_MS)
            || within(self.maybe_refused_hello_ms, HANDOVER_HOLD_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_link_holds_nothing() {
        // Arrange / Act / Assert
        assert!(!LinkStamps::default().holds_local_mining(0));
    }

    #[test]
    fn heartbeats_hold_until_the_link_has_been_quiet_long_enough() {
        // Arrange
        let mut stamps = LinkStamps::default();
        stamps.note_activity(1_000);
        // Act / Assert
        assert!(stamps.holds_local_mining(1_000 + LINK_QUIET_MS - 1));
        assert!(!stamps.holds_local_mining(1_000 + LINK_QUIET_MS));
    }

    #[test]
    fn a_refused_hello_holds_while_the_operator_connects_again() {
        // Arrange
        let mut stamps = LinkStamps::default();
        stamps.note_hello(1_000, false);
        // Act / Assert: long after the link went quiet, still within the hold.
        assert!(stamps.holds_local_mining(1_000 + HANDOVER_HOLD_MS - 1));
        assert!(!stamps.holds_local_mining(1_000 + HANDOVER_HOLD_MS));
    }

    #[test]
    fn an_admitted_hello_ends_the_handover_hold() {
        // Arrange
        let mut stamps = LinkStamps::default();
        stamps.note_hello(1_000, false);
        // Act
        stamps.note_hello(2_000, true);
        // Assert: only the ordinary quiet window remains.
        assert!(!stamps.holds_local_mining(2_000 + LINK_QUIET_MS));
    }

    #[test]
    fn a_stamp_taken_after_now_counts_as_recent() {
        // Arrange
        let mut stamps = LinkStamps::default();
        stamps.note_activity(2_000);
        // Act / Assert
        assert!(stamps.holds_local_mining(1_000));
    }
}
