//! Closed, value-free breadcrumbs for the firmware's private fault recorder.

/// Last control-owner boundary entered; this is not an allocation call stack.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlDiagnosticPhase {
    Idle = 0,
    FrameParse = 1,
    ControlDispatch = 2,
    StartPrepare = 3,
    RenewPrepare = 4,
    V2Snapshot = 5,
    V2Value = 6,
    ReplySerialize = 7,
    ReplyQueued = 8,
    Cleanup = 9,
}
