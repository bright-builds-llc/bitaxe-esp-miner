//! Session-scoped, replayable Plan 13 boot evidence.

use std::sync::OnceLock;

use esp_idf_svc::sys;

use crate::{asic_adapter, log_buffer};

static BOOT_SESSION: OnceLock<BootSessionNonce> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootSessionNonce([u32; 4]);

impl BootSessionNonce {
    fn from_hardware_rng() -> Self {
        Self([
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
        ])
    }

    fn as_hex(self) -> String {
        let [first, second, third, fourth] = self.0;
        format!("{first:08x}{second:08x}{third:08x}{fourth:08x}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootEvidenceState {
    Booted,
    ListenerArmed,
}

impl BootEvidenceState {
    const fn label(self) -> &'static str {
        match self {
            Self::Booted => "booted",
            Self::ListenerArmed => "listener_armed",
        }
    }
}

/// Generates the per-boot nonce and records boot proof in Plan 13 evidence mode.
pub fn initialize_and_record_booted() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    let nonce = BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng);
    record(*nonce, BootEvidenceState::Booted);
}

/// Records listener readiness against the same per-boot nonce.
pub fn record_listener_armed() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    let nonce = BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng);
    record(*nonce, BootEvidenceState::ListenerArmed);
}

fn record(nonce: BootSessionNonce, state: BootEvidenceState) {
    let marker = marker(nonce, state);
    log::info!("{marker}");
    log_buffer::append_runtime_log_line(&marker);
}

fn marker(nonce: BootSessionNonce, state: BootEvidenceState) -> String {
    format!(
        "plan13_boot_evidence session={} state={} redacted=true",
        nonce.as_hex(),
        state.label()
    )
}

#[cfg(test)]
mod tests {
    use super::{marker, BootEvidenceState, BootSessionNonce};

    #[test]
    fn boot_evidence_marker_is_fixed_width_and_redacted() {
        // Arrange
        let nonce = BootSessionNonce([0, 1, u32::MAX, 0x1234_abcd]);

        // Act
        let marker = marker(nonce, BootEvidenceState::ListenerArmed);

        // Assert
        assert_eq!(
            marker,
            "plan13_boot_evidence session=0000000000000001ffffffff1234abcd state=listener_armed redacted=true"
        );
    }
}
