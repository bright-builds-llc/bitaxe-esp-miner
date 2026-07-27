//! Explicit-input, read-only classification of immutable Attempt 31 companions.

use std::collections::BTreeSet;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use thiserror::Error;

use crate::operator_evidence::{
    publish_phase36_generation, read_phase36_public_checklist, Phase36GenerationDocuments,
    Phase36PublicationOptions,
};
use crate::phase35_evidence::sha256_hex;
use crate::phase36_evidence::effects::{
    classify_independent_effect_document, ValidatedIndependentEffectInterval,
};
use crate::phase36_evidence::runtime_identity::{
    validate_observed_runtime_identity_documents, ValidatedObservedRuntimeIdentity,
};
use crate::phase36_evidence::{
    validate_substantive_snapshot_components, ComponentInsufficiency, SubstantiveSnapshotJoin,
    ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};
use crate::phase36_promotion::{
    current_phase36_evaluator_digest, evaluate_phase36_promotion, Phase36ChecklistSnapshot,
    Phase36ClaimPrerequisites, ValidatedHostnameDurabilityFacts,
};
use crate::protected_input::{ProtectedFile, ProtectedRoot};
use crate::ReevaluatePhase36Attempt31Args;

const PHASE35_ROOT: &str =
    "docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion";
const PHASE36_ROOT: &str =
    "docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion";
const CHECKLIST: &str = "docs/parity/checklist.md";
const PHASE36_STAGING: &str =
    "docs/parity/evidence/.phase-36-substantive-evidence-admission-and-exact-re-promotion.staging";

#[derive(Debug)]
pub(crate) struct Phase36OfflineRequest {
    workspace_root: Utf8PathBuf,
    maybe_protected_root: Option<Utf8PathBuf>,
    companion_paths: CompanionPaths,
}

impl Phase36OfflineRequest {
    pub(crate) fn from_args(args: &ReevaluatePhase36Attempt31Args) -> Self {
        Self {
            workspace_root: args.workspace_root.clone(),
            maybe_protected_root: args.maybe_protected_root.clone(),
            companion_paths: CompanionPaths {
                maybe_api: args.maybe_api_document.clone(),
                maybe_websocket: args.maybe_websocket_document.clone(),
                maybe_retained: args.maybe_retained_document.clone(),
                maybe_exact_package: args.maybe_exact_package_document.clone(),
                maybe_request: args.maybe_request_document.clone(),
                maybe_event_ledger: args.maybe_event_ledger_document.clone(),
                maybe_private_result: args.maybe_private_result_document.clone(),
                maybe_public_projection: args.maybe_public_projection_document.clone(),
                maybe_independent_effect: args.maybe_independent_effect_document.clone(),
            },
        }
    }
}

