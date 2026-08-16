use bitaxe_api::SystemInfoWire;

use super::validation::advances;
use super::watchdog::{window_failure, WatchdogFailure};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SerialWindowEvidence {
    marker_count: u64,
    first_poll_request_count: Option<u64>,
    last_poll_request_count: Option<u64>,
}

impl SerialWindowEvidence {
    pub(super) fn observe(&mut self, poll_request_count: u64) {
        self.marker_count = self.marker_count.saturating_add(1);
        self.first_poll_request_count
            .get_or_insert(poll_request_count);
        self.last_poll_request_count = Some(poll_request_count);
    }

    pub(super) fn work_renewed(self) -> bool {
        self.marker_count >= 2
            && self
                .first_poll_request_count
                .zip(self.last_poll_request_count)
                .is_some_and(|(first, last)| last > first)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct TransportWindowEvidence {
    sample_count: u64,
    first_watchdog_feed_sequence: Option<u64>,
    last_watchdog_feed_sequence: Option<u64>,
    first_checkpoint_sequence: Option<u64>,
    last_checkpoint_sequence: Option<u64>,
}

impl TransportWindowEvidence {
    pub(super) fn observe(&mut self, sample: &SystemInfoWire) {
        self.sample_count = self.sample_count.saturating_add(1);
        if let Some(sequence) = sample.runtime_health.maybe_task_watchdog_feed_sequence {
            self.first_watchdog_feed_sequence.get_or_insert(sequence);
            self.last_watchdog_feed_sequence = Some(sequence);
        }
        if let Some(sequence) = sample.runtime_health.maybe_checkpoint_sequence {
            self.first_checkpoint_sequence.get_or_insert(sequence);
            self.last_checkpoint_sequence = Some(sequence);
        }
    }

    pub(super) fn complete(self) -> bool {
        self.sample_count >= 2
    }

    fn checkpoint_advanced(self) -> bool {
        advances(
            self.first_checkpoint_sequence,
            self.last_checkpoint_sequence,
        )
    }

    fn watchdog_feed_advanced(self) -> bool {
        advances(
            self.first_watchdog_feed_sequence,
            self.last_watchdog_feed_sequence,
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ContinuityWindowEvidence {
    pub(super) http: TransportWindowEvidence,
    pub(super) websocket: TransportWindowEvidence,
    pub(super) serial: SerialWindowEvidence,
    pub(super) complete: bool,
}

impl ContinuityWindowEvidence {
    pub(super) fn watchdog_failure(self) -> WatchdogFailure {
        window_failure(
            self.http.checkpoint_advanced(),
            self.http.watchdog_feed_advanced(),
            self.websocket.checkpoint_advanced(),
            self.websocket.watchdog_feed_advanced(),
        )
    }
}
