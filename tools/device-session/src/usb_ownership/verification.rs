use super::{inspect_usb_profile, UsbProfile};
use crate::{UsbSession, UsbSessionError, UsbTerminalCategory};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationTransportObservation {
    pub transport: UsbProfile,
    pub reenumerated: bool,
    /// FORCE_DOWNLOAD state observed before the conditional masked clear.
    pub force_download_bit_set: bool,
}

/// Returns through the official native reset after the caller admits ROM and validates tools.
/// Shared USB descriptors alone never prove that the application subsequently executed.
pub fn run_installed_application(
    session: &mut UsbSession,
    esptool_bin: &Path,
    espflash_bin: &Path,
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
    let port = session.port().to_owned();
    let force_download_bit_set = reset_admitted_application(
        &port,
        esptool_bin,
        espflash_bin,
        |program, args, timeout| {
            session
                .run_espflash_probe(program, args, timeout)
                .map(|output| output.stdout)
        },
    )?;
    let (transport, reenumerated) = session.reacquire_application_transport()?;
    Ok(ApplicationTransportObservation {
        transport,
        reenumerated,
        force_download_bit_set,
    })
}

const RTC_OPTION1: &str = "0x6000812c";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

fn reset_admitted_application(
    port: &str,
    esptool: &Path,
    espflash: &Path,
    mut run: impl FnMut(&Path, &[String], Duration) -> Result<Vec<u8>, UsbSessionError>,
) -> Result<bool, UsbSessionError> {
    let read = force_download_args(port, false);
    let was_set = force_download_bit(&run(esptool, &read, COMMAND_TIMEOUT)?)?;
    if was_set {
        run(esptool, &force_download_args(port, true), COMMAND_TIMEOUT)?;
        if force_download_bit(&run(esptool, &read, COMMAND_TIMEOUT)?)? {
            return Err(reset_admission_error("force_download_clear_unconfirmed"));
        }
    }
    run(espflash, &installed_application_args(port), COMMAND_TIMEOUT)?;
    Ok(was_set)
}

fn force_download_args(port: &str, clear: bool) -> Vec<String> {
    let mut args = [
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--before",
        "no_reset",
        "--after",
        "no_reset",
        "--no-stub",
        if clear { "write_mem" } else { "read_mem" },
        RTC_OPTION1,
    ]
    .map(str::to_owned)
    .to_vec();
    if clear {
        // ROM read-modify-write updates only FORCE_DOWNLOAD; every other bit is preserved.
        args.extend(["0x00000000".to_owned(), "0x00000001".to_owned()]);
    }
    args
}

fn force_download_bit(bytes: &[u8]) -> Result<bool, UsbSessionError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| reset_admission_error("force_download_read_malformed"))?;
    let mut maybe_value = None;
    for line in text.lines().filter(|line| line.starts_with(RTC_OPTION1)) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if maybe_value.is_some()
            || fields.len() != 3
            || fields[0] != RTC_OPTION1
            || fields[1] != "="
        {
            return Err(reset_admission_error("force_download_read_malformed"));
        }
        let raw = fields[2]
            .strip_prefix("0x")
            .filter(|raw| raw.len() == 8 && raw.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .ok_or_else(|| reset_admission_error("force_download_read_malformed"))?;
        maybe_value = Some(
            u32::from_str_radix(raw, 16)
                .map_err(|_| reset_admission_error("force_download_read_malformed"))?,
        );
    }
    maybe_value
        .map(|value| value & 1 != 0)
        .ok_or_else(|| reset_admission_error("force_download_read_missing"))
}

fn reset_admission_error(detail: &str) -> UsbSessionError {
    UsbSessionError {
        category: UsbTerminalCategory::RomAdmissionFailed,
        detail: detail.to_owned(),
    }
}

