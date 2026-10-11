//! A real-time simulated device for an external host: bytes in, bytes out.
//!
//! Each step mirrors one pass of the firmware's link, owner and writer loops: the
//! revocation deadline, link supervision and input, the Worker owner's frame
//! handling (reply, or rejection then revocation), and device output. Board time
//! follows the caller's wall clock and never regresses. The device advertises the
//! capability it is given and the real serial manifest digest, and reports no
//! simulation status evidence, so an unmodified Gate can admit it. One logical
//! session per device: the board begins its revocation generation once.

use super::device::{SimWriterView, WireSink};
use super::{LinkState, OwnerEvent, SimEffects, DEVICE_ENTROPY};
use crate::controller::{self, Controller, ControllerProfile};
use crate::SimulationError;
use bitaxe_runtime::revocation::WorkerGeneration;
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};
use bitaxe_worker_control::serial::link::SerialLinkReceiver;
use bitaxe_worker_control::serial::serial_manifest_sha256;
use bitaxe_worker_control::serial::trace::SerialTraceCorrelation;
use bitaxe_worker_control::serial::writer::SerialRecordWriter;
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
    pub maybe_link_failure: Option<String>,
    pub maybe_output_failure: Option<String>,
    pub maybe_owner_failure: Option<String>,
}

impl StdioOutcome {
    /// Admitted, authenticated and closed with a terminal credit, with no failure.
    #[must_use]
    pub fn clean(&self) -> bool {
        self.hello_acknowledged
            && self.authenticated
            && self.terminal_credit_sent
            && self.maybe_link_failure.is_none()
            && self.maybe_output_failure.is_none()
            && self.maybe_owner_failure.is_none()
    }

    fn owner_failure(&mut self, category: &str) {
        self.maybe_owner_failure
            .get_or_insert_with(|| category.to_owned());
    }
}

pub struct StdioDevice {
    worker: Box<Controller>,
    receiver: SerialLinkReceiver<WorkerGeneration>,
    state: LinkState,
    writer: SerialRecordWriter,
    wire: Vec<u8>,
    start_board_ms: u64,
    owner_epoch: u32,
    outcome: StdioOutcome,
}

impl StdioDevice {
    /// A fresh board whose controller advertises `capability`.
    pub fn new(seed: u64, capability: Value) -> Result<Self, SimulationError> {
        let board = Rc::new(RefCell::new(VirtualBoard::new(BoardConfig {
            seed,
            ..BoardConfig::default()
        })?));
        let profile = ControllerProfile {
            capability,
            manifest_sha256: serial_manifest_sha256()?,
            report_status_evidence: false,
        };
        let mut worker = controller::controller_with(board, "serial-stdio", profile)?;
        worker.session_mut().actuation.host_heartbeats = false;
        let state = LinkState::default();
        state.device_entropy.set(seed ^ DEVICE_ENTROPY);
        state.apply_heartbeats.set(true);
        let start_board_ms = worker.session().actuation.now_ms();
        Ok(Self {
            worker,
            receiver: SerialLinkReceiver::new(),
            state,
            writer: SerialRecordWriter::new(),
            wire: Vec::new(),
            start_board_ms,
            owner_epoch: 0,
            outcome: StdioOutcome::default(),
        })
    }

    /// The canonical capability digest a host must bind into possession.
    #[must_use]
    pub fn capability_sha256(&self) -> &str {
        self.worker.capability_sha256()
    }

    #[must_use]
    pub fn outcome(&self) -> &StdioOutcome {
        &self.outcome
    }

    /// Consumes host bytes received by `wall_ms` and returns the device's output.
    pub fn step(&mut self, input: &[u8], wall_ms: u64) -> Vec<u8> {
        let now = self.advance_to(wall_ms);
        self.worker.session().actuation.gate.check_deadline(now);
        self.release_revoked_owner(now);
        {
            let effects = SimEffects {
                state: &self.state,
                actuation: &self.worker.session().actuation,
            };
            self.receiver.supervise(now, &effects);
            if !input.is_empty() {
                self.receiver.receive(input, &effects);
            }
        }
        self.drain_owner(now);
        if input.is_empty() {
            if let Err(error) = self.worker.tick(now) {
                self.outcome.owner_failure(error.category());
            }
        }
        self.release_revoked_owner(now);
        self.write_output(now);
        self.note_link_failure();
        std::mem::take(&mut self.wire)
    }

    /// The host closed its side: the link and any session end.
    pub fn input_closed(&mut self, wall_ms: u64) -> Vec<u8> {
        let now = self.advance_to(wall_ms);
        {
            let effects = SimEffects {
                state: &self.state,
                actuation: &self.worker.session().actuation,
            };
            self.receiver.read_failed(&effects);
        }
        self.release_revoked_owner(now);
        self.write_output(now);
        // A read failure after a clean Close is expected; only earlier failures count.
        std::mem::take(&mut self.wire)
    }

    fn advance_to(&mut self, wall_ms: u64) -> u64 {
        let target = self.start_board_ms.saturating_add(wall_ms);
        let actuation = &mut self.worker.session_mut().actuation;
        let current = actuation.now_ms();
        if target > current {
            if let Err(category) = actuation.advance(target - current, false) {
                self.outcome.owner_failure(category);
            }
        }
        self.worker.session().actuation.now_ms()
    }

