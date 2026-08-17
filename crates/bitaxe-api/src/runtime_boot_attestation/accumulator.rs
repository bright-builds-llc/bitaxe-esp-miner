use super::{
    ExpectedRuntimeAttestationIdentity, ResetReasonCategory, RuntimeAttestationParseFailure,
    RuntimeAttestationParseFailureCounts, RuntimeAttestationStatus, RuntimeBootAttestation,
};

/// Bounded, incremental classification state for a stream of attestation markers.
#[derive(Debug, Default)]
pub struct RuntimeAttestationAccumulator {
    sample_count: u64,
    maybe_first: Option<RuntimeBootAttestation>,
    maybe_previous_uptime_ms: Option<u64>,
    maybe_parse_failure: Option<RuntimeAttestationStatus>,
    maybe_first_parse_failure: Option<RuntimeAttestationParseFailure>,
    parse_failure_counts: RuntimeAttestationParseFailureCounts,
    mixed_session_or_ordinal: bool,
    maybe_mixed_reset_reason: Option<ResetReasonCategory>,
    static_fields_mismatch: bool,
    non_monotonic_uptime: bool,
}

impl RuntimeAttestationAccumulator {
    /// Observes one complete marker candidate without retaining its source text.
    pub fn observe_line(&mut self, line: &str) {
        let sample = match RuntimeBootAttestation::parse(line) {
            Ok(sample) => sample,
            Err(error) => {
                let failure = RuntimeAttestationParseFailure::from(error);
                self.maybe_first_parse_failure.get_or_insert(failure);
                self.parse_failure_counts.record(failure);
                self.maybe_parse_failure.get_or_insert(
                    if failure == RuntimeAttestationParseFailure::IncompleteReadiness {
                        RuntimeAttestationStatus::IncompleteReadiness
                    } else {
                        RuntimeAttestationStatus::Malformed
                    },
                );
                return;
            }
        };

        self.sample_count = self.sample_count.saturating_add(1);
        if let Some(first) = self.maybe_first.as_ref() {
            if !sample.same_session_and_ordinal(first) {
                self.mixed_session_or_ordinal = true;
                self.maybe_mixed_reset_reason
                    .get_or_insert(sample.reset_reason);
            }
            self.static_fields_mismatch |= !sample.same_static_fields(first);
        } else {
            self.maybe_first = Some(sample.clone());
        }
        if let Some(previous_uptime_ms) = self.maybe_previous_uptime_ms {
            self.non_monotonic_uptime |= sample.uptime_ms <= previous_uptime_ms;
        }
        self.maybe_previous_uptime_ms = Some(sample.uptime_ms);
    }

    /// Returns the first closed parse-failure category in observation order.
    #[must_use]
    pub const fn maybe_first_parse_failure(&self) -> Option<RuntimeAttestationParseFailure> {
        self.maybe_first_parse_failure
    }

    /// Returns saturating closed parse-failure counts without retained input.
    #[must_use]
    pub const fn parse_failure_counts(&self) -> RuntimeAttestationParseFailureCounts {
        self.parse_failure_counts
    }

    /// Returns the reset category on the first mixed-session transition.
    #[must_use]
    pub const fn mixed_reset_reason_label(&self) -> &'static str {
        match self.maybe_mixed_reset_reason {
            Some(reason) => reason.label(),
            None => "none",
        }
    }

    /// Classifies every complete marker observed so far against one admitted package.
    #[must_use]
    pub fn status(
        &self,
        expected: &ExpectedRuntimeAttestationIdentity,
    ) -> RuntimeAttestationStatus {
        if let Some(status) = self.maybe_parse_failure {
            return status;
        }
        let Some(first) = self.maybe_first.as_ref() else {
            return RuntimeAttestationStatus::Missing;
        };
        if self.sample_count < 2 {
            return RuntimeAttestationStatus::InsufficientSamples;
        }
        if self.mixed_session_or_ordinal {
            return RuntimeAttestationStatus::MixedSessionOrOrdinal;
        }
        if self.static_fields_mismatch {
            return RuntimeAttestationStatus::StaticFieldsMismatch;
        }
        if self.non_monotonic_uptime {
            return RuntimeAttestationStatus::NonMonotonicUptime;
        }
        if !first.matches_expected(expected) {
            return RuntimeAttestationStatus::PackageIdentityMismatch;
        }
        RuntimeAttestationStatus::Trusted
    }
}
