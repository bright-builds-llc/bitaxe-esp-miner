use super::*;

#[derive(Serialize)]
pub(super) struct CampaignStatusProjection {
    pub(super) schema: &'static str,
    pub(super) stage: &'static str,
    pub(super) lease_id: Option<u64>,
    pub(super) campaign_state: &'static str,
    pub(super) profile: &'static str,
    pub(super) active_ms: u64,
    pub(super) submit_outcome: &'static str,
    pub(super) qualified_candidate_count: u64,
    pub(super) below_pool_target_count: u64,
    pub(super) duplicate_candidate_count: u64,
    pub(super) accepted_share_count: u64,
    pub(super) rejected_share_count: u64,
    pub(super) job_transition: CampaignJobTransitionProjection,
    pub(super) asic_bridge: AsicBridgeEvidence,
    pub(super) terminal_reason: &'static str,
    pub(super) protocol_gate: &'static str,
    pub(super) readiness_transition: CampaignReadinessTransitionProjection,
    pub(super) operator_sensor: OperatorSensorDiagnosticProjection,
    pub(super) resumable_pause_safe_stop: &'static str,
    pub(super) safety: &'static str,
    pub(super) fresh_observation_count: u8,
    pub(super) observation_freshness: CampaignObservationFreshness,
    pub(super) observation_requirements: CampaignObservationRequirements,
    pub(super) pool_config: &'static str,
    pub(super) pool_config_persisted: bool,
    pub(super) actuation: &'static str,
    pub(super) mineonboot: bool,
    pub(super) safe_stop: &'static str,
    pub(super) failure: CampaignFailureDiagnostic,
}

#[derive(Serialize)]
pub(super) struct OperatorSensorDiagnosticProjection {
    available: bool,
    boot_session: u64,
    revision: u64,
    stage: &'static str,
    outcome: &'static str,
    duration_bucket: &'static str,
}

impl From<Option<OperatorSensorDiagnostic>> for OperatorSensorDiagnosticProjection {
    fn from(maybe_diagnostic: Option<OperatorSensorDiagnostic>) -> Self {
        let Some(diagnostic) = maybe_diagnostic else {
            return Self {
                available: false,
                boot_session: 0,
                revision: 0,
                stage: "none",
                outcome: "none",
                duration_bucket: "none",
            };
        };
        Self {
            available: true,
            boot_session: diagnostic.boot_session(),
            revision: diagnostic.revision(),
            stage: diagnostic.stage().label(),
            outcome: diagnostic.outcome().label(),
            duration_bucket: diagnostic.duration_bucket().label(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct CampaignReadinessTransitionProjection {
    wakeup: &'static str,
    previous_blocker: &'static str,
    current_blocker: &'static str,
    session_phase: &'static str,
    campaign_state: &'static str,
    hardware_state: &'static str,
    safety_sample: &'static str,
    observation_epoch: &'static str,
    pending_observation_recovered: bool,
}

impl From<ReadinessTransitionEvidence> for CampaignReadinessTransitionProjection {
    fn from(evidence: ReadinessTransitionEvidence) -> Self {
        Self {
            wakeup: evidence.wakeup_label(),
            previous_blocker: evidence.previous_blocker_label(),
            current_blocker: evidence.current_blocker_label(),
            session_phase: evidence.session_phase_label(),
            campaign_state: evidence.campaign_state_label(),
            hardware_state: evidence.hardware_state_label(),
            safety_sample: if evidence.safety_sample_fresh {
                "fresh"
            } else {
                "stale"
            },
            observation_epoch: evidence.observation_epoch_relation.label(),
            pending_observation_recovered: evidence.pending_observation_recovered,
        }
    }
}

#[derive(Serialize)]
pub(super) struct CampaignJobTransitionProjection {
    pool_notify_count: u64,
    clean_jobs_notify_count: u64,
    previous_block_change_count: u64,
    new_block_generation_count: u64,
    replacement_dispatch_count: u64,
    post_transition_correlated_result_count: u64,
    completed_transition_count: u64,
    stale_generation_result_discard_count: u64,
    stale_generation_submit_count: u64,
    reconnect_count: u64,
    latest_state: &'static str,
}

impl From<JobTransitionEvidence> for CampaignJobTransitionProjection {
    fn from(evidence: JobTransitionEvidence) -> Self {
        Self {
            pool_notify_count: evidence.pool_notify_count,
            clean_jobs_notify_count: evidence.clean_jobs_notify_count,
            previous_block_change_count: evidence.previous_block_change_count,
            new_block_generation_count: evidence.new_block_generation_count,
            replacement_dispatch_count: evidence.replacement_dispatch_count,
            post_transition_correlated_result_count: evidence.replacement_result_count,
            completed_transition_count: evidence.completed_transition_count,
            stale_generation_result_discard_count: evidence.stale_generation_result_discard_count,
            stale_generation_submit_count: evidence.stale_generation_submit_count,
            reconnect_count: evidence.reconnect_count,
            latest_state: evidence.latest_state.label(),
        }
    }
}