    /// The firmware owner releases a session as soon as its epoch is revoked.
    fn release_revoked_owner(&mut self, now: u64) {
        if self.owner_epoch == 0 || self.state.current.get() == self.owner_epoch {
            return;
        }
        if self.state.authenticated.get() == self.owner_epoch {
            self.state.authenticated.set(0);
        }
        if self.worker.disconnect(now).is_err() {
            self.outcome.owner_failure("restoration_pending");
        }
        self.owner_epoch = 0;
    }

    fn drain_owner(&mut self, now: u64) {
        loop {
            let Some(event) = self.state.owner.borrow_mut().pop_front() else {
                return;
            };
            match event {
                OwnerEvent::Session(binding) => {
                    let epoch = self.state.current.get();
                    if self.owner_epoch != 0 || self.outcome.authenticated {
                        self.outcome.owner_failure("second_session_unsupported");
                        self.state.revoke(epoch);
                        continue;
                    }
                    if self.worker.begin_serial_session(binding).is_err() {
                        self.state.revoke(epoch);
                        continue;
                    }
                    self.owner_epoch = epoch;
                }
                OwnerEvent::Frame(correlation, bytes) => {
                    let current = self.state.current.get();
                    if correlation.epoch != self.owner_epoch || current != self.owner_epoch {
                        continue;
                    }
                    self.process_frame(correlation, &bytes, now);
                }
            }
        }
    }

    /// Mirrors firmware `process_frame`: a rejection is answered, then revokes the epoch.
    fn process_frame(&mut self, correlation: SerialTraceCorrelation, bytes: &[u8], now: u64) {
        let epoch = correlation.epoch;
        let response = match self.worker.prepare_frame(bytes, now) {
            Ok(response) => response,
            Err(error) => {
                self.outcome.frames_rejected += 1;
                if self.state.current.get() == epoch {
                    if let Ok(rejection) = self.worker.prepare_rejection(bytes, &error) {
                        let frame = rejection.frame().to_vec();
                        self.send_reply(correlation, &frame);
                    }
                }
                self.state.revoke(epoch);
                return;
            }
        };
        if self.state.current.get() != epoch {
            return;
        }
        let frame = response.frame().to_vec();
        let sent = self.send_reply(correlation, &frame);
        if !sent
            || self.state.current.get() != epoch
            || self.worker.confirm_sent_at(response, now).is_err()
        {
            self.state.revoke(epoch);
            return;
        }
        self.outcome.frames_answered += 1;
        if self.worker.is_admitted() && self.state.current.get() == epoch {
            self.state.authenticated.set(epoch);
            self.outcome.authenticated = true;
        }
    }

    fn send_reply(&mut self, correlation: SerialTraceCorrelation, frame: &[u8]) -> bool {
        let view = SimWriterView {
            state: &self.state,
            actuation: &self.worker.session().actuation,
        };
        let mut sink = WireSink(&mut self.wire);
        let result = self
            .writer
            .control(correlation, frame, |_| true, &view, &mut sink);
        if result.is_err() {
            self.outcome
                .maybe_output_failure
                .get_or_insert_with(|| "control_reply".into());
        }
        result.is_ok()
    }

    /// One writer pass: heartbeat, credit (including a terminal credit), a waiting ack.
    fn write_output(&mut self, now: u64) {
        let view = SimWriterView {
            state: &self.state,
            actuation: &self.worker.session().actuation,
        };
        let mut sink = WireSink(&mut self.wire);
        if self.writer.heartbeat_if_due(now, &view, &mut sink).is_err() {
            self.outcome
                .maybe_output_failure
                .get_or_insert_with(|| "heartbeat".into());
        }
        let epoch = self.writer.epoch();
        let terminal_pending = epoch != 0
            && self.state.current.get() != epoch
            && self.state.credit.maybe_terminal_bytes(epoch, now).is_some();
        let before = sink.0.len();
        match self.writer.receive_credit(&view, &mut sink) {
            Ok(()) if terminal_pending && sink.0.len() > before => {
                self.outcome.terminal_credit_sent = true;
            }
            Ok(()) => {}
            Err(_) => {
                self.outcome
                    .maybe_output_failure
                    .get_or_insert_with(|| "receive_credit".into());
            }
        }
        let maybe_hello = self.state.pending_hello.borrow_mut().take();
        if let Some((hello_epoch, session_id, payload)) = maybe_hello {
            match self
                .writer
                .hello(hello_epoch, session_id, &payload, &view, &mut sink)
            {
                Some(Ok(())) => self.outcome.hello_acknowledged = true,
                Some(Err(_)) => {
                    self.outcome
                        .maybe_output_failure
                        .get_or_insert_with(|| "hello_ack".into());
                }
                None => {}
            }
        }
    }

    fn note_link_failure(&mut self) {
        if let Some(failure) = self.state.failure.get() {
            self.outcome
                .maybe_link_failure
                .get_or_insert_with(|| failure.label().to_owned());
        }
    }
}

#[cfg(test)]
mod tests;
