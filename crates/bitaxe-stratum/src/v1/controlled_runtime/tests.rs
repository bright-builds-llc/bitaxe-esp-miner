use bitaxe_asic::bm1366::{
    production::Bm1366ProductionCommand, result::Bm1366NonceResult, work::Bm1366JobId,
};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence, mining_preconditions::ProductionMiningPreconditionDecision,
    power::PowerEvidenceToken, status::SafetyStatus, thermal::ThermalEvidenceToken,
};

use super::*;
use crate::v1::messages::{
    ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse, StratumResponseError,
    StratumV1ClientMessage,
};
use crate::v1::mining_loop::{MiningLoopGate, POWER_PREFLIGHT_EVIDENCE_MISSING};
use crate::v1::state::{MiningActivityStatus, PoolLifecycleStatus, WorkSubmissionGate};

#[test]
fn controlled_runtime_blocked_gate_has_no_side_effect_plan() {
    // Arrange
    let input = ControlledMiningRuntimeInput {
        pool: sample_pool_config(),
        gate: MiningLoopGate::default(),
        transcript: sample_transcript(),
        maybe_nonce_result: None,
        maybe_submit_response: None,
    };

    // Act
    let plan = ControlledMiningRuntimePlan::build(input)
        .expect("blocked gate should still produce evidence");

    // Assert
    assert_eq!(plan.status, ControlledMiningRuntimeStatus::Blocked);
    assert_eq!(plan.block_reason, Some(POWER_PREFLIGHT_EVIDENCE_MISSING));
    assert!(plan.client_messages.is_empty());
    assert!(plan.guarded_plan.maybe_dispatch.is_none());
    assert!(plan.guarded_plan.maybe_submit_intent.is_none());
    assert_eq!(
        plan.guarded_plan.runtime_state.work_submission,
        WorkSubmissionGate::Blocked
    );
    assert_eq!(
        plan.guarded_plan.runtime_state.mining_activity,
        MiningActivityStatus::SafeBlocked
    );
}

#[test]
fn controlled_runtime_redacted_summary_excludes_sensitive_pool_identity() {
    // Arrange
    let input = ControlledMiningRuntimeInput {
        pool: ControlledPoolConfig::parse(
            "stratum+tcp://private.pool.example:3333",
            "private-worker",
            "private-password",
            Some("private-worker-name"),
        )
        .expect("sample pool config should parse"),
        gate: ready_gate(),
        transcript: sample_transcript(),
        maybe_nonce_result: None,
        maybe_submit_response: None,
    };

    // Act
    let plan =
        ControlledMiningRuntimePlan::build(input).expect("ready transcript should produce a plan");
    let summary = plan.evidence.redacted_summary();

    // Assert
    assert!(matches!(
        plan.client_messages.as_slice(),
        [
            StratumV1ClientMessage::Subscribe { .. },
            StratumV1ClientMessage::Authorize { .. },
            ..
        ]
    ));
    for forbidden in [
        "private.pool.example",
        "private-worker",
        "private-password",
        "private-worker-name",
        "DEVICE_URL",
        "192.168.",
        "aa:bb:cc",
        "token",
        "wifi",
        "nvs",
    ] {
        assert!(
            !summary.contains(forbidden),
            "redacted summary leaked {forbidden}: {summary}"
        );
    }
}

#[test]
fn controlled_runtime_redacted_summary_labels_rejected_share_without_reason() {
    // Arrange
    let input = ControlledMiningRuntimeInput {
        pool: ControlledPoolConfig::parse(
            "stratum+tcp://redaction-sentinel.pool.invalid:3333",
            "redaction-sentinel-worker",
            "redaction-sentinel-password",
            Some("redaction-sentinel-worker-name"),
        )
        .expect("sample pool config should parse"),
        gate: ready_gate(),
        transcript: sample_transcript(),
        maybe_nonce_result: Some(sample_nonce_result()),
        maybe_submit_response: Some(StratumResponse {
            maybe_id: Some(ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID),
            success: false,
            maybe_error: Some(StratumResponseError {
                maybe_code: Some(21),
                message: concat!(
                    "rejected redaction-sentinel-worker ",
                    "redaction-sentinel.pool.invalid token DEVICE_URL wifi nvs"
                )
                .to_owned(),
            }),
            maybe_extranonce: None,
            maybe_version_mask: None,
        }),
    };

    // Act
    let plan =
        ControlledMiningRuntimePlan::build(input).expect("rejected share should produce a plan");
    let summary = plan.evidence.redacted_summary();

    // Assert
    assert!(summary.contains("share_outcome: rejected"));
    for forbidden in [
        "redaction-sentinel.pool.invalid",
        "redaction-sentinel-worker",
        concat!("DEVICE", "_URL"),
        "token",
        "wifi",
        "nvs",
    ] {
        assert!(
            !summary.contains(forbidden),
            "redacted summary leaked {forbidden}: {summary}"
        );
    }
}

