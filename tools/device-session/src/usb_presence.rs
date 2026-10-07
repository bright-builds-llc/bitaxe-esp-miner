//! One presence sample for an admitted physical USB device.
//!
//! The device is selected by its stable physical identity (location or USB
//! serial number), never by its device node. The enumeration digest changes
//! with every re-enumeration, so callers compare the two independently.

use anyhow::Result;

use crate::macos::MacOsDeviceAdapter;
use crate::UsbProfile;

/// Enumeration-scoped facts about the admitted device while it is present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbPresenceSample {
    /// Digest of the device node, its inode and its mode; it never names the node.
    pub enumeration_sha256: String,
    pub accessible: bool,
    pub holder_count: u16,
    pub profile: UsbProfile,
}

/// Sample the device with `expected_physical_identity`; `None` means it is absent.
///
/// # Errors
///
/// Fails when the platform probe fails or more than one candidate carries the
/// same physical identity.
pub fn sample_usb_presence(expected_physical_identity: &str) -> Result<Option<UsbPresenceSample>> {
    let maybe_snapshot = MacOsDeviceAdapter::maybe_physical_snapshot(expected_physical_identity)?;
    Ok(maybe_snapshot.map(|snapshot| UsbPresenceSample {
        enumeration_sha256: snapshot.enumeration_token,
        accessible: snapshot.accessible,
        holder_count: snapshot.holder_count,
        profile: snapshot.profile,
    }))
}
