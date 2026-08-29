//! Pure possession-bound BWG Worker-control state machine.

mod authorization;
mod codec;
mod controller;
mod effect_marker;
mod identity;
mod lease;
mod possession;
mod session;

pub use authorization::{
    AcceptedSequenceStore, LeaseAuthorizationError, SequenceStoreResult, WorkLeaseAuthorityTrust,
    WorkLeaseAuthorizationVerifier, WorkerLeaseAuthorizationContext,
};
pub use codec::{WorkerControlFrameAccumulator, WorkerControlFrameAccumulatorError};
pub use controller::{PreparedResponse, WorkerControl, WorkerControlError};
pub use effect_marker::PersistedWorkerEffectState;
pub use identity::{
    load_or_generate_device_identity, DeviceIdentity, DeviceIdentitySeedGenerator,
    DeviceIdentitySeedStore, IdentityLoadError,
};
pub use lease::{LeaseDeadlines, WorkerLeaseGrant, WorkerLeaseRenewal};
pub use possession::{PossessionRequest, PossessionResponse};
pub use session::{
    LeaseAuthorizationVerifier, RestorationReason, WorkerSession, WorkerSessionError,
};
