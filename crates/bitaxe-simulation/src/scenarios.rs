use crate::{
    authorization, controller, CheckStatus, Failure, ProfileCheck, ScenarioResult, SimulationError,
    SCENARIO_VERSION,
};
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};
use serde_json::json;
use std::{cell::RefCell, rc::Rc};
const SCENARIOS: &[&str] = &[
    "healthy-lifecycle",
    "stale-safety-step5",
    "zero-fan-step5",
    "unsafe-reading-step5",
    "heartbeat-loss",
    "heap-fragmented",
    "internal-allocation-failure",
    "max-status",
    "status-allocation-failure",
    "delayed-i2c",
    "queue-saturation",
    "cancel-preparation",
    "cancel-shutdown",
    "reboot-before-status",
    "retained-record-missing",
    "persistence-failure",
    "replay-challenge",
    "stop-rejection",
    "close-rejection",
    "fixture-completion-failure",
    "live-writer",
];
pub fn scenario_names() -> &'static [&'static str] {
    SCENARIOS
}
pub fn run_scenario(name: &str, seed: u64) -> Result<ScenarioResult, SimulationError> {
    run_scenario_with_observer(name, seed, &mut |_| {})
}

/// Runs identical scenario policy with allocation-free target diagnostic checkpoints.
pub fn run_scenario_with_observer(
    name: &str,
    seed: u64,
    observer: &mut dyn FnMut(crate::ScenarioPhase),
) -> Result<ScenarioResult, SimulationError> {
    observer(crate::ScenarioPhase::Entry);
    if !SCENARIOS.contains(&name) {
        return Err(SimulationError::UnknownScenario);
    }
    if matches!(name, "cancel-preparation" | "cancel-shutdown") {
        return crate::boundary_scenarios::run_cancel_matrix(name, seed);
    }
    if matches!(
        name,
        "retained-record-missing" | "status-allocation-failure"
    ) {
        return crate::boundary_scenarios::run_retained_or_snapshot(name, seed);
    }
    if name == "close-rejection" {
        return crate::boundary_scenarios::run_close_rejection(seed);
    }
    if matches!(name, "heap-fragmented" | "internal-allocation-failure") {
        return Ok(crate::resource_scenarios::run(name, seed));
    }
    let supported = matches!(
        name,
        "healthy-lifecycle"
            | "stale-safety-step5"
            | "zero-fan-step5"
            | "unsafe-reading-step5"
            | "heartbeat-loss"
            | "cancel-preparation"
            | "persistence-failure"
            | "replay-challenge"
            | "stop-rejection"
            | "max-status"
            | "queue-saturation"
            | "delayed-i2c"
            | "reboot-before-status"
    );
    if !supported {
        return Ok(ScenarioResult {
            schema: SCENARIO_VERSION.into(),
            scenario: name.into(),
            seed,
            checks: vec![ProfileCheck {
                id: name.into(),
                status: CheckStatus::Unsupported,
                detail: "production_boundary_not_integrated".into(),
            }],
            expected_outcome: "boundary_exercised".into(),
            actual_outcome: "unsupported".into(),
            journal: vec![],
            maybe_earliest_failure: None,
            cleanup_failures: vec![],
            virtual_time_ms: 0,
            modeled_memory: true,
            hardware_qualified: false,
        });
    }
    let context = initialize_controller(name, seed, observer)?;
    execute_controller_scenario(name, seed, context)
}

struct ScenarioContext {
    board: Rc<RefCell<VirtualBoard>>,
    worker: Box<controller::Controller>,
    binding: String,
    possession: Vec<u8>,
}

// The Controller/key constructor must execute before the larger command and
// status frame exists. Boxing the owner keeps this stage's return value small.
// Fixed peripheral/model return temporaries end before authority construction.
#[inline(never)]
fn initialize_board(seed: u64) -> Result<Rc<RefCell<VirtualBoard>>, SimulationError> {
    Ok(Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
        seed,
        ..BoardConfig::default()
    })?)))
}

