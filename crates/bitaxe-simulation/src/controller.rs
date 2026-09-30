//! Real WorkerSession adapter; modeled I/O supplies facts and stores production ledgers.
use crate::{actuation::RuntimeBoardAdapter, authorization, SimulationError};
mod v2;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_runtime::mining_actuation::{execute_preparation, execute_safe_shutdown};
use bitaxe_virtual_board::VirtualBoard;
use bitaxe_worker_control::{
    DeviceIdentity, FirmwareIdentity, FirmwareSourceCommit, LeaseDeadlines, QualificationLedger,
    RestorationReason, WorkerControl, WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession,
    WorkerSessionError,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Session {
    pub actuation: RuntimeBoardAdapter,
    pub ledger: QualificationLedger,
    pub reject_stop: bool,
    pub fail_v2_snapshot: bool,
    pub diagnostic_phases: RefCell<Vec<bitaxe_worker_control::ControlDiagnosticPhase>>,
    pub seed: u64,
    pub maybe_v2_record: Rc<RefCell<Option<bitaxe_worker_control::v2::V2Record>>>,
    pub maybe_encrypted_share: Option<crate::v2::EncryptedShareFacts>,
}
impl Session {
    fn persist(&mut self, ledger: QualificationLedger) -> Result<(), WorkerSessionError> {
        let bytes = serde_json::to_vec(&ledger).map_err(|_| WorkerSessionError::Rejected)?;
        self.actuation
            .board
            .borrow_mut()
            .storage
            .transaction(&[("qualification_ledger".into(), bytes)])
            .map_err(|_| WorkerSessionError::Rejected)?;
        self.ledger = ledger;
        Ok(())
    }
}
impl WorkerSession for Session {
    fn diagnostic_phase(&self, phase: bitaxe_worker_control::ControlDiagnosticPhase) {
        let mut phases = self.diagnostic_phases.borrow_mut();
        if phases.len() < phases.capacity() {
            phases.push(phase);
        }
    }
    fn v2_status(
        &self,
        scope: bitaxe_worker_control::v2::Scope,
    ) -> Result<Option<bitaxe_worker_control::v2::V2Status>, WorkerSessionError> {
        if self.fail_v2_snapshot {
            return Err(WorkerSessionError::Rejected);
        }
        let maybe_record = self
            .maybe_v2_record
            .borrow()
            .as_ref()
            .map(|record| record.snapshot());
        Ok(Some(bitaxe_worker_control::v2::V2Status {
            schema: "worker-stratum-v2-status-v1",
            scope,
            state: maybe_record
                .as_ref()
                .map_or(bitaxe_worker_control::v2::State::Idle, |record| {
                    record.state
                }),
            observation: self.v2_observation(),
            maybe_connection: None,
            maybe_record,
        }))
    }
    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        let allowance = grant
            .maybe_qualification_attempt()
            .ok_or(WorkerSessionError::Rejected)?;
        self.persist(
            self.ledger
                .reserve(allowance)
                .map_err(|_| WorkerSessionError::Rejected)?,
        )?;
        let generation = self.actuation.generation;
        if !self
            .actuation
            .gate
            .admit_budget(generation, allowance.maximum_active_milliseconds())
            || !self
                .actuation
                .gate
                .set_lease_deadline(generation, deadlines.expires_at_monotonic_milliseconds())
            || !self
                .actuation
                .gate
                .activate_at(generation, self.actuation.now_ms())
        {
            return Err(WorkerSessionError::Rejected);
        }
        if grant.maybe_v2().is_some() {
            self.admit_v2_record(grant)?;
        }
        execute_preparation(&mut self.actuation, RuntimeBoardAdapter::profile())
            .map_err(|_| WorkerSessionError::Rejected)?;
        if grant.maybe_v2().is_some() {
            self.run_encrypted_share(grant)?;
        }
        Ok(())
    }
    fn renew(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }
    fn safe_stop(&mut self, _reason: RestorationReason) -> Result<(), WorkerSessionError> {
        if self.reject_stop {
            return Err(WorkerSessionError::SafeStopFailed);
        }
        self.actuation
            .gate
            .revoke_at(self.actuation.generation, self.actuation.now_ms());
        execute_safe_shutdown(&mut self.actuation)
            .map_err(|_| WorkerSessionError::SafeStopFailed)?;
        self.persist(
            self.ledger
                .finish()
                .map_err(|_| WorkerSessionError::SafeStopFailed)?,
        )?;
        self.actuation
            .gate
            .finish_shutdown(self.actuation.generation);
        if let Some(record) = self.maybe_v2_record.borrow_mut().as_mut() {
            record.release_fence(Some(self.actuation.now_ms() * 1000));
        }
        Ok(())
    }
    fn qualification_attempt_review(&self) -> Result<Option<Value>, WorkerSessionError> {
        Ok(Some(
            json!({"schema":"worker-qualification-ledger-v1","next_ordinal":self.ledger.next_ordinal(),
            "last_completed_ordinal":self.ledger.last_completed_ordinal(),"total_charged_ms":self.ledger.total_charged_ms(),"pending":self.ledger.pending()}),
        ))
    }
    fn status_evidence(&self) -> Option<Value> {
        let facts = self.actuation.board.borrow().peripherals.sample;
        Some(
            json!({"schema":"simulation-observation-v1","sampled_at_ms":facts.sampled_at_ms,"fan_rpm":facts.fan_rpm,
            "asic_enabled":self.actuation.board.borrow().peripherals.power_enabled}),
        )
    }
}

