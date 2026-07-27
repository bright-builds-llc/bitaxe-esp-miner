//! Versioned, read-only successor classification for immutable Phase 35 evidence.

use std::collections::BTreeMap;

use camino::Utf8Path;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;

mod authority;
pub mod capture;
mod classification;
mod contract;
pub mod effects;
mod facts;
pub mod runtime_identity;
pub mod substance;

use authority::{authenticate_artifact_graph, Phase36Authority};
pub(crate) use classification::load_and_classify_phase36_root;
#[cfg(test)]
use classification::{
    classify_phase36_envelope, computed_claim_digests, load_and_classify_with_authority,
};
use facts::{
    derive_sufficiency, validate_health_projection, validate_sensor_projection,
    validate_shareable_facts,
};

use crate::phase35_evidence::{
    sha256_hex, validate_phase35_evidence, InventoryArtifact, Phase35EvidenceRootInput,
};
use crate::phase36_evidence::effects::{
    classify_independent_effect_document, IndependentEffectAdmission,
    IndependentEffectObservationSource,
};
use crate::phase36_evidence::runtime_identity::{
    validate_observed_runtime_identity_documents, ObservedRuntimeIdentityAdmission,
};
use crate::phase36_promotion::ValidatedHostnameDurabilityFacts;
use crate::protected_input::{ProtectedFile, ProtectedInputError, ProtectedRoot};

pub use contract::ComponentInsufficiency;
#[cfg(test)]
pub(crate) use contract::Phase36ClaimDigests;
pub(crate) use contract::{
    Attempt31Sufficiency, EffectIntervalState, EffectObservationSource,
    ImmutableArtifactAssessment, Phase36ArtifactRole, Phase36Classification,
    Phase36EvidenceEnvelope, Phase36EvidenceError, PowerSensorFacts, ProvenanceJoinFacts,
    RuntimeHealthCategory, RuntimeHealthFacts, RuntimeIdentityObservationSource, SensorReason,
    SensorTruthState, ShareablePhase36FactsV1, SufficiencyResult, PHASE36_INPUT_DOCUMENT,
    PHASE36_SCHEMA, SHAREABLE_PHASE36_FACTS_SCHEMA,
};
pub use substance::{
    validate_substantive_snapshot_components, validate_substantive_snapshot_documents,
    ObservationState, SubstantiveEvidenceAdmission, SubstantiveSnapshotJoin,
    ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};

fn validate_relative_path(relative_path: &str) -> Result<(), Phase36EvidenceError> {
    if relative_path.is_empty()
        || relative_path.starts_with('/')
        || relative_path.starts_with('\\')
        || relative_path.contains('\\')
        || relative_path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
        || Utf8Path::new(relative_path).is_absolute()
    {
        return Err(Phase36EvidenceError::UnsafeArtifactPath);
    }
    Ok(())
}

