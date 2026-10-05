use std::time::Duration;

use super::identity::{IdentityScanner, IdentityVerdict};
use super::model::{CycleRun, FinalCleanup};
use super::runner::{run_cycles, CycleEffects, ObserveFacts, ProfileFacts, ResetFacts};
use super::*;
use crate::{
    UsbCommandTermination, UsbProfile, UsbRuntimeIdentity, UsbSessionError, UsbTerminalCategory,
};

mod contract;
#[cfg(unix)]
mod private_root;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const PORT: &str = "/dev/cu.usbmodem-private-node";

fn elf() -> String {
    "e".repeat(64)
}

fn physical() -> String {
    "c".repeat(64)
}

fn config(cycles: u32) -> ResetEnduranceConfig {
    ResetEnduranceConfig::new(&physical(), COMMIT, &elf(), cycles, 20).expect("valid config")
}

fn identity_line(commit: &str) -> String {
    format!(
        "usb_runtime_identity schema=v1 firmware_commit={commit} app_elf_sha256={} redacted=true\n",
        elf()
    )
}

fn discriminator(ordinal: u16) -> String {
    format!(
        "usb_reboot_discriminator schema=v1 boot_ordinal={ordinal} reset_reason=other uptime_ms=900 redacted=true\n"
    )
}

