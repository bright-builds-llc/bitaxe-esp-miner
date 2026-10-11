//! A real-time simulated device for an external host: bytes in, bytes out.
//!
//! Each step mirrors one pass of the firmware's link, owner and writer loops: the
//! revocation deadline, link supervision and input, the Worker owner's frame
//! handling (reply, or rejection then revocation), and device output. Board time
//! follows the caller's wall clock and never regresses. The device advertises the
//! capability it is given and the real serial manifest digest, verifies leases with
//! the deployment trust it is given, and reports Gate-shaped qualification evidence,
//! so an unmodified Gate can admit it and run leases on it.
//!
//! As in firmware, each Hello begins its own revocation generation, and the link and
//! writer keep running while the owner actuates: the port is serviced at board time
//! during Start preparation and safe shutdown, so host heartbeats, Close and device
//! output are not held behind the owner. One logical session per device.

mod link;

use super::device::{SimWriterView, WireSink};
use super::{LinkState, OwnerEvent, SimEffects, DEVICE_ENTROPY};
use crate::actuation::LinkPump;
use crate::controller::{self, Controller, ControllerProfile, StatusEvidence};
use crate::SimulationError;
use bitaxe_runtime::mining_actuation::SafeShutdownStep;
use bitaxe_runtime::revocation::RevocationReason;
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};
use bitaxe_worker_control::serial::link::SerialLinkReceiver;
use bitaxe_worker_control::serial::serial_manifest_sha256;
use bitaxe_worker_control::serial::trace::SerialTraceCorrelation;
use bitaxe_worker_control::serial::writer::SerialRecordWriter;
use link::{DeviceIo, DeviceLink};
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// What happened on the device's one logical session; categories only, no secrets.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StdioOutcome {
    pub hello_acknowledged: bool,
    pub authenticated: bool,
    pub terminal_credit_sent: bool,
    pub frames_answered: u32,
    pub frames_rejected: u32,
    pub leases_started: u32,
    pub renewals: u32,
    pub restorations: u32,
    /// Restorations whose safe shutdown failed; the device keeps the lease to restore.
    pub rejected_restorations: u32,
    /// A reserved qualification attempt the device never finished restoring.
    pub lease_pending: bool,
    pub maybe_link_failure: Option<String>,
    pub maybe_output_failure: Option<String>,
    pub maybe_owner_failure: Option<String>,
}

impl StdioOutcome {
    /// Admitted, authenticated, restored and closed with a terminal credit, with no failure.
    #[must_use]
    pub fn clean(&self) -> bool {
        self.hello_acknowledged
            && self.authenticated
            && self.terminal_credit_sent
            && !self.lease_pending
            && self.maybe_link_failure.is_none()
            && self.maybe_output_failure.is_none()
            && self.maybe_owner_failure.is_none()
    }

    fn owner_failure(&mut self, category: &str) {
        self.maybe_owner_failure
            .get_or_insert_with(|| category.to_owned());
    }

    fn output_failure(&mut self, category: &str) {
        self.maybe_output_failure
            .get_or_insert_with(|| category.to_owned());
    }
}

/// Host bytes the port received by a wall-clock instant.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PortInput {
    pub bytes: Vec<u8>,
    /// The host closed its side; no later bytes arrive.
    pub closed: bool,
}

/// The host transport as the firmware link and writer threads see it while the owner
/// is busy. Wall time is milliseconds since the session began.
pub trait StdioPort {
    /// Waits until `wall_ms` and returns the host bytes received by then.
    fn input_until(&mut self, wall_ms: u64) -> PortInput;
    /// Writes device bytes; false once the host stopped reading.
    fn output(&mut self, bytes: &[u8]) -> bool;
}

/// What the device is built from; every input is runtime data, none compiled in.
pub struct StdioConfig {
    pub seed: u64,
    pub capability: Value,
    /// The deployment trust whose work-lease authority verifies leases.
    pub trust_json: String,
    pub maybe_fault: Option<StdioFault>,
}

/// A device-side fault a host-facing scenario injects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StdioFault {
    /// The first restoration of a started lease cannot prove cooling, so the device
    /// rejects it after removing power; a later restoration completes.
    RejectRestore,
}

pub struct StdioDevice {
    worker: Box<Controller>,
    link: Rc<DeviceLink>,
    pump: LinkPump,
    owner_epoch: u32,
}

