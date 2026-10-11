//! The simulated device's output side: the shared writer core over board state and a byte buffer.

use super::LinkState;
use crate::actuation::RuntimeBoardAdapter;
use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
use bitaxe_worker_control::serial::writer::{
    SerialOutputPurpose, SerialOutputSink, SerialWriterView,
};

/// Device state seen by the writer; replayed instants override the board clock.
pub(super) struct SimWriterView<'a> {
    pub(super) state: &'a LinkState,
    pub(super) actuation: &'a RuntimeBoardAdapter,
}

impl SerialWriterView for SimWriterView<'_> {
    fn now_ms(&self) -> u64 {
        self.state
            .maybe_replay_ms
            .get()
            .unwrap_or_else(|| self.actuation.now_ms())
    }
    fn current_epoch(&self) -> u32 {
        self.state.current.get()
    }
    /// Simulated writes are never interrupted mid-record.
    fn has_partial_output(&self) -> bool {
        false
    }
    fn received_bytes(&self, epoch: u32) -> Option<u32> {
        self.state.credit.maybe_received_bytes(epoch)
    }
    fn terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        self.state.credit.maybe_terminal_bytes(epoch, now_ms)
    }
    fn finish_terminal(&self, epoch: u32) {
        self.state.credit.finish_terminal(epoch);
    }
    fn revoke_epoch(&self, epoch: u32) {
        self.state.revoke(epoch);
    }
    fn trace(&self, _: SerialTraceCorrelation, _: SerialTraceStage, _: usize) {}
    fn diagnostic_dropped(&self) {}
}

/// The admission check at the start and end of a record stands in for the
/// transport's per-chunk checks; an inadmissible record writes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Cancelled;

pub(super) struct WireSink<'a>(pub(super) &'a mut Vec<u8>);

impl SerialOutputSink for WireSink<'_> {
    type Error = Cancelled;
    type DiagnosticKind = ();

    fn write<A: Fn() -> bool>(
        &mut self,
        record: &[u8],
        admitted: A,
        _: Option<SerialTraceCorrelation>,
        _: SerialOutputPurpose<()>,
    ) -> Result<(), Cancelled> {
        if !admitted() {
            return Err(Cancelled);
        }
        self.0.extend_from_slice(record);
        if !admitted() {
            return Err(Cancelled);
        }
        Ok(())
    }
    fn resynchronize<A: Fn() -> bool>(&mut self, _: A) -> Result<(), Cancelled> {
        Ok(())
    }
    fn is_transport_failure(_: &Cancelled) -> bool {
        true
    }
}
