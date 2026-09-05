//! Qualification of complete application records from one freshly owned serial capture.
use crate::*;
use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};
use bitaxe_api::panic_receipt::{
    AllocationFailureContextMarker, AllocationFailureMarker, RustPanicMarker,
};
use bitaxe_api::UsbBootTransport;
use bitaxe_device_session::{UsbMemoryCheckpoint, UsbStartupProgress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FixedSerialIssue {
    MissingPackageIdentity,
    IdentityMissing,
    IdentityMismatch,
    MixedIdentity,
    MalformedRecord,
    BaselineUnconfirmed,
    StartupIncomplete,
    StartupFailed,
    RebootObserved,
    NonMonotonicUptime,
    InsufficientAdvancingSamples,
    ErrorDiagnostic,
    UnsupportedTransport,
    LegacyApplicationRecord,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixedSerialAssessment {
    pub(crate) execution_present: bool,
    pub(crate) safe_baseline_confirmed: bool,
    pub(crate) startup_complete: bool,
    pub(crate) startup_failed: bool,
    pub(crate) stable_boot: bool,
    #[serde(default)]
    pub(crate) retained_failure_history: bool,
    pub(crate) issues: Vec<FixedSerialIssue>,
}
impl FixedSerialAssessment {
    pub(crate) fn qualified(&self) -> bool {
        self.execution_present
            && self.safe_baseline_confirmed
            && self.startup_complete
            && !self.startup_failed
            && self.stable_boot
            && self.issues.is_empty()
    }
    fn issue(&mut self, issue: FixedSerialIssue) {
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
    }
    pub(crate) fn conclusion(&self) -> String {
        let execution = if self.execution_present {
            "present"
        } else {
            "missing"
        };
        let startup = if self.startup_failed {
            "failed"
        } else if self.startup_complete {
            "complete"
        } else {
            "incomplete"
        };
        let issues = serde_json::to_string(&self.issues).expect("closed issue labels serialize");
        format!("unqualified - fixed Serial/JTAG execution {execution}; startup {startup}; issues={issues}")
    }
}

#[derive(Default)]
struct Samples {
    assessment: FixedSerialAssessment,
    maybe_identity: Option<UsbRuntimeIdentity>,
    maybe_profile: Option<UsbBootProfileMarker>,
    runtime_identity_matches: bool,
    profile_identity_matches: bool,
    maybe_boot: Option<WorkerUsbBootMarker>,
    maybe_first_boot_ms: Option<u64>,
    boot_advances: usize,
    maybe_startup: Option<UsbStartupProgress>,
    maybe_first_complete_ms: Option<u64>,
    complete_advances: usize,
}

pub(crate) fn assess(
    log: &str,
    maybe_expected: Option<&ExpectedRuntimeAttestationIdentity>,
) -> FixedSerialAssessment {
    let mut samples = Samples::default();
    let Some(expected) = maybe_expected else {
        samples
            .assessment
            .issue(FixedSerialIssue::MissingPackageIdentity);
        return samples.assessment;
    };
    for record in log.split_inclusive('\n') {
        // Captures can start in ROM noise and end inside the next record. Only LF admits a record.
        if !record.ends_with('\n') {
            continue;
        }
        let line = record.trim_end_matches(['\r', '\n']);
        samples.observe(line, expected);
    }
    samples.finish()
}

impl Samples {
    fn observe(&mut self, line: &str, expected: &ExpectedRuntimeAttestationIdentity) {
        if line.len() > 66560
            && (line.starts_with("usb_") || line.starts_with("wifi_startup_failure"))
        {
            self.assessment.issue(FixedSerialIssue::MalformedRecord);
            return;
        }
        if line.starts_with("usb_runtime_identity") {
            match UsbRuntimeIdentity::parse(line) {
                Ok(identity) => {
                    if self
                        .maybe_identity
                        .as_ref()
                        .is_some_and(|prior| prior != &identity)
                    {
                        self.assessment.issue(FixedSerialIssue::MixedIdentity);
                    }
                    let exact = identity.firmware_commit == expected.firmware_commit
                        && identity.app_elf_sha256 == expected.app_elf_sha256;
                    self.runtime_identity_matches |= exact;
                    self.assessment.execution_present |= exact;
                    if !exact {
                        self.assessment.issue(FixedSerialIssue::IdentityMismatch);
                    }
                    self.maybe_identity = Some(identity);
                }
                Err(_) => self.assessment.issue(FixedSerialIssue::MalformedRecord),
            }
        } else if line.starts_with("usb_boot_profile") {
            self.profile(line, expected);
        } else if line.starts_with("usb_reboot_discriminator") {
            self.boot(line);
        } else if line.starts_with("usb_startup") {
            self.startup(line);
        } else if line.starts_with("wifi_startup_failure")
            || line.starts_with("bwg_worker_start_failure")
        {
            self.assessment.startup_failed = true;
            self.assessment.issue(FixedSerialIssue::ErrorDiagnostic);
            if !bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(line) {
                self.assessment.issue(FixedSerialIssue::MalformedRecord);
            }
        } else if line.starts_with("usb_memory_checkpoint") {
            if UsbMemoryCheckpoint::parse(line).is_err() {
                self.assessment.issue(FixedSerialIssue::MalformedRecord);
            }
        } else if line.starts_with("rust_panic_receipt") {
            self.assessment.retained_failure_history = true;
            if RustPanicMarker::parse(line).is_none() {
                self.assessment.issue(FixedSerialIssue::MalformedRecord);
            }
        } else if line.starts_with("allocation_failure_receipt") {
            self.assessment.retained_failure_history = true;
            if AllocationFailureMarker::parse(line).is_none() {
                self.assessment.issue(FixedSerialIssue::MalformedRecord);
            }
        } else if line.starts_with("allocation_failure_context") {
            self.assessment.retained_failure_history = true;
            if AllocationFailureContextMarker::maybe_parse(line).is_none() {
                self.assessment.issue(FixedSerialIssue::MalformedRecord);
            }
        } else if line.starts_with("usb_tx_failure") {
            self.assessment.issue(FixedSerialIssue::ErrorDiagnostic);
        } else if line.starts_with("runtime_boot_attestation")
            || line.starts_with("runtime_boot_identity")
            || line.starts_with("firmware_commit=")
        {
            self.assessment
                .issue(FixedSerialIssue::LegacyApplicationRecord);
        } else if line.starts_with("Guru Meditation Error:")
            || line.starts_with("abort() was called")
            || line.starts_with("assert failed:")
            || matches!(
                line,
                "invalid_frame"
                    | "invalid_request"
                    | "authentication_failed"
                    | "persistence_failed"
                    | "monotonic_reset"
                    | "session_failed"
                    | "restoration_pending"
                    | "encoding_failed"
            )
        {
            self.assessment.issue(FixedSerialIssue::ErrorDiagnostic);
        }
        // Valid panic/allocation receipts remain history, never a claim about this boot.
    }

    fn profile(&mut self, line: &str, expected: &ExpectedRuntimeAttestationIdentity) {
        let Ok(profile) = UsbBootProfileMarker::parse(line) else {
            self.assessment.issue(FixedSerialIssue::MalformedRecord);
            return;
        };
        if self
            .maybe_profile
            .as_ref()
            .is_some_and(|prior| prior != &profile)
        {
            self.assessment.issue(FixedSerialIssue::MixedIdentity);
        }
        let exact = profile.firmware_commit() == expected.firmware_commit
            && profile.app_elf_sha256() == expected.app_elf_sha256;
        self.profile_identity_matches |= exact;
        self.assessment.execution_present |= exact;
        if !exact {
            self.assessment.issue(FixedSerialIssue::IdentityMismatch);
        }
        if profile.transport() != UsbBootTransport::SerialJtagRuntime {
            self.assessment
                .issue(FixedSerialIssue::UnsupportedTransport);
        }
        let wire = serde_json::to_value(&profile).expect("typed profile serializes");
        let safe = exact
            && profile.transport() == UsbBootTransport::SerialJtagRuntime
            && wire["baseline"] == "confirmed"
            && wire["reason"] == "worker_started";
        self.assessment.safe_baseline_confirmed |= safe;
        if !safe {
            self.assessment.issue(FixedSerialIssue::BaselineUnconfirmed);
        }
        if self
            .maybe_boot
            .is_some_and(|boot| boot.boot_ordinal() != profile.boot_ordinal())
        {
            self.assessment.issue(FixedSerialIssue::RebootObserved);
        }
        self.maybe_profile = Some(profile);
    }

    fn boot(&mut self, line: &str) {
        let Some(boot) = WorkerUsbBootMarker::parse(line).filter(|boot| boot.marker() == line)
        else {
            self.assessment.issue(FixedSerialIssue::MalformedRecord);
            return;
        };
        if matches!(
            boot.reset_reason(),
            ResetReasonCategory::Panic
                | ResetReasonCategory::Watchdog
                | ResetReasonCategory::Brownout
        ) {
            self.assessment.issue(FixedSerialIssue::RebootObserved);
        }
        if let Some(prior) = self.maybe_boot {
            if prior.boot_ordinal() != boot.boot_ordinal()
                || prior.reset_reason() != boot.reset_reason()
            {
                self.assessment.issue(FixedSerialIssue::RebootObserved);
            } else if boot.uptime_ms() < prior.uptime_ms() {
                self.assessment.issue(FixedSerialIssue::NonMonotonicUptime);
            } else if boot.uptime_ms() > prior.uptime_ms() {
                self.boot_advances += 1;
            }
        } else {
            self.maybe_first_boot_ms = Some(boot.uptime_ms());
        }
        if self
            .maybe_profile
            .as_ref()
            .is_some_and(|profile| profile.boot_ordinal() != boot.boot_ordinal())
        {
            self.assessment.issue(FixedSerialIssue::RebootObserved);
        }
        self.maybe_boot = Some(boot);
    }

    fn startup(&mut self, line: &str) {
        let Ok(startup) = UsbStartupProgress::parse(line) else {
            self.assessment.issue(FixedSerialIssue::MalformedRecord);
            return;
        };
        let complete = startup.stage == "runtime_ready" && startup.state == "complete";
        self.assessment.startup_complete = complete;
        self.assessment.startup_failed |=
            startup.maybe_first_failure.is_some() || startup.state == "failed";
        if let Some(prior) = &self.maybe_startup {
            if prior.stage == "runtime_ready" && prior.state == "complete" && !complete {
                self.assessment.issue(FixedSerialIssue::StartupIncomplete);
            }
            if startup.uptime_ms < prior.uptime_ms {
                self.assessment.issue(FixedSerialIssue::NonMonotonicUptime);
            } else if complete
                && prior.stage == "runtime_ready"
                && prior.state == "complete"
                && startup.uptime_ms > prior.uptime_ms
            {
                self.complete_advances += 1;
            }
        }
        if complete && self.maybe_first_complete_ms.is_none() {
            self.maybe_first_complete_ms = Some(startup.uptime_ms);
        }
        self.maybe_startup = Some(startup);
    }

    fn finish(mut self) -> FixedSerialAssessment {
        if !self.runtime_identity_matches || !self.profile_identity_matches {
            self.assessment.issue(FixedSerialIssue::IdentityMissing);
        }
        if !self.assessment.safe_baseline_confirmed {
            self.assessment.issue(FixedSerialIssue::BaselineUnconfirmed);
        }
        if self.assessment.startup_failed {
            self.assessment.issue(FixedSerialIssue::StartupFailed);
        }
        let startup_span = self
            .maybe_startup
            .as_ref()
            .zip(self.maybe_first_complete_ms)
            .map_or(0, |(last, first)| last.uptime_ms.saturating_sub(first));
        if !self.assessment.startup_complete || self.complete_advances == 0 || startup_span < 1000 {
            self.assessment.issue(FixedSerialIssue::StartupIncomplete);
        }
        let boot_span = self
            .maybe_boot
            .zip(self.maybe_first_boot_ms)
            .map_or(0, |(last, first)| last.uptime_ms().saturating_sub(first));
        self.assessment.stable_boot = self.boot_advances > 0
            && boot_span >= 1000
            && !self
                .assessment
                .issues
                .contains(&FixedSerialIssue::RebootObserved)
            && !self
                .assessment
                .issues
                .contains(&FixedSerialIssue::NonMonotonicUptime);
        if !self.assessment.stable_boot {
            self.assessment
                .issue(FixedSerialIssue::InsufficientAdvancingSamples);
        }
        self.assessment
    }
}

#[cfg(test)]
mod tests;
