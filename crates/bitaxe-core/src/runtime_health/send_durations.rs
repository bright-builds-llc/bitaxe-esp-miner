//! Durations of blocking sends on the HTTP server task: response writes and
//! WebSocket frame sends. A soak observer reads them to tell a slow server-task
//! write apart from a network stall. Counts only; no request or client identity.

/// The rolling window for `recent_worst_ms`: the current and previous minute.
pub const SEND_WINDOW_MS: u64 = 60_000;

/// One send category since boot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SendDurationSnapshot {
    pub sends: u64,
    pub failures: u64,
    /// The longest send since boot.
    pub worst_ms: u32,
    /// The longest send in the current and the previous minute.
    pub recent_worst_ms: u32,
}

/// Records send durations with a rolling one-minute worst.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SendDurationTracker {
    sends: u64,
    failures: u64,
    worst_ms: u32,
    window: u64,
    window_worst_ms: u32,
    previous_window_worst_ms: u32,
}

impl SendDurationTracker {
    /// An empty tracker, usable in a `static`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sends: 0,
            failures: 0,
            worst_ms: 0,
            window: 0,
            window_worst_ms: 0,
            previous_window_worst_ms: 0,
        }
    }

    /// Records one finished send.
    pub fn record(&mut self, now_ms: u64, duration_ms: u32, succeeded: bool) {
        self.advance(now_ms);
        self.sends = self.sends.saturating_add(1);
        if !succeeded {
            self.failures = self.failures.saturating_add(1);
        }
        self.worst_ms = self.worst_ms.max(duration_ms);
        self.window_worst_ms = self.window_worst_ms.max(duration_ms);
    }

    /// The counts and worst durations as seen at `now_ms`.
    #[must_use]
    pub fn snapshot(&self, now_ms: u64) -> SendDurationSnapshot {
        let mut current = *self;
        current.advance(now_ms);
        SendDurationSnapshot {
            sends: current.sends,
            failures: current.failures,
            worst_ms: current.worst_ms,
            recent_worst_ms: current
                .window_worst_ms
                .max(current.previous_window_worst_ms),
        }
    }

    fn advance(&mut self, now_ms: u64) {
        let window = now_ms / SEND_WINDOW_MS;
        if window <= self.window {
            return;
        }
        self.previous_window_worst_ms = if window == self.window + 1 {
            self.window_worst_ms
        } else {
            0
        };
        self.window_worst_ms = 0;
        self.window = window;
    }
}

/// The HTTP server task's two blocking send kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HttpServerSendsSnapshot {
    pub http_responses: SendDurationSnapshot,
    pub websocket_sends: SendDurationSnapshot,
}

impl super::RuntimeHealthSnapshot {
    /// Attaches the HTTP server task's send durations.
    #[must_use]
    pub fn with_http_server_sends(mut self, sends: HttpServerSendsSnapshot) -> Self {
        *self.http_server_sends = sends;
        self
    }

    #[must_use]
    pub fn http_server_sends(&self) -> HttpServerSendsSnapshot {
        *self.http_server_sends
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_failures_and_the_worst_duration_accumulate() {
        // Arrange
        let mut tracker = SendDurationTracker::default();
        // Act
        tracker.record(1_000, 12, true);
        tracker.record(2_000, 4_600, false);
        tracker.record(3_000, 30, true);
        // Assert
        assert_eq!(
            tracker.snapshot(3_000),
            SendDurationSnapshot {
                sends: 3,
                failures: 1,
                worst_ms: 4_600,
                recent_worst_ms: 4_600,
            }
        );
    }

    #[test]
    fn the_recent_worst_covers_the_previous_minute_then_ages_out() {
        // Arrange: a 4.6 s write at 30 s.
        let mut tracker = SendDurationTracker::default();
        tracker.record(30_000, 4_600, true);
        // Act
        let next_minute = tracker.snapshot(90_000).recent_worst_ms;
        let two_minutes_later = tracker.snapshot(150_000).recent_worst_ms;
        // Assert: the lifetime worst stays; the recent worst ages out.
        assert_eq!((next_minute, two_minutes_later), (4_600, 0));
        assert_eq!(tracker.snapshot(150_000).worst_ms, 4_600);
    }

    #[test]
    fn a_skipped_minute_does_not_carry_an_old_worst() {
        // Arrange
        let mut tracker = SendDurationTracker::default();
        tracker.record(10_000, 3_000, true);
        // Act: the next send comes three minutes later.
        tracker.record(190_000, 5, true);
        // Assert
        assert_eq!(tracker.snapshot(190_000).recent_worst_ms, 5);
    }
}
