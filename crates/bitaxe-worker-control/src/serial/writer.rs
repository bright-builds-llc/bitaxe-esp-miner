//! Outbound serial record decisions shared by the firmware writer thread and host simulation.
//!
//! One writer owns the device sequence counter for every record kind: the Hello
//! acknowledgement at sequence 0, heartbeats, receive credit, control replies and
//! framed diagnostics. Output admission is re-checked by the sink during each
//! bounded write, so a revoked epoch never completes a record. Device state is
//! read through a shared [`SerialWriterView`] while the sole [`SerialOutputSink`]
//! is mutably borrowed; both are statically dispatched.

use super::trace::{SerialTraceCorrelation, SerialTraceStage};
use super::{SerialEnvelope, SerialError, SerialKind};
use serde_json::{json, value::RawValue, Value};
use zeroize::Zeroizing;

/// Device heartbeat cadence while a session is current.
pub const DEVICE_HEARTBEAT_INTERVAL_MILLISECONDS: u64 = 1000;
/// Control payloads longer than this get a fresh heartbeat before the indivisible record.
pub const LONG_RECORD_HEARTBEAT_BYTES: usize = 4096;

/// Where a diagnostic line is going and why. Declaration order is a stable table index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticOrigin {
    /// Plain text outside a current session.
    Bootstrap,
    /// A framed record of the current session.
    Session,
    /// A retained marker repeated on the replay schedule.
    Replay,
}

/// A wire-protocol record; fieldless so sinks can map it by table lookup.
/// Declaration order is a stable table index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolRecord {
    Hello,
    Heartbeat,
    ReceiveCredit,
    ControlReply,
}

/// What a record is for; sinks map it onto their own measurement categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialOutputPurpose<K> {
    Protocol(ProtocolRecord),
    Diagnostic(DiagnosticOrigin, K),
}

/// Shared device state and side effects, readable while the sink writes.
pub trait SerialWriterView {
    fn now_ms(&self) -> u64;
    fn current_epoch(&self) -> u32;
    fn has_partial_output(&self) -> bool;
    fn received_bytes(&self, epoch: u32) -> Option<u32>;
    fn terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32>;
    fn finish_terminal(&self, epoch: u32);
    fn revoke_epoch(&self, epoch: u32);
    fn trace(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    );
    fn diagnostic_dropped(&self);
}

/// The sole transport writer; `admitted` must be re-checked at every bounded attempt.
pub trait SerialOutputSink {
    type Error;
    type DiagnosticKind: Copy;

    /// `maybe_correlation` is `None` for plain lines, which report no write observations.
    fn write<A: Fn() -> bool>(
        &mut self,
        record: &[u8],
        admitted: A,
        maybe_correlation: Option<SerialTraceCorrelation>,
        purpose: SerialOutputPurpose<Self::DiagnosticKind>,
    ) -> Result<(), Self::Error>;
    /// Terminates an interrupted line before a fresh Hello acknowledgement.
    fn resynchronize<A: Fn() -> bool>(&mut self, admitted: A) -> Result<(), Self::Error>;
    /// Transport failures are already traced as abandoned writes.
    fn is_transport_failure(error: &Self::Error) -> bool;
}

/// Why a record was not written.
#[derive(Debug)]
pub enum SerialWriteError<E> {
    /// The record's epoch lost output admission.
    Revoked,
    SequenceExhausted,
    Payload(serde_json::Error),
    Encoding(SerialError),
    Sink(E),
}

impl<E> SerialWriteError<E> {
    #[must_use]
    pub fn maybe_sink(&self) -> Option<&E> {
        match self {
            Self::Sink(error) => Some(error),
            _ => None,
        }
    }
}

type WriteResult<E> = Result<(), SerialWriteError<E>>;

#[derive(Clone, Copy)]
struct OutputAdmission {
    epoch: u32,
    terminal: bool,
}

impl OutputAdmission {
    fn active(epoch: u32) -> Self {
        Self {
            epoch,
            terminal: false,
        }
    }

    fn permits<V: SerialWriterView>(self, view: &V) -> bool {
        let current = view.current_epoch();
        if self.terminal {
            return current == 0
                && !view.has_partial_output()
                && view.terminal_bytes(self.epoch, view.now_ms()).is_some();
        }
        self.epoch != 0 && current == self.epoch
    }
}

/// Per-session output state: epoch, session, sequence, credit and heartbeat clock.
#[derive(Default)]
pub struct SerialRecordWriter {
    epoch: u32,
    session_id: String,
    sequence: u32,
    credited_bytes: u32,
    last_heartbeat_ms: u64,
}