#[cfg(test)]
fn digest_serializable(value: &impl Serialize) -> Result<String, Phase36EvidenceError> {
    let bytes = serde_json::to_vec(value).map_err(|_| Phase36EvidenceError::PartialPublicOutput)?;
    Ok(sha256_hex(&bytes))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY: &[(&str, &str)] = &[
    ("phase36_evidence.rs", include_str!("phase36_evidence.rs")),
    (
        "phase36_evidence/classification.rs",
        include_str!("phase36_evidence/classification.rs"),
    ),
    (
        "phase36_evidence/authority.rs",
        include_str!("phase36_evidence/authority.rs"),
    ),
    (
        "phase36_evidence/facts.rs",
        include_str!("phase36_evidence/facts.rs"),
    ),
    (
        "phase36_evidence/substance.rs",
        include_str!("phase36_evidence/substance.rs"),
    ),
    (
        "phase36_evidence/substance/types.rs",
        include_str!("phase36_evidence/substance/types.rs"),
    ),
    (
        "phase36_evidence/runtime_identity.rs",
        include_str!("phase36_evidence/runtime_identity.rs"),
    ),
    (
        "phase36_evidence/runtime_identity/ledger.rs",
        include_str!("phase36_evidence/runtime_identity/ledger.rs"),
    ),
    (
        "phase36_evidence/capture.rs",
        include_str!("phase36_evidence/capture.rs"),
    ),
    (
        "phase36_evidence/capture/filesystem.rs",
        include_str!("phase36_evidence/capture/filesystem.rs"),
    ),
    (
        "phase36_evidence/capture/hardware.rs",
        include_str!("phase36_evidence/capture/hardware.rs"),
    ),
    (
        "phase36_broker/contract.rs",
        include_str!("phase36_broker/contract.rs"),
    ),
    (
        "phase36_broker/ledger.rs",
        include_str!("phase36_broker/ledger.rs"),
    ),
    (
        "phase36_broker/hardware.rs",
        include_str!("phase36_broker/hardware.rs"),
    ),
    (
        "phase36_broker/hardware_process.rs",
        include_str!("phase36_broker/hardware_process.rs"),
    ),
    (
        "phase36_broker/hardware_process/process_boundary.rs",
        include_str!("phase36_broker/hardware_process/process_boundary.rs"),
    ),
    (
        "phase36_broker/hardware_process/effect_result.rs",
        include_str!("phase36_broker/hardware_process/effect_result.rs"),
    ),
    (
        "scripts/phase36-substantive-evidence.sh",
        include_str!("../../../scripts/phase36-substantive-evidence.sh"),
    ),
    (
        "scripts/phase36-hardware-effect.sh",
        include_str!("../../../scripts/phase36-hardware-effect.sh"),
    ),
    (
        "tools/device-session/src/model.rs",
        include_str!("../../device-session/src/model.rs"),
    ),
    (
        "tools/device-session/src/model/state.rs",
        include_str!("../../device-session/src/model/state.rs"),
    ),
    (
        "phase36_evidence/effects.rs",
        include_str!("phase36_evidence/effects.rs"),
    ),
    (
        "operator_snapshot_evidence.rs",
        include_str!("operator_snapshot_evidence.rs"),
    ),
    (
        "crates/bitaxe-api/src/operator_snapshot.rs",
        include_str!("../../../crates/bitaxe-api/src/operator_snapshot.rs"),
    ),
    ("phase35_evidence.rs", include_str!("phase35_evidence.rs")),
    (
        "phase35_evidence/contract.rs",
        include_str!("phase35_evidence/contract.rs"),
    ),
    (
        "phase35_evidence/digests.rs",
        include_str!("phase35_evidence/digests.rs"),
    ),
    (
        "phase35_evidence/inventory.rs",
        include_str!("phase35_evidence/inventory.rs"),
    ),
    (
        "phase35_evidence/projection.rs",
        include_str!("phase35_evidence/projection.rs"),
    ),
    (
        "phase36_promotion/types.rs",
        include_str!("phase36_promotion/types.rs"),
    ),
    ("protected_input.rs", include_str!("protected_input.rs")),
];

fn phase36_evidence_evaluator_digest_from_inventory<I, P, S>(inventory: I) -> String
where
    I: IntoIterator<Item = (P, S)>,
    P: AsRef<str>,
    S: AsRef<str>,
{
    let mut digest_input = b"phase36-evidence-evaluator-v2\0".to_vec();
    for (path, source) in inventory {
        append_length_delimited(&mut digest_input, path.as_ref());
        append_length_delimited(&mut digest_input, source.as_ref());
    }
    sha256_hex(&digest_input)
}

fn append_length_delimited(output: &mut Vec<u8>, value: &str) {
    let length = u64::try_from(value.len()).expect("source inventory entry length fits u64");
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

pub(crate) fn current_phase36_evidence_evaluator_digest() -> String {
    phase36_evidence_evaluator_digest_from_inventory(
        PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
            .iter()
            .map(|(path, source)| (*path, *source)),
    )
}

fn phase36_evidence_contract_digest_for_evaluator(evaluator_digest: &str) -> String {
    sha256_hex(
        [
            "phase36-evidence-contract-v1\0",
            include_str!("phase36_evidence/contract.rs"),
            evaluator_digest,
        ]
        .concat()
        .as_bytes(),
    )
}

pub(crate) fn current_phase36_evidence_contract_digest() -> String {
    phase36_evidence_contract_digest_for_evaluator(&current_phase36_evidence_evaluator_digest())
}

fn map_protected_error(error: ProtectedInputError) -> Phase36EvidenceError {
    match error {
        ProtectedInputError::RootInvalid => Phase36EvidenceError::ProtectedRootInvalid,
        ProtectedInputError::RootSymlink => Phase36EvidenceError::ProtectedRootSymlink,
        ProtectedInputError::UnsafePath => Phase36EvidenceError::UnsafeArtifactPath,
        ProtectedInputError::Missing => Phase36EvidenceError::ProtectedInputMissing,
        ProtectedInputError::Symlink => Phase36EvidenceError::ProtectedInputSymlink,
        ProtectedInputError::WrongPermissions => Phase36EvidenceError::WrongPermissions,
        ProtectedInputError::Changed => Phase36EvidenceError::ProtectedInputChanged,
        ProtectedInputError::NotUtf8 => Phase36EvidenceError::ArtifactInvalid,
    }
}

#[cfg(test)]
pub(crate) mod tests;
