use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbExecutionOwner {
    Unknown,
    Rom,
    Application,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbOwnershipIdentity {
    pub transport: UsbProfile,
    pub execution_owner: UsbExecutionOwner,
}

#[must_use]
pub fn classify_usb_ownership(
    vendor: Option<u16>,
    product: Option<u16>,
    product_name: Option<&str>,
) -> UsbOwnershipIdentity {
    let transport = classify_usb_profile(vendor, product, product_name, false);
    let execution_owner = if transport == UsbProfile::WorkerRuntime {
        UsbExecutionOwner::Application
    } else {
        UsbExecutionOwner::Unknown
    };
    UsbOwnershipIdentity {
        transport,
        execution_owner,
    }
}

pub fn admit_rom_execution(
    inspection: &UsbProfileInspection,
    board_info: &[u8],
) -> Result<UsbOwnershipIdentity, UsbSessionError> {
    admit_rom_downloader(inspection.clone(), board_info)?;
    Ok(UsbOwnershipIdentity {
        transport: UsbProfile::SerialJtagRuntime,
        execution_owner: UsbExecutionOwner::Rom,
    })
}

pub fn admit_application_execution(
    transport: UsbProfile,
    marker: &bitaxe_api::UsbBootProfileMarker,
    expected_firmware_commit: &str,
    expected_app_elf_sha256: &str,
) -> Result<UsbOwnershipIdentity, UsbSessionError> {
    let marker_transport = match marker.transport() {
        bitaxe_api::UsbBootTransport::WorkerRuntime => UsbProfile::WorkerRuntime,
        bitaxe_api::UsbBootTransport::SerialJtagRuntime => UsbProfile::SerialJtagRuntime,
    };
    if transport != marker_transport
        || marker.firmware_commit() != expected_firmware_commit
        || marker.app_elf_sha256() != expected_app_elf_sha256
    {
        return Err(handoff_error(
            UsbTerminalCategory::ApplicationIdentityMismatch,
            "the boot-profile marker did not match the admitted application",
        ));
    }
    Ok(UsbOwnershipIdentity {
        transport,
        execution_owner: UsbExecutionOwner::Application,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_profiles_keep_execution_owner_independent() {
        // Arrange / Act
        let worker = classify_usb_ownership(
            Some(0x1209),
            Some(0xb17a),
            Some("Bitaxe Ultra 205 BWG Worker"),
        );
        let serial_jtag = classify_usb_ownership(Some(0x303a), Some(0x1001), None);

        // Assert
        assert_eq!(worker.transport, UsbProfile::WorkerRuntime);
        assert_eq!(worker.execution_owner, UsbExecutionOwner::Application);
        assert_eq!(serial_jtag.transport, UsbProfile::SerialJtagRuntime);
        assert_eq!(serial_jtag.execution_owner, UsbExecutionOwner::Unknown);
    }

    #[test]
    fn board_info_admits_rom_owner_without_relabeling_the_transport() {
        // Arrange
        let inspection = UsbProfileInspection {
            profile: UsbProfile::SerialJtagRuntime,
            port: "private".to_owned(),
            physical_identity_digest: "1".repeat(64),
            enumeration_token: "epoch".to_owned(),
        };

        // Act
        let identity = admit_rom_execution(&inspection, b"Chip type: ESP32-S3\n")
            .expect("board-info should admit ROM ownership");

        // Assert
        assert_eq!(identity.transport, UsbProfile::SerialJtagRuntime);
        assert_eq!(identity.execution_owner, UsbExecutionOwner::Rom);
        assert!(admit_rom_execution(&inspection, b"not board info").is_err());
    }

    #[test]
    fn exact_boot_marker_admits_serial_jtag_application_owner() {
        // Arrange
        let marker = bitaxe_api::UsbBootProfileMarker::new(
            bitaxe_api::UsbBootTransport::SerialJtagRuntime,
            bitaxe_api::UsbBootProfileReason::BootBaselineUnconfirmed,
            bitaxe_api::UsbBootBaseline::Unconfirmed,
            "1".repeat(40),
            "2".repeat(64),
            9,
        )
        .expect("valid marker");

        // Act
        let identity = admit_application_execution(
            UsbProfile::SerialJtagRuntime,
            &marker,
            &"1".repeat(40),
            &"2".repeat(64),
        )
        .expect("exact marker should admit the application");

        // Assert
        assert_eq!(identity.transport, UsbProfile::SerialJtagRuntime);
        assert_eq!(identity.execution_owner, UsbExecutionOwner::Application);
        assert!(admit_application_execution(
            UsbProfile::SerialJtagRuntime,
            &marker,
            &"3".repeat(40),
            &"2".repeat(64),
        )
        .is_err());
    }
}
