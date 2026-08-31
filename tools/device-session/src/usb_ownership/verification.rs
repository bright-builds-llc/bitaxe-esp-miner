use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::{
    admit_rom_downloader, handoff_worker_to_rom, inspect_usb_profile, ProfileObservationCounts,
    UsbProfile,
};
use crate::{UsbSession, UsbSessionError, UsbTerminalCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct NativeUsbTransitionOutcome {
    pub ready_received: bool,
    pub committed_received: bool,
    pub bus_reset_observed: bool,
    pub profile_counts: ProfileObservationCounts,
    pub rom_admitted: bool,
    pub application_reappeared: bool,
}

#[must_use]
pub fn native_usb_transition_module_sha256() -> String {
    let sources = [
        ("verification.rs", include_str!("verification.rs")),
        ("maintenance.rs", include_str!("maintenance.rs")),
        ("profile_trace.rs", include_str!("profile_trace.rs")),
        (
            "profile_reacquire.rs",
            include_str!("../usb/profile_reacquire.rs"),
        ),
        ("lifecycle.rs", include_str!("../usb/lifecycle.rs")),
    ];
    let mut digest = Sha256::new();
    for (path, source) in sources {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(source.as_bytes());
        digest.update([0xff]);
    }
    let output = digest.finalize();
    let mut encoded = String::with_capacity(output.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in output {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub fn verify_native_usb_transition(
    session: &mut UsbSession,
    espflash_bin: &Path,
) -> Result<NativeUsbTransitionOutcome, UsbSessionError> {
    let handoff = handoff_worker_to_rom(session)?;
    let serial_jtag = inspect_usb_profile(session.port()).map_err(|error| UsbSessionError {
        category: UsbTerminalCategory::RuntimeProfileUnknown,
        detail: error.to_string(),
    })?;
    if serial_jtag.profile != UsbProfile::SerialJtagRuntime
        || serial_jtag.physical_identity_digest != session.physical_identity_digest()
    {
        return Err(UsbSessionError {
            category: UsbTerminalCategory::PhysicalIdentityDrift,
            detail: "the pre-admission Serial/JTAG profile did not match the retained lease"
                .to_owned(),
        });
    }
    let args = [
        "board-info".to_owned(),
        "--chip".to_owned(),
        "esp32s3".to_owned(),
        "--port".to_owned(),
        session.port().to_owned(),
        "--non-interactive".to_owned(),
        "--before".to_owned(),
        "no-reset".to_owned(),
        "--after".to_owned(),
        "hard-reset".to_owned(),
    ];
    let output = session.run_espflash_probe(espflash_bin, &args, Duration::from_secs(30))?;
    let mut board_info = output.stdout;
    board_info.extend_from_slice(&output.stderr);
    let rom = admit_rom_downloader(serial_jtag, &board_info).map_err(|error| UsbSessionError {
        category: UsbTerminalCategory::RomAdmissionFailed,
        detail: error.detail,
    })?;
    if rom.physical_identity_digest != session.physical_identity_digest() {
        return Err(UsbSessionError {
            category: UsbTerminalCategory::PhysicalIdentityDrift,
            detail: "the admitted ROM profile did not match the retained lease".to_owned(),
        });
    }
    let application_counts = session.reacquire_profile(UsbProfile::WorkerRuntime)?;
    Ok(NativeUsbTransitionOutcome {
        ready_received: handoff.ready_received,
        committed_received: handoff.committed_received,
        bus_reset_observed: handoff.bus_reset_observed,
        profile_counts: handoff.profile_counts.merge(application_counts),
        rom_admitted: true,
        application_reappeared: true,
    })
}

pub fn run_installed_application(
    session: &mut UsbSession,
    esptool_bin: &Path,
) -> Result<ProfileObservationCounts, UsbSessionError> {
    let downloader = inspect_usb_profile(session.port()).map_err(|error| UsbSessionError {
        category: UsbTerminalCategory::RuntimeProfileUnknown,
        detail: error.to_string(),
    })?;
    if downloader.physical_identity_digest != session.physical_identity_digest() {
        return Err(UsbSessionError {
            category: UsbTerminalCategory::PhysicalIdentityDrift,
            detail: "the downloader profile did not match the retained lease".to_owned(),
        });
    }
    if !matches!(
        downloader.profile,
        UsbProfile::SerialJtagRuntime | UsbProfile::RomDownloader
    ) {
        return Err(UsbSessionError {
            category: UsbTerminalCategory::RuntimeProfileUnknown,
            detail: "application run requires an admitted downloader profile".to_owned(),
        });
    }
    let args = installed_application_args(session.port());
    session.run_espflash_probe(esptool_bin, &args, Duration::from_secs(30))?;
    session.reacquire_profile(UsbProfile::WorkerRuntime)
}

fn installed_application_args(port: &str) -> [String; 10] {
    [
        "--chip".to_owned(),
        "esp32s3".to_owned(),
        "--port".to_owned(),
        port.to_owned(),
        "--before".to_owned(),
        "no_reset".to_owned(),
        "--after".to_owned(),
        "no_reset".to_owned(),
        "--no-stub".to_owned(),
        "run".to_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use super::installed_application_args;

    #[test]
    fn transition_module_excludes_every_device_write_surface() {
        // Arrange
        let source = include_str!("verification.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes its tests");

        // Act / Assert
        for forbidden in [
            "write-bin",
            "write_flash",
            "erase_flash",
            "generate_nvs_partition",
            "wifi_credentials",
            "pool_credentials",
            "mining-campaign",
        ] {
            assert!(
                !source.contains(forbidden),
                "transition module contains forbidden surface {forbidden}"
            );
        }
    }

    #[test]
    fn installed_application_uses_only_the_exact_rom_run_command() {
        // Arrange / Act
        let args = installed_application_args("admitted");

        // Assert
        assert_eq!(
            args,
            [
                "--chip",
                "esp32s3",
                "--port",
                "admitted",
                "--before",
                "no_reset",
                "--after",
                "no_reset",
                "--no-stub",
                "run",
            ]
        );
    }
}
