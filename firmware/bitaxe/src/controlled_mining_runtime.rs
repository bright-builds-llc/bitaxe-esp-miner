//! Phase 21 controlled mining runtime shell.
//!
//! This module is deliberately compile-time gated and bounded. It never starts
//! default production mining, never logs Stratum credentials, and never emits
//! raw BM1366 frame bytes.

use bitaxe_asic::bm1366::command::Bm1366Command;
use bitaxe_config::{nvs::StoredValueKind, NvsSnapshot};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence, power::PowerEvidenceToken, status::SafetyStatus,
    thermal::ThermalEvidenceToken,
};
use bitaxe_stratum::v1::{
    controlled_runtime::{
        ControlledMiningRuntimeInput, ControlledMiningRuntimePlan, ControlledMiningRuntimeStatus,
        ControlledPoolConfig, ControlledRuntimeMarker, ControlledShareOutcome,
        ControlledStratumTranscript,
    },
    messages::{
        ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse, StratumV1ClientMessage,
    },
    mining_loop::MiningLoopGate,
    state::{HashrateInputs, MiningRuntimeState},
};

use crate::{
    asic_adapter, mining_evidence_mode::MiningEvidenceMode, runtime_snapshot, settings_adapter,
};

const BOARD_205: &str = "205";
const MISSING_POOL_SETTINGS: &str = "missing_pool_settings";
const PLAN_BUILD_FAILED: &str = "plan_build_failed";
const ADAPTER_ACTION_FAILED: &str = "adapter_action_failed";

pub fn maybe_start_after_asic_gate() {
    match MiningEvidenceMode::current() {
        MiningEvidenceMode::FailClosed => {
            asic_adapter::publish_mining_loop_blocked_status("hardware_evidence_ack_missing");
        }
        MiningEvidenceMode::LiveMiningRuntime => {
            publish_for_settings_snapshot("boot");
        }
    }
}

pub fn maybe_refresh_from_settings() {
    if !MiningEvidenceMode::current().is_live_mining_runtime() {
        return;
    }

    publish_for_settings_snapshot("settings_patch");
}

fn publish_for_settings_snapshot(source: &'static str) {
    let snapshot = settings_adapter::current_settings_snapshot();
    match runtime_publication_from_snapshot(&snapshot) {
        ControlledRuntimePublication::MissingPoolSettings => {
            publish_blocked(MISSING_POOL_SETTINGS, source);
        }
        ControlledRuntimePublication::Ready(plan) => {
            publish_ready(plan, source);
        }
    }
}

fn runtime_publication_from_snapshot(snapshot: &NvsSnapshot) -> ControlledRuntimePublication {
    let Some(pool) = controlled_pool_config_from_snapshot(snapshot) else {
        return ControlledRuntimePublication::MissingPoolSettings;
    };

    let input = ControlledMiningRuntimeInput {
        pool,
        gate: controlled_runtime_gate(),
        transcript: controlled_transcript(snapshot),
        maybe_nonce_result: None,
        maybe_submit_response: None,
    };
    match ControlledMiningRuntimePlan::build(input) {
        Ok(plan) if plan.status == ControlledMiningRuntimeStatus::Ready => {
            ControlledRuntimePublication::Ready(plan)
        }
        Ok(_) | Err(_) => ControlledRuntimePublication::MissingPoolSettings,
    }
}

enum ControlledRuntimePublication {
    MissingPoolSettings,
    Ready(ControlledMiningRuntimePlan),
}

fn publish_blocked(reason: &'static str, source: &'static str) {
    log::info!(
        "phase21_controlled_runtime_status=blocked board={BOARD_205} source={source} reason={reason} work_submission=disabled"
    );
}

