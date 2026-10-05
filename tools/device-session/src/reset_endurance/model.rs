//! Pure contract, configuration and evidence shapes for the USB reset endurance loop.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::{UsbCommandTermination, UsbRuntimeIdentity};

/// Redacted projection schema shared outside the private evidence root.
pub const RESET_ENDURANCE_PROJECTION_SCHEMA: &str = "usb-reset-endurance-v1";
/// Private result schema stored beneath the mode-0700 evidence root.
pub const RESET_ENDURANCE_PRIVATE_SCHEMA: &str = "usb-reset-endurance-private-v1";

const TASK: &str = "task-usb-stuck-link-after-reset";
const ENABLE_LINE: &str = "USB reset endurance hardware: enabled.";
const MAX_CYCLES: u32 = 500;
const MIN_OBSERVE_SECONDS: u64 = 5;
const MAX_OBSERVE_SECONDS: u64 = 60;

/// Admits the effect only from one exact enabled line inside the single active task block.
pub fn admit_reset_endurance_task(tasks: &str) -> Result<()> {
    let mut active = false;
    let mut in_task = false;
    let mut headings = 0_usize;
    let mut enabled = false;
    for line in tasks.lines() {
        if line.starts_with("## ") {
            active = line.trim() == "## Active";
            in_task = false;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            in_task = heading.split([' ', '|']).next() == Some(TASK);
            headings += usize::from(in_task);
        }
        if active && in_task && line.trim() == ENABLE_LINE {
            enabled = true;
        }
    }
    if headings != 1 || !enabled {
        bail!("usb_reset_endurance=blocked reason=active_contract_disabled_or_ambiguous");
    }
    Ok(())
}

/// Validated, bounded loop parameters; constructing one performs no device effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetEnduranceConfig {
    expected_physical_sha256: String,
    expected_identity: UsbRuntimeIdentity,
    cycles: u16,
    observe_timeout: Duration,
}

impl ResetEnduranceConfig {
    /// Validates every caller bound before any task, file or device access.
    pub fn new(
        expected_physical_sha256: &str,
        expected_firmware_commit: &str,
        expected_app_elf_sha256: &str,
        cycles: u32,
        observe_timeout_seconds: u64,
    ) -> Result<Self> {
        if !is_lower_hex(expected_physical_sha256, 64) {
            bail!("usb_reset_endurance=invalid reason=expected_physical_sha256");
        }
        let expected_identity =
            UsbRuntimeIdentity::new(expected_firmware_commit, expected_app_elf_sha256)?;
        if !(1..=MAX_CYCLES).contains(&cycles) {
            bail!("usb_reset_endurance=invalid reason=cycles_out_of_bounds");
        }
        if !(MIN_OBSERVE_SECONDS..=MAX_OBSERVE_SECONDS).contains(&observe_timeout_seconds) {
            bail!("usb_reset_endurance=invalid reason=observe_timeout_out_of_bounds");
        }
        Ok(Self {
            expected_physical_sha256: expected_physical_sha256.to_owned(),
            expected_identity,
            cycles: u16::try_from(cycles)?,
            observe_timeout: Duration::from_secs(observe_timeout_seconds),
        })
    }

    #[must_use]
    pub fn expected_physical_sha256(&self) -> &str {
        &self.expected_physical_sha256
    }

    #[must_use]
    pub const fn expected_identity(&self) -> &UsbRuntimeIdentity {
        &self.expected_identity
    }

    #[must_use]
    pub const fn cycles(&self) -> u16 {
        self.cycles
    }

    #[must_use]
    pub const fn observe_timeout(&self) -> Duration {
        self.observe_timeout
    }
}

/// Closed reason the loop stopped; the first one is preserved through cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetEnduranceStop {
    SessionAdmissionFailed,
    PhysicalIdentityDrift,
    RuntimeProfileUnexpected,
    ResetFailed,
    TransportNotReacquired,
    ObserverFailed,
    Interrupted,
    IdentityMismatch,
    BootOrdinalAmbiguous,
    RebootNotProven,
    ApplicationNotObserved,
    CleanupUnproven,
}