fn installed_application_args(port: &str) -> Vec<String> {
    // espflash4.5 reset() calls Connection::reset() directly, which selects the native
    // USB DTR=false/RTS sequence. No separate ROM 'run' or subsequent reset is sent.
    [
        "reset",
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--before",
        "no-reset-no-sync",
        "--after",
        "hard-reset",
        "--no-stub",
        "--non-interactive",
        "--skip-update-check",
    ]
    .map(str::to_owned)
    .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn application_return_uses_official_native_reset_without_run_or_stub() {
        // Arrange
        let args = installed_application_args("admitted-port");
        // Act / Assert
        assert_eq!(args[0], "reset");
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--before", "no-reset-no-sync"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--after", "hard-reset"]));
        assert!(args.iter().any(|arg| arg == "--no-stub"));
        assert!(!args.iter().any(|arg| arg == "run" || arg == "hard_reset"));
    }
    #[test]
    fn force_download_clear_is_conditional_masked_and_verified_before_reset() {
        // Arrange
        let mut calls = Vec::new();
        let mut reads = 0;
        // Act
        let was_set = reset_admitted_application(
            "admitted",
            Path::new("esptool"),
            Path::new("espflash"),
            |program, args, timeout| {
                calls.push((program.to_owned(), args.to_vec()));
                assert_eq!(timeout, Duration::from_secs(30));
                if args.iter().any(|arg| arg == "read_mem") {
                    reads += 1;
                    return Ok(format!(
                        "0x6000812c = 0x{:08x}\n",
                        if reads == 1 {
                            0xa5a50001u32
                        } else {
                            0xa5a50000u32
                        }
                    )
                    .into_bytes());
                }
                Ok(Vec::new())
            },
        )
        .expect("qualified reset sequence");
        // Assert
        assert!(was_set);
        assert_eq!(calls.len(), 4);
        assert_eq!(
            &calls[1].1[9..],
            ["write_mem", RTC_OPTION1, "0x00000000", "0x00000001"]
        );
        assert_eq!(calls[2].1[9], "read_mem");
        assert_eq!(calls[3].0, Path::new("espflash"));
        assert_eq!(calls[3].1, installed_application_args("admitted"));
    }

    #[test]
    fn already_clear_force_download_never_writes_a_register() {
        // Arrange
        let mut calls = Vec::new();
        // Act
        let was_set = reset_admitted_application(
            "admitted",
            Path::new("esptool"),
            Path::new("espflash"),
            |_, args, _| {
                calls.push(args.to_vec());
                Ok(b"0x6000812c = 0xa5a50000\n".to_vec())
            },
        )
        .expect("already clear reset");
        // Assert
        assert!(!was_set);
        assert_eq!(calls.len(), 2);
        assert!(!calls
            .iter()
            .flatten()
            .any(|arg| arg == "write_mem" || arg == "run"));
    }

    #[test]
    fn failed_clear_readback_prevents_reset() {
        // Arrange
        let mut reset_calls = 0;
        // Act
        let failed = reset_admitted_application(
            "admitted",
            Path::new("esptool"),
            Path::new("espflash"),
            |program, _, _| {
                if program == Path::new("espflash") {
                    reset_calls += 1;
                }
                Ok(b"0x6000812c = 0x00000001\n".to_vec())
            },
        );
        // Assert
        assert_eq!(
            failed.expect_err("uncleared mode").detail,
            "force_download_clear_unconfirmed"
        );
        assert_eq!(reset_calls, 0);
    }

    #[test]
    fn failed_clear_keeps_earliest_error_and_stops_before_readback_or_reset() {
        // Arrange
        let earliest = UsbSessionError {
            category: UsbTerminalCategory::CleanupFailed,
            detail: "fixture_child_unsettled".to_owned(),
        };
        let mut calls = 0;
        // Act
        let failed = reset_admitted_application(
            "admitted",
            Path::new("esptool"),
            Path::new("espflash"),
            |_, _, _| {
                calls += 1;
                if calls == 1 {
                    Ok(b"0x6000812c = 0x00000001\n".to_vec())
                } else {
                    Err(earliest.clone())
                }
            },
        );
        // Assert
        assert_eq!(failed.expect_err("failed clear"), earliest);
        assert_eq!(calls, 2);
    }

    #[test]
    fn malformed_or_missing_register_evidence_never_authorizes_reset() {
        // Arrange / Act / Assert
        for input in [
            "",
            "0x6000812c = nope\n",
            "0x6000812c = 0x00000000 extra\n",
            "0x6000812c = 0x00000000\n0x6000812c = 0x00000000\n",
        ] {
            let mut calls = 0;
            assert!(reset_admitted_application(
                "admitted",
                Path::new("esptool"),
                Path::new("espflash"),
                |_, _, _| {
                    calls += 1;
                    Ok(input.as_bytes().to_vec())
                }
            )
            .is_err());
            assert_eq!(calls, 1);
        }
    }
}