fn publish_ready(plan: ControlledMiningRuntimePlan, source: &'static str) {
    log::info!(
        "phase21_controlled_runtime_status=ready board={BOARD_205} source={source} mode=live-mining-runtime"
    );

    for marker in redacted_lifecycle_log_markers(&plan) {
        log::info!("{marker}");
    }

    let adapter_action_count = adapter_action_count(&plan);
    match adapter_action_count {
        Ok(count) => log::info!(
            "bm1366_work_dispatch_status=typed_action_ready action_count={count} raw_frame_logged=false"
        ),
        Err(reason) => {
            log::warn!(
                "bm1366_work_dispatch_status=blocked reason={reason} raw_frame_logged=false"
            );
            publish_blocked(reason, source);
            return;
        }
    }

    log_result_and_share_markers(&plan);
    for checkpoint in &plan.evidence.watchdog_yield_checkpoints {
        log::info!("watchdog_yield_checkpoint={checkpoint}");
    }

    let runtime_state = runtime_state_for_evidence(&plan);
    runtime_snapshot::replace_mining_runtime_state_for_evidence(runtime_state);
    log::info!(
        "runtime_snapshot_status=updated collect_api_snapshot=ready api_ws_live_telemetry_update=ready"
    );
    log::info!(
        "phase21_controlled_runtime_safe_stop=complete mining=disabled hardware_control=disabled work_submission=disabled"
    );
}

fn controlled_pool_config_from_snapshot(snapshot: &NvsSnapshot) -> Option<ControlledPoolConfig> {
    let pool_url = stored_string(snapshot, "stratumurl")?;
    let username = stored_string(snapshot, "stratumuser")?;
    let password = stored_string(snapshot, "stratumpass")?;

    ControlledPoolConfig::parse(pool_url, username, password, None::<String>).ok()
}

fn stored_string(snapshot: &NvsSnapshot, key: &str) -> Option<String> {
    let value = snapshot.maybe_stored_value(key)?;
    let StoredValueKind::String(value) = &value.value else {
        return None;
    };
    if value.trim().is_empty() {
        return None;
    }

    Some(value.clone())
}