impl ResetEnduranceStop {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionAdmissionFailed => "session_admission_failed",
            Self::PhysicalIdentityDrift => "physical_identity_drift",
            Self::RuntimeProfileUnexpected => "runtime_profile_unexpected",
            Self::ResetFailed => "reset_failed",
            Self::TransportNotReacquired => "transport_not_reacquired",
            Self::ObserverFailed => "observer_failed",
            Self::Interrupted => "interrupted",
            Self::IdentityMismatch => "identity_mismatch",
            Self::BootOrdinalAmbiguous => "boot_ordinal_ambiguous",
            Self::RebootNotProven => "reboot_not_proven",
            Self::ApplicationNotObserved => "application_not_observed",
            Self::CleanupUnproven => "cleanup_unproven",
        }
    }
}

/// One private per-cycle row; it never contains ports, USB identity or process data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CycleRow {
    pub cycle: u16,
    pub reset_termination: UsbCommandTermination,
    pub reenumerated: bool,
    pub identity_observed: bool,
    pub bytes_observed: u64,
    #[serde(rename = "reset_reason")]
    pub maybe_reset_reason: Option<&'static str>,
    #[serde(rename = "boot_ordinal")]
    pub maybe_boot_ordinal: Option<u64>,
    #[serde(rename = "identity_latency_ms")]
    pub maybe_identity_latency_ms: Option<u64>,
    pub elapsed_ms: u64,
    #[serde(rename = "stop")]
    pub maybe_stop: Option<ResetEnduranceStop>,
}

impl CycleRow {
    pub(crate) const fn new(cycle: u16) -> Self {
        Self {
            cycle,
            reset_termination: UsbCommandTermination::NotStarted,
            reenumerated: false,
            identity_observed: false,
            bytes_observed: 0,
            maybe_reset_reason: None,
            maybe_boot_ordinal: None,
            maybe_identity_latency_ms: None,
            elapsed_ms: 0,
            maybe_stop: None,
        }
    }
}

/// Earliest stop, with the byte count that separates a stalled stream from silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FirstFailure {
    pub cycle: u16,
    pub category: ResetEnduranceStop,
    pub bytes_observed: u64,
}

/// Complete loop result before final session cleanup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CycleRun {
    pub rows: Vec<CycleRow>,
    pub maybe_first_failure: Option<FirstFailure>,
    pub maybe_failure_detail: Option<String>,
}

impl CycleRun {
    /// Records a failure that happened before cycle 1 could start.
    pub(crate) fn admission_failure(detail: String) -> Self {
        Self {
            rows: Vec::new(),
            maybe_first_failure: Some(FirstFailure {
                cycle: 0,
                category: ResetEnduranceStop::SessionAdmissionFailed,
                bytes_observed: 0,
            }),
            maybe_failure_detail: Some(detail),
        }
    }

    fn cycles_completed(&self) -> u16 {
        let completed = self.rows.iter().filter(|row| row.maybe_stop.is_none());
        u16::try_from(completed.count()).unwrap_or(u16::MAX)
    }
}

/// Final cleanup outcome, recorded separately so it never replaces the first failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalCleanup {
    pub proven: bool,
    pub maybe_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectionCounts {
    pub reenumerated_cycles: u16,
    pub cycles_with_bytes: u16,
    pub reset_reasons: BTreeMap<&'static str, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectionDurations {
    pub total_elapsed_ms: u64,
    pub max_cycle_elapsed_ms: u64,
    #[serde(rename = "max_identity_latency_ms")]
    pub maybe_max_identity_latency_ms: Option<u64>,
}

/// Shareable summary containing only expected identity, closed categories, counts and durations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResetEnduranceProjection {
    pub schema: &'static str,
    pub status: &'static str,
    pub expected_firmware_commit: String,
    pub expected_app_elf_sha256: String,
    pub cycles_requested: u16,
    pub cycles_completed: u16,
    pub first_failure: Option<FirstFailure>,
    pub final_cleanup_proven: bool,
    pub counts: ProjectionCounts,
    pub durations: ProjectionDurations,
}