#[inline(never)]
fn initialize_controller(
    name: &str,
    seed: u64,
    observer: &mut dyn FnMut(crate::ScenarioPhase),
) -> Result<ScenarioContext, SimulationError> {
    let board = initialize_board(seed)?;
    observer(crate::ScenarioPhase::BeforeController);
    let mut worker = controller::controller(board.clone(), name)?;
    observer(crate::ScenarioPhase::BeforePossession);
    let (binding, possession) = controller::possession(&mut worker)?;
    observer(crate::ScenarioPhase::AfterPossession);
    Ok(ScenarioContext {
        board,
        worker,
        binding,
        possession,
    })
}

#[inline(never)]
fn execute_controller_scenario(
    name: &str,
    seed: u64,
    context: ScenarioContext,
) -> Result<ScenarioResult, SimulationError> {
    let ScenarioContext {
        board,
        mut worker,
        binding,
        possession,
    } = context;
    if name == "persistence-failure" {
        board.borrow_mut().storage.reject_writes = true;
    }
    if name == "replay-challenge" {
        let rejected = worker
            .prepare_frame(&possession, 100)
            .err()
            .map(|e| e.category());
        return Ok(result(
            name,
            seed,
            &worker,
            "invalid_proof",
            rejected.unwrap_or("unexpected_success"),
            vec![],
        ));
    }
    if name == "healthy-lifecycle" {
        let now = worker.session().actuation.now_ms();
        let query = controller::frame(
            "stratum_v2_status",
            json!({"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":null}),
        )?;
        let observation = worker
            .prepare_frame(&query, now)
            .map_err(|_| SimulationError::Boundary("v2_observation"))?;
        worker
            .confirm_sent_at(observation, now)
            .map_err(|_| SimulationError::Boundary("v2_observation_delivery"))?;
    }
    let start = controller::frame(
        "start_lease",
        authorization::signed_start_for_profile(
            &binding,
            worker.session().ledger.next_ordinal(),
            name == "healthy-lifecycle",
        )?,
    )?;
    let prepared = worker.prepare_frame(&start, 100);
    let actual = match prepared {
        Ok(reply) => {
            worker
                .confirm_sent(reply)
                .map_err(|_| SimulationError::Boundary("start_delivery"))?;
            "started"
        }
        Err(ref error) => error.category(),
    };
    let expected = match name {
        "persistence-failure" => "authentication_failed",
        "stale-safety-step5"
        | "zero-fan-step5"
        | "unsafe-reading-step5"
        | "cancel-preparation"
        | "queue-saturation"
        | "delayed-i2c" => "session_failed",
        _ => "started",
    };
    let mut extra = vec![];
    if name == "heartbeat-loss" && actual == "started" {
        worker
            .session_mut()
            .actuation
            .advance(2800, false)
            .map_err(|_| SimulationError::Boundary("heartbeat_advance"))?;
        let timing = worker
            .session()
            .actuation
            .gate
            .timing(worker.session().actuation.now_ms())
            .ok_or(SimulationError::Boundary("timing"))?;
        extra.push(ProfileCheck {
            id: "heartbeat_cutoff".into(),
            status: if timing.revocation_reason
                == bitaxe_runtime::revocation::RevocationReason::HeartbeatTimeout
            {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: timing.revocation_reason.label().into(),
        });
    }
    if actual == "started" {
        let now = worker.session().actuation.now_ms();
        let status =
            worker.prepare_frame(&controller::frame("status", serde_json::Value::Null)?, now);
        extra.push(ProfileCheck {
            id: "authenticated_status".into(),
            status: if status.is_ok() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: if status.is_ok() {
                "response_prepared"
            } else {
                "status_failed"
            }
            .into(),
        });
        let stop = worker.prepare_frame(
            &controller::frame("restore", json!({"reason":"paused"}))?,
            now,
        );
        if name == "stop-rejection" {
            extra.push(ProfileCheck {
                id: "stop_rejected".into(),
                status: if stop.is_err() && worker.has_active_lease() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                detail: "cleanup_obligation_retained".into(),
            });
        } else {
            extra.push(ProfileCheck {
                id: "stop".into(),
                status: if stop.is_ok() && !worker.has_active_lease() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                detail: "restoration_checked".into(),
            });
        }
    }
    if name == "persistence-failure" {
        board.borrow_mut().storage.reject_writes = false;
    }
    // Cleanup is independently evaluated even after the scenario's earliest error.
    worker.session_mut().reject_stop = false;
    let now = worker.session().actuation.now_ms();
    let close = worker.disconnect(now);
    extra.push(ProfileCheck {
        id: "close".into(),
        status: if close.is_ok() && !worker.has_active_lease() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        },
        detail: "controller_released".into(),
    });
    let ledger = worker.session().ledger.clone();
    extra.push(ProfileCheck {
        id: "accounting".into(),
        status: if !ledger.pending()
            && (name == "persistence-failure" || ledger.total_charged_ms() == 180000)
        {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        },
        detail: "durable_ledger_checked".into(),
    });
    let mut answer = result(name, seed, &worker, expected, actual, extra);
    if name == "healthy-lifecycle" {
        let facts = worker.session().maybe_encrypted_share.as_ref();
        answer.checks.push(ProfileCheck {
            id: "controller_owned_encrypted_share".into(),
            status: if facts.is_some_and(|value| {
                value.acknowledged_shares == 1
                    && value.encrypted_frames_delivered == 9
                    && value.resources_released
                    && value.independently_target_valid
            }) {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: "generic_public_genesis_profile".into(),
        });
        let timing = worker
            .session()
            .actuation
            .gate
            .timing(worker.session().actuation.now_ms());
        answer.checks.push(ProfileCheck {
            id: "guarded_dispatch_accounting".into(),
            status: if timing.is_some_and(|value| {
                value.work_dispatched == 1
                    && value.submitted == 1
                    && value.accepted == 1
                    && value.nonce_work_correlations == 1
            }) {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: "actual_gate_io_observations".into(),
        });
        answer.checks.push(ProfileCheck {
            id: "retained_accepted_share".into(),
            status: if worker
                .session()
                .maybe_v2_record
                .borrow()
                .as_ref()
                .is_some_and(|record| {
                    record.has_accepted_share()
                        && record.snapshot().state == bitaxe_worker_control::v2::State::Terminal
                        && record.snapshot().maybe_outcome
                            == Some(bitaxe_worker_control::v2::Outcome::Accepted)
                        && record.snapshot().maybe_first_failure.is_none()
                }) {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: "actual_share_fact_and_release_fence".into(),
        });
        answer.checks.push(ProfileCheck {
            id: "strict_live_profile_share".into(),
            status: CheckStatus::Unsupported,
            detail: "genesis_not_fixed_regtest_1024_fixture".into(),
        });
    }
    Ok(answer)
}
fn result(
    name: &str,
    seed: u64,
    worker: &controller::Controller,
    expected: &str,
    actual: &str,
    mut checks: Vec<ProfileCheck>,
) -> ScenarioResult {
    checks.insert(
        0,
        ProfileCheck {
            id: "controller_outcome".into(),
            status: if expected == actual {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: actual.into(),
        },
    );
    ScenarioResult {
        schema: SCENARIO_VERSION.into(),
        scenario: name.into(),
        seed,
        checks,
        expected_outcome: expected.into(),
        actual_outcome: actual.into(),
        journal: worker.session().actuation.journal.clone(),
        maybe_earliest_failure: worker
            .session()
            .actuation
            .maybe_earliest_failure
            .clone()
            .or_else(|| {
                (actual != "started" && actual != "invalid_proof").then(|| Failure {
                    phase: "controller".into(),
                    category: actual.into(),
                })
            }),
        cleanup_failures: worker.session().actuation.cleanup_failures.clone(),
        virtual_time_ms: worker.session().actuation.now_ms(),
        modeled_memory: true,
        hardware_qualified: false,
    }
}
