use std::time::Duration;

use anyhow::{bail, Result};

use crate::SessionEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UsbDeviceSnapshot {
    pub(crate) port: String,
    pub(crate) physical_identity_digest: String,
    pub(crate) enumeration_token: String,
    pub(crate) accessible: bool,
    pub(crate) holder_count: u16,
}

// The unsupported adapter preserves the observation contract but never returns one.
#[allow(dead_code)]
pub(crate) enum PhysicalSnapshotObservation {
    Absent,
    PhysicalMismatch,
    Match(UsbDeviceSnapshot),
}

pub(crate) struct UsbProfileFields {
    pub(crate) port: String,
    pub(crate) physical_identity_digest: String,
    pub(crate) enumeration_token: String,
    pub(crate) vendor: String,
    pub(crate) product: String,
    pub(crate) product_name: Option<String>,
}

pub(crate) struct DeviceObservation {
    pub(crate) event: SessionEvent,
    pub(crate) maybe_port: Option<String>,
}

pub(crate) struct ReconnectingReceiveCapture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) open_count: u16,
}

pub(crate) fn capture_reconnecting_receive_only(
    _requested_port: &str,
    _timeout: Duration,
) -> Result<ReconnectingReceiveCapture> {
    bail!("reconnecting receive-only capture is unsupported on this platform")
}

pub(crate) struct ReceiveOnlyReader;

impl Drop for ReceiveOnlyReader {
    fn drop(&mut self) {
        // Mirror the resource-owning reader's explicit release contract on supported hosts.
    }
}

impl ReceiveOnlyReader {
    pub(crate) fn open(_port: &str) -> Result<Self> {
        bail!("receive-only reader is unsupported on this platform")
    }

    pub(crate) fn read_into(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::Unsupported.into())
    }

    pub(crate) fn read_available(&mut self) -> Result<Vec<u8>> {
        bail!("receive-only reader is unsupported on this platform")
    }

    pub(crate) fn port(&self) -> &str {
        ""
    }
}

pub(crate) struct MacOsDeviceAdapter;

impl MacOsDeviceAdapter {
    pub(crate) fn maybe_profile_fields(_port: &str) -> Result<Option<UsbProfileFields>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }
    pub(crate) fn candidate_ports() -> Result<Vec<String>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn maybe_exact_snapshot(_port: &str) -> Result<Option<UsbDeviceSnapshot>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn maybe_physical_snapshot(
        _expected_physical_identity: &str,
    ) -> Result<Option<UsbDeviceSnapshot>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn profile_transition_snapshot(
        _expected_physical_identity: &str,
        _previous_port: &str,
    ) -> Result<PhysicalSnapshotObservation> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn initial_sample(
        _admitted_port: &str,
        _expected_physical_identity: &str,
    ) -> Result<DeviceObservation> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn recovery_sample(
        _expected_physical_identity: &str,
        _previous_port: &str,
    ) -> Result<DeviceObservation> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn holder_count(_port: &str) -> Result<u16> {
        bail!("macOS ownership adapter is unsupported on this platform")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_transition_rejects_unsupported_platform() {
        // Arrange
        let physical_identity = "fixture-physical-identity";
        let previous_port = "/fixture/serial";

        // Act
        let result =
            MacOsDeviceAdapter::profile_transition_snapshot(physical_identity, previous_port);

        // Assert
        let Err(error) = result else {
            panic!("unsupported adapter must not produce a USB observation");
        };
        assert_eq!(
            error.to_string(),
            "macOS identity adapter is unsupported on this platform"
        );
    }
}