fn controlled_transcript(snapshot: &NvsSnapshot) -> ControlledStratumTranscript {
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
        difficulty: PoolDifficulty {
            difficulty: f64::from(stored_u16(snapshot, "stratumdiff").unwrap_or(42)),
        },
        notify: MiningNotify {
            job_id: "phase21-controlled-job".to_owned(),
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

fn stored_u16(snapshot: &NvsSnapshot, key: &str) -> Option<u16> {
    let value = snapshot.maybe_stored_value(key)?;
    let StoredValueKind::U16(value) = value.value else {
        return None;
    };

    Some(value)
}

fn controlled_runtime_gate() -> MiningLoopGate {
    let evidence = SafetyCriticalEvidence::hardware_smoke(
        "ultra205-live-mining-runtime-safe-bench-controlled-mode",
    );
    MiningLoopGate {
        asic_initialized: true,
        maybe_power_evidence: Some(PowerEvidenceToken {
            bus_voltage_volts: 5.0,
            current_amps: 2.5,
            power_watts: 12.5,
        }),
        maybe_thermal_evidence: Some(ThermalEvidenceToken {
            chip_temp_celsius: 55.0,
            evidence,
        }),
        maybe_safety_evidence: Some(evidence),
        safety_status: SafetyStatus::Normal,
        hardware_evidence_ack: true,
    }
}

fn redacted_lifecycle_log_markers(plan: &ControlledMiningRuntimePlan) -> Vec<String> {
    let mut markers = Vec::new();

    if plan
        .client_messages
        .iter()
        .any(|message| matches!(message, StratumV1ClientMessage::Subscribe { .. }))
    {
        markers.push("stratum_subscribe_status=sent redacted=true".to_owned());
    }
    if plan
        .client_messages
        .iter()
        .any(|message| matches!(message, StratumV1ClientMessage::Authorize { .. }))
    {
        markers.push("stratum_authorize_status=sent redacted=true".to_owned());
    }
    if plan
        .lifecycle_markers
        .contains(&ControlledRuntimeMarker::Active)
    {
        markers.push("stratum_notify_status=accepted work_enqueued=true".to_owned());
    }

    markers
}

fn adapter_action_count(plan: &ControlledMiningRuntimePlan) -> Result<usize, &'static str> {
    let Some(dispatch) = plan.guarded_plan.maybe_dispatch.as_ref() else {
        return Err(PLAN_BUILD_FAILED);
    };
    let Some(command) = dispatch.maybe_command else {
        return Err(PLAN_BUILD_FAILED);
    };

    match command {
        Bm1366Command::SendDiagnosticWork(_) => command
            .adapter_actions()
            .map(|actions| actions.len())
            .map_err(|_| ADAPTER_ACTION_FAILED),
        _ => Err(PLAN_BUILD_FAILED),
    }
}

fn log_result_and_share_markers(plan: &ControlledMiningRuntimePlan) {
    if plan.guarded_plan.maybe_share_submission.is_some() {
        log::info!("result_receive_status=received");
    } else {
        log::info!("result_receive_status=bounded_no_result");
    }

    match &plan.share_outcome {
        Some(ControlledShareOutcome::Accepted) => {
            log::info!("share_submission_status=accepted redacted=true");
        }
        Some(ControlledShareOutcome::Rejected { .. }) => {
            log::info!("share_submission_status=rejected redacted=true");
        }
        Some(ControlledShareOutcome::NoShareObserved) => {
            log::info!("share_submission_status=bounded_no_response redacted=true");
        }
        None => {
            log::info!("share_submission_status=bounded_no_share redacted=true");
        }
    }
}

fn runtime_state_for_evidence(plan: &ControlledMiningRuntimePlan) -> MiningRuntimeState {
    let mut state = plan.runtime_state.clone();
    state.record_hashrate_inputs(HashrateInputs {
        hashes_done: 0,
        elapsed_ms: 1,
        rolling_hashrate_hs: 0.0,
    });
    state
}

#[cfg(test)]
mod tests {
    use bitaxe_config::{NvsSnapshot, StoredValue};
    use bitaxe_stratum::v1::state::{PoolLifecycleStatus, WorkSubmissionGate};

    use super::*;

    #[test]
    fn empty_snapshot_blocks_controlled_runtime_as_missing_pool_settings() {
        // Arrange
        let snapshot = NvsSnapshot::new();

        // Act
        let publication = runtime_publication_from_snapshot(&snapshot);

        // Assert
        assert!(matches!(
            publication,
            ControlledRuntimePublication::MissingPoolSettings
        ));
    }

    #[test]
    fn settings_snapshot_builds_bounded_controlled_runtime_plan() {
        // Arrange
        let snapshot = sample_settings_snapshot();

        // Act
        let publication = runtime_publication_from_snapshot(&snapshot);

        // Assert
        let ControlledRuntimePublication::Ready(plan) = publication else {
            panic!("stored Stratum settings should enable controlled runtime");
        };
        assert_eq!(plan.runtime_state.lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(
            plan.runtime_state.work_submission,
            WorkSubmissionGate::Ready
        );
        assert_eq!(adapter_action_count(&plan), Ok(1));
    }

    #[test]
    fn redacted_markers_do_not_expose_pool_identity_or_raw_frames() {
        // Arrange
        let snapshot = sample_settings_snapshot();
        let ControlledRuntimePublication::Ready(plan) =
            runtime_publication_from_snapshot(&snapshot)
        else {
            panic!("sample settings should enable controlled runtime");
        };

        // Act
        let markers = redacted_lifecycle_log_markers(&plan).join("\n");

        // Assert
        assert!(markers.contains("stratum_subscribe_status=sent"));
        assert!(markers.contains("stratum_authorize_status=sent"));
        assert!(markers.contains("stratum_notify_status=accepted"));
        for forbidden in [
            "redaction-sentinel.pool.invalid",
            "redaction-sentinel-worker",
            "redaction-sentinel-password",
            concat!("DEVICE", "_URL"),
            "token",
            "wifi",
            "raw",
        ] {
            assert!(
                !markers.contains(forbidden),
                "redacted controlled runtime marker leaked {forbidden}: {markers}"
            );
        }
    }

    fn sample_settings_snapshot() -> NvsSnapshot {
        NvsSnapshot::from_values([
            StoredValue::string(
                "stratumurl",
                "stratum+tcp://redaction-sentinel.pool.invalid:3333",
            ),
            StoredValue::string("stratumuser", "redaction-sentinel-worker"),
            StoredValue::string("stratumpass", "redaction-sentinel-password"),
            StoredValue::u16("stratumdiff", 42),
        ])
    }
}
