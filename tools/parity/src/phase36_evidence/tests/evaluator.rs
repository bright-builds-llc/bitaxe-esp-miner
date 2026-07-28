use super::*;

#[test]
fn phase36_evaluator_inventory_binds_every_material_owned_validator() {
    // Arrange
    const CLASSIFICATION_DECLARATION: &str = "pub(crate) fn classify_phase36_envelope(";
    const LOADING_DECLARATION: &str = "pub(crate) fn load_and_classify_phase36_root(";
    const HOSTNAME_VALIDATOR_DECLARATION: &str = "pub(crate) fn from_public_generation(";
    const SESSION_REQUEST_VALIDATOR_DECLARATION: &str = "pub fn schema_is_valid(&self) -> bool {";
    const SESSION_STATE_VALIDATOR_DECLARATION: &str =
        "pub fn apply(&mut self, event: SessionEvent) {";
    const HARDWARE_TRANSACTION_DECLARATION: &str =
        "pub(super) fn run_phase36_hardware_transaction_with(";
    let expected_sources = [
        "phase36_evidence.rs",
        "phase36_evidence/classification.rs",
        "phase36_evidence/loading.rs",
        "phase36_evidence/authority.rs",
        "phase36_evidence/facts.rs",
        "phase36_evidence/substance.rs",
        "phase36_evidence/substance/types.rs",
        "phase36_evidence/runtime_identity.rs",
        "phase36_evidence/runtime_identity/ledger.rs",
        "phase36_evidence/capture.rs",
        "phase36_evidence/capture/filesystem.rs",
        "phase36_evidence/capture/hardware.rs",
        "phase36_broker/contract.rs",
        "phase36_broker/ledger.rs",
        "phase36_broker/hardware.rs",
        "phase36_broker/hardware_process.rs",
        "phase36_broker/hardware_process/process_boundary.rs",
        "phase36_broker/hardware_process/effect_result.rs",
        "scripts/phase36-substantive-evidence.sh",
        "scripts/phase36-hardware-effect.sh",
        "tools/device-session/src/model.rs",
        "tools/device-session/src/model/state.rs",
        "phase36_evidence/effects.rs",
        "operator_snapshot_evidence.rs",
        "crates/bitaxe-api/src/operator_snapshot.rs",
        "phase35_evidence.rs",
        "phase35_evidence/contract.rs",
        "phase35_evidence/digests.rs",
        "phase35_evidence/inventory.rs",
        "phase35_evidence/projection.rs",
        "phase36_promotion/types.rs",
        "protected_input.rs",
    ];
    let inventory_sources = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .map(|(path, _)| *path)
        .collect::<Vec<_>>();
    let hostname_validator_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_promotion/types.rs")
        .map(|(_, source)| *source)
        .expect("hostname generation validator source should be inventoried");
    let classification_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_evidence/classification.rs")
        .map(|(_, source)| *source)
        .expect("Phase 36 classification source should be inventoried");
    let loading_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_evidence/loading.rs")
        .map(|(_, source)| *source)
        .expect("Phase 36 loading adapter source should be inventoried");
    let session_validator_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "tools/device-session/src/model.rs")
        .map(|(_, source)| *source)
        .expect("device-session replay validator source should be inventoried");
    let session_state_validator_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "tools/device-session/src/model/state.rs")
        .map(|(_, source)| *source)
        .expect("device-session state validator source should be inventoried");
    let hardware_transaction_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_broker/hardware.rs")
        .map(|(_, source)| *source)
        .expect("hardware transaction source should be inventoried");
    let drift_identity = |target_path: &str, declaration: &str, replacement: &str| {
        let drifted_inventory =
            PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
                .iter()
                .map(|(path, source)| {
                    let source = if *path == target_path {
                        std::borrow::Cow::Owned(source.replacen(declaration, replacement, 1))
                    } else {
                        std::borrow::Cow::Borrowed(*source)
                    };
                    (*path, source)
                });
        let evaluator = phase36_evidence_evaluator_digest_from_inventory(drifted_inventory);
        let contract = phase36_evidence_contract_digest_for_evaluator(&evaluator);
        (evaluator, contract)
    };

    // Act
    let hostname_drift = drift_identity(
        "phase36_promotion/types.rs",
        HOSTNAME_VALIDATOR_DECLARATION,
        "pub(crate) fn from_public_generation_drift(",
    );
    let classification_drift = drift_identity(
        "phase36_evidence/classification.rs",
        CLASSIFICATION_DECLARATION,
        "pub(crate) fn classify_phase36_envelope_drift(",
    );
    let loading_drift = drift_identity(
        "phase36_evidence/loading.rs",
        LOADING_DECLARATION,
        "pub(crate) fn load_and_classify_phase36_root_drift(",
    );
    let session_state_drift = drift_identity(
        "tools/device-session/src/model/state.rs",
        SESSION_STATE_VALIDATOR_DECLARATION,
        "pub fn apply_drift(&mut self, event: SessionEvent) {",
    );
    let hardware_transaction_drift = drift_identity(
        "phase36_broker/hardware.rs",
        HARDWARE_TRANSACTION_DECLARATION,
        "pub(super) fn run_phase36_hardware_transaction_with_drift(",
    );
    let path_drifted_inventory =
        PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
            .iter()
            .map(|(path, source)| {
                let path = if *path == "tools/device-session/src/model/state.rs" {
                    "tools/device-session/src/model/state-drift.rs"
                } else {
                    *path
                };
                (path, *source)
            });
    let path_drifted_evaluator =
        phase36_evidence_evaluator_digest_from_inventory(path_drifted_inventory);
    let path_drift = (
        path_drifted_evaluator.clone(),
        phase36_evidence_contract_digest_for_evaluator(&path_drifted_evaluator),
    );

    // Assert
    assert_eq!(inventory_sources, expected_sources);
    assert!(classification_source.contains(CLASSIFICATION_DECLARATION));
    for prohibited in [
        "ProtectedRoot",
        "ProtectedFile",
        "open_file(",
        "authenticate_artifact_graph",
        "verify_unchanged",
    ] {
        assert!(
            !classification_source.contains(prohibited),
            "pure Phase 36 classifier contains effectful loading token {prohibited}"
        );
    }
    for required in [
        LOADING_DECLARATION,
        "ProtectedRoot::open",
        "open_file(",
        "authenticate_artifact_graph",
        "verify_unchanged",
        "classify_phase36_envelope",
    ] {
        assert!(
            loading_source.contains(required),
            "Phase 36 loading adapter is missing {required}"
        );
    }
    assert!(hostname_validator_source.contains(HOSTNAME_VALIDATOR_DECLARATION));
    assert!(session_validator_source.contains(SESSION_REQUEST_VALIDATOR_DECLARATION));
    assert!(session_state_validator_source.contains(SESSION_STATE_VALIDATOR_DECLARATION));
    assert!(hardware_transaction_source.contains(HARDWARE_TRANSACTION_DECLARATION));
    for (drifted_evaluator, drifted_contract) in [
        classification_drift,
        loading_drift,
        hostname_drift,
        session_state_drift,
        hardware_transaction_drift,
        path_drift,
    ] {
        assert_ne!(
            drifted_evaluator,
            current_phase36_evidence_evaluator_digest()
        );
        assert_ne!(drifted_contract, current_phase36_evidence_contract_digest());
    }
}