#[derive(Debug)]
struct CompanionPaths {
    maybe_api: Option<Utf8PathBuf>,
    maybe_websocket: Option<Utf8PathBuf>,
    maybe_retained: Option<Utf8PathBuf>,
    maybe_exact_package: Option<Utf8PathBuf>,
    maybe_request: Option<Utf8PathBuf>,
    maybe_event_ledger: Option<Utf8PathBuf>,
    maybe_private_result: Option<Utf8PathBuf>,
    maybe_public_projection: Option<Utf8PathBuf>,
    maybe_independent_effect: Option<Utf8PathBuf>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Phase36OfflineOutcome {
    status: &'static str,
    component_insufficiencies: Vec<ComponentInsufficiency>,
    supported_rows: Vec<String>,
    authoritative_generation_digest: String,
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum Phase36OfflineError {
    #[error("offline_public_inputs_invalid")]
    PublicInputsInvalid,
    #[error("offline_companion_invalid")]
    CompanionInvalid,
    #[error("offline_protected_snapshot_changed")]
    ProtectedSnapshotChanged,
    #[error("offline_publication_failed")]
    PublicationFailed,
}

pub(crate) fn reevaluate_attempt31(
    request: &Phase36OfflineRequest,
) -> Result<Phase36OfflineOutcome, Phase36OfflineError> {
    let checklist = read_phase36_public_checklist(
        &request.workspace_root,
        Utf8Path::new(PHASE36_STAGING),
        Utf8Path::new(PHASE36_ROOT),
        Utf8Path::new(CHECKLIST),
        Utf8Path::new(&format!("{PHASE35_ROOT}/.phase35-generation-manifest.json")),
    )
    .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let public = PublicGeneration::load(&request.workspace_root, checklist)?;
    let companions = CompanionSnapshot::load(
        request.maybe_protected_root.as_deref(),
        &request.companion_paths,
    );
    let facts = classify_companions(&companions)?;
    companions.verify_unchanged()?;
    let component_insufficiencies = facts.component_insufficiencies();
    let prerequisites = Phase36ClaimPrerequisites {
        phase35_root_digest: public.hostname.phase35_root_digest.clone(),
        superseded_phase35_generation_digest: public.generation_digest,
        evaluator_digest: current_phase36_evaluator_digest(),
        maybe_hostname: Some(public.hostname),
        maybe_sensors: facts.maybe_sensors,
        maybe_snapshot_join: facts.maybe_join,
        maybe_runtime_health: facts.maybe_runtime_health,
        maybe_runtime_identity: facts.maybe_runtime_identity,
        maybe_independent_effect: facts.maybe_independent_effect,
    };
    let checklist = Phase36ChecklistSnapshot::capture(public.checklist)
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let matrix = evaluate_phase36_promotion(&prerequisites, &checklist)
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let supported_rows = matrix
        .supported_row_ids()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let authoritative_generation_digest = matrix
        .resolver
        .authoritative_phase36_generation_digest
        .clone();
    publish_phase36_generation(
        &request.workspace_root,
        Utf8Path::new(PHASE36_STAGING),
        Utf8Path::new(PHASE36_ROOT),
        Utf8Path::new(CHECKLIST),
        Utf8Path::new(&format!("{PHASE35_ROOT}/.phase35-generation-manifest.json")),
        &Phase36GenerationDocuments::new(prerequisites, matrix),
        Phase36PublicationOptions::default(),
    )
    .map_err(|_| Phase36OfflineError::PublicationFailed)?;
    let status = if component_insufficiencies.is_empty() {
        "immutable_artifacts_sufficient"
    } else {
        "immutable_artifacts_insufficient"
    };
    Ok(Phase36OfflineOutcome {
        status,
        component_insufficiencies,
        supported_rows,
        authoritative_generation_digest,
    })
}

struct PublicGeneration {
    hostname: ValidatedHostnameDurabilityFacts,
    generation_digest: String,
    checklist: String,
}

impl PublicGeneration {
    fn load(workspace: &Utf8Path, checklist: String) -> Result<Self, Phase36OfflineError> {
        let manifest = read_public(
            workspace,
            &format!("{PHASE35_ROOT}/.phase35-generation-manifest.json"),
        )?;
        let projection = read_public(workspace, &format!("{PHASE35_ROOT}/projection.json"))?;
        let matrix = read_public(workspace, &format!("{PHASE35_ROOT}/decision-matrix.json"))?;
        let verdict = read_public(workspace, &format!("{PHASE35_ROOT}/admitted.json"))?;
        let hostname = ValidatedHostnameDurabilityFacts::from_public_generation(
            &manifest,
            &projection,
            &matrix,
            &verdict,
        )
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
        Ok(Self {
            hostname,
            generation_digest: sha256_hex(manifest.as_bytes()),
            checklist,
        })
    }
}

fn read_public(workspace: &Utf8Path, relative: &str) -> Result<String, Phase36OfflineError> {
    fs::read_to_string(workspace.join(relative).as_std_path())
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)
}

#[derive(Default)]
struct ClassifiedFacts {
    maybe_sensors: Option<ValidatedSensorSubstance>,
    maybe_join: Option<SubstantiveSnapshotJoin>,
    maybe_runtime_health: Option<ValidatedRuntimeHealthSubstance>,
    maybe_runtime_identity: Option<ValidatedObservedRuntimeIdentity>,
    maybe_independent_effect: Option<ValidatedIndependentEffectInterval>,
}

impl ClassifiedFacts {
    fn component_insufficiencies(&self) -> Vec<ComponentInsufficiency> {
        let mut categories = BTreeSet::new();
        if self.maybe_sensors.is_none() || self.maybe_join.is_none() {
            categories.insert(ComponentInsufficiency::SnapshotSubstance);
        }
        if self.maybe_runtime_health.is_none() || self.maybe_join.is_none() {
            categories.insert(ComponentInsufficiency::RuntimeHealth);
        }
        if self.maybe_runtime_identity.is_none() {
            categories.insert(ComponentInsufficiency::RuntimeIdentityObservation);
        }
        if self.maybe_independent_effect.is_none() {
            categories.insert(ComponentInsufficiency::IndependentEffectObservation);
        }
        categories.into_iter().collect()
    }
}

fn classify_companions(
    companions: &CompanionSnapshot,
) -> Result<ClassifiedFacts, Phase36OfflineError> {
    if let (Some(api), Some(websocket), Some(retained)) = (
        companions.text(&companions.api),
        companions.text(&companions.websocket),
        companions.text(&companions.retained),
    ) {
        validate_substantive_snapshot_components(api, websocket, retained)
            .map_err(|_| Phase36OfflineError::CompanionInvalid)?;
    }
    if let Some(package) = companions.text(&companions.exact_package) {
        validate_observed_runtime_identity_documents(
            package,
            companions.text(&companions.request),
            companions.text(&companions.event_ledger),
            companions.text(&companions.private_result),
            companions.text(&companions.public_projection),
        )
        .map_err(|_| Phase36OfflineError::CompanionInvalid)?;
    } else if [
        &companions.request,
        &companions.event_ledger,
        &companions.private_result,
        &companions.public_projection,
    ]
    .into_iter()
    .any(Option::is_some)
    {
        return Err(Phase36OfflineError::CompanionInvalid);
    }
    classify_independent_effect_document(companions.text(&companions.independent_effect), None)
        .map_err(|_| Phase36OfflineError::CompanionInvalid)?;

    // The immutable Phase 35 generation does not anchor any of these companion
    // roles or their digests. Caller-supplied documents cannot create evidence
    // authority and therefore remain insufficient.
    Ok(ClassifiedFacts::default())
}

#[derive(Default)]
struct CompanionSnapshot {
    _protected_root: Option<ProtectedRoot>,
    api: Option<ProtectedFile>,
    websocket: Option<ProtectedFile>,
    retained: Option<ProtectedFile>,
    exact_package: Option<ProtectedFile>,
    request: Option<ProtectedFile>,
    event_ledger: Option<ProtectedFile>,
    private_result: Option<ProtectedFile>,
    public_projection: Option<ProtectedFile>,
    independent_effect: Option<ProtectedFile>,
}

impl CompanionSnapshot {
    fn load(maybe_root: Option<&Utf8Path>, paths: &CompanionPaths) -> Self {
        let Some(root) = maybe_root.and_then(|root| ProtectedRoot::open(root).ok()) else {
            return Self::default();
        };
        Self {
            api: snapshot_optional(&root, paths.maybe_api.as_deref()),
            websocket: snapshot_optional(&root, paths.maybe_websocket.as_deref()),
            retained: snapshot_optional(&root, paths.maybe_retained.as_deref()),
            exact_package: snapshot_optional(&root, paths.maybe_exact_package.as_deref()),
            request: snapshot_optional(&root, paths.maybe_request.as_deref()),
            event_ledger: snapshot_optional(&root, paths.maybe_event_ledger.as_deref()),
            private_result: snapshot_optional(&root, paths.maybe_private_result.as_deref()),
            public_projection: snapshot_optional(&root, paths.maybe_public_projection.as_deref()),
            independent_effect: snapshot_optional(&root, paths.maybe_independent_effect.as_deref()),
            _protected_root: Some(root),
        }
    }

    fn text<'a>(&self, maybe_file: &'a Option<ProtectedFile>) -> Option<&'a str> {
        maybe_file.as_ref().and_then(|file| file.text().ok())
    }

    fn verify_unchanged(&self) -> Result<(), Phase36OfflineError> {
        for file in [
            &self.api,
            &self.websocket,
            &self.retained,
            &self.exact_package,
            &self.request,
            &self.event_ledger,
            &self.private_result,
            &self.public_projection,
            &self.independent_effect,
        ]
        .into_iter()
        .flatten()
        {
            file.verify_unchanged()
                .map_err(|_| Phase36OfflineError::ProtectedSnapshotChanged)?;
        }
        Ok(())
    }
}

fn snapshot_optional(
    root: &ProtectedRoot,
    maybe_relative: Option<&Utf8Path>,
) -> Option<ProtectedFile> {
    maybe_relative.and_then(|relative| root.open_file(relative).ok())
}

#[cfg(test)]
mod tests;
