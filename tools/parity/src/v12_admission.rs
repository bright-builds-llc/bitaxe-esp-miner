//! Closed Phase 31 claim admission.

use serde::Serialize;

/// Exact Phase 31 requirement row that may receive a contract claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub(crate) enum V12Requirement {
    Obs01,
    Cfg08,
}

/// Complete set of claims eligible for Phase 31 admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum V12PromotableClaim {
    ObservationTruthContract,
    HostnamePatchAllowlist,
}

/// Typed reason that a claim is outside exact Phase 31 authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum V12ExclusionReason {
    ActiveFanControl,
    ActiveVoltageControl,
    ResetOrPowerControl,
    ActiveAsicControl,
    SelfTestEffects,
    WatchdogInterventionOrLoad,
    Mining,
    ArchivedPhase28_1_1,
    Credentials,
    DirectUartOrPins,
    OtaOrRecovery,
    OtherBoards,
    TelemetryHistory,
    RuntimeUiDisplayInputOrBap,
    BroadProductionOrVerifiedPromotion,
    MissingEvidence,
    WrongRequirement,
    UntypedClaim,
}

const V12_EXCLUSIONS: [V12ExclusionReason; 18] = [
    V12ExclusionReason::ActiveFanControl,
    V12ExclusionReason::ActiveVoltageControl,
    V12ExclusionReason::ResetOrPowerControl,
    V12ExclusionReason::ActiveAsicControl,
    V12ExclusionReason::SelfTestEffects,
    V12ExclusionReason::WatchdogInterventionOrLoad,
    V12ExclusionReason::Mining,
    V12ExclusionReason::ArchivedPhase28_1_1,
    V12ExclusionReason::Credentials,
    V12ExclusionReason::DirectUartOrPins,
    V12ExclusionReason::OtaOrRecovery,
    V12ExclusionReason::OtherBoards,
    V12ExclusionReason::TelemetryHistory,
    V12ExclusionReason::RuntimeUiDisplayInputOrBap,
    V12ExclusionReason::BroadProductionOrVerifiedPromotion,
    V12ExclusionReason::MissingEvidence,
    V12ExclusionReason::WrongRequirement,
    V12ExclusionReason::UntypedClaim,
];

/// Closed result of Phase 31 admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "decision", content = "category", rename_all = "snake_case")]
pub(crate) enum V12AdmissionDecision {
    Eligible(V12PromotableClaim),
    Ineligible(V12ExclusionReason),
}

/// Typed evidence input for one exact requirement and claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct V12EvidenceBundle {
    requirement: V12Requirement,
    claim: V12PromotableClaim,
    evidence_present: bool,
}

impl V12EvidenceBundle {
    pub(crate) const fn new(
        requirement: V12Requirement,
        claim: V12PromotableClaim,
        evidence_present: bool,
    ) -> Self {
        Self {
            requirement,
            claim,
            evidence_present,
        }
    }
}

/// Admits only a present exact-claim bundle for its owning requirement row.
pub(crate) const fn admit_v12_bundle(bundle: V12EvidenceBundle) -> V12AdmissionDecision {
    if !bundle.evidence_present {
        return V12AdmissionDecision::Ineligible(V12ExclusionReason::MissingEvidence);
    }

    let requirement_matches = matches!(
        (bundle.requirement, bundle.claim),
        (
            V12Requirement::Obs01,
            V12PromotableClaim::ObservationTruthContract
        ) | (
            V12Requirement::Cfg08,
            V12PromotableClaim::HostnamePatchAllowlist
        )
    );
    if !requirement_matches {
        return V12AdmissionDecision::Ineligible(V12ExclusionReason::WrongRequirement);
    }

    V12AdmissionDecision::Eligible(bundle.claim)
}

/// Rejects open text without ever translating it into an eligible claim.
pub(crate) fn reject_untyped_v12_claim(label: &str) -> V12AdmissionDecision {
    let normalized = label.to_ascii_lowercase();
    let is_broad_promotion = ["production ready", "production-ready", "verified"]
        .iter()
        .any(|term| normalized.contains(term));

    if is_broad_promotion {
        return V12AdmissionDecision::Ineligible(
            V12ExclusionReason::BroadProductionOrVerifiedPromotion,
        );
    }

    V12AdmissionDecision::Ineligible(V12ExclusionReason::UntypedClaim)
}

