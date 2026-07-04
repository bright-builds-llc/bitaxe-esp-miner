//! Phase 21 controlled Stratum runtime contract.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_socket.c`
//! - `reference/esp-miner/components/stratum/mining.c`
//! - `reference/esp-miner/main/tasks/protocol_coordinator.c`
//! - Parity checklist rows `ASIC-007`, `STR-006`, `STR-007`, and `SAFE-09`

use std::fmt;

use bitaxe_asic::bm1366::result::Bm1366NonceResult;
use bitaxe_asic::bm1366::work::Bm1366JobId;
use bitaxe_config::ultra_205_defaults;

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;
use crate::v1::messages::{
    ExtranonceAssignment, PoolDifficulty, StratumResponse, StratumV1ClientMessage,
};
use crate::v1::mining::{MiningWorkBuilder, ShareSubmission};
use crate::v1::mining_loop::{
    GuardedMiningLoopInputs, GuardedMiningLoopPlan, GuardedMiningLoopSource, MiningLoopDecision,
    MiningLoopGate,
};
use crate::v1::queue::MiningWorkQueue;
use crate::v1::state::{MiningRuntimeState, PoolLifecycleStatus, ShareDifficulty};

#[derive(Clone, PartialEq, Eq)]
pub struct ControlledPoolConfig {
    pool_url: String,
    username: String,
    password: String,
    maybe_worker: Option<String>,
}

impl ControlledPoolConfig {
    pub fn parse(
        pool_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        maybe_worker: Option<impl Into<String>>,
    ) -> Result<Self, StratumV1Error> {
        let pool_url = pool_url.into();
        let username = username.into();
        let password = password.into();
        let maybe_worker = maybe_worker.map(Into::into);

        if pool_url.trim().is_empty() {
            return Err(StratumV1Error::InvalidField {
                field: "pool_url",
                reason: "expected non-empty controlled pool URL",
            });
        }
        if username.trim().is_empty() {
            return Err(StratumV1Error::InvalidField {
                field: "username",
                reason: "expected non-empty controlled pool username",
            });
        }

        Ok(Self {
            pool_url,
            username,
            password,
            maybe_worker,
        })
    }

    fn authorize_message(&self) -> StratumV1ClientMessage {
        StratumV1ClientMessage::authorize(
            ControlledMiningRuntimePlan::AUTHORIZE_REQUEST_ID,
            &self.username,
            &self.password,
        )
    }

    fn redacted_summary(&self) -> ControlledPoolSummary {
        ControlledPoolSummary {
            endpoint_configured: !self.pool_url.trim().is_empty(),
            username_configured: !self.username.trim().is_empty(),
            password_configured: !self.password.trim().is_empty(),
            worker_configured: self
                .maybe_worker
                .as_deref()
                .map(|worker| !worker.trim().is_empty())
                .unwrap_or(false),
        }
    }
}