#[test]
fn controlled_runtime_transcript_enqueues_work_and_emits_typed_bm1366_dispatch() {
    // Arrange
    let input = ControlledMiningRuntimeInput {
        pool: sample_pool_config(),
        gate: ready_gate(),
        transcript: sample_transcript(),
        maybe_nonce_result: None,
        maybe_submit_response: None,
    };

    // Act
    let plan = ControlledMiningRuntimePlan::build(input)
        .expect("ready transcript should produce a dispatch plan");

    // Assert
    assert_eq!(plan.status, ControlledMiningRuntimeStatus::Ready);
    assert_eq!(
        plan.lifecycle_markers,
        vec![
            ControlledRuntimeMarker::Subscribed,
            ControlledRuntimeMarker::Authorized,
            ControlledRuntimeMarker::Active,
        ]
    );
    let dispatch = plan
        .guarded_plan
        .maybe_dispatch
        .expect("active notify should dispatch BM1366 work");
    assert!(matches!(
        dispatch.maybe_production_command,
        Some(Bm1366ProductionCommand::SendProductionWork(payload))
            if payload.job_id() == Bm1366JobId::new(0x28)
    ));
    assert_eq!(
        plan.guarded_plan.runtime_state.lifecycle,
        PoolLifecycleStatus::Active
    );
}

#[test]
fn controlled_runtime_nonce_result_maps_to_share_submit_and_pool_outcomes() {
    // Arrange
    let accepted_input = ControlledMiningRuntimeInput {
        pool: sample_pool_config(),
        gate: ready_gate(),
        transcript: sample_transcript(),
        maybe_nonce_result: Some(sample_nonce_result()),
        maybe_submit_response: Some(StratumResponse {
            maybe_id: Some(ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID),
            success: true,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: None,
        }),
    };
    let rejected_input = ControlledMiningRuntimeInput {
        maybe_submit_response: Some(StratumResponse {
            maybe_id: Some(ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID),
            success: false,
            maybe_error: Some(StratumResponseError {
                maybe_code: Some(21),
                message: "low difficulty".to_owned(),
            }),
            maybe_extranonce: None,
            maybe_version_mask: None,
        }),
        ..accepted_input.clone()
    };

    // Act
    let accepted_plan = ControlledMiningRuntimePlan::build(accepted_input)
        .expect("accepted share path should produce a plan");
    let rejected_plan = ControlledMiningRuntimePlan::build(rejected_input)
        .expect("rejected share path should produce a plan");

    // Assert
    assert!(accepted_plan
        .client_messages
        .iter()
        .any(|message| matches!(message, StratumV1ClientMessage::SubmitShare { .. })));
    assert_eq!(
        accepted_plan.share_outcome,
        Some(ControlledShareOutcome::Accepted)
    );
    assert_eq!(accepted_plan.runtime_state.counters.accepted, 1);
    assert_eq!(
        rejected_plan.share_outcome,
        Some(ControlledShareOutcome::Rejected {
            reason: "low difficulty".to_owned()
        })
    );
    assert_eq!(rejected_plan.runtime_state.counters.rejected, 1);
}

