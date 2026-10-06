//! Pure possession-bound BWG Worker-control state machine.

mod authorization;
mod codec;
mod controller;
mod effect_marker;
mod identity;
mod lease;
mod possession;
mod preservation;
pub use preservation::{SettingsPreservation, StateFingerprint};
pub mod cadence;
pub mod noise;
mod restart;
pub mod serial;
mod session;
pub mod v2;
pub use restart::{QualificationResetKind, QualificationRestartContext};

pub use authorization::{
    AcceptedSequenceStore, LeaseAuthorizationError, SequenceStoreResult, WorkLeaseAuthorityTrust,
    WorkLeaseAuthorizationVerifier, WorkerLeaseAuthorizationContext,
};
pub use controller::{PreparedResponse, WorkerControl, WorkerControlError};
pub use effect_marker::PersistedWorkerEffectState;
pub use identity::{
    load_or_generate_device_identity, DeviceIdentity, DeviceIdentitySeedGenerator,
    DeviceIdentitySeedStore, IdentityLoadError,
};
pub use lease::{AcceptanceCampaign, LeaseDeadlines, WorkerLeaseGrant, WorkerLeaseRenewal};
pub use possession::{
    FirmwareIdentity, FirmwareSourceCommit, PossessionRequest, PossessionResponse,
};
mod diagnostic_phase;
pub use diagnostic_phase::ControlDiagnosticPhase;
pub use session::{
    LeaseAuthorizationVerifier, RestorationReason, WorkerSession, WorkerSessionError,
};

mod soak;
pub use soak::{
    HardwareProfile, SoakAllowance, SoakLedger, SoakRejected, SOAK_MAXIMUM_ACTIVE_MS,
    SOAK_WORK_GATE_MS,
};

mod qualification;
pub use qualification::{
    QualificationAttempt, QualificationLedger, QualificationPurpose, QualificationRejected,
};
