//! Single boot-seeded cryptographic stream; runtime requests never touch ADC or RF.
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntropyError {
    AlreadyInitialized,
    Unavailable,
    Source,
}
impl std::fmt::Display for EntropyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::AlreadyInitialized => "crypto_entropy_already_initialized",
            Self::Unavailable => "crypto_entropy_unavailable",
            Self::Source => "crypto_entropy_source_failed",
        })
    }
}
impl std::error::Error for EntropyError {}

pub(super) trait StartupEntropySource {
    fn enable(&mut self);
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), EntropyError>;
    fn disable(&mut self);
}
struct EnabledSource<'a, S: StartupEntropySource>(&'a mut S);
impl<S: StartupEntropySource> Drop for EnabledSource<'_, S> {
    fn drop(&mut self) {
        self.0.disable();
    }
}

pub(super) struct EntropyOwner {
    initialized: AtomicBool,
    maybe_rng: Mutex<Option<StdRng>>,
}
impl EntropyOwner {
    pub(super) const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            maybe_rng: Mutex::new(None),
        }
    }
    pub(super) fn initialize(
        &self,
        source: &mut impl StartupEntropySource,
    ) -> Result<(), EntropyError> {
        if self.initialized.swap(true, Ordering::AcqRel) {
            return Err(EntropyError::AlreadyInitialized);
        }
        let mut owner = self
            .maybe_rng
            .try_lock()
            .map_err(|_| EntropyError::Unavailable)?;
        let mut seed = Zeroizing::new([0u8; 32]);
        source.enable();
        let enabled = EnabledSource(source);
        enabled.0.fill_seed(&mut seed)?;
        drop(enabled);
        *owner = Some(StdRng::from_seed(*seed));
        Ok(())
    }
    pub(super) fn fill(&self, bytes: &mut [u8]) -> Result<(), EntropyError> {
        // Caller buffers cannot retain an earlier successful nonce after a failed request.
        bytes.zeroize();
        let mut owner = self
            .maybe_rng
            .try_lock()
            .map_err(|_| EntropyError::Unavailable)?;
        let rng = owner.as_mut().ok_or(EntropyError::Unavailable)?;
        rng.try_fill_bytes(bytes).map_err(|_| {
            bytes.zeroize();
            EntropyError::Source
        })
    }
}

#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
