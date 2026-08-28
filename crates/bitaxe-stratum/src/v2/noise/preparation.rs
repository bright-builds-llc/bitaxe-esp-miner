use rand::{CryptoRng, Rng};

use super::{NoiseCompletionFailure, NoiseInitiator, NoiseTransport, ACT_ONE_LEN};
use crate::v2::StratumV2Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoisePreparationStage {
    KeypairReady,
    ActOneReady,
}

pub struct PreparedNoiseInitiator {
    initiator: NoiseInitiator,
    act_one: [u8; ACT_ONE_LEN],
}

impl NoiseInitiator {
    pub fn prepare<R: Rng + CryptoRng + ?Sized>(
        maybe_authority_public_key: Option<[u8; 32]>,
        rng: &mut R,
    ) -> Result<PreparedNoiseInitiator, StratumV2Error> {
        Self::prepare_with_observer(maybe_authority_public_key, rng, |_| {})
    }

    pub fn prepare_with_observer<R, Observe>(
        maybe_authority_public_key: Option<[u8; 32]>,
        rng: &mut R,
        mut observe: Observe,
    ) -> Result<PreparedNoiseInitiator, StratumV2Error>
    where
        R: Rng + CryptoRng + ?Sized,
        Observe: FnMut(NoisePreparationStage),
    {
        let mut initiator = Self::new(maybe_authority_public_key, rng)?;
        observe(NoisePreparationStage::KeypairReady);
        let act_one = initiator.act_one()?;
        observe(NoisePreparationStage::ActOneReady);
        Ok(PreparedNoiseInitiator { initiator, act_one })
    }
}

impl PreparedNoiseInitiator {
    #[must_use]
    pub const fn act_one(&self) -> &[u8; ACT_ONE_LEN] {
        &self.act_one
    }

    pub fn complete(
        self,
        act_two: &[u8],
        unix_time_seconds: u32,
    ) -> Result<NoiseTransport, StratumV2Error> {
        self.initiator.complete(act_two, unix_time_seconds)
    }

    pub fn complete_diagnostic(
        self,
        act_two: &[u8],
        unix_time_seconds: u32,
    ) -> Result<NoiseTransport, NoiseCompletionFailure> {
        self.initiator
            .complete_diagnostic(act_two, unix_time_seconds)
    }
}
