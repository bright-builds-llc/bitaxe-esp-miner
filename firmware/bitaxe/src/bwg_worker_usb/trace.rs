//! Boot-local serial trace; no unsolicited output, durable writes, or authority effects.
use bitaxe_worker_control::serial::trace::{SerialTrace, SerialTraceCorrelation, SerialTraceStage};

pub(super) static TRACE: SerialTrace = SerialTrace::new();

pub(super) fn event(
    correlation: SerialTraceCorrelation,
    stage: SerialTraceStage,
    wire_bytes: usize,
) {
    TRACE.record(
        correlation,
        stage,
        crate::runtime_uptime::millis(),
        wire_bytes,
        0,
    );
}

pub(super) fn observe(
    correlation: SerialTraceCorrelation,
    observation: crate::usb_runtime::WriteObservation,
) {
    observe_in(&TRACE, correlation, observation);
}

pub(super) fn observe_in(
    trace: &SerialTrace,
    correlation: SerialTraceCorrelation,
    observation: crate::usb_runtime::WriteObservation,
) {
    use crate::usb_runtime::WriteObservationStage;
    let stage = match observation.stage {
        WriteObservationStage::Queued => SerialTraceStage::WriterQueued,
        WriteObservationStage::Completed => SerialTraceStage::WriterCompleted,
        WriteObservationStage::Abandoned => SerialTraceStage::WriterAbandoned,
    };
    trace.record(
        correlation,
        stage,
        observation.at_ms,
        observation.record_bytes,
        observation.queued_bytes,
    );
}

pub(crate) fn snapshot() -> bitaxe_worker_control::serial::trace::SerialTraceSnapshot {
    TRACE.snapshot()
}
