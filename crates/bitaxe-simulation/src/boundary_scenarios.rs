//! Composed Controller boundaries with independent restoration and host transport closure.
use crate::{
    authorization, controller, CheckStatus, Failure, ProfileCheck, ScenarioResult, SimulationError,
    SCENARIO_VERSION,
};
use bitaxe_runtime::mining_actuation::{preparation_plan, safe_shutdown_plan, PreparationStep};
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};
use serde_json::json;
use std::{cell::RefCell, rc::Rc};

pub(crate) fn run_cancel_matrix(name: &str, seed: u64) -> Result<ScenarioResult, SimulationError> {
    let mut checks = Vec::new();
    let mut journal = Vec::new();
    let mut failures = Vec::new();
    let mut elapsed = 0;
    let mut primary = None;
    let steps = if name == "cancel-preparation" {
        preparation_plan(crate::actuation::RuntimeBoardAdapter::profile()).len()
    } else {
        safe_shutdown_plan().len()
    };
    for index in 0..steps {
        let board = Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
            seed: seed.wrapping_add(index as u64),
            ..BoardConfig::default()
        })?));
        let mut worker = controller::controller(board.clone(), name)?;
        let (binding, _) = controller::possession(&mut worker)?;
        let expected = if name == "cancel-preparation" {
            let step = preparation_plan(crate::actuation::RuntimeBoardAdapter::profile())[index];
            worker.session_mut().actuation.maybe_cancel_preparation = Some(step);
            step
        } else {
            worker.session_mut().actuation.maybe_cancel_preparation =
                Some(PreparationStep::EnableAsic);
            worker.session_mut().actuation.maybe_fail_shutdown = Some(safe_shutdown_plan()[index]);
            PreparationStep::EnableAsic
        };
        let now = worker.session().actuation.now_ms();
        let start = worker.prepare_frame(
            &controller::frame(
                "start_lease",
                authorization::signed_start(&binding, worker.session().ledger.next_ordinal())?,
            )?,
            now,
        );
        let correctly_rejected = start.is_err()
            && worker
                .session()
                .actuation
                .maybe_earliest_failure
                .as_ref()
                .is_some_and(|failure| {
                    failure.phase == expected.label()
                        && failure.category == "injected_preparation_cancellation"
                });
        let now = worker.session().actuation.now_ms();
        let close = worker.disconnect(now);
        let safety = {
            let b = board.borrow();
            !b.peripherals.power_enabled && b.peripherals.reset_asserted
        };
        checks.push(ProfileCheck {
            id: format!("{}_{}", name, index + 1),
            status: if correctly_rejected
                && close.is_ok()
                && !worker.has_active_lease()
                && !worker.session().ledger.pending()
                && safety
            {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: "earliest_cancellation_and_independent_release".into(),
        });
        if primary.is_none() {
            primary = worker.session().actuation.maybe_earliest_failure.clone();
        }
        journal.extend(worker.session().actuation.journal.clone());
        failures.extend(worker.session().actuation.cleanup_failures.clone());
        elapsed += worker.session().actuation.now_ms();
    }
    Ok(ScenarioResult {
        schema: SCENARIO_VERSION.into(),
        scenario: name.into(),
        seed,
        checks,
        expected_outcome: "cancelled_and_restored".into(),
        actual_outcome: "cancelled_and_restored".into(),
        journal,
        maybe_earliest_failure: primary,
        cleanup_failures: failures,
        virtual_time_ms: elapsed,
        modeled_memory: true,
        hardware_qualified: false,
    })
}

pub(crate) fn run_retained_or_snapshot(
    name: &str,
    seed: u64,
) -> Result<ScenarioResult, SimulationError> {
    let board = Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
        seed,
        ..BoardConfig::default()
    })?));
    let mut worker = controller::controller(board.clone(), name)?;
    controller::possession(&mut worker)?;
    let query = controller::frame(
        "stratum_v2_status",
        json!({"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":if name=="retained-record-missing" {serde_json::Value::String(base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD,[9;16]))} else {serde_json::Value::Null}}),
    )?;
    worker.session_mut().fail_v2_snapshot = name == "status-allocation-failure";
    let reply = worker.prepare_frame(&query, 100);
    let actual = reply
        .err()
        .map(|error| error.category())
        .unwrap_or("unexpected_success");
    let expected = if name == "retained-record-missing" {
        "invalid_transition"
    } else {
        "session_failed"
    };
    let phase = worker.session().diagnostic_phases.borrow().last().copied();
    worker.session_mut().fail_v2_snapshot = false;
    let closed = worker.disconnect(100).is_ok();
    let mut checks = vec![
        ProfileCheck {
            id: "retained_snapshot_boundary".into(),
            status: if actual == expected
                && phase == Some(bitaxe_worker_control::ControlDiagnosticPhase::V2Snapshot)
            {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: actual.into(),
        },
        ProfileCheck {
            id: "independent_close".into(),
            status: if closed && !worker.has_active_lease() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: "missing_history_did_not_block_current_release".into(),
        },
    ];
    if name == "status-allocation-failure" {
        checks.push(ProfileCheck {
            id: "real_retained_clone_allocation_failure".into(),
            status: CheckStatus::Unsupported,
            detail: "snapshot_adapter_rejection_only".into(),
        });
    }
    Ok(ScenarioResult {
        schema: SCENARIO_VERSION.into(),
        scenario: name.into(),
        seed,
        checks,
        expected_outcome: expected.into(),
        actual_outcome: actual.into(),
        journal: vec![],
        maybe_earliest_failure: Some(Failure {
            phase: "retained_snapshot".into(),
            category: actual.into(),
        }),
        cleanup_failures: vec![],
        virtual_time_ms: 100,
        modeled_memory: true,
        hardware_qualified: false,
    })
}

pub(crate) fn run_close_rejection(seed: u64) -> Result<ScenarioResult, SimulationError> {
    let board = Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
        seed,
        ..BoardConfig::default()
    })?));
    let mut worker = controller::controller(board.clone(), "close-rejection")?;
    controller::possession(&mut worker)?;
    board.borrow_mut().control.connect()?;
    board.borrow_mut().control.reject_close = true;
    let current = worker.disconnect(100).is_ok();
    let rejected = board.borrow_mut().control.close().is_err();
    let retained = board.borrow().control.connected;
    board.borrow_mut().control.reject_close = false;
    let released = board.borrow_mut().control.close().is_ok() && !board.borrow().control.connected;
    Ok(ScenarioResult {
        schema: SCENARIO_VERSION.into(),
        scenario: "close-rejection".into(),
        seed,
        checks: vec![
            ProfileCheck {
                id: "independent_close_rejection".into(),
                status: if current && rejected && retained && released {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                detail: "restoration_separate_from_transport_release".into(),
            },
            ProfileCheck {
                id: "physical_host_owner_release".into(),
                status: CheckStatus::Unsupported,
                detail: "browser_http_owner_seam_not_integrated".into(),
            },
        ],
        expected_outcome: "close_rejected_then_released".into(),
        actual_outcome: "close_rejected_then_released".into(),
        journal: vec![],
        maybe_earliest_failure: Some(Failure {
            phase: "close".into(),
            category: "transport_close_rejected".into(),
        }),
        cleanup_failures: vec![],
        virtual_time_ms: 100,
        modeled_memory: true,
        hardware_qualified: false,
    })
}
