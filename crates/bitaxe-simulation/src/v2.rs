//! Real Noise and production generic SV2 session against independent fixture facts.
//! This public genesis vector does not qualify the strict live regtest profile.
mod exchange;
mod scenario;
#[cfg(test)]
mod tests;

pub use exchange::synthetic_authority_public_key;
pub use scenario::{
    run_encrypted_share_on_board, run_encrypted_share_on_board_with_dispatch,
    run_encrypted_share_on_board_with_io, run_encrypted_share_profile, EncryptedShareFacts,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    None,
    WrongAuthority,
    TamperedEncryptedSubmission,
    ReplayedEncryptedSubmission,
    TruncatedEncryptedSubmission,
    InvalidPlainSubmission,
    WrongAcknowledgementChannel,
    InflatedAcknowledgement,
    TamperedEncryptedAcknowledgement,
    IncompleteSubmissionWrite,
    RejectedClose,
    InvalidSubmissionAndRejectedClose,
}

/// Completion observations emitted after the actual operation, never queue admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileIoEvent {
    AsicWriteCompleted,
    ShareWriteCompleted,
    AcknowledgementValidated,
}

/// Immutable synthetic lease terms, never physical credentials or endpoints.
pub use bitaxe_stratum_v2_fixture::functional::{
    SYNTHETIC_ENDPOINT_HOST, SYNTHETIC_ENDPOINT_PORT, SYNTHETIC_USER_IDENTITY,
};
