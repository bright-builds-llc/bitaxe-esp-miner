use serde::{Deserialize, Serialize};

pub const PUBLIC_PROJECTION_SCHEMA: &str = "esp-device-session-v1";
pub const PRIVATE_RESULT_SCHEMA: &str = "esp-device-session-private-result-v1";
pub const REQUEST_SCHEMA: &str = "esp-device-session-reboot-request-v1";
const REQUIRED_STABLE_SAMPLES: u8 = 3;
const MAX_DURATION_MILLIS: u64 = 600_000;
const MAX_REPORTED_SERIAL_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformCategory {
    Macos,
    Linux,
    Windows,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DevicePhase {
    Initial,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialPhase {
    PreRestart,
    PostRestart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicalMatch {
    None,
    UniqueSame,
    UniqueDifferent,
    Multiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestOutcome {
    NotAttempted,
    NotTransmitted,
    TransmissionAmbiguous,
    ResponseMissing,
    ResponseReceived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialDelivery {
    Correlated,
    Silent,
    Reacquired,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalCategory {
    Incomplete,
    Ready,
    ObserverUnqualified,
    RestartRequestNotSent,
    RestartAttributionAmbiguous,
    UsbIdentityUnavailable,
    UsbIdentityDrift,
    ServiceRecoveryTimeout,
    BootIdentityInvalid,
    BuildIdentityMismatch,
    SessionNotAdvanced,
    ResetReasonWrong,
    OrdinalNotNext,
    PostconditionMismatch,
}

impl TerminalCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Ready => "ready",
            Self::ObserverUnqualified => "observer_unqualified",
            Self::RestartRequestNotSent => "restart_request_not_sent",
            Self::RestartAttributionAmbiguous => "restart_attribution_ambiguous",
            Self::UsbIdentityUnavailable => "usb_identity_unavailable",
            Self::UsbIdentityDrift => "usb_identity_drift",
            Self::ServiceRecoveryTimeout => "service_recovery_timeout",
            Self::BootIdentityInvalid => "boot_identity_invalid",
            Self::BuildIdentityMismatch => "build_identity_mismatch",
            Self::SessionNotAdvanced => "session_not_advanced",
            Self::ResetReasonWrong => "reset_reason_wrong",
            Self::OrdinalNotNext => "ordinal_not_next",
            Self::PostconditionMismatch => "postcondition_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineApplication {
    pub boot_session: String,
    pub boot_ordinal: u64,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedPostcondition {
    pub hostname_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest {
    pub schema_version: String,
    pub board_category: String,
    pub admitted_port: String,
    pub physical_identity_digest: String,
    pub trusted_origin: String,
    pub baseline: BaselineApplication,
    pub expected_postcondition: ExpectedPostcondition,
}

impl SessionRequest {
    #[must_use]
    pub fn schema_is_valid(&self) -> bool {
        self.schema_version == REQUEST_SCHEMA
            && self.board_category == "205"
            && !self.admitted_port.is_empty()
            && is_sha256(&self.physical_identity_digest)
            && (self.trusted_origin.starts_with("http://")
                || self.trusted_origin.starts_with("https://"))
            && !self.baseline.boot_session.is_empty()
            && is_sha256(&self.baseline.app_elf_sha256)
            && is_sha256(&self.expected_postcondition.hostname_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateBootB {
    pub boot_session: String,
    pub boot_ordinal: u64,
    pub reset_reason_category: String,
    pub trusted_origin: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub hostname_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionEvent {
    PlatformObserved {
        category: PlatformCategory,
    },
    DeviceSample {
        phase: DevicePhase,
        physical_match: PhysicalMatch,
        enumeration_token: String,
        accessible: bool,
        holder_count: u16,
    },
    DeviceAbsent,
    ReaderArmed,
    ReaderLost,
    ReaderReacquired,
    ReaderStartFailed,
    SerialBytes {
        phase: SerialPhase,
        count: u64,
    },
    BaselineConfirmed,
    BaselineMismatch,
    RestartRequestStarted,
    RestartRequestBytesWritten {
        count: u64,
    },
    RestartRequestWriteComplete,
    RestartResponseReceived,
    RestartResponseRejected,
    ServiceLossObserved,
    BootBObserved {
        boot_b: PrivateBootB,
    },
    ObservationWindowExpired {
        duration_millis: u64,
    },
    CleanupComplete,
    CleanupFailed,
    AdmissionRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublicProjection {
    pub schema_version: &'static str,
    pub terminal_category: TerminalCategory,
    pub platform_category: PlatformCategory,
    pub board_category: &'static str,
    pub same_physical_device: bool,
    pub stable_enumeration: bool,
    pub reenumerated: bool,
    pub reader_armed: bool,
    pub pre_restart_serial_delivery: bool,
    pub post_restart_serial_delivery: bool,
    pub serial_delivery: SerialDelivery,
    pub request_outcome: RequestOutcome,
    pub request_attempt_count: u8,
    pub service_loss_observed: bool,
    pub trusted_origin_preserved: bool,
    pub application_recovered: bool,
    pub build_identity_matches: bool,
    pub boot_session_changed: bool,
    pub boot_ordinal_advanced_by_one: bool,
    pub software_reset_observed: bool,
    pub postcondition_matches: bool,
    pub cleanup_complete: bool,
    pub usb_disappearance_count: u16,
    pub enumeration_change_count: u16,
    pub serial_byte_count: u64,
    pub http_observation_count: u16,
    pub duration_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateSessionResult {
    pub schema_version: &'static str,
    pub terminal_category: TerminalCategory,
    pub request_outcome: RequestOutcome,
    pub maybe_secondary_cleanup_failure: bool,
    pub boot_b: Option<PrivateBootB>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    baseline: BaselineApplication,
    expected_postcondition: ExpectedPostcondition,
    trusted_origin: String,
    platform_category: PlatformCategory,
    terminal_category: TerminalCategory,
    maybe_secondary_cleanup_failure: bool,
    maybe_sample_token: Option<String>,
    sample_count: u8,
    initial_device_ready: bool,
    recovery_device_ready: bool,
    same_physical_device: bool,
    maybe_initial_enumeration: Option<String>,
    maybe_current_enumeration: Option<String>,
    reader_armed: bool,
    reader_reacquired: bool,
    baseline_confirmed: bool,
    pre_restart_serial_bytes: u64,
    post_restart_serial_bytes: u64,
    request_attempt_count: u8,
    request_bytes_written: u64,
    request_write_complete: bool,
    restart_response_received: bool,
    service_loss_observed: bool,
    trusted_origin_matches: bool,
    maybe_boot_b: Option<PrivateBootB>,
    application_recovered: bool,
    build_identity_matches: bool,
    boot_session_changed: bool,
    boot_ordinal_advanced_by_one: bool,
    software_reset_observed: bool,
    postcondition_matches: bool,
    cleanup_complete: bool,
    usb_disappearance_count: u16,
    enumeration_change_count: u16,
    http_observation_count: u16,
    duration_millis: u64,
}

impl SessionState {
    #[must_use]
    pub fn new(
        baseline: BaselineApplication,
        expected_postcondition: ExpectedPostcondition,
        trusted_origin: String,
    ) -> Self {
        Self {
            baseline,
            expected_postcondition,
            trusted_origin,
            platform_category: PlatformCategory::Other,
            terminal_category: TerminalCategory::Incomplete,
            maybe_secondary_cleanup_failure: false,
            maybe_sample_token: None,
            sample_count: 0,
            initial_device_ready: false,
            recovery_device_ready: false,
            same_physical_device: false,
            maybe_initial_enumeration: None,
            maybe_current_enumeration: None,
            reader_armed: false,
            reader_reacquired: false,
            baseline_confirmed: false,
            pre_restart_serial_bytes: 0,
            post_restart_serial_bytes: 0,
            request_attempt_count: 0,
            request_bytes_written: 0,
            request_write_complete: false,
            restart_response_received: false,
            service_loss_observed: false,
            trusted_origin_matches: false,
            maybe_boot_b: None,
            application_recovered: false,
            build_identity_matches: false,
            boot_session_changed: false,
            boot_ordinal_advanced_by_one: false,
            software_reset_observed: false,
            postcondition_matches: false,
            cleanup_complete: false,
            usb_disappearance_count: 0,
            enumeration_change_count: 0,
            http_observation_count: 0,
            duration_millis: 0,
        }
    }

    pub fn apply(&mut self, event: SessionEvent) {
        if self.terminal_category != TerminalCategory::Incomplete {
            self.apply_after_terminal(event);
            return;
        }
        match event {
            SessionEvent::PlatformObserved { category } => self.observe_platform(category),
            SessionEvent::DeviceSample {
                phase,
                physical_match,
                enumeration_token,
                accessible,
                holder_count,
            } => self.observe_device(
                phase,
                physical_match,
                enumeration_token,
                accessible,
                holder_count,
            ),
            SessionEvent::DeviceAbsent => self.observe_absence(),
            SessionEvent::ReaderArmed => self.arm_reader(),
            SessionEvent::ReaderLost => self.lose_reader(),
            SessionEvent::ReaderReacquired => self.reacquire_reader(),
            SessionEvent::ReaderStartFailed => self.fail(TerminalCategory::ObserverUnqualified),
            SessionEvent::SerialBytes { phase, count } => self.observe_serial(phase, count),
            SessionEvent::BaselineConfirmed => self.baseline_confirmed = true,
            SessionEvent::BaselineMismatch => self.fail(TerminalCategory::BootIdentityInvalid),
            SessionEvent::RestartRequestStarted => self.start_restart_request(),
            SessionEvent::RestartRequestBytesWritten { count } => {
                if self.request_attempt_count != 1 {
                    self.fail(TerminalCategory::ObserverUnqualified);
                } else {
                    self.request_bytes_written = self.request_bytes_written.saturating_add(count);
                }
            }
            SessionEvent::RestartRequestWriteComplete => {
                if self.request_attempt_count != 1 || self.request_bytes_written == 0 {
                    self.fail(TerminalCategory::ObserverUnqualified);
                } else {
                    self.request_write_complete = true;
                }
            }
            SessionEvent::RestartResponseReceived => {
                if !self.request_write_complete {
                    self.fail(TerminalCategory::ObserverUnqualified);
                } else {
                    self.restart_response_received = true;
                }
            }
            SessionEvent::RestartResponseRejected => {
                self.fail(TerminalCategory::RestartRequestNotSent);
            }
            SessionEvent::ServiceLossObserved => self.service_loss_observed = true,
            SessionEvent::BootBObserved { boot_b } => self.observe_boot_b(boot_b),
            SessionEvent::ObservationWindowExpired { duration_millis } => {
                self.expire(duration_millis)
            }
            SessionEvent::CleanupComplete => self.finish_cleanup(),
            SessionEvent::CleanupFailed => self.fail(TerminalCategory::ObserverUnqualified),
            SessionEvent::AdmissionRejected => self.fail(TerminalCategory::ObserverUnqualified),
        }
    }

    #[must_use]
    pub fn projection(&self) -> PublicProjection {
        PublicProjection {
            schema_version: PUBLIC_PROJECTION_SCHEMA,
            terminal_category: self.terminal_category,
            platform_category: self.platform_category,
            board_category: "205",
            same_physical_device: self.same_physical_device,
            stable_enumeration: self.initial_device_ready || self.recovery_device_ready,
            reenumerated: self.enumeration_change_count > 0,
            reader_armed: self.reader_armed,
            pre_restart_serial_delivery: self.pre_restart_serial_bytes > 0,
            post_restart_serial_delivery: self.post_restart_serial_bytes > 0,
            serial_delivery: self.serial_delivery(),
            request_outcome: self.request_outcome(),
            request_attempt_count: self.request_attempt_count,
            service_loss_observed: self.service_loss_observed,
            trusted_origin_preserved: self.trusted_origin_matches,
            application_recovered: self.application_recovered,
            build_identity_matches: self.build_identity_matches,
            boot_session_changed: self.boot_session_changed,
            boot_ordinal_advanced_by_one: self.boot_ordinal_advanced_by_one,
            software_reset_observed: self.software_reset_observed,
            postcondition_matches: self.postcondition_matches,
            cleanup_complete: self.cleanup_complete,
            usb_disappearance_count: self.usb_disappearance_count,
            enumeration_change_count: self.enumeration_change_count,
            serial_byte_count: self
                .pre_restart_serial_bytes
                .saturating_add(self.post_restart_serial_bytes),
            http_observation_count: self.http_observation_count,
            duration_millis: self.duration_millis,
        }
    }

    #[must_use]
    pub fn private_result(&self) -> PrivateSessionResult {
        PrivateSessionResult {
            schema_version: PRIVATE_RESULT_SCHEMA,
            terminal_category: self.terminal_category,
            request_outcome: self.request_outcome(),
            maybe_secondary_cleanup_failure: self.maybe_secondary_cleanup_failure,
            boot_b: self.maybe_boot_b.clone(),
        }
    }

    #[must_use]
    pub const fn terminal_category(&self) -> TerminalCategory {
        self.terminal_category
    }

    #[must_use]
    pub(crate) const fn device_ready(&self, phase: DevicePhase) -> bool {
        match phase {
            DevicePhase::Initial => self.initial_device_ready,
            DevicePhase::Recovery => self.recovery_device_ready,
        }
    }

    #[must_use]
    pub(crate) fn authoritative_quorum_satisfied(&self) -> bool {
        self.quorum_satisfied()
    }

    fn observe_platform(&mut self, category: PlatformCategory) {
        self.platform_category = category;
        if category != PlatformCategory::Macos {
            self.fail(TerminalCategory::ObserverUnqualified);
        }
    }

    fn observe_device(
        &mut self,
        phase: DevicePhase,
        physical_match: PhysicalMatch,
        enumeration_token: String,
        accessible: bool,
        holder_count: u16,
    ) {
        match physical_match {
            PhysicalMatch::None => {
                self.reset_samples();
                return;
            }
            PhysicalMatch::UniqueDifferent => {
                self.fail(TerminalCategory::UsbIdentityDrift);
                return;
            }
            PhysicalMatch::Multiple => {
                self.fail(TerminalCategory::UsbIdentityDrift);
                return;
            }
            PhysicalMatch::UniqueSame => {}
        }
        if !accessible {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        if holder_count > 0 {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        if self.maybe_sample_token.as_deref() == Some(enumeration_token.as_str()) {
            self.sample_count = self.sample_count.saturating_add(1);
        } else {
            self.maybe_sample_token = Some(enumeration_token.clone());
            self.sample_count = 1;
        }
        if self.sample_count < REQUIRED_STABLE_SAMPLES {
            return;
        }
        self.same_physical_device = true;
        self.maybe_current_enumeration = Some(enumeration_token.clone());
        match phase {
            DevicePhase::Initial => {
                self.initial_device_ready = true;
                self.maybe_initial_enumeration = Some(enumeration_token);
            }
            DevicePhase::Recovery => {
                self.recovery_device_ready = true;
                if self.maybe_initial_enumeration.as_deref() != Some(enumeration_token.as_str()) {
                    self.enumeration_change_count = self.enumeration_change_count.saturating_add(1);
                }
            }
        }
    }

    fn observe_absence(&mut self) {
        self.usb_disappearance_count = self.usb_disappearance_count.saturating_add(1);
        self.recovery_device_ready = false;
        self.same_physical_device = false;
        self.maybe_current_enumeration = None;
        self.reset_samples();
    }

    fn arm_reader(&mut self) {
        if !self.initial_device_ready {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        self.reader_armed = true;
    }

    fn lose_reader(&mut self) {
        if !self.reader_armed {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        self.recovery_device_ready = false;
        self.same_physical_device = false;
        self.reset_samples();
    }

    fn reacquire_reader(&mut self) {
        if !self.recovery_device_ready || !self.same_physical_device {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        self.reader_reacquired = true;
    }

    fn observe_serial(&mut self, phase: SerialPhase, count: u64) {
        if !self.reader_armed {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        match phase {
            SerialPhase::PreRestart => {
                self.pre_restart_serial_bytes = self
                    .pre_restart_serial_bytes
                    .saturating_add(count)
                    .min(MAX_REPORTED_SERIAL_BYTES)
            }
            SerialPhase::PostRestart => {
                self.post_restart_serial_bytes = self
                    .post_restart_serial_bytes
                    .saturating_add(count)
                    .min(MAX_REPORTED_SERIAL_BYTES)
            }
        }
    }

    fn start_restart_request(&mut self) {
        self.request_attempt_count = self.request_attempt_count.saturating_add(1);
        if self.request_attempt_count > 1 {
            self.fail(TerminalCategory::RestartAttributionAmbiguous);
            return;
        }
        if !self.reader_armed || self.pre_restart_serial_bytes == 0 {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        if !self.baseline_confirmed {
            self.fail(TerminalCategory::BootIdentityInvalid);
        }
    }

    fn observe_boot_b(&mut self, boot_b: PrivateBootB) {
        if self.request_attempt_count != 1 {
            self.fail(TerminalCategory::ObserverUnqualified);
            return;
        }
        self.http_observation_count = self.http_observation_count.saturating_add(1);
        self.application_recovered = true;
        self.build_identity_matches = boot_b.source_commit == self.baseline.source_commit
            && boot_b.reference_commit == self.baseline.reference_commit
            && boot_b.app_elf_sha256 == self.baseline.app_elf_sha256;
        self.boot_session_changed = boot_b.boot_session != self.baseline.boot_session;
        self.boot_ordinal_advanced_by_one = self
            .baseline
            .boot_ordinal
            .checked_add(1)
            .is_some_and(|next| boot_b.boot_ordinal == next);
        self.software_reset_observed = boot_b.reset_reason_category == "software_cpu";
        self.postcondition_matches =
            boot_b.hostname_sha256 == self.expected_postcondition.hostname_sha256;
        let trusted_origin_matches = boot_b.trusted_origin == self.trusted_origin;
        self.trusted_origin_matches = trusted_origin_matches;
        self.maybe_boot_b = Some(boot_b);
        if trusted_origin_matches
            && self.same_physical_device
            && self.build_identity_matches
            && self.boot_session_changed
            && self.boot_ordinal_advanced_by_one
            && self.software_reset_observed
            && self.postcondition_matches
        {
            self.application_recovered = true;
        }
    }

    fn expire(&mut self, duration_millis: u64) {
        self.duration_millis = duration_millis.min(MAX_DURATION_MILLIS);
        if !self.reader_armed || self.pre_restart_serial_bytes == 0 {
            self.fail(TerminalCategory::ObserverUnqualified);
        } else if self.request_attempt_count != 1 || self.request_bytes_written == 0 {
            self.fail(TerminalCategory::RestartRequestNotSent);
        } else if !self.request_write_complete
            || (!self.restart_response_received && self.maybe_boot_b.is_none())
        {
            self.fail(TerminalCategory::RestartAttributionAmbiguous);
        } else if !self.initial_device_ready
            || !self.recovery_device_ready
            || !self.same_physical_device
        {
            self.fail(TerminalCategory::UsbIdentityUnavailable);
        } else if !self.application_recovered || self.maybe_boot_b.is_none() {
            self.fail(TerminalCategory::ServiceRecoveryTimeout);
        } else if !self.trusted_origin_matches {
            self.fail(TerminalCategory::BootIdentityInvalid);
        } else if !self.build_identity_matches {
            self.fail(TerminalCategory::BuildIdentityMismatch);
        } else if !self.boot_session_changed {
            self.fail(TerminalCategory::SessionNotAdvanced);
        } else if !self.software_reset_observed {
            self.fail(TerminalCategory::ResetReasonWrong);
        } else if !self.boot_ordinal_advanced_by_one {
            self.fail(TerminalCategory::OrdinalNotNext);
        } else if !self.postcondition_matches {
            self.fail(TerminalCategory::PostconditionMismatch);
        }
    }

    fn finish_cleanup(&mut self) {
        self.cleanup_complete = true;
        if self.quorum_satisfied() {
            self.terminal_category = TerminalCategory::Ready;
        } else {
            self.fail(TerminalCategory::ObserverUnqualified);
        }
    }

    fn quorum_satisfied(&self) -> bool {
        let trusted_origin_matches = self
            .maybe_boot_b
            .as_ref()
            .is_some_and(|boot_b| boot_b.trusted_origin == self.trusted_origin);
        trusted_origin_matches
            && self.request_attempt_count == 1
            && self.request_bytes_written > 0
            && self.request_write_complete
            && self.recovery_device_ready
            && self.same_physical_device
            && self.build_identity_matches
            && self.boot_session_changed
            && self.boot_ordinal_advanced_by_one
            && self.software_reset_observed
            && self.postcondition_matches
    }

    fn request_outcome(&self) -> RequestOutcome {
        if self.request_attempt_count == 0 {
            RequestOutcome::NotAttempted
        } else if self.request_bytes_written == 0 {
            RequestOutcome::NotTransmitted
        } else if !self.request_write_complete {
            RequestOutcome::TransmissionAmbiguous
        } else if !self.restart_response_received {
            RequestOutcome::ResponseMissing
        } else {
            RequestOutcome::ResponseReceived
        }
    }

    fn serial_delivery(&self) -> SerialDelivery {
        if !self.reader_armed {
            SerialDelivery::Failed
        } else if self.post_restart_serial_bytes == 0 {
            SerialDelivery::Silent
        } else if self.reader_reacquired {
            SerialDelivery::Reacquired
        } else {
            SerialDelivery::Correlated
        }
    }

    fn reset_samples(&mut self) {
        self.maybe_sample_token = None;
        self.sample_count = 0;
    }

    fn apply_after_terminal(&mut self, event: SessionEvent) {
        match event {
            SessionEvent::CleanupComplete => self.cleanup_complete = true,
            SessionEvent::CleanupFailed => self.maybe_secondary_cleanup_failure = true,
            _ => {}
        }
    }

    fn fail(&mut self, category: TerminalCategory) {
        if self.terminal_category == TerminalCategory::Incomplete {
            self.terminal_category = category;
        }
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
