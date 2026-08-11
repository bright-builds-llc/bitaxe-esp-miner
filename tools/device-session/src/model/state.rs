use super::*;

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
        let expected_app_elf_sha256 = self
            .expected_postcondition
            .app_elf_sha256
            .as_ref()
            .unwrap_or(&self.baseline.app_elf_sha256);
        self.build_identity_matches = boot_b.source_commit == self.baseline.source_commit
            && boot_b.reference_commit == self.baseline.reference_commit
            && &boot_b.app_elf_sha256 == expected_app_elf_sha256;
        self.boot_session_changed = boot_b.boot_session != self.baseline.boot_session;
        self.boot_ordinal_advanced_by_one = self
            .baseline
            .boot_ordinal
            .checked_add(1)
            .is_some_and(|next| boot_b.boot_ordinal == next);
        self.software_reset_observed = boot_b.reset_reason_category == "software_cpu";
        self.postcondition_matches = boot_b.hostname_sha256
            == self.expected_postcondition.hostname_sha256
            && self
                .expected_postcondition
                .running_partition
                .as_ref()
                .is_none_or(|expected| &boot_b.running_partition == expected);
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
