use super::*;
use bitaxe_stratum::v2::noise::diagnostic::{self, Event, Failure, Observer, Phase};
#[path = "transport/failure.rs"]
mod failure_projection;
#[path = "transport/input.rs"]
mod input;
#[path = "transport/rng.rs"]
mod rng;
pub(super) use input::{maybe_prepare, PreparedInput};

pub(super) fn run(shared: &Shared, input: PreparedInput) {
    let mut generators = match rng::prepare(crate::crypto_entropy::fill) {
        Ok(generators) => generators,
        Err(failure) => {
            fail(
                shared,
                FailureStage::NoisePrepared,
                NoiseCategory::Preparation,
                match failure {
                    rng::Failure::Allocation => NoiseDetail::Allocation,
                    rng::Failure::Entropy => NoiseDetail::Rng,
                },
            );
            return;
        }
    };
    let Some(generator) = generators.first_mut() else {
        fail(
            shared,
            FailureStage::NoisePrepared,
            NoiseCategory::EvidenceIncomplete,
            NoiseDetail::Missing,
        );
        return;
    };
    diagnostic::run(
        input.endpoint.into(),
        input.authority,
        generator,
        &mut NativeObserver(shared),
    );
}
struct NativeObserver<'a>(&'a Shared);
impl Observer for NativeObserver<'_> {
    fn permitted(&mut self) -> bool {
        allowed(self.0, FailureStage::Evidence)
    }
    fn now_us(&self) -> Option<u64> {
        now_us()
    }
    fn event(&mut self, event: Event) {
        let Ok(mut maybe_record) = self.0.record.lock() else {
            return;
        };
        let Some(record) = maybe_record.as_mut() else {
            return;
        };
        match event {
            Event::SocketClosedWithoutClock => record.socket_closed_without_clock(),
            Event::SocketOpened(port) => record.socket_opened(port),
            Event::Cleanup => record.cleanup(now_us()),
            Event::Complete {
                phase: Phase::Close,
                at_us,
                duration_us,
                ..
            } => record.socket_closed(at_us, duration_us),
            Event::Complete {
                phase,
                at_us,
                duration_us,
                bytes,
            } => {
                let stage = match phase {
                    Phase::Prepare => NoiseStage::NoisePrepared,
                    Phase::Connect => NoiseStage::TcpConnected,
                    Phase::ActOne => NoiseStage::ActOneWritten,
                    Phase::ActTwo => NoiseStage::ActTwoReceived,
                    Phase::Authenticate => NoiseStage::AuthorityVerified,
                    Phase::Proof => NoiseStage::ProofWritten,
                    Phase::Close => return,
                };
                record.stage(stage, at_us, duration_us, bytes);
            }
        }
    }
    fn failed(&mut self, phase: Phase, failure: Failure) {
        // The fence already retained the actual cancellation cause.
        if failure == Failure::Cancelled {
            return;
        }
        let (stage, category, detail) = failure_projection::project(phase, failure);
        fail(self.0, stage, category, detail);
    }
}