impl fmt::Debug for ControlledPoolConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ControlledPoolConfig")
            .field("pool_url", &"<redacted>")
            .field("username", &"<redacted>")
            .field("password", &"<redacted>")
            .field(
                "maybe_worker",
                &self.maybe_worker.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ControlledPoolSummary {
    endpoint_configured: bool,
    username_configured: bool,
    password_configured: bool,
    worker_configured: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlledStratumTranscript {
    pub subscribe_response: StratumResponse,
    pub authorize_response: StratumResponse,
    pub difficulty: PoolDifficulty,
    pub notify: crate::v1::messages::MiningNotify,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlledMiningRuntimeInput {
    pub pool: ControlledPoolConfig,
    pub gate: MiningLoopGate,
    pub transcript: ControlledStratumTranscript,
    pub maybe_nonce_result: Option<Bm1366NonceResult>,
    pub maybe_submit_response: Option<StratumResponse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledMiningRuntimeStatus {
    Ready,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledRuntimeMarker {
    Subscribed,
    Authorized,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlledShareOutcome {
    Accepted,
    Rejected { reason: String },
    NoShareObserved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlledMiningRuntimeEvidence {
    pub safe_stop_required: bool,
    pub watchdog_yield_checkpoints: Vec<&'static str>,
    status: ControlledMiningRuntimeStatus,
    pool_summary: ControlledPoolSummary,
    lifecycle_markers: Vec<ControlledRuntimeMarker>,
    share_outcome: Option<ControlledShareOutcome>,
}

impl ControlledMiningRuntimeEvidence {
    #[must_use]
    pub fn redacted_summary(&self) -> String {
        format!(
            concat!(
                "controlled_runtime_harness_status: {status}\n",
                "pool_endpoint_configured: {endpoint}\n",
                "pool_username_configured: {username}\n",
                "pool_password_configured: {password}\n",
                "pool_worker_configured: {worker}\n",
                "lifecycle_markers: {markers:?}\n",
                "share_outcome: {share}\n",
                "safe_stop_required: {safe_stop}\n",
                "watchdog_yield_checkpoints: {watchdog:?}\n"
            ),
            status = match self.status {
                ControlledMiningRuntimeStatus::Ready => "ready",
                ControlledMiningRuntimeStatus::Blocked => "blocked",
            },
            endpoint = self.pool_summary.endpoint_configured,
            username = self.pool_summary.username_configured,
            password = self.pool_summary.password_configured,
            worker = self.pool_summary.worker_configured,
            markers = self.lifecycle_markers,
            share = redacted_share_outcome_label(&self.share_outcome),
            safe_stop = self.safe_stop_required,
            watchdog = self.watchdog_yield_checkpoints,
        )
    }
}

fn redacted_share_outcome_label(outcome: &Option<ControlledShareOutcome>) -> &'static str {
    match outcome {
        Some(ControlledShareOutcome::Accepted) => "accepted",
        Some(ControlledShareOutcome::Rejected { .. }) => "rejected",
        Some(ControlledShareOutcome::NoShareObserved) => "no_share_observed",
        None => "none",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlledMiningRuntimePlan {
    pub status: ControlledMiningRuntimeStatus,
    pub block_reason: Option<&'static str>,
    pub client_messages: Vec<StratumV1ClientMessage>,
    pub guarded_plan: GuardedMiningLoopPlan,
    pub runtime_state: MiningRuntimeState,
    pub lifecycle_markers: Vec<ControlledRuntimeMarker>,
    pub share_outcome: Option<ControlledShareOutcome>,
    pub evidence: ControlledMiningRuntimeEvidence,
}

impl ControlledMiningRuntimePlan {
    pub const SUBSCRIBE_REQUEST_ID: StratumRequestId = StratumRequestId::new(1);
    pub const AUTHORIZE_REQUEST_ID: StratumRequestId = StratumRequestId::new(2);
    pub const SUBMIT_REQUEST_ID: StratumRequestId = StratumRequestId::new(3);

    pub fn build(input: ControlledMiningRuntimeInput) -> Result<Self, StratumV1Error> {
        let pool_summary = input.pool.redacted_summary();
        let decision = input.gate.decision();

        if let MiningLoopDecision::Blocked { reason } = decision {
            return Self::blocked(input, reason, pool_summary);
        }

        Self::ready(input, pool_summary)
    }

    fn blocked(
        input: ControlledMiningRuntimeInput,
        reason: &'static str,
        pool_summary: ControlledPoolSummary,
    ) -> Result<Self, StratumV1Error> {
        let guarded_plan = GuardedMiningLoopInputs {
            gate: input.gate,
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::FakePool,
            work_queue: MiningWorkQueue::new(),
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: None,
        }
        .plan()?;
        let evidence = ControlledMiningRuntimeEvidence {
            safe_stop_required: true,
            watchdog_yield_checkpoints: vec!["safe_stop"],
            status: ControlledMiningRuntimeStatus::Blocked,
            pool_summary,
            lifecycle_markers: Vec::new(),
            share_outcome: None,
        };

        Ok(Self {
            status: ControlledMiningRuntimeStatus::Blocked,
            block_reason: Some(reason),
            client_messages: Vec::new(),
            runtime_state: guarded_plan.runtime_state.clone(),
            guarded_plan,
            lifecycle_markers: Vec::new(),
            share_outcome: None,
            evidence,
        })
    }

    fn ready(
        input: ControlledMiningRuntimeInput,
        pool_summary: ControlledPoolSummary,
    ) -> Result<Self, StratumV1Error> {
        let mut client_messages = vec![
            StratumV1ClientMessage::subscribe(Self::SUBSCRIBE_REQUEST_ID, "ultra", "205"),
            input.pool.authorize_message(),
        ];
        let mut runtime_state = MiningRuntimeState::default();
        let mut lifecycle_markers = Vec::new();
        let extranonce = apply_subscribe_response(
            &input.transcript.subscribe_response,
            &mut runtime_state,
            &mut lifecycle_markers,
        )?;
        apply_authorize_response(
            &input.transcript.authorize_response,
            &mut runtime_state,
            &mut lifecycle_markers,
        )?;
        runtime_state.set_pool_difficulty(input.transcript.difficulty);
        runtime_state.set_lifecycle(PoolLifecycleStatus::Active);
        lifecycle_markers.push(ControlledRuntimeMarker::Active);

        let mut work_queue = MiningWorkQueue::new();
        let work = MiningWorkBuilder::new(input.transcript.notify, extranonce)
            .with_pool_difficulty(input.transcript.difficulty)
            .build(Bm1366JobId::new(0x28))?;
        work_queue.enqueue_work(work)?;

        let dispatch_plan = GuardedMiningLoopInputs {
            gate: input.gate.clone(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue,
            runtime_state,
            maybe_nonce_result: None,
        }
        .plan()?;

        let mut guarded_plan = if let Some(nonce_result) = input.maybe_nonce_result {
            GuardedMiningLoopInputs {
                gate: input.gate,
                pool_defaults: ultra_205_defaults(),
                source: GuardedMiningLoopSource::Notify,
                work_queue: dispatch_plan.work_queue,
                runtime_state: dispatch_plan.runtime_state,
                maybe_nonce_result: Some(nonce_result),
            }
            .plan()?
        } else {
            dispatch_plan
        };

        let maybe_share = guarded_plan.maybe_share_submission.clone();
        let share_outcome = apply_share_response(
            maybe_share.as_ref(),
            input.maybe_submit_response.as_ref(),
            &input.pool,
            &mut client_messages,
            &mut guarded_plan.runtime_state,
        );
        guarded_plan
            .runtime_state
            .set_lifecycle(PoolLifecycleStatus::Active);
        let runtime_state = guarded_plan.runtime_state.clone();
        let evidence = ControlledMiningRuntimeEvidence {
            safe_stop_required: true,
            watchdog_yield_checkpoints: vec![
                "subscribe",
                "authorize",
                "notify",
                "dispatch",
                "result",
                "share",
                "safe_stop",
            ],
            status: ControlledMiningRuntimeStatus::Ready,
            pool_summary,
            lifecycle_markers: lifecycle_markers.clone(),
            share_outcome: share_outcome.clone(),
        };

        Ok(Self {
            status: ControlledMiningRuntimeStatus::Ready,
            block_reason: None,
            client_messages,
            guarded_plan,
            runtime_state,
            lifecycle_markers,
            share_outcome,
            evidence,
        })
    }
}

fn apply_subscribe_response(
    response: &StratumResponse,
    runtime_state: &mut MiningRuntimeState,
    lifecycle_markers: &mut Vec<ControlledRuntimeMarker>,
) -> Result<ExtranonceAssignment, StratumV1Error> {
    if !response.success {
        return Err(StratumV1Error::InvalidParams {
            method: "controlled_runtime.subscribe",
        });
    }
    let Some(extranonce) = response.maybe_extranonce.clone() else {
        return Err(StratumV1Error::InvalidParams {
            method: "controlled_runtime.subscribe",
        });
    };

    runtime_state.set_lifecycle(PoolLifecycleStatus::Subscribed);
    lifecycle_markers.push(ControlledRuntimeMarker::Subscribed);
    Ok(extranonce)
}

fn apply_authorize_response(
    response: &StratumResponse,
    runtime_state: &mut MiningRuntimeState,
    lifecycle_markers: &mut Vec<ControlledRuntimeMarker>,
) -> Result<(), StratumV1Error> {
    if !response.success {
        return Err(StratumV1Error::InvalidParams {
            method: "controlled_runtime.authorize",
        });
    }

    runtime_state.set_lifecycle(PoolLifecycleStatus::Authorized);
    lifecycle_markers.push(ControlledRuntimeMarker::Authorized);
    Ok(())
}

fn apply_share_response(
    maybe_share: Option<&ShareSubmission>,
    maybe_response: Option<&StratumResponse>,
    pool: &ControlledPoolConfig,
    client_messages: &mut Vec<StratumV1ClientMessage>,
    runtime_state: &mut MiningRuntimeState,
) -> Option<ControlledShareOutcome> {
    let share = maybe_share?;

    client_messages.push(share.to_client_message(
        ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID,
        &pool.username,
    ));

    let Some(response) = maybe_response.filter(|response| {
        response.maybe_id == Some(ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID)
    }) else {
        return Some(ControlledShareOutcome::NoShareObserved);
    };

    if response.success {
        let difficulty = runtime_state
            .maybe_pool_difficulty
            .map(|pool| ShareDifficulty::new(pool.difficulty))
            .unwrap_or_else(|| ShareDifficulty::new(0.0));
        runtime_state.record_accepted_share(difficulty);
        return Some(ControlledShareOutcome::Accepted);
    }

    let reason = response
        .maybe_error
        .as_ref()
        .map(|error| error.message.clone())
        .unwrap_or_else(|| "pool rejected share".to_owned());
    runtime_state.record_rejected_share(reason.clone());
    Some(ControlledShareOutcome::Rejected { reason })
}

#[cfg(test)]
mod tests;
