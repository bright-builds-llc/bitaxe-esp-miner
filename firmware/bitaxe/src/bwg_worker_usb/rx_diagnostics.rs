//! First closed receive failure per logical session; no payload or identifier retention.
use bitaxe_worker_control::serial::link::SerialLinkFailure;
use std::sync::atomic::{AtomicU32, Ordering};

pub(super) struct Receipt(AtomicU32);
impl Receipt {
    const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn clear(&self) {
        self.0.store(0, Ordering::Release);
    }
    pub fn record(&self, failure: SerialLinkFailure, observed_bytes: usize) {
        let value = (u32::from(failure.code()) << 24) | observed_bytes.min(66560) as u32;
        let _ = self
            .0
            .compare_exchange(0, value, Ordering::AcqRel, Ordering::Acquire);
    }
    pub fn marker(&self) -> Option<String> {
        let value = self.0.load(Ordering::Acquire);
        let stage = SerialLinkFailure::from_code(value >> 24)?.label();
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
        receipt.record(SerialLinkFailure::Framing, usize::MAX);
        receipt.record(SerialLinkFailure::SessionRevoked, 0);
        // Assert
        assert_eq!(
            receipt.marker().as_deref(),
            Some("usb_rx_failure schema=v1 stage=framing observed_bytes=66560 redacted=true")
        );
        // Act
        receipt.clear();
        receipt.record(SerialLinkFailure::HeartbeatTimeout, 4096);
        // Assert
        assert_eq!(receipt.marker().as_deref(), Some("usb_rx_failure schema=v1 stage=heartbeat_timeout observed_bytes=4096 redacted=true"));
    }
}