impl StdioDevice {
    /// A fresh board whose controller advertises `config.capability`.
    pub fn new(config: StdioConfig, port: Box<dyn StdioPort>) -> Result<Self, SimulationError> {
        let board = Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
            seed: config.seed,
            ..BoardConfig::default()
        })?));
        let profile = ControllerProfile {
            capability: config.capability,
            manifest_sha256: serial_manifest_sha256()?,
            evidence: StatusEvidence::Qualification,
            maybe_trust_json: Some(config.trust_json),
        };
        let mut worker = controller::controller_with(board, "serial-stdio", profile)?;
        if config.maybe_fault == Some(StdioFault::RejectRestore) {
            worker.session_mut().maybe_restore_fault =
                Some(SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C);
        }
        let actuation = &mut worker.session_mut().actuation;
        actuation.host_heartbeats = false;
        // Firmware boots with no generation; each Hello begins one.
        let start_board_ms = actuation.now_ms();
        actuation.gate.revoke_reason_at(
            actuation.generation,
            start_board_ms,
            RevocationReason::LinkClosed,
        );
        let state = LinkState::default();
        state.device_entropy.set(config.seed ^ DEVICE_ENTROPY);
        state.real_time.set(true);
        let link = Rc::new(DeviceLink {
            state,
            io: RefCell::new(DeviceIo {
                receiver: SerialLinkReceiver::new(),
                writer: SerialRecordWriter::new(),
                wire: Vec::new(),
                port,
                input_closed: false,
            }),
            outcome: RefCell::new(StdioOutcome::default()),
            start_board_ms,
        });
        let serviced = Rc::clone(&link);
        let pump: LinkPump = Rc::new(move |actuation| serviced.service(actuation));
        Ok(Self {
            worker,
            link,
            pump,
            owner_epoch: 0,
        })
    }

    /// The canonical capability digest a host must bind into possession.
    #[must_use]
    pub fn capability_sha256(&self) -> &str {
        self.worker.capability_sha256()
    }

    #[must_use]
    pub fn outcome(&self) -> StdioOutcome {
        let session = self.worker.session();
        let mut outcome = self.link.outcome.borrow().clone();
        outcome.leases_started = session.starts;
        outcome.renewals = session.renewals;
        outcome.restorations = session.restorations;
        outcome.rejected_restorations = session.rejected_restorations;
        outcome.lease_pending = session.ledger.pending();
        outcome
    }

    /// Consumes host bytes received by `wall_ms` and returns the device's output.
    pub fn step(&mut self, input: &[u8], wall_ms: u64) -> Vec<u8> {
        let now = self.advance_to(wall_ms);
        self.worker.session().actuation.gate.check_deadline(now);
        self.release_revoked_owner();
        {
            let mut io = self.link.io.borrow_mut();
            let effects = SimEffects {
                state: &self.link.state,
                actuation: &self.worker.session().actuation,
            };
            io.receiver.supervise(now, &effects);
            if !input.is_empty() {
                io.receiver.receive(input, &effects);
            }
        }
        self.drain_owner();
        if input.is_empty() {
            if let Err(error) = self.owned(|worker, now| worker.tick(now)) {
                self.link
                    .outcome
                    .borrow_mut()
                    .owner_failure(error.category());
            }
        }
        self.release_revoked_owner();
        self.finish_pass()
    }

    /// The host closed its side: the link and any session end.
    pub fn input_closed(&mut self, wall_ms: u64) -> Vec<u8> {
        self.advance_to(wall_ms);
        self.link.close_input(&self.worker.session().actuation);
        self.release_revoked_owner();
        // A read failure after a clean Close is expected; only earlier failures count.
        self.finish_pass()
    }

    fn finish_pass(&mut self) -> Vec<u8> {
        let actuation = &self.worker.session().actuation;
        let mut io = self.link.io.borrow_mut();
        self.link
            .write_output(&mut io, actuation, actuation.now_ms());
        self.link.note_link_failure();
        std::mem::take(&mut io.wire)
    }

    fn advance_to(&mut self, wall_ms: u64) -> u64 {
        let target = self.link.start_board_ms.saturating_add(wall_ms);
        let actuation = &mut self.worker.session_mut().actuation;
        let current = actuation.now_ms();
        if target > current {
            if let Err(category) = actuation.advance(target - current, false) {
                self.link.outcome.borrow_mut().owner_failure(category);
            }
        }
        self.worker.session().actuation.now_ms()
    }

    /// Runs one owner call with the link and writer serviced beside it, at the board
    /// instant the call begins.
    fn owned<R>(&mut self, call: impl FnOnce(&mut Controller, u64) -> R) -> R {
        let now = self.worker.session().actuation.now_ms();
        self.worker.session_mut().actuation.maybe_pump = Some(Rc::clone(&self.pump));
        let result = call(&mut self.worker, now);
        self.worker.session_mut().actuation.maybe_pump = None;
        result
    }

    /// The firmware owner releases a session as soon as its epoch is revoked.
    fn release_revoked_owner(&mut self) {
        let state = &self.link.state;
        if self.owner_epoch == 0 || state.current.get() == self.owner_epoch {
            return;
        }
        if state.authenticated.get() == self.owner_epoch {
            state.authenticated.set(0);
        }
        if self.owned(|worker, now| worker.disconnect(now)).is_err() {
            self.link
                .outcome
                .borrow_mut()
                .owner_failure("restoration_pending");
        }
        self.owner_epoch = 0;
    }

    fn drain_owner(&mut self) {
        loop {
            let Some(event) = self.link.state.owner.borrow_mut().pop_front() else {
                return;
            };
            match event {
                OwnerEvent::Session(binding, generation) => {
                    let epoch = self.link.state.current.get();
                    if self.owner_epoch != 0 || self.link.outcome.borrow().authenticated {
                        self.link
                            .outcome
                            .borrow_mut()
                            .owner_failure("second_session_unsupported");
                        self.link.state.revoke(epoch);
                        continue;
                    }
                    if self.worker.begin_serial_session(binding).is_err() {
                        self.link.state.revoke(epoch);
                        continue;
                    }
                    self.worker.session_mut().actuation.generation = generation;
                    self.owner_epoch = epoch;
                }
                OwnerEvent::Frame(correlation, bytes) => {
                    let current = self.link.state.current.get();
                    if correlation.epoch != self.owner_epoch || current != self.owner_epoch {
                        continue;
                    }
                    self.process_frame(correlation, &bytes);
                }
            }
        }
    }

    /// Mirrors firmware `process_frame`: a rejection is answered, then revokes the epoch.
    fn process_frame(&mut self, correlation: SerialTraceCorrelation, bytes: &[u8]) {
        let epoch = correlation.epoch;
        let link = Rc::clone(&self.link);
        let state = &link.state;
        let response = match self.owned(|worker, now| worker.prepare_frame(bytes, now)) {
            Ok(response) => response,
            Err(error) => {
                self.link.outcome.borrow_mut().frames_rejected += 1;
                if state.current.get() == epoch {
                    if let Ok(rejection) = self.worker.prepare_rejection(bytes, &error) {
                        let frame = rejection.frame().to_vec();
                        self.send_reply(correlation, &frame);
                    }
                }
                state.revoke(epoch);
                return;
            }
        };
        if state.current.get() != epoch {
            return;
        }
        let frame = response.frame().to_vec();
        let sent = self.send_reply(correlation, &frame);
        if !sent
            || state.current.get() != epoch
            || self
                .owned(|worker, now| worker.confirm_sent_at(response, now))
                .is_err()
        {
            state.revoke(epoch);
            return;
        }
        let mut outcome = self.link.outcome.borrow_mut();
        outcome.frames_answered += 1;
        if self.worker.is_admitted() && state.current.get() == epoch {
            state.authenticated.set(epoch);
            outcome.authenticated = true;
        }
    }

    fn send_reply(&mut self, correlation: SerialTraceCorrelation, frame: &[u8]) -> bool {
        let actuation = &self.worker.session().actuation;
        let mut io = self.link.io.borrow_mut();
        let io = &mut *io;
        let view = SimWriterView {
            state: &self.link.state,
            actuation,
        };
        let result = io.writer.control(
            correlation,
            frame,
            |_| true,
            &view,
            &mut WireSink(&mut io.wire),
        );
        if result.is_err() {
            self.link
                .outcome
                .borrow_mut()
                .output_failure("control_reply");
        }
        result.is_ok()
    }
}

#[cfg(test)]
mod tests;
