/// Diagnostic accepted-state replay window after listener readiness.
///
/// This covers the 30-minute native-USB appearance window, the 60-second
/// attachment bound, and two replay intervals of alignment headroom.
pub const ACCEPTED_STATE_REPLAY_WINDOW_MS: u64 = 1_880_000;
/// Fixed accepted-state replay interval inside the bounded window.
pub const ACCEPTED_STATE_REPLAY_INTERVAL_MS: u64 = 10_000;
/// Bounded wait for the selected native-USB node to appear after arming.
pub const ACCEPTED_STATE_RESTORE_WATCH_MS: u64 = 1_800_000;
/// Bounded readiness and passive-monitor ownership acquisition after appearance.
pub const ACCEPTED_STATE_MONITOR_ATTACHMENT_MS: u64 = 60_000;
/// Initial runtime-heartbeat cadence through the first two minutes.
pub const RUNTIME_HEARTBEAT_EARLY_CADENCE_MS: u64 = 1_000;
/// Runtime-heartbeat cadence after the first two minutes.
pub const RUNTIME_HEARTBEAT_STEADY_CADENCE_MS: u64 = 10_000;
/// Inclusive upper bound for the early runtime-heartbeat cadence.
pub const RUNTIME_HEARTBEAT_EARLY_WINDOW_MS: u64 = 120_000;

/// One immutable, redaction-safe runtime-heartbeat observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeHeartbeatSample {
    session_words: [u32; 4],
    sequence: u64,
    uptime_ms: u64,
    cadence_ms: u64,
    listener_armed: bool,
}

impl RuntimeHeartbeatSample {
    /// Renders the exact serial-only runtime-heartbeat marker.
    #[must_use]
    pub fn marker(self) -> String {
        let [first, second, third, fourth] = self.session_words;
        format!(
            "runtime_heartbeat session={first:08x}{second:08x}{third:08x}{fourth:08x} sequence={} uptime_ms={} cadence_ms={} listener_armed={} redacted=true",
            self.sequence, self.uptime_ms, self.cadence_ms, self.listener_armed
        )
    }
}

/// Pure boot-lifetime runtime-heartbeat state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeHeartbeatModel {
    session_words: [u32; 4],
    next_sequence: u64,
    next_deadline_ms: u64,
    listener_armed: bool,
}

impl RuntimeHeartbeatModel {
    /// Creates a heartbeat schedule for one opaque boot session.
    #[must_use]
    pub const fn new(session_words: [u32; 4]) -> Self {
        Self {
            session_words,
            next_sequence: 0,
            next_deadline_ms: RUNTIME_HEARTBEAT_EARLY_CADENCE_MS,
            listener_armed: false,
        }
    }

    /// Latches listener readiness for the rest of this boot.
    pub fn arm_listener(&mut self) {
        self.listener_armed = true;
    }

    /// Returns the next monotonic deadline at which the observer should wake.
    #[must_use]
    pub const fn next_deadline_ms(self) -> u64 {
        self.next_deadline_ms
    }

    /// Emits at most one due sample and schedules from the observed time.
    pub fn maybe_take_due(&mut self, observed_uptime_ms: u64) -> Option<RuntimeHeartbeatSample> {
        if observed_uptime_ms < self.next_deadline_ms {
            return None;
        }

        let cadence_ms = heartbeat_cadence_ms(observed_uptime_ms);
        let sample = RuntimeHeartbeatSample {
            session_words: self.session_words,
            sequence: self.next_sequence,
            uptime_ms: observed_uptime_ms,
            cadence_ms,
            listener_armed: self.listener_armed,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        let next_cadence_ms = if observed_uptime_ms < RUNTIME_HEARTBEAT_EARLY_WINDOW_MS {
            RUNTIME_HEARTBEAT_EARLY_CADENCE_MS
        } else {
            RUNTIME_HEARTBEAT_STEADY_CADENCE_MS
        };
        self.next_deadline_ms = observed_uptime_ms.saturating_add(next_cadence_ms);
        Some(sample)
    }
}

const fn heartbeat_cadence_ms(uptime_ms: u64) -> u64 {
    if uptime_ms <= RUNTIME_HEARTBEAT_EARLY_WINDOW_MS {
        RUNTIME_HEARTBEAT_EARLY_CADENCE_MS
    } else {
        RUNTIME_HEARTBEAT_STEADY_CADENCE_MS
    }
}

/// Host-testable bounded cadence for replaying retained diagnostic markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedStateReplayCadence {
    next_due_ms: u64,
    exhausted_at_ms: u64,
}

impl AcceptedStateReplayCadence {
    /// Arms a replay cadence at listener readiness. The first replay is due
    /// immediately, then repeats at the fixed interval for the bounded window.
    #[must_use]
    pub fn armed(armed_at_ms: u64) -> Self {
        Self {
            next_due_ms: armed_at_ms,
            exhausted_at_ms: armed_at_ms.saturating_add(ACCEPTED_STATE_REPLAY_WINDOW_MS),
        }
    }

    /// Consumes one due replay opportunity for the supplied monotonic time.
    pub fn take_due(&mut self, now_ms: u64) -> bool {
        if now_ms >= self.exhausted_at_ms || now_ms < self.next_due_ms {
            return false;
        }

        self.next_due_ms = now_ms.saturating_add(ACCEPTED_STATE_REPLAY_INTERVAL_MS);
        true
    }
}
