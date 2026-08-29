//! Pure native-USB profile and operation planning.

use std::io::Read;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::macos::MacOsDeviceAdapter;
use crate::{UsbSession, UsbSessionError, UsbTerminalCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbProfile {
    WorkerRuntime,
    SerialJtagRuntime,
    RomDownloader,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbIntent {
    Inspect,
    Flash,
    Observe,
    Recover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbOperationPlan {
    InspectOnly,
    ObserveOnly,
    DirectEspflash,
    HandoffThenEspflash,
    RejectUnknownProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProfileTransitionSample {
    Absent,
    Ambiguous,
    Candidate {
        profile: UsbProfile,
        physical_identity_matches: bool,
        accessible: bool,
        holder_count: u16,
        stability_key: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileTransitionDecision {
    Pending,
    Ready,
    Failed(UsbTerminalCategory),
}

pub(crate) struct ProfileTransition {
    expected: UsbProfile,
    maybe_stability_key: Option<String>,
    stable_samples: u8,
}

impl ProfileTransition {
    pub(crate) const fn new(expected: UsbProfile) -> Self {
        Self {
            expected,
            maybe_stability_key: None,
            stable_samples: 0,
        }
    }

    pub(crate) fn observe(&mut self, sample: ProfileTransitionSample) -> ProfileTransitionDecision {
        let ProfileTransitionSample::Candidate {
            profile,
            physical_identity_matches,
            accessible,
            holder_count,
            stability_key,
        } = sample
        else {
            self.reset();
            return if sample == ProfileTransitionSample::Ambiguous {
                ProfileTransitionDecision::Failed(UsbTerminalCategory::BootloaderAmbiguous)
            } else {
                ProfileTransitionDecision::Pending
            };
        };
        if !physical_identity_matches {
            self.reset();
            return ProfileTransitionDecision::Failed(UsbTerminalCategory::PhysicalIdentityDrift);
        }
        if !accessible || holder_count > 0 {
            self.reset();
            return ProfileTransitionDecision::Failed(UsbTerminalCategory::ForeignHolder);
        }
        if profile != self.expected {
            self.reset();
            return ProfileTransitionDecision::Pending;
        }
        let same_sample = self
            .maybe_stability_key
            .as_ref()
            .is_some_and(|previous| previous == &stability_key);
        self.stable_samples = if same_sample {
            self.stable_samples.saturating_add(1)
        } else {
            1
        };
        self.maybe_stability_key = Some(stability_key);
        if self.stable_samples >= 3 {
            ProfileTransitionDecision::Ready
        } else {
            ProfileTransitionDecision::Pending
        }
    }

    pub(crate) fn timeout(&mut self) -> ProfileTransitionDecision {
        self.reset();
        ProfileTransitionDecision::Failed(UsbTerminalCategory::HandoffTransitionTimeout)
    }

    fn reset(&mut self) {
        self.maybe_stability_key = None;
        self.stable_samples = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbProfileInspection {
    pub profile: UsbProfile,
    pub port: String,
    pub physical_identity_digest: String,
    pub enumeration_token: String,
}

pub fn inspect_usb_profile(port: &str) -> Result<UsbProfileInspection> {
    let fields = MacOsDeviceAdapter::maybe_profile_fields(port)?
        .context("the selected USB profile is absent")?;
    let vendor = parse_usb_number(&fields.vendor);
    let product = parse_usb_number(&fields.product);
    let profile = classify_usb_profile(vendor, product, fields.product_name.as_deref(), false);
    Ok(UsbProfileInspection {
        profile,
        port: fields.port,
        physical_identity_digest: fields.physical_identity_digest,
        enumeration_token: fields.enumeration_token,
    })
}

#[must_use]
pub fn classify_usb_profile(
    vendor: Option<u16>,
    product: Option<u16>,
    product_name: Option<&str>,
    rom_admitted: bool,
) -> UsbProfile {
    if matches!((vendor, product), (Some(0x1209), Some(0xb17a)))
        && matches!(product_name, Some("Bitaxe Ultra 205 BWG Worker"))
    {
        UsbProfile::WorkerRuntime
    } else if rom_admitted && matches!((vendor, product), (Some(0x303a), Some(0x1001))) {
        UsbProfile::RomDownloader
    } else if matches!((vendor, product), (Some(0x303a), Some(0x1001))) {
        UsbProfile::SerialJtagRuntime
    } else {
        UsbProfile::Unknown
    }
}

pub fn admit_rom_downloader(
    mut inspection: UsbProfileInspection,
    board_info: &[u8],
) -> Result<UsbProfileInspection, UsbSessionError> {
    if inspection.profile != UsbProfile::SerialJtagRuntime
        || !String::from_utf8_lossy(board_info).contains("ESP32-S3")
    {
        return Err(handoff_error(
            UsbTerminalCategory::BootloaderSyncFailed,
            "board-info did not admit the selected ESP32-S3 ROM downloader",
        ));
    }
    inspection.profile = UsbProfile::RomDownloader;
    Ok(inspection)
}

fn parse_usb_number(value: &str) -> Option<u16> {
    value
        .strip_prefix("0x")
        .map(|hex| u16::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse().ok())
}

pub fn handoff_worker_to_rom(session: &mut UsbSession) -> Result<(), UsbSessionError> {
    if !cfg!(target_os = "macos") {
        return Err(handoff_error(
            UsbTerminalCategory::HandoffUnsupported,
            "native USB maintenance handoff is qualified only on macOS",
        ));
    }
    let inspection = inspect_usb_profile(session.port())
        .map_err(|error| handoff_error(UsbTerminalCategory::RuntimeProfileUnknown, error))?;
    if inspection.profile != UsbProfile::WorkerRuntime {
        return Err(handoff_error(
            UsbTerminalCategory::HandoffUnsupported,
            "maintenance handoff requires the admitted Worker runtime profile",
        ));
    }
    if inspection.physical_identity_digest != session.physical_identity_digest() {
        return Err(handoff_error(
            UsbTerminalCategory::PhysicalIdentityDrift,
            "the Worker profile does not match the retained USB lease",
        ));
    }
    #[cfg(target_os = "macos")]
    {
        use std::fs::OpenOptions;
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(session.port())
            .map_err(|error| handoff_error(UsbTerminalCategory::ForeignHolder, error))?;
        let fd = file.as_raw_fd();
        let mut dtr = libc::TIOCM_DTR;
        if unsafe { libc::ioctl(fd, libc::TIOCMBIS, &mut dtr) } != 0 {
            return Err(handoff_error(
                UsbTerminalCategory::HandoffUnsupported,
                "the Worker CDC adapter rejected DTR assertion",
            ));
        }
        let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
        if unsafe { libc::tcgetattr(fd, &mut termios) } != 0
            || unsafe { libc::cfsetspeed(&mut termios, libc::B1200) } != 0
            || unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) } != 0
        {
            clear_dtr(fd, &mut dtr);
            return Err(handoff_error(
                UsbTerminalCategory::HandoffUnsupported,
                "the Worker CDC adapter rejected exact 1200-baud line coding",
            ));
        }
        let deadline = Instant::now() + Duration::from_secs(6);
        let mut observed = Vec::new();
        while Instant::now() < deadline {
            let mut chunk = [0_u8; 256];
            match file.read(&mut chunk) {
                Ok(count) if observed.len().saturating_add(count) <= 4_096 => {
                    observed.extend_from_slice(&chunk[..count]);
                }
                Ok(_) => {
                    clear_dtr(fd, &mut dtr);
                    return Err(handoff_error(
                        UsbTerminalCategory::HandoffRejectedUnsafeState,
                        "the readiness transcript exceeded its fixed bound",
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    clear_dtr(fd, &mut dtr);
                    return Err(handoff_error(
                        UsbTerminalCategory::HandoffRejectedUnsafeState,
                        error,
                    ));
                }
            }
            if observed
                .windows(b"usb_maintenance={\"status\":\"ready\"}".len())
                .any(|window| window == b"usb_maintenance={\"status\":\"ready\"}")
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if !observed
            .windows(b"usb_maintenance={\"status\":\"ready\"}".len())
            .any(|window| window == b"usb_maintenance={\"status\":\"ready\"}")
        {
            clear_dtr(fd, &mut dtr);
            return Err(handoff_error(
                UsbTerminalCategory::HandoffReadyTimeout,
                "the Worker did not emit the maintenance readiness receipt",
            ));
        }
        if unsafe { libc::ioctl(fd, libc::TIOCMBIC, &mut dtr) } != 0 {
            return Err(handoff_error(
                UsbTerminalCategory::HandoffUnsupported,
                "the Worker CDC adapter rejected the commit edge",
            ));
        }
        drop(file);
        session.reacquire_profile(UsbProfile::SerialJtagRuntime)
    }
    #[cfg(not(target_os = "macos"))]
    unreachable!()
}

#[cfg(target_os = "macos")]
fn clear_dtr(fd: std::os::fd::RawFd, dtr: &mut libc::c_int) {
    let _result = unsafe { libc::ioctl(fd, libc::TIOCMBIC, dtr) };
}

fn handoff_error(category: UsbTerminalCategory, detail: impl std::fmt::Display) -> UsbSessionError {
    UsbSessionError {
        category,
        detail: detail.to_string(),
    }
}

#[must_use]
pub const fn plan_usb_operation(intent: UsbIntent, profile: UsbProfile) -> UsbOperationPlan {
    match (intent, profile) {
        (UsbIntent::Inspect, UsbProfile::Unknown) => UsbOperationPlan::RejectUnknownProfile,
        (UsbIntent::Inspect, _) => UsbOperationPlan::InspectOnly,
        (UsbIntent::Observe, UsbProfile::WorkerRuntime | UsbProfile::SerialJtagRuntime) => {
            UsbOperationPlan::ObserveOnly
        }
        (UsbIntent::Flash | UsbIntent::Recover, UsbProfile::WorkerRuntime) => {
            UsbOperationPlan::HandoffThenEspflash
        }
        (
            UsbIntent::Flash | UsbIntent::Recover,
            UsbProfile::SerialJtagRuntime | UsbProfile::RomDownloader,
        ) => UsbOperationPlan::DirectEspflash,
        _ => UsbOperationPlan::RejectUnknownProfile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_runtime_flash_requires_handoff_before_espflash() {
        // Arrange / Act
        let plan = plan_usb_operation(UsbIntent::Flash, UsbProfile::WorkerRuntime);

        // Assert
        assert_eq!(plan, UsbOperationPlan::HandoffThenEspflash);
    }

    #[test]
    fn exact_usb_descriptors_classify_profiles_without_claiming_physical_identity() {
        // Arrange / Act / Assert
        assert_eq!(
            classify_usb_profile(
                Some(0x1209),
                Some(0xb17a),
                Some("Bitaxe Ultra 205 BWG Worker"),
                false,
            ),
            UsbProfile::WorkerRuntime
        );
        assert_eq!(
            classify_usb_profile(Some(0x303a), Some(0x1001), None, false),
            UsbProfile::SerialJtagRuntime
        );
        assert_eq!(
            classify_usb_profile(Some(0x303a), Some(0x1001), None, true),
            UsbProfile::RomDownloader
        );
        assert_eq!(
            classify_usb_profile(Some(0x1209), Some(0xb17a), Some("other"), false),
            UsbProfile::Unknown
        );
    }

    #[test]
    fn board_info_is_required_to_promote_serial_jtag_to_rom() {
        // Arrange
        let inspection = UsbProfileInspection {
            profile: UsbProfile::SerialJtagRuntime,
            port: "port".to_owned(),
            physical_identity_digest: "physical".to_owned(),
            enumeration_token: "enumeration".to_owned(),
        };

        // Act
        let admitted = admit_rom_downloader(inspection.clone(), b"Chip type: ESP32-S3\n");
        let rejected = admit_rom_downloader(inspection, b"not a board-info response");

        // Assert
        assert_eq!(
            admitted.expect("ROM admission").profile,
            UsbProfile::RomDownloader
        );
        assert_eq!(
            rejected.expect_err("missing board-info must fail").category,
            UsbTerminalCategory::BootloaderSyncFailed
        );
    }

    #[test]
    fn monitoring_never_arms_handoff() {
        // Arrange / Act
        let worker = plan_usb_operation(UsbIntent::Observe, UsbProfile::WorkerRuntime);
        let serial = plan_usb_operation(UsbIntent::Observe, UsbProfile::SerialJtagRuntime);

        // Assert
        assert_eq!(worker, UsbOperationPlan::ObserveOnly);
        assert_eq!(serial, UsbOperationPlan::ObserveOnly);
    }

    #[test]
    fn direct_profiles_never_request_maintenance_traffic() {
        // Arrange / Act
        let serial = plan_usb_operation(UsbIntent::Flash, UsbProfile::SerialJtagRuntime);
        let rom = plan_usb_operation(UsbIntent::Recover, UsbProfile::RomDownloader);

        // Assert
        assert_eq!(serial, UsbOperationPlan::DirectEspflash);
        assert_eq!(rom, UsbOperationPlan::DirectEspflash);
    }

    #[test]
    fn profile_transition_requires_three_stable_same_physical_samples() {
        // Arrange
        let mut transition = ProfileTransition::new(UsbProfile::SerialJtagRuntime);
        let sample = || ProfileTransitionSample::Candidate {
            profile: UsbProfile::SerialJtagRuntime,
            physical_identity_matches: true,
            accessible: true,
            holder_count: 0,
            stability_key: "rom-epoch".to_owned(),
        };

        // Act / Assert
        assert_eq!(
            transition.observe(sample()),
            ProfileTransitionDecision::Pending
        );
        assert_eq!(
            transition.observe(sample()),
            ProfileTransitionDecision::Pending
        );
        assert_eq!(
            transition.observe(sample()),
            ProfileTransitionDecision::Ready
        );
    }

    #[test]
    fn profile_transition_fails_closed_on_ambiguity_identity_or_holder() {
        for (sample, category) in [
            (
                ProfileTransitionSample::Ambiguous,
                UsbTerminalCategory::BootloaderAmbiguous,
            ),
            (
                ProfileTransitionSample::Candidate {
                    profile: UsbProfile::SerialJtagRuntime,
                    physical_identity_matches: false,
                    accessible: true,
                    holder_count: 0,
                    stability_key: "foreign".to_owned(),
                },
                UsbTerminalCategory::PhysicalIdentityDrift,
            ),
            (
                ProfileTransitionSample::Candidate {
                    profile: UsbProfile::SerialJtagRuntime,
                    physical_identity_matches: true,
                    accessible: true,
                    holder_count: 1,
                    stability_key: "held".to_owned(),
                },
                UsbTerminalCategory::ForeignHolder,
            ),
        ] {
            // Arrange
            let mut transition = ProfileTransition::new(UsbProfile::SerialJtagRuntime);

            // Act / Assert
            assert_eq!(
                transition.observe(sample),
                ProfileTransitionDecision::Failed(category)
            );
        }
    }

    #[test]
    fn profile_transition_timeout_is_closed() {
        // Arrange
        let mut transition = ProfileTransition::new(UsbProfile::SerialJtagRuntime);

        // Act / Assert
        assert_eq!(
            transition.timeout(),
            ProfileTransitionDecision::Failed(UsbTerminalCategory::HandoffTransitionTimeout)
        );
    }
}
