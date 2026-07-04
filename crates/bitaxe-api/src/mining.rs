//! Pure mining-state response mapping for AxeOS-compatible API surfaces.
//!
//! Reference breadcrumbs:
//! - `crates/bitaxe-stratum/src/v1/state.rs`
//! - `crates/bitaxe-stratum/src/v1/mining_loop.rs`
//! - `reference/esp-miner/main/http_server/system_api_json.c`

use bitaxe_stratum::v1::mining_loop::HARDWARE_EVIDENCE_ACK_MISSING;
use bitaxe_stratum::v1::state::{
    MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus, WorkSubmissionGate,
};
use serde::{Deserialize, Serialize};

/// Aggregated rejected-share reason in the upstream system-info shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharesRejectedReasonWire {
    pub message: String,
    pub count: u64,
}

/// Mining state fields used by system info and future live telemetry mappers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MiningStateWire {
    #[serde(rename = "hashRate")]
    pub hash_rate: f64,
    #[serde(rename = "hashRate_1m")]
    pub hash_rate_1m: f64,
    #[serde(rename = "hashRate_10m")]
    pub hash_rate_10m: f64,
    #[serde(rename = "hashRate_1h")]
    pub hash_rate_1h: f64,
    #[serde(rename = "sharesAccepted")]
    pub shares_accepted: u64,
    #[serde(rename = "sharesRejected")]
    pub shares_rejected: u64,
    #[serde(rename = "sharesRejectedReasons")]
    pub shares_rejected_reasons: Vec<SharesRejectedReasonWire>,
    #[serde(rename = "bestDiff")]
    pub best_diff: f64,
    #[serde(rename = "bestSessionDiff")]
    pub best_session_diff: f64,
    #[serde(rename = "poolDifficulty")]
    pub pool_difficulty: f64,
    #[serde(rename = "isUsingFallbackStratum")]
    pub is_using_fallback_stratum: u8,
    #[serde(rename = "poolConnectionInfo")]
    pub pool_connection_info: String,
    #[serde(rename = "responseTime")]
    pub response_time: f64,
    #[serde(rename = "responseShareBatch")]
    pub response_share_batch: u64,
    #[serde(rename = "processTime")]
    pub process_time: f64,
    #[serde(rename = "miningPaused")]
    pub mining_paused: bool,
    #[serde(rename = "miningActivity")]
    pub mining_activity: String,
    #[serde(rename = "workSubmission")]
    pub work_submission: String,
    #[serde(rename = "blockedReason")]
    pub blocked_reason: String,
}

/// Maps Phase 4 typed mining runtime data into API-visible mining fields.
#[must_use]
pub fn mining_state_from_runtime(state: &MiningRuntimeState) -> MiningStateWire {
    let hash_rate_ghs = state.hashrate_inputs.rolling_hashrate_hs / 1_000_000_000.0;
    let best_diff = state
        .counters
        .maybe_best_difficulty
        .map(|difficulty| difficulty.raw())
        .unwrap_or(0.0);

    MiningStateWire {
        hash_rate: hash_rate_ghs,
        hash_rate_1m: hash_rate_ghs,
        hash_rate_10m: 0.0,
        hash_rate_1h: 0.0,
        shares_accepted: state.counters.accepted,
        shares_rejected: state.counters.rejected,
        shares_rejected_reasons: rejected_reasons(&state.counters.rejected_reasons),
        best_diff,
        best_session_diff: best_diff,
        pool_difficulty: state
            .maybe_pool_difficulty
            .map(|difficulty| difficulty.difficulty)
            .unwrap_or(0.0),
        is_using_fallback_stratum: u8::from(state.fallback_active),
        pool_connection_info: lifecycle_label(state.lifecycle).to_owned(),
        response_time: 0.0,
        response_share_batch: 0,
        process_time: 0.0,
        mining_paused: !matches!(state.mining_activity, MiningActivityStatus::Active),
        mining_activity: mining_activity_label(state.mining_activity).to_owned(),
        work_submission: work_submission_label(state.work_submission).to_owned(),
        blocked_reason: blocked_reason(state).to_owned(),
    }
}

fn rejected_reasons(reasons: &[String]) -> Vec<SharesRejectedReasonWire> {
    let mut aggregate = Vec::new();

    for reason in reasons {
        if let Some(existing) = aggregate
            .iter_mut()
            .find(|entry: &&mut SharesRejectedReasonWire| entry.message == *reason)
        {
            existing.count += 1;
            continue;
        }

        aggregate.push(SharesRejectedReasonWire {
            message: reason.clone(),
            count: 1,
        });
    }

    aggregate
}

fn lifecycle_label(status: PoolLifecycleStatus) -> &'static str {
    match status {
        PoolLifecycleStatus::Disconnected => "disconnected",
        PoolLifecycleStatus::Connecting => "connecting",
        PoolLifecycleStatus::Subscribed => "subscribed",
        PoolLifecycleStatus::Authorized => "authorized",
        PoolLifecycleStatus::Active => "active",
        PoolLifecycleStatus::Reconnecting => "reconnecting",
        PoolLifecycleStatus::FallbackActive => "fallback_active",
        PoolLifecycleStatus::Error => "error",
    }
}

fn mining_activity_label(status: MiningActivityStatus) -> &'static str {
    match status {
        MiningActivityStatus::Paused => "paused",
        MiningActivityStatus::Active => "active",
        MiningActivityStatus::SafeBlocked => "safe_blocked",
    }
}

fn work_submission_label(gate: WorkSubmissionGate) -> &'static str {
    match gate {
        WorkSubmissionGate::Blocked => "blocked",
        WorkSubmissionGate::Ready => "ready",
    }
}

fn blocked_reason(state: &MiningRuntimeState) -> &'static str {
    if state.work_submission == WorkSubmissionGate::Blocked
        && state.mining_activity == MiningActivityStatus::SafeBlocked
    {
        return HARDWARE_EVIDENCE_ACK_MISSING;
    }

    ""
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::mining_loop::HARDWARE_EVIDENCE_ACK_MISSING;
    use bitaxe_stratum::v1::state::{MiningActivityStatus, MiningRuntimeState, WorkSubmissionGate};

    use crate::mining::mining_state_from_runtime;

    #[test]
    fn mining_state_keeps_hardware_evidence_block_visible_and_not_active() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        state.set_mining_activity(MiningActivityStatus::SafeBlocked);
        state.work_submission = WorkSubmissionGate::Blocked;

        // Act
        let response = mining_state_from_runtime(&state);

        // Assert
        assert_eq!(response.mining_activity, "safe_blocked");
        assert_eq!(response.work_submission, "blocked");
        assert_eq!(response.blocked_reason, HARDWARE_EVIDENCE_ACK_MISSING);
        assert!(response.mining_paused);
    }

    #[test]
    fn mining_state_projects_exact_runtime_blocked_reason() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        state.block_work_submission("voltage_observation_stale");

        // Act
        let response = mining_state_from_runtime(&state);

        // Assert
        assert_eq!(response.mining_activity, "safe_blocked");
        assert_eq!(response.work_submission, "blocked");
        assert_eq!(response.blocked_reason, "voltage_observation_stale");
        assert!(response.mining_paused);
    }
}