#[test]
fn controlled_runtime_ignores_submit_responses_with_wrong_or_missing_id() {
    // Arrange
    let cases = [
        Some(ControlledMiningRuntimePlan::AUTHORIZE_REQUEST_ID),
        None,
    ];

    // Act / Assert
    for maybe_response_id in cases {
        let input = ControlledMiningRuntimeInput {
            pool: sample_pool_config(),
            gate: ready_gate(),
            transcript: sample_transcript(),
            maybe_nonce_result: Some(sample_nonce_result()),
            maybe_submit_response: Some(StratumResponse {
                maybe_id: maybe_response_id,
                success: true,
                maybe_error: None,
                maybe_extranonce: None,
                maybe_version_mask: None,
            }),
        };
        let plan = ControlledMiningRuntimePlan::build(input)
            .expect("uncorrelated submit response should still produce a plan");

        assert_eq!(
            plan.share_outcome,
            Some(ControlledShareOutcome::NoShareObserved)
        );
        assert_eq!(plan.runtime_state.counters.accepted, 0);
        assert_eq!(plan.runtime_state.counters.rejected, 0);
    }
}

#[test]
fn controlled_runtime_requires_safe_stop_and_watchdog_yields() {
    // Arrange
    let input = ControlledMiningRuntimeInput {
        pool: sample_pool_config(),
        gate: ready_gate(),
        transcript: sample_transcript(),
        maybe_nonce_result: Some(sample_nonce_result()),
        maybe_submit_response: None,
    };

    // Act
    let plan = ControlledMiningRuntimePlan::build(input)
        .expect("bounded no-result path should produce a plan");

    // Assert
    assert!(plan.evidence.safe_stop_required);
    assert_eq!(
        plan.evidence.watchdog_yield_checkpoints,
        vec![
            "subscribe",
            "authorize",
            "notify",
            "dispatch",
            "result",
            "share",
            "safe_stop",
        ]
    );
}

fn sample_pool_config() -> ControlledPoolConfig {
    ControlledPoolConfig::parse(
        "stratum+tcp://example.invalid:3333",
        "synthetic-user",
        "synthetic-password",
        Some("synthetic-worker"),
    )
    .expect("sample pool config should parse")
}

fn sample_transcript() -> ControlledStratumTranscript {
    ControlledStratumTranscript {
        subscribe_response: StratumResponse {
            maybe_id: Some(ControlledMiningRuntimePlan::SUBSCRIBE_REQUEST_ID),
            success: true,
            maybe_error: None,
            maybe_extranonce: Some(ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            }),
            maybe_version_mask: None,
        },
        authorize_response: StratumResponse {
            maybe_id: Some(ControlledMiningRuntimePlan::AUTHORIZE_REQUEST_ID),
            success: true,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: None,
        },
        difficulty: PoolDifficulty { difficulty: 42.0 },
        notify: MiningNotify {
            job_id: "job-40".to_owned(),
            prev_block_hash: "00".repeat(32),
            coinbase_1: "0200000001".to_owned(),
            coinbase_2: "ffffffff".to_owned(),
            merkle_branches: Vec::new(),
            version: 0x2000_0004,
            nbits: 0x1705_ae3a,
            ntime: 0x6470_25b5,
            clean_jobs: false,
        },
    }
}

fn ready_gate() -> MiningLoopGate {
    MiningLoopGate {
        production_preconditions: ProductionMiningPreconditionDecision::Ready,
        asic_initialized: true,
        maybe_power_evidence: Some(PowerEvidenceToken {
            bus_voltage_volts: 5.0,
            current_amps: 2.5,
            power_watts: 12.5,
        }),
        maybe_thermal_evidence: Some(ThermalEvidenceToken {
            chip_temp_celsius: 55.0,
            evidence: SafetyCriticalEvidence::hardware_smoke("phase-21-thermal"),
        }),
        maybe_safety_evidence: Some(SafetyCriticalEvidence::hardware_smoke("phase-21-safety")),
        safety_status: SafetyStatus::Normal,
        hardware_evidence_ack: true,
    }
}

fn sample_nonce_result() -> Bm1366NonceResult {
    Bm1366NonceResult {
        job_id: Bm1366JobId::new(0x28),
        nonce: 0x1234_5678,
        asic_index: 0,
        core_id: 1,
        small_core_id: 0,
        version_bits: 0x0000_2000,
    }
}
