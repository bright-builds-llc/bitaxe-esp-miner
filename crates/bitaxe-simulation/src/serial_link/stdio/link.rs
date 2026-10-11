//! The real-time device's link and writer side: the firmware link and writer thread
//! passes over the shared receiver and writer cores, run between owner steps and, through
//! the pump, while the owner actuates.

use super::{StdioOutcome, StdioPort};
use crate::actuation::RuntimeBoardAdapter;
use crate::serial_link::device::{SimWriterView, WireSink};
use crate::serial_link::{LinkState, SimEffects};
use bitaxe_runtime::revocation::WorkerGeneration;
use bitaxe_worker_control::serial::link::SerialLinkReceiver;
use bitaxe_worker_control::serial::writer::SerialRecordWriter;
use std::cell::RefCell;

/// The device's link and writer side, shared with the pump that runs it while the
/// owner actuates. Its I/O is borrowed only for one link or writer pass at a time.
pub(super) struct DeviceLink {
    pub(super) state: LinkState,
    pub(super) io: RefCell<DeviceIo>,
    pub(super) outcome: RefCell<StdioOutcome>,
    pub(super) start_board_ms: u64,
}

pub(super) struct DeviceIo {
    pub(super) receiver: SerialLinkReceiver<WorkerGeneration>,
    pub(super) writer: SerialRecordWriter,
    pub(super) wire: Vec<u8>,
    pub(super) port: Box<dyn StdioPort>,
    pub(super) input_closed: bool,
}

impl DeviceLink {
    /// One pass of the firmware link and writer threads while the owner actuates: wait
    /// for wall time to reach board time, take the host bytes received by then, then
    /// write device output. Pending replies are written first, so order holds.
    pub(super) fn service(&self, actuation: &RuntimeBoardAdapter) {
        // The link never calls back into the owner, so a pass never nests.
        let Ok(mut io) = self.io.try_borrow_mut() else {
            return;
        };
        let now = actuation.now_ms();
        self.flush(&mut io, actuation);
        if !io.input_closed {
            let wall_ms = now.saturating_sub(self.start_board_ms);
            let input = io.port.input_until(wall_ms);
            let effects = SimEffects {
                state: &self.state,
                actuation,
            };
            io.receiver.supervise(now, &effects);
            if !input.bytes.is_empty() {
                io.receiver.receive(&input.bytes, &effects);
            }
            if input.closed {
                io.input_closed = true;
                io.receiver.read_failed(&effects);
            }
        }
        self.write_output(&mut io, actuation, now);
        self.flush(&mut io, actuation);
        self.note_link_failure();
    }

    pub(super) fn close_input(&self, actuation: &RuntimeBoardAdapter) {
        let mut io = self.io.borrow_mut();
        if io.input_closed {
            return;
        }
        io.input_closed = true;
        let effects = SimEffects {
            state: &self.state,
            actuation,
        };
        io.receiver.read_failed(&effects);
    }

    /// Writes buffered device bytes to the port; a host that stopped reading is gone.
    fn flush(&self, io: &mut DeviceIo, actuation: &RuntimeBoardAdapter) {
        if io.wire.is_empty() {
            return;
        }
        let bytes = std::mem::take(&mut io.wire);
        if io.port.output(&bytes) || io.input_closed {
            return;
        }
        io.input_closed = true;
        let effects = SimEffects {
            state: &self.state,
            actuation,
        };
        io.receiver.read_failed(&effects);
    }

    /// One writer pass: heartbeat, credit (including a terminal credit), a waiting ack.
    pub(super) fn write_output(
        &self,
        io: &mut DeviceIo,
        actuation: &RuntimeBoardAdapter,
        now: u64,
    ) {
        let view = SimWriterView {
            state: &self.state,
            actuation,
        };
        let mut outcome = self.outcome.borrow_mut();
        let mut sink = WireSink(&mut io.wire);
        if io.writer.heartbeat_if_due(now, &view, &mut sink).is_err() {
            outcome.output_failure("heartbeat");
        }
        let epoch = io.writer.epoch();
        let terminal_pending = epoch != 0
            && self.state.current.get() != epoch
            && self.state.credit.maybe_terminal_bytes(epoch, now).is_some();
        let before = sink.0.len();
        match io.writer.receive_credit(&view, &mut sink) {
            Ok(()) if terminal_pending && sink.0.len() > before => {
                outcome.terminal_credit_sent = true;
            }
            Ok(()) => {}
            Err(_) => outcome.output_failure("receive_credit"),
        }
        let maybe_hello = self.state.pending_hello.borrow_mut().take();
        if let Some((hello_epoch, session_id, payload)) = maybe_hello {
            match io
                .writer
                .hello(hello_epoch, session_id, &payload, &view, &mut sink)
            {
                Some(Ok(())) => outcome.hello_acknowledged = true,
                Some(Err(_)) => outcome.output_failure("hello_ack"),
                None => {}
            }
        }
    }

    pub(super) fn note_link_failure(&self) {
        if let Some(failure) = self.state.failure.get() {
            self.outcome
                .borrow_mut()
                .maybe_link_failure
                .get_or_insert_with(|| failure.label().to_owned());
        }
    }
}
