use super::*;
use crate::serial::ReceiveCreditMailbox;
use std::cell::{Cell, RefCell};

const SESSION: &str = "AAAAAAAAAAAAAAAAAAAAAA";

/// Device state over a real credit mailbox; effects are recorded in order.
#[derive(Default)]
struct View {
    now: Cell<u64>,
    current: Cell<u32>,
    partial: Cell<bool>,
    credit: ReceiveCreditMailbox,
    dropped: Cell<u32>,
    log: RefCell<Vec<String>>,
}

impl View {
    fn begin(&self, epoch: u32) {
        self.current.set(epoch);
        self.credit.begin(epoch);
    }
    fn lines(&self, prefix: &str) -> Vec<String> {
        self.log
            .borrow()
            .iter()
            .filter(|line| line.starts_with(prefix))
            .cloned()
            .collect()
    }
}

impl SerialWriterView for View {
    fn now_ms(&self) -> u64 {
        self.now.get()
    }
    fn current_epoch(&self) -> u32 {
        self.current.get()
    }
    fn has_partial_output(&self) -> bool {
        self.partial.get()
    }
    fn received_bytes(&self, epoch: u32) -> Option<u32> {
        self.credit.maybe_received_bytes(epoch)
    }
    fn terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        self.credit.maybe_terminal_bytes(epoch, now_ms)
    }
    fn finish_terminal(&self, epoch: u32) {
        self.log
            .borrow_mut()
            .push(format!("finish_terminal {epoch}"));
        self.credit.finish_terminal(epoch);
    }
    fn revoke_epoch(&self, epoch: u32) {
        self.log.borrow_mut().push(format!("revoke {epoch}"));
        if self.current.get() == epoch {
            self.current.set(0);
            self.credit.close(epoch);
        }
    }
    fn trace(&self, correlation: SerialTraceCorrelation, stage: SerialTraceStage, _: usize) {
        self.log.borrow_mut().push(format!(
            "trace {stage:?} {} {}",
            correlation.epoch, correlation.request_sequence
        ));
    }
    fn diagnostic_dropped(&self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

#[derive(Debug, PartialEq)]
enum SinkError {
    Transport,
    Cancelled,
}

/// Collects written records; can fail the nth write or revoke inside admission.
#[derive(Default)]
struct Sink {
    records: Vec<(Vec<u8>, SerialOutputPurpose<u8>)>,
    writes: u32,
    maybe_fail_write: Option<u32>,
    resynchronized: u32,
}

impl SerialOutputSink for Sink {
    type Error = SinkError;
    type DiagnosticKind = u8;
    fn write<A: Fn() -> bool>(
        &mut self,
        record: &[u8],
        admitted: A,
        _: Option<SerialTraceCorrelation>,
        purpose: SerialOutputPurpose<u8>,
    ) -> Result<(), SinkError> {
        self.writes += 1;
        if !admitted() {
            return Err(SinkError::Cancelled);
        }
        if self.maybe_fail_write == Some(self.writes) {
            return Err(SinkError::Transport);
        }
        self.records.push((record.to_vec(), purpose));
        Ok(())
    }
    fn resynchronize<A: Fn() -> bool>(&mut self, admitted: A) -> Result<(), SinkError> {
        if !admitted() {
            return Err(SinkError::Cancelled);
        }
        self.resynchronized += 1;
        Ok(())
    }
    fn is_transport_failure(error: &SinkError) -> bool {
        *error == SinkError::Transport
    }
}

impl Sink {
    fn envelopes(&self) -> Vec<SerialEnvelope> {
        self.records
            .iter()
            .filter(|(record, _)| record.first() == Some(&b'{'))
            .map(|(record, _)| SerialEnvelope::parse(record).expect("every framed record parses"))
            .collect()
    }
    fn sequences(&self) -> Vec<u32> {
        self.envelopes()
            .iter()
            .map(|envelope| envelope.sequence)
            .collect()
    }
}

fn open() -> (SerialRecordWriter, View, Sink) {
    let view = View::default();
    view.begin(1);
    let mut sink = Sink::default();
    let mut writer = SerialRecordWriter::new();
    writer
        .hello(
            1,
            SESSION.into(),
            &json!({"op": "hello_ack"}),
            &view,
            &mut sink,
        )
        .expect("current Hello")
        .expect("acknowledged");
    (writer, view, sink)
}

fn correlation(epoch: u32, request_sequence: u32) -> SerialTraceCorrelation {
    SerialTraceCorrelation {
        epoch,
        request_sequence,
    }
}

mod records;
mod session;
