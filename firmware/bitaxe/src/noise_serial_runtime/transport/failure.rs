//! Exact native diagnostic error projection, shared with its host regression.
use bitaxe_stratum::v2::noise::{
    diagnostic::{Failure, Phase},
    NoiseCompletionFailure,
};
use bitaxe_worker_control::noise::{FailureStage, NoiseCategory, NoiseDetail};

pub(crate) fn project(
    phase: Phase,
    failure: Failure,
) -> (FailureStage, NoiseCategory, NoiseDetail) {
    let stage = match phase {
        Phase::Prepare => FailureStage::NoisePrepared,
        Phase::Connect => FailureStage::TcpConnected,
        Phase::ActOne => FailureStage::ActOneWritten,
        Phase::ActTwo => FailureStage::ActTwoReceived,
        Phase::Authenticate => FailureStage::AuthorityVerified,
        Phase::Proof => FailureStage::ProofWritten,
        Phase::Close => FailureStage::SocketClosed,
    };
    let category = match phase {
        Phase::Prepare => NoiseCategory::Preparation,
        Phase::Connect => NoiseCategory::Connect,
        Phase::ActOne => NoiseCategory::Write,
        Phase::ActTwo => NoiseCategory::Read,
        Phase::Authenticate => NoiseCategory::Authentication,
        Phase::Proof if failure == Failure::Timeout => NoiseCategory::Write,
        Phase::Proof => NoiseCategory::Proof,
        Phase::Close => NoiseCategory::Cleanup,
    };
    let detail = match failure {
        Failure::Cancelled => NoiseDetail::CancelRequested,
        Failure::Clock => NoiseDetail::ClockDiscontinuity,
        Failure::Timeout => NoiseDetail::Timeout,
        Failure::Eof => NoiseDetail::Eof,
        Failure::Extra => NoiseDetail::Extra,
        Failure::Allocation => NoiseDetail::Allocation,
        Failure::Io => NoiseDetail::Io,
        Failure::BeforeEpoch => NoiseDetail::BeforeEpoch,
        Failure::TimeOverflow => NoiseDetail::TimeOverflow,
        Failure::Authentication(NoiseCompletionFailure::CertificateTime) => {
            NoiseDetail::CertificateTime
        }
        Failure::Authentication(
            NoiseCompletionFailure::CertificateSignature | NoiseCompletionFailure::PublicKey,
        ) => NoiseDetail::WrongAuthority,
        Failure::Authentication(_) => NoiseDetail::Malformed,
    };
    (
        stage,
        if failure == Failure::Clock {
            NoiseCategory::ClockInvalid
        } else {
            category
        },
        detail,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proof_write_timeout_projects_to_expired_write_cause() {
        // Arrange / Act
        let cause = project(Phase::Proof, Failure::Timeout);
        // Assert
        assert_eq!(
            cause,
            (
                FailureStage::ProofWritten,
                NoiseCategory::Write,
                NoiseDetail::Timeout
            )
        );
    }
    #[test]
    fn proof_codec_failure_does_not_masquerade_as_a_write_timeout() {
        // Arrange / Act
        let cause = project(Phase::Proof, Failure::Io);
        // Assert
        assert_eq!(
            cause,
            (
                FailureStage::ProofWritten,
                NoiseCategory::Proof,
                NoiseDetail::Io
            )
        );
    }
}