impl ResetEnduranceProjection {
    #[must_use]
    pub fn build(
        config: &ResetEnduranceConfig,
        run: &CycleRun,
        cleanup: &FinalCleanup,
        total_elapsed_ms: u64,
    ) -> Self {
        let passed = run.maybe_first_failure.is_none() && cleanup.proven;
        Self {
            schema: RESET_ENDURANCE_PROJECTION_SCHEMA,
            status: if passed { "passed" } else { "failed" },
            expected_firmware_commit: config.expected_identity.firmware_commit.clone(),
            expected_app_elf_sha256: config.expected_identity.app_elf_sha256.clone(),
            cycles_requested: config.cycles,
            cycles_completed: run.cycles_completed(),
            first_failure: run.maybe_first_failure,
            final_cleanup_proven: cleanup.proven,
            counts: counts(&run.rows),
            durations: ProjectionDurations {
                total_elapsed_ms,
                max_cycle_elapsed_ms: run.rows.iter().map(|row| row.elapsed_ms).max().unwrap_or(0),
                maybe_max_identity_latency_ms: run
                    .rows
                    .iter()
                    .filter_map(|row| row.maybe_identity_latency_ms)
                    .max(),
            },
        }
    }

    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == "passed"
    }

    /// One redacted console line for the operator.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let failure = self.first_failure.map_or_else(
            || "first_failure=none".to_owned(),
            |failure| {
                format!(
                    "first_failure_cycle={} first_failure_category={} first_failure_bytes={}",
                    failure.cycle,
                    failure.category.as_str(),
                    failure.bytes_observed
                )
            },
        );
        format!(
            "usb_reset_endurance status={} cycles_completed={} cycles_requested={} {failure} final_cleanup_proven={}",
            self.status, self.cycles_completed, self.cycles_requested, self.final_cleanup_proven
        )
    }
}

/// Private result; protected operational fields stay beneath the mode-0700 root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResetEndurancePrivateResult {
    pub schema: &'static str,
    pub expected_physical_sha256: String,
    pub expected_firmware_commit: String,
    pub expected_app_elf_sha256: String,
    pub cycles_requested: u16,
    pub cycles_completed: u16,
    pub first_failure: Option<FirstFailure>,
    #[serde(rename = "first_failure_detail")]
    pub maybe_first_failure_detail: Option<String>,
    pub final_cleanup_proven: bool,
    #[serde(rename = "final_cleanup_detail")]
    pub maybe_final_cleanup_detail: Option<String>,
    pub total_elapsed_ms: u64,
}

impl ResetEndurancePrivateResult {
    #[must_use]
    pub fn build(
        config: &ResetEnduranceConfig,
        run: &CycleRun,
        cleanup: &FinalCleanup,
        total_elapsed_ms: u64,
    ) -> Self {
        Self {
            schema: RESET_ENDURANCE_PRIVATE_SCHEMA,
            expected_physical_sha256: config.expected_physical_sha256.clone(),
            expected_firmware_commit: config.expected_identity.firmware_commit.clone(),
            expected_app_elf_sha256: config.expected_identity.app_elf_sha256.clone(),
            cycles_requested: config.cycles,
            cycles_completed: run.cycles_completed(),
            first_failure: run.maybe_first_failure,
            maybe_first_failure_detail: run.maybe_failure_detail.clone(),
            final_cleanup_proven: cleanup.proven,
            maybe_final_cleanup_detail: cleanup.maybe_detail.clone(),
            total_elapsed_ms,
        }
    }
}

fn counts(rows: &[CycleRow]) -> ProjectionCounts {
    let count = |predicate: fn(&CycleRow) -> bool| {
        u16::try_from(rows.iter().filter(|row| predicate(row)).count()).unwrap_or(u16::MAX)
    };
    let mut reset_reasons = BTreeMap::new();
    for row in rows {
        let label = row.maybe_reset_reason.unwrap_or("missing");
        let entry = reset_reasons.entry(label).or_insert(0_u16);
        *entry = entry.saturating_add(1);
    }
    ProjectionCounts {
        reenumerated_cycles: count(|row| row.reenumerated),
        cycles_with_bytes: count(|row| row.bytes_observed > 0),
        reset_reasons,
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
