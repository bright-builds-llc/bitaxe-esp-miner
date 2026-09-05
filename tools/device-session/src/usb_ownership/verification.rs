use super::{inspect_usb_profile, UsbProfile};
use crate::{UsbSession, UsbSessionError, UsbTerminalCategory};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationTransportObservation {
    pub transport: UsbProfile,
    pub reenumerated: bool,
}

pub fn run_installed_application(
    session: &mut UsbSession,
    esptool_bin: &Path,
) -> Result<ApplicationTransportObservation, UsbSessionError> {
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
    let (transport, reenumerated) = session.reacquire_application_transport()?;
    Ok(ApplicationTransportObservation {
        transport,
        reenumerated,
    })
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
        "hard_reset".to_owned(),
        "--no-stub".to_owned(),
        "run".to_owned(),
    ]
}
