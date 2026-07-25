//! Exclusive Phase 36 effect capability and independently owned typed ledger.

mod contract;
mod hardware;
mod hardware_process;
mod ipc;
mod ledger;

pub use contract::{
    Phase36AllowedOperation, Phase36BrokerCapability, Phase36BrokerFailure, Phase36CapabilityError,
    Phase36CapabilityGuard, Phase36CapabilityPresentation, Phase36CapabilityScope,
    Phase36RecoveryDisposition, Phase36ValidatedCapability,
};
pub use hardware::{
    run_phase36_hardware_pre_capture_gate, Phase36HardwareDisposition, Phase36HardwareGateError,
    Phase36HardwareGateStatus, Phase36HardwareTransactionError,
};
pub use hardware_process::run_phase36_hardware_transaction;
pub use ipc::{write_broker_frame, Phase36BrokerFrameReceiver, Phase36BrokerIpcError};
pub use ledger::{
    Phase36EffectInterval, Phase36LedgerError, Phase36LedgerRecord, Phase36LedgerState,
    Phase36LedgerTransition, PrivateAppendOnlyLedger,
};

#[cfg(test)]
mod tests;
