//! Installed image identity, learned from the mounted version file after a successful read.
use std::sync::OnceLock;

use bitaxe_api::{parse_static_asset_version, PlatformFact, PlatformUnavailableReason};

/// One boot/mount lifetime. Future same-boot WWW replacement or remount support must
/// replace or invalidate this holder before publishing facts from the new image.
pub(super) struct InstalledAssetVersion {
    value: OnceLock<String>,
}

impl InstalledAssetVersion {
    pub(super) const fn new() -> Self {
        Self {
            value: OnceLock::new(),
        }
    }

    pub(super) fn observe(&self, read: impl FnOnce() -> Option<Vec<u8>>) -> PlatformFact<String> {
        if let Some(value) = self.value.get() {
            return PlatformFact::available(value.clone());
        }
        let Some(value) = read().and_then(|bytes| parse_static_asset_version(&bytes).ok()) else {
            // An early unavailable mount, failed read, or malformed marker cannot
            // permanently hide a later successful observation of the installed image.
            return PlatformFact::unavailable(PlatformUnavailableReason::StaticAssetUnavailable);
        };
        // Loading happens before initialization, so file I/O never holds this lock.
        PlatformFact::available(self.value.get_or_init(|| value).clone())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn repeated_snapshots_read_the_installed_marker_once() {
        // Arrange
        let version = InstalledAssetVersion::new();
        let reads = Cell::new(0);

        // Act
        let observations = (0..100)
            .map(|_| {
                version.observe(|| {
                    reads.set(reads.get() + 1);
                    Some(b"installed-assets-v1\n".to_vec())
                })
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(reads.get(), 1);
        assert!(observations
            .iter()
            .all(|value| value == &PlatformFact::available("installed-assets-v1".to_owned())));
    }

    #[test]
    fn unavailable_or_malformed_marker_does_not_poison_later_success() {
        // Arrange
        let version = InstalledAssetVersion::new();
        let unavailable =
            PlatformFact::unavailable(PlatformUnavailableReason::StaticAssetUnavailable);

        // Act
        let before_mount = version.observe(|| None);
        let malformed = version.observe(|| Some(b"invalid asset version".to_vec()));
        let recovered = version.observe(|| Some(b"installed-assets-v2\n".to_vec()));
        let retained = version
            .observe(|| panic!("a proved identity does not re-probe filesystem availability"));

        // Assert
        assert_eq!(before_mount, unavailable);
        assert_eq!(malformed, unavailable);
        assert_eq!(
            recovered,
            PlatformFact::available("installed-assets-v2".to_owned())
        );
        assert_eq!(retained, recovered);
    }

    #[test]
    fn installed_assets_are_not_substituted_with_the_firmware_label() {
        // Arrange
        let version = InstalledAssetVersion::new();
        let firmware_label = "firmware-build-v1";

        // Act
        let observed = version.observe(|| Some(b"asset-build-v2\n".to_vec()));

        // Assert
        assert_eq!(
            observed,
            PlatformFact::available("asset-build-v2".to_owned())
        );
        assert_ne!(observed, PlatformFact::available(firmware_label.to_owned()));
    }

    #[test]
    fn a_fresh_mount_holder_learns_the_new_installed_image() {
        // Arrange
        let prior = InstalledAssetVersion::new();
        let first = prior.observe(|| Some(b"assets-v1\n".to_vec()));
        let next = InstalledAssetVersion::new();

        // Act
        let old_mount = prior.observe(|| Some(b"assets-v2\n".to_vec()));
        let new_mount = next.observe(|| Some(b"assets-v2\n".to_vec()));

        // Assert
        assert_eq!(old_mount, first);
        assert_eq!(new_mount, PlatformFact::available("assets-v2".to_owned()));
    }
}