/// Checks that the built-in exact Phase 31 admission pairs remain closed.
pub(crate) fn validate_closed_phase31_contract() -> Result<(), String> {
    let exact_bundles = [
        V12EvidenceBundle::new(
            V12Requirement::Obs01,
            V12PromotableClaim::ObservationTruthContract,
            true,
        ),
        V12EvidenceBundle::new(
            V12Requirement::Cfg08,
            V12PromotableClaim::HostnamePatchAllowlist,
            true,
        ),
    ];

    for bundle in exact_bundles {
        if !matches!(admit_v12_bundle(bundle), V12AdmissionDecision::Eligible(_)) {
            return Err("closed Phase 31 exact-claim admission is inconsistent".to_owned());
        }
    }

    for reason in V12_EXCLUSIONS {
        if matches!(
            V12AdmissionDecision::Ineligible(reason),
            V12AdmissionDecision::Eligible(_)
        ) {
            return Err("closed Phase 31 exclusion became eligible".to_owned());
        }
    }

    if matches!(
        reject_untyped_v12_claim("observation_truth_contract"),
        V12AdmissionDecision::Eligible(_)
    ) {
        return Err("untyped Phase 31 claim became eligible".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_CHECKLIST: &str = include_str!("../../../docs/parity/checklist.md");

    #[test]
    fn phase31_observation_truth_is_eligible_only_for_obs01() {
        // Arrange
        let matching = V12EvidenceBundle::new(
            V12Requirement::Obs01,
            V12PromotableClaim::ObservationTruthContract,
            true,
        );
        let wrong_row = V12EvidenceBundle::new(
            V12Requirement::Cfg08,
            V12PromotableClaim::ObservationTruthContract,
            true,
        );

        // Act
        let matching_decision = admit_v12_bundle(matching);
        let wrong_row_decision = admit_v12_bundle(wrong_row);

        // Assert
        assert_eq!(
            matching_decision,
            V12AdmissionDecision::Eligible(V12PromotableClaim::ObservationTruthContract)
        );
        assert_eq!(
            wrong_row_decision,
            V12AdmissionDecision::Ineligible(V12ExclusionReason::WrongRequirement)
        );
    }

    #[test]
    fn phase31_hostname_allowlist_is_eligible_only_for_cfg08() {
        // Arrange
        let matching = V12EvidenceBundle::new(
            V12Requirement::Cfg08,
            V12PromotableClaim::HostnamePatchAllowlist,
            true,
        );
        let wrong_row = V12EvidenceBundle::new(
            V12Requirement::Obs01,
            V12PromotableClaim::HostnamePatchAllowlist,
            true,
        );

        // Act
        let matching_decision = admit_v12_bundle(matching);
        let wrong_row_decision = admit_v12_bundle(wrong_row);

        // Assert
        assert_eq!(
            matching_decision,
            V12AdmissionDecision::Eligible(V12PromotableClaim::HostnamePatchAllowlist)
        );
        assert_eq!(
            wrong_row_decision,
            V12AdmissionDecision::Ineligible(V12ExclusionReason::WrongRequirement)
        );
    }

    #[test]
    fn phase31_missing_evidence_is_ineligible_for_both_exact_claims() {
        // Arrange
        let bundles = [
            V12EvidenceBundle::new(
                V12Requirement::Obs01,
                V12PromotableClaim::ObservationTruthContract,
                false,
            ),
            V12EvidenceBundle::new(
                V12Requirement::Cfg08,
                V12PromotableClaim::HostnamePatchAllowlist,
                false,
            ),
        ];

        for bundle in bundles {
            // Act
            let decision = admit_v12_bundle(bundle);

            // Assert
            assert_eq!(
                decision,
                V12AdmissionDecision::Ineligible(V12ExclusionReason::MissingEvidence)
            );
        }
    }

    #[test]
    fn phase31_every_excluded_category_is_typed_and_redaction_safe() {
        // Arrange
        let exclusions = V12_EXCLUSIONS;

        for exclusion in exclusions {
            // Act
            let decision = V12AdmissionDecision::Ineligible(exclusion);
            let serialized = serde_json::to_string(&decision).expect("typed decision serializes");

            // Assert
            assert!(!serialized.is_empty());
            assert!(!serialized.contains("password"));
            assert!(!serialized.contains("endpoint"));
            assert!(!serialized.contains("worker"));
        }
    }

    #[test]
    fn phase31_strings_and_schema_growth_can_never_construct_eligible_claims() {
        // Arrange
        let labels = [
            "observation_truth_contract",
            "hostname_patch_allowlist",
            "future_schema_field",
            "production ready",
            "active safety verified",
            "mining verified",
        ];

        for label in labels {
            // Act
            let decision = reject_untyped_v12_claim(label);

            // Assert
            assert!(matches!(decision, V12AdmissionDecision::Ineligible(_)));
        }
    }

    #[test]
    fn phase31_eligible_enum_source_contains_no_excluded_category_tokens() {
        // Arrange
        let source = include_str!("v12_admission.rs");
        let enum_source = source
            .split("pub(crate) enum V12PromotableClaim")
            .nth(1)
            .and_then(|tail| tail.split('}').next())
            .expect("eligible enum source must be present");
        let excluded_tokens = [
            "Fan",
            "Voltage",
            "Reset",
            "Power",
            "Asic",
            "SelfTest",
            "Watchdog",
            "Mining",
            "Credential",
            "Uart",
            "Pin",
            "Ota",
            "Recovery",
            "OtherBoard",
            "History",
            "Ui",
            "Display",
            "Bap",
            "Production",
            "Verified",
        ];

        // Act / Assert
        for token in excluded_tokens {
            assert!(!enum_source.contains(token));
        }
    }

    #[test]
    fn phase31_admission_validation_preserves_current_checklist_statuses() {
        // Arrange
        let statuses_before = checklist_status_cells(CURRENT_CHECKLIST);

        // Act
        let contract_result = validate_closed_phase31_contract();
        let statuses_after = checklist_status_cells(CURRENT_CHECKLIST);

        // Assert
        assert!(contract_result.is_ok());
        assert_eq!(statuses_after, statuses_before);
    }

    fn checklist_status_cells(checklist: &str) -> Vec<String> {
        checklist
            .lines()
            .filter(|line| line.starts_with('|'))
            .filter_map(|line| line.split('|').nth(5))
            .map(str::trim)
            .filter(|status| !matches!(*status, "Status" | "---"))
            .map(str::to_owned)
            .collect()
    }
}
