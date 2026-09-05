//! First closed receive failure per logical session; no payload or identifier retention.
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy)]
#[repr(u8)]
pub(super) enum Stage {
    Read = 1,
    Framing,
    Envelope,
    Sequence,
    HeartbeatPayload,
    HeartbeatTimeout,
    ControlAllocation,
    ControlQueue,
    UnexpectedKind,
    SessionRevoked,
}

pub(super) struct Receipt(AtomicU32);
impl Receipt {
    const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn clear(&self) {
        self.0.store(0, Ordering::Release);
    }
    pub fn record(&self, stage: Stage, observed_bytes: usize) {
        let value = ((stage as u32) << 24) | observed_bytes.min(66560) as u32;
        let _ = self
            .0
            .compare_exchange(0, value, Ordering::AcqRel, Ordering::Acquire);
    }
    pub fn marker(&self) -> Option<String> {
        let value = self.0.load(Ordering::Acquire);
        let stage = match value >> 24 {
            0 => return None,
            1 => "read",
            2 => "framing",
            3 => "envelope",
            4 => "sequence",
            5 => "heartbeat_payload",
            6 => "heartbeat_timeout",
            7 => "control_allocation",
            8 => "control_queue",
            9 => "unexpected_kind",
            10 => "session_revoked",
            _ => return None,
        };
        Some(format!(
            "usb_rx_failure schema=v1 stage={stage} observed_bytes={} redacted=true",
            value & 0x00ff_ffff
        ))
    }
}
pub(super) static FAILURE: Receipt = Receipt::new();

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_failure_retains_only_bounded_count_until_fresh_session() {
        // Arrange
        let receipt = Receipt::new();
        // Act
        receipt.record(Stage::Framing, usize::MAX);
        receipt.record(Stage::SessionRevoked, 0);
        // Assert
        assert_eq!(
            receipt.marker().as_deref(),
            Some("usb_rx_failure schema=v1 stage=framing observed_bytes=66560 redacted=true")
        );
        // Act
        receipt.clear();
        receipt.record(Stage::HeartbeatTimeout, 4096);
        // Assert
        assert_eq!(receipt.marker().as_deref(), Some("usb_rx_failure schema=v1 stage=heartbeat_timeout observed_bytes=4096 redacted=true"));
    }
}