impl SerialRecordWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn epoch(&self) -> u32 {
        self.epoch
    }

    /// When the next heartbeat falls due, if a session was ever acknowledged.
    #[must_use]
    pub fn maybe_next_heartbeat_ms(&self) -> Option<u64> {
        (self.epoch != 0).then(|| self.last_heartbeat_ms + DEVICE_HEARTBEAT_INTERVAL_MILLISECONDS)
    }

    /// Acknowledges a Hello; `None` means it was stale and nothing changed.
    /// A failed acknowledgement revokes the new epoch.
    pub fn hello<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        epoch: u32,
        session_id: String,
        payload: &Value,
        view: &V,
        sink: &mut S,
    ) -> Option<WriteResult<S::Error>> {
        if view.current_epoch() != epoch {
            return None;
        }
        self.epoch = epoch;
        self.session_id = session_id;
        self.sequence = 0;
        self.credited_bytes = 0;
        self.last_heartbeat_ms = view.now_ms();
        let result = self.acknowledge(payload, view, sink);
        if result.is_err() {
            view.revoke_epoch(epoch);
        }
        Some(result)
    }

    fn acknowledge<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        payload: &Value,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        let raw = serde_json::value::to_raw_value(payload).map_err(SerialWriteError::Payload)?;
        let admission = OutputAdmission::active(self.epoch);
        sink.resynchronize(|| admission.permits(view))
            .map_err(SerialWriteError::Sink)?;
        self.emit(
            admission,
            SerialKind::Session,
            0,
            &raw,
            None,
            SerialOutputPurpose::Protocol(ProtocolRecord::Hello),
            view,
            sink,
        )
    }

    /// Sends a heartbeat when one is due; a failure revokes the epoch.
    pub fn heartbeat_if_due<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        now_ms: u64,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        if self.epoch == 0
            || view.current_epoch() != self.epoch
            || now_ms.saturating_sub(self.last_heartbeat_ms)
                < DEVICE_HEARTBEAT_INTERVAL_MILLISECONDS
        {
            return Ok(());
        }
        self.last_heartbeat_ms = now_ms;
        let result = self.next_record(
            OutputAdmission::active(self.epoch),
            SerialKind::Heartbeat,
            b"{}",
            None,
            SerialOutputPurpose::Protocol(ProtocolRecord::Heartbeat),
            view,
            sink,
        );
        if result.is_err() {
            view.revoke_epoch(self.epoch);
        }
        result
    }

    /// Sends receive credit, or the closing epoch's terminal credit; a failure revokes the epoch.
    pub fn receive_credit<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        let result = self.send_receive_credit(view, sink);
        if result.is_err() {
            view.revoke_epoch(self.epoch);
        }
        result
    }

    fn send_receive_credit<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        let admission = OutputAdmission {
            epoch: self.epoch,
            terminal: view.current_epoch() != self.epoch,
        };
        let maybe_received = if admission.terminal {
            view.terminal_bytes(self.epoch, view.now_ms())
        } else {
            view.received_bytes(self.epoch)
        };
        let Some(received) = maybe_received else {
            return Ok(());
        };
        if received <= self.credited_bytes || !admission.permits(view) {
            if admission.terminal {
                view.finish_terminal(self.epoch);
            }
            return Ok(());
        }
        let payload = serde_json::to_vec(&json!({
            "op": "receive_credit", "receivedBytes": received,
        }))
        .map_err(SerialWriteError::Payload)?;
        let result = self.next_record(
            admission,
            SerialKind::Session,
            &payload,
            None,
            SerialOutputPurpose::Protocol(ProtocolRecord::ReceiveCredit),
            view,
            sink,
        );
        if admission.terminal {
            view.finish_terminal(self.epoch);
        }
        result?;
        self.credited_bytes = received;
        Ok(())
    }

    /// Writes one control reply after any due long-record heartbeat and credit.
    ///
    /// `deliver_receipt` reports whether the reply was sent and returns whether the
    /// requester still listened; an unsent or unheard reply revokes its epoch.
    pub fn control<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        correlation: SerialTraceCorrelation,
        payload: &[u8],
        deliver_receipt: impl FnOnce(bool) -> bool,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        let wanted = correlation.epoch;
        let current = wanted == self.epoch && view.current_epoch() == self.epoch;
        if !current {
            view.trace(correlation, SerialTraceStage::WriterRejected, 0);
        }
        let mut result = Ok(());
        if current && payload.len() > LONG_RECORD_HEARTBEAT_BYTES {
            // An indivisible long record gets a fresh peer deadline before transmission.
            self.last_heartbeat_ms = view.now_ms();
            result = self.next_record(
                OutputAdmission::active(self.epoch),
                SerialKind::Heartbeat,
                b"{}",
                None,
                SerialOutputPurpose::Protocol(ProtocolRecord::Heartbeat),
                view,
                sink,
            );
        }
        if current && result.is_ok() {
            result = self.send_receive_credit(view, sink);
        }
        if current && result.is_ok() {
            result = self.next_record(
                OutputAdmission::active(self.epoch),
                SerialKind::Control,
                payload,
                Some(correlation),
                SerialOutputPurpose::Protocol(ProtocolRecord::ControlReply),
                view,
                sink,
            );
        }
        if let Err(error) = &result {
            if !error.maybe_sink().is_some_and(S::is_transport_failure) {
                view.trace(correlation, SerialTraceStage::WriterRejected, 0);
            }
        }
        let sent = current && result.is_ok() && OutputAdmission::active(self.epoch).permits(view);
        if !deliver_receipt(sent) || !sent {
            view.revoke_epoch(wanted);
        }
        result
    }

    /// Writes one diagnostic line: plain text outside a current session, else framed.
    ///
    /// Callers revoke the writer epoch once per failed diagnostic step.
    pub fn diagnostic_line<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        line: &str,
        kind: S::DiagnosticKind,
        replay: bool,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        let origin = if replay {
            DiagnosticOrigin::Replay
        } else if self.epoch == 0 || view.current_epoch() != self.epoch {
            DiagnosticOrigin::Bootstrap
        } else {
            DiagnosticOrigin::Session
        };
        let purpose = SerialOutputPurpose::Diagnostic(origin, kind);
        if view.current_epoch() != self.epoch || self.epoch == 0 {
            let line = format!("{line}\n");
            let result = sink.write(line.as_bytes(), || view.current_epoch() == 0, None, purpose);
            if result.is_err() {
                view.diagnostic_dropped();
            }
            return result.map_err(SerialWriteError::Sink);
        }
        let payload =
            serde_json::to_vec(&json!({"line": line})).map_err(SerialWriteError::Payload)?;
        self.next_record(
            OutputAdmission::active(self.epoch),
            SerialKind::Diagnostic,
            &payload,
            None,
            purpose,
            view,
            sink,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn next_record<V: SerialWriterView, S: SerialOutputSink>(
        &mut self,
        admission: OutputAdmission,
        kind: SerialKind,
        payload: &[u8],
        maybe_correlation: Option<SerialTraceCorrelation>,
        purpose: SerialOutputPurpose<S::DiagnosticKind>,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        if !admission.permits(view) {
            return Err(SerialWriteError::Revoked);
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or(SerialWriteError::SequenceExhausted)?;
        // Borrowed, so validating the payload copies nothing.
        let raw: &RawValue = serde_json::from_slice(payload).map_err(SerialWriteError::Payload)?;
        self.emit(
            admission,
            kind,
            self.sequence,
            raw,
            maybe_correlation,
            purpose,
            view,
            sink,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn emit<V: SerialWriterView, S: SerialOutputSink>(
        &self,
        admission: OutputAdmission,
        kind: SerialKind,
        sequence: u32,
        payload: &RawValue,
        maybe_correlation: Option<SerialTraceCorrelation>,
        purpose: SerialOutputPurpose<S::DiagnosticKind>,
        view: &V,
        sink: &mut S,
    ) -> WriteResult<S::Error> {
        if !admission.permits(view) {
            return Err(SerialWriteError::Revoked);
        }
        let bytes = Zeroizing::new(
            SerialEnvelope::encode(kind, Some(&self.session_id), sequence, payload)
                .map_err(SerialWriteError::Encoding)?,
        );
        let correlation = maybe_correlation.unwrap_or(SerialTraceCorrelation {
            epoch: admission.epoch,
            request_sequence: 0,
        });
        view.trace(correlation, SerialTraceStage::WriterStarted, bytes.len());
        sink.write(
            &bytes,
            || admission.permits(view),
            Some(correlation),
            purpose,
        )
        .map_err(SerialWriteError::Sink)
    }

    #[cfg(test)]
    pub(crate) fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = sequence;
        self
    }
}

#[cfg(test)]
mod tests;
