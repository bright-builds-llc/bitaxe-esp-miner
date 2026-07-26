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

pub(crate) struct DeviceObservation {
    pub(crate) event: SessionEvent,
    pub(crate) maybe_port: Option<String>,
}

pub(crate) struct ReceiveOnlyReader;

impl ReceiveOnlyReader {
    pub(crate) fn open(_port: &str) -> Result<Self> {
        bail!("receive-only reader is unsupported on this platform")
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
    pub(crate) fn candidate_ports() -> Result<Vec<String>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn exact_snapshot(_port: &str) -> Result<Option<UsbDeviceSnapshot>> {
        bail!("macOS identity adapter is unsupported on this platform")
    }

    pub(crate) fn physical_snapshot(
        _expected_physical_identity: &str,
    ) -> Result<Option<UsbDeviceSnapshot>> {
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