pub(crate) type Controller = WorkerControl<authorization::Verifier, Session>;
struct ControllerAuthority {
    identity: DeviceIdentity,
    verifier: authorization::Verifier,
}

// Key generation and strict trust parsing run without Session/Controller moves.
#[inline(never)]
fn prepare_controller_authority(
    board: Rc<RefCell<VirtualBoard>>,
) -> Result<ControllerAuthority, SimulationError> {
    let identity = prepare_device_identity();
    let verifier = authorization::verifier(board)?;
    Ok(ControllerAuthority { identity, verifier })
}

#[inline(never)]
fn prepare_device_identity() -> DeviceIdentity {
    DeviceIdentity::from_seed([7; 32])
}

#[inline(never)]
pub(crate) fn controller(
    board: Rc<RefCell<VirtualBoard>>,
    scenario: &str,
) -> Result<Box<Controller>, SimulationError> {
    let authority = prepare_controller_authority(board.clone())?;
    assemble_controller(board, scenario, authority)
}

#[inline(never)]
fn assemble_controller(
    board: Rc<RefCell<VirtualBoard>>,
    scenario: &str,
    authority: ControllerAuthority,
) -> Result<Box<Controller>, SimulationError> {
    let ledger = board
        .borrow()
        .storage
        .read("qualification_ledger")
        .map(serde_json::from_slice)
        .transpose()?
        .unwrap_or_default();
    let restoration =
        (board.borrow().storage.boot_ordinal > 1).then_some(RestorationReason::Reboot);
    board.borrow_mut().network.connect()?;
    let seed = board.borrow_mut().scheduler.next_random();
    let session = Session {
        actuation: RuntimeBoardAdapter::new(board.clone(), scenario)?,
        ledger,
        reject_stop: scenario == "stop-rejection",
        fail_v2_snapshot: false,
        diagnostic_phases: RefCell::new(Vec::with_capacity(128)),
        seed,
        maybe_v2_record: Rc::new(RefCell::new(None)),
        maybe_encrypted_share: None,
    };
    WorkerControl::new(authority.identity, authority.verifier, session, restoration,
        FirmwareIdentity::new(FirmwareSourceCommit::parse(&"a".repeat(40)).map_err(|_| SimulationError::Boundary("identity"))?, &"b".repeat(64))
        .map_err(|_| SimulationError::Boundary("identity"))?,
        json!({"protocolVersion":"bwg-worker-controller/0.4","transportProfile":"bwg-worker-serial/0.2"}),
        &URL_SAFE_NO_PAD.encode([1;32])).map(Box::new).map_err(|_| SimulationError::Boundary("controller"))
}

pub(crate) fn frame(command: &str, payload: Value) -> Result<Vec<u8>, SimulationError> {
    let mut value = json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_simulation","command":command});
    if !payload.is_null() {
        value["payload"] = payload;
    }
    let mut bytes = serde_json::to_vec(&value)?;
    bytes.push(b'\n');
    Ok(bytes)
}
pub(crate) fn possession(worker: &mut Controller) -> Result<(String, Vec<u8>), SimulationError> {
    let session = URL_SAFE_NO_PAD.encode([2; 16]);
    let nonce = URL_SAFE_NO_PAD.encode([3; 32]);
    worker
        .begin_serial_session(
            bitaxe_worker_control::serial::SerialSessionBinding::parse(&session, &nonce, &nonce)
                .map_err(|_| SimulationError::Boundary("serial_binding"))?,
        )
        .map_err(|_| SimulationError::Boundary("serial_binding"))?;
    let request = json!({"profile":"bwg-worker-possession/0.2","requestId":"pos_simulation","command":"prove_possession",
        "payload":{"purpose":"initial_admission","possessionNonce":URL_SAFE_NO_PAD.encode([4;32]),
        "challengeBindingSha256":URL_SAFE_NO_PAD.encode([5;32]),"controllerCapabilitySha256":worker.capability_sha256(),
        "sessionId":session,"hostNonce":nonce,"deviceNonce":nonce,"serialManifestSha256":URL_SAFE_NO_PAD.encode([1;32])}});
    let mut bytes = serde_json::to_vec(&request)?;
    bytes.push(b'\n');
    let prepared = worker
        .prepare_frame(&bytes, worker.session().actuation.now_ms())
        .map_err(|e| SimulationError::Boundary(e.category()))?;
    let response: Value = serde_json::from_slice(prepared.frame())?;
    let transcript =
        json!({"profile":"bwg-worker-control-session/0.2","request":request,"response":response});
    let binding = URL_SAFE_NO_PAD.encode(Sha256::digest(serde_json::to_vec(&transcript)?));
    worker
        .confirm_sent(prepared)
        .map_err(|_| SimulationError::Boundary("possession_delivery"))?;
    Ok((binding, bytes))
}