fn tasks(section: &str, line: &str) -> String {
    format!(
        "# Tasks\n\n{section}\n\n### task-usb-stuck-link-after-reset | 2026-10-05 | Stop the USB link wedging\n\nStatus: Active.\n{line}\n\n## Accepted Debt and Constraints\n"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fault {
    InspectDrift,
    WrongProfile,
    ResetExit,
    ResetCleanup,
    ReacquireDrift,
    ReacquireLost,
    Silent,
    Stall,
    Mismatch,
    ObserverError,
    CleanupUnproven,
    StaleIdentity,
    RepeatedOrdinal,
    AmbiguousOrdinal,
}

struct Fake {
    cycle: u16,
    clock: u64,
    maybe_fault: Option<(u16, Fault)>,
    calls: Vec<&'static str>,
    recorded: Vec<CycleRow>,
}

impl Fake {
    fn new(maybe_fault: Option<(u16, Fault)>) -> Self {
        Self {
            cycle: 0,
            clock: 0,
            maybe_fault,
            calls: Vec::new(),
            recorded: Vec::new(),
        }
    }

    fn fault(&self, fault: Fault) -> bool {
        self.maybe_fault == Some((self.cycle, fault))
    }
}

fn error(category: UsbTerminalCategory) -> UsbSessionError {
    UsbSessionError {
        category,
        detail: "fixture".to_owned(),
    }
}

impl CycleEffects for Fake {
    fn monotonic_ms(&mut self) -> u64 {
        self.clock += 10;
        self.clock
    }

    fn inspect_profile(&mut self) -> Result<ProfileFacts, UsbSessionError> {
        self.cycle += 1;
        self.calls.push("inspect");
        Ok(ProfileFacts {
            profile: if self.fault(Fault::WrongProfile) {
                UsbProfile::Unknown
            } else {
                UsbProfile::SerialJtagRuntime
            },
            physical_identity_matches: !self.fault(Fault::InspectDrift),
        })
    }

    fn reset_application(&mut self) -> ResetFacts {
        self.calls.push("reset");
        if self.fault(Fault::ResetExit) {
            return ResetFacts {
                termination: UsbCommandTermination::ExitedFailure,
                maybe_error: Some(error(UsbTerminalCategory::FlashFailedBeforeTransfer)),
            };
        }
        if self.fault(Fault::ResetCleanup) {
            return ResetFacts {
                termination: UsbCommandTermination::NotStarted,
                maybe_error: Some(error(UsbTerminalCategory::CleanupFailed)),
            };
        }
        ResetFacts {
            termination: UsbCommandTermination::ExitedSuccess,
            maybe_error: None,
        }
    }

    fn reacquire_transport(&mut self) -> Result<(UsbProfile, bool), UsbSessionError> {
        self.calls.push("reacquire");
        if self.fault(Fault::ReacquireDrift) {
            return Err(error(UsbTerminalCategory::PhysicalIdentityDrift));
        }
        if self.fault(Fault::ReacquireLost) {
            return Err(error(UsbTerminalCategory::RecoveryNotObserved));
        }
        Ok((UsbProfile::SerialJtagRuntime, self.cycle % 2 == 0))
    }

    fn observe(
        &mut self,
        timeout: Duration,
        scanner: &mut IdentityScanner,
    ) -> Result<ObserveFacts, UsbSessionError> {
        self.calls.push("observe");
        assert_eq!(timeout, Duration::from_secs(20));
        if self.fault(Fault::ObserverError) {
            return Err(error(UsbTerminalCategory::MonitorFailed));
        }
        let mut serial = Vec::new();
        let chunks: Vec<String> = if self.fault(Fault::Silent) {
            Vec::new()
        } else if self.fault(Fault::Stall) {
            vec!["I (31) boot: Disabling RNG early entropy source...\n".to_owned()]
        } else if self.fault(Fault::Mismatch) {
            vec![identity_line(&"f".repeat(40))]
        } else if self.fault(Fault::StaleIdentity) {
            vec![identity_line(COMMIT)]
        } else if self.fault(Fault::AmbiguousOrdinal) {
            vec![
                discriminator(self.cycle + 1),
                discriminator(self.cycle + 2),
                identity_line(COMMIT),
            ]
        } else {
            // Cycle N normally reports ordinal N + 1; a repeat reuses cycle N - 1's ordinal.
            let ordinal = if self.fault(Fault::RepeatedOrdinal) {
                self.cycle
            } else {
                self.cycle + 1
            };
            let line = format!("{}{}", discriminator(ordinal), identity_line(COMMIT));
            let (head, tail) = line.split_at(line.len() / 2);
            vec![head.to_owned(), tail.to_owned()]
        };
        for chunk in chunks {
            serial.extend_from_slice(chunk.as_bytes());
            scanner.feed(chunk.as_bytes());
        }
        Ok(ObserveFacts {
            interrupted: false,
            maybe_identity_latency_ms: (scanner.verdict() == IdentityVerdict::Complete)
                .then_some(5),
            serial,
        })
    }

    fn prove_released(&mut self) -> Result<(), UsbSessionError> {
        self.calls.push("prove_released");
        if self.fault(Fault::CleanupUnproven) {
            return Err(error(UsbTerminalCategory::ForeignHolder));
        }
        Ok(())
    }

    fn record_cycle(&mut self, row: &CycleRow, _serial: &[u8]) -> anyhow::Result<()> {
        self.recorded.push(row.clone());
        Ok(())
    }
}

fn run_with(cycles: u32, maybe_fault: Option<(u16, Fault)>) -> (CycleRun, Fake) {
    let mut fake = Fake::new(maybe_fault);
    let run = run_cycles(&config(cycles), &mut fake).expect("fake run");
    (run, fake)
}

#[test]
fn passing_cycles_complete_the_requested_count() {
    // Arrange / Act
    let (run, fake) = run_with(5, None);

    // Assert
    assert_eq!(
        (run.rows.len(), run.maybe_first_failure, fake.recorded.len()),
        (5, None, 5)
    );
    assert!(run.rows.iter().all(|row| row.identity_observed));
}

#[test]
fn loop_stops_at_first_unobserved_application_and_keeps_prior_rows() {
    // Arrange / Act
    let (run, fake) = run_with(10, Some((3, Fault::Silent)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category, failure.bytes_observed),
        (3, ResetEnduranceStop::ApplicationNotObserved, 0)
    );
    assert_eq!(fake.recorded.len(), 3);
    assert!(fake.recorded[..2]
        .iter()
        .all(|row| row.maybe_stop.is_none()));
    assert_eq!(
        fake.calls.iter().filter(|call| **call == "reset").count(),
        3
    );
}

#[test]
fn mid_boot_stall_is_distinguished_from_silence_by_byte_count() {
    // Arrange / Act
    let (run, _fake) = run_with(4, Some((2, Fault::Stall)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(failure.category, ResetEnduranceStop::ApplicationNotObserved);
    assert!(failure.bytes_observed > 0);
}

#[test]
fn each_failed_boundary_maps_to_its_closed_category() {
    for (fault, category) in [
        (
            Fault::InspectDrift,
            ResetEnduranceStop::PhysicalIdentityDrift,
        ),
        (
            Fault::WrongProfile,
            ResetEnduranceStop::RuntimeProfileUnexpected,
        ),
        (Fault::ResetExit, ResetEnduranceStop::ResetFailed),
        (Fault::ResetCleanup, ResetEnduranceStop::CleanupUnproven),
        (
            Fault::ReacquireDrift,
            ResetEnduranceStop::PhysicalIdentityDrift,
        ),
        (
            Fault::ReacquireLost,
            ResetEnduranceStop::TransportNotReacquired,
        ),
        (Fault::Mismatch, ResetEnduranceStop::IdentityMismatch),
        (Fault::ObserverError, ResetEnduranceStop::ObserverFailed),
        (Fault::CleanupUnproven, ResetEnduranceStop::CleanupUnproven),
        (Fault::StaleIdentity, ResetEnduranceStop::RebootNotProven),
        (Fault::RepeatedOrdinal, ResetEnduranceStop::RebootNotProven),
        (
            Fault::AmbiguousOrdinal,
            ResetEnduranceStop::BootOrdinalAmbiguous,
        ),
    ] {
        // Act
        let (run, _fake) = run_with(3, Some((2, fault)));

        // Assert
        let failure = run.maybe_first_failure.expect("failure");
        assert_eq!(
            (failure.cycle, failure.category),
            (2, category),
            "{fault:?}"
        );
    }
}

#[test]
fn increasing_ordinals_across_three_cycles_pass() {
    // Arrange / Act
    let (run, _fake) = run_with(3, None);

    // Assert
    let ordinals: Vec<_> = run.rows.iter().map(|row| row.maybe_boot_ordinal).collect();
    assert_eq!(
        (run.maybe_first_failure, ordinals),
        (None, vec![Some(2), Some(3), Some(4)])
    );
}

#[test]
fn stale_identity_without_discriminator_fails_and_records_bytes() {
    // Arrange / Act
    let (run, _fake) = run_with(3, Some((1, Fault::StaleIdentity)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category, failure.bytes_observed),
        (
            1,
            ResetEnduranceStop::RebootNotProven,
            u64::try_from(identity_line(COMMIT).len()).expect("length")
        )
    );
}

#[test]
fn non_increasing_ordinal_fails_reboot_proof() {
    // Arrange / Act
    let (run, _fake) = run_with(3, Some((2, Fault::RepeatedOrdinal)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (
            failure.cycle,
            failure.category,
            run.rows[1].maybe_boot_ordinal
        ),
        (2, ResetEnduranceStop::RebootNotProven, Some(2))
    );
}

#[test]
fn ambiguous_ordinals_in_one_window_fail() {
    // Arrange / Act
    let (run, _fake) = run_with(3, Some((1, Fault::AmbiguousOrdinal)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category),
        (1, ResetEnduranceStop::BootOrdinalAmbiguous)
    );
}

#[test]
fn identity_drift_stops_before_any_reset() {
    // Arrange / Act
    let (_run, fake) = run_with(3, Some((1, Fault::InspectDrift)));

    // Assert
    assert_eq!(fake.calls, ["inspect"]);
}

#[test]
fn projection_excludes_port_and_physical_identity() {
    // Arrange
    let (run, _fake) = run_with(2, Some((2, Fault::Silent)));
    let cleanup = FinalCleanup {
        proven: true,
        maybe_detail: Some(format!("detail for {PORT}")),
    };

    // Act
    let projection = ResetEnduranceProjection::build(&config(2), &run, &cleanup, 1_000);
    let encoded = serde_json::to_string(&projection).expect("projection JSON");

    // Assert
    assert!(
        !encoded.contains(PORT) && !encoded.contains(&physical()) && !encoded.contains("fixture")
    );
}

#[test]
fn projection_fails_when_final_cleanup_is_unproven() {
    // Arrange
    let (run, _fake) = run_with(2, None);
    let cleanup = FinalCleanup {
        proven: false,
        maybe_detail: None,
    };

    // Act
    let projection = ResetEnduranceProjection::build(&config(2), &run, &cleanup, 1_000);

    // Assert
    assert_eq!(
        (projection.status, projection.cycles_completed),
        ("failed", 2)
    );
}

#[test]
fn projection_summarizes_counts_and_durations() {
    // Arrange
    let (run, _fake) = run_with(4, None);
    let cleanup = FinalCleanup {
        proven: true,
        maybe_detail: None,
    };

    // Act
    let projection = ResetEnduranceProjection::build(&config(4), &run, &cleanup, 1_000);

    // Assert
    assert_eq!(
        (
            projection.passed(),
            projection.counts.reenumerated_cycles,
            projection.counts.reset_reasons.get("other").copied(),
            projection.durations.maybe_max_identity_latency_ms,
        ),
        (true, 2, Some(4), Some(5))
    );
}

#[test]
fn endurance_reset_enters_the_downloader_from_the_running_application_then_hard_resets() {
    // Arrange
    let args = super::session::endurance_reset_args("admitted-port");
    // Act
    let pairs: Vec<_> = args
        .windows(2)
        .map(|pair| (pair[0].as_str(), pair[1].as_str()))
        .collect();
    // Assert: `no-reset-no-sync` waits for a downloader that a running application never provides.
    assert_eq!(args[0], "reset");
    assert!(pairs.contains(&("--before", "usb-reset")));
    assert!(pairs.contains(&("--after", "hard-reset")));
    assert!(!args
        .iter()
        .any(|arg| arg == "no-reset-no-sync" || arg == "write_flash" || arg == "erase_flash"));
}
