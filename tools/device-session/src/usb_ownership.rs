//! Pure native-USB profile and operation planning.

use anyhow::{Context, Result};

use crate::macos::MacOsDeviceAdapter;
use crate::{UsbSessionError, UsbTerminalCategory};

mod execution;
mod profile_trace;
mod verification;

pub use execution::{
    admit_application_execution, admit_rom_execution, classify_usb_ownership, UsbExecutionOwner,
    UsbOwnershipIdentity,
};
pub use profile_trace::ProfileObservationCounts;
#[cfg(test)]
use profile_trace::MAX_PROFILE_OBSERVATION_SAMPLES;
pub(crate) use profile_trace::{
    profile_observation_category, ProfileObservationCategory, ProfileObservationTrace,
};
pub use verification::{run_installed_application, ApplicationTransportObservation};

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
        || !board_info_reports_esp32s3(board_info)
    {
        return Err(handoff_error(
            UsbTerminalCategory::BootloaderSyncFailed,
            "board-info did not admit the selected ESP32-S3 ROM downloader",
        ));
    }
    inspection.profile = UsbProfile::RomDownloader;
    Ok(inspection)
}

#[must_use]
pub fn board_info_reports_esp32s3(board_info: &[u8]) -> bool {
    String::from_utf8_lossy(board_info).lines().any(|line| {
        let Some(chip) = line
            .trim_start()
            .strip_prefix("Chip type:")
            .and_then(|value| value.split_whitespace().next())
        else {
            return false;
        };
        chip.eq_ignore_ascii_case("esp32s3") || chip.eq_ignore_ascii_case("esp32-s3")
    })
}

fn parse_usb_number(value: &str) -> Option<u16> {
    value
        .strip_prefix("0x")
        .map(|hex| u16::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse().ok())
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
        (UsbIntent::Observe, UsbProfile::SerialJtagRuntime) => UsbOperationPlan::ObserveOnly,
        (UsbIntent::Flash | UsbIntent::Recover, UsbProfile::WorkerRuntime) => {
            UsbOperationPlan::RejectUnknownProfile
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
    fn legacy_tinyusb_flash_is_rejected_without_maintenance() {
        // Arrange / Act
        let plan = plan_usb_operation(UsbIntent::Flash, UsbProfile::WorkerRuntime);

        // Assert
        assert_eq!(plan, UsbOperationPlan::RejectUnknownProfile);
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
    fn real_espflash_chip_type_spelling_admits_rom_downloader() {
        // Arrange
        let inspection = UsbProfileInspection {
            profile: UsbProfile::SerialJtagRuntime,
            port: "port".to_owned(),
            physical_identity_digest: "physical".to_owned(),
            enumeration_token: "enumeration".to_owned(),
        };
        let board_info = b"Chip type:         esp32s3 (revision v0.2)\n";

        // Act
        let admitted = admit_rom_downloader(inspection, board_info);

        // Assert
        assert_eq!(
            admitted.expect("real espflash ROM admission").profile,
            UsbProfile::RomDownloader
        );
    }

    #[test]
    fn observation_rejects_legacy_tinyusb_and_accepts_fixed_serial() {
        // Arrange / Act
        let worker = plan_usb_operation(UsbIntent::Observe, UsbProfile::WorkerRuntime);
        let serial = plan_usb_operation(UsbIntent::Observe, UsbProfile::SerialJtagRuntime);

        // Assert
        assert_eq!(worker, UsbOperationPlan::RejectUnknownProfile);
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
    fn closed_profile_trace_distinguishes_wrong_profile_absence_and_identity_drift() {
        // Arrange
        let mut trace = ProfileObservationTrace::new(UsbProfile::SerialJtagRuntime);

        // Act
        trace.observe(ProfileObservationCategory::SameWorker);
        trace.observe(ProfileObservationCategory::Absent);
        trace.observe(ProfileObservationCategory::PhysicalMismatch);

        // Assert
        assert_eq!(
            trace.samples(),
            [
                ProfileObservationCategory::SameWorker,
                ProfileObservationCategory::Absent,
                ProfileObservationCategory::PhysicalMismatch,
            ]
        );
        assert!(!trace.overflowed());
    }

    #[test]
    fn closed_profile_trace_is_bounded() {
        // Arrange
        let mut trace = ProfileObservationTrace::new(UsbProfile::SerialJtagRuntime);

        // Act
        for _ in 0..=MAX_PROFILE_OBSERVATION_SAMPLES {
            trace.observe(ProfileObservationCategory::SameWorker);
        }

        // Assert
        assert_eq!(trace.samples().len(), MAX_PROFILE_OBSERVATION_SAMPLES);
        assert!(trace.overflowed());
    }

    #[test]
    fn closed_profile_categories_cover_every_observable_profile() {
        // Arrange / Act / Assert
        assert_eq!(
            profile_observation_category(Some(UsbProfile::WorkerRuntime), true),
            ProfileObservationCategory::SameWorker
        );
        assert_eq!(
            profile_observation_category(Some(UsbProfile::SerialJtagRuntime), true),
            ProfileObservationCategory::SameSerialJtag
        );
        assert_eq!(
            profile_observation_category(Some(UsbProfile::Unknown), true),
            ProfileObservationCategory::SameUnknown
        );
        assert_eq!(
            profile_observation_category(None, true),
            ProfileObservationCategory::Absent
        );
        assert_eq!(
            profile_observation_category(Some(UsbProfile::WorkerRuntime), false),
            ProfileObservationCategory::PhysicalMismatch
        );
    }

    #[test]
    fn serialized_profile_trace_excludes_raw_identity_fields() {
        // Arrange
        let mut trace = ProfileObservationTrace::new(UsbProfile::SerialJtagRuntime);
        trace.observe(ProfileObservationCategory::SameWorker);

        // Act
        let encoded = serde_json::to_string(&trace).expect("profile trace JSON");

        // Assert
        assert!(encoded.contains("same_worker"));
        for forbidden in ["port", "address", "serial", "location", "digest"] {
            assert!(!encoded.contains(&format!("\"{forbidden}\":")));
        }
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
