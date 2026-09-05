//! Startup-only physical entropy admission, before application ADC and Wi-Fi ownership.
mod model;
pub(crate) use model::EntropyError;
use model::{EntropyOwner, StartupEntropySource};

static OWNER: EntropyOwner = EntropyOwner::new();

struct EspStartupEntropy;
impl StartupEntropySource for EspStartupEntropy {
    fn enable(&mut self) {
        unsafe { esp_idf_sys::bootloader_random_enable() };
    }
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), EntropyError> {
        unsafe { esp_idf_sys::esp_fill_random(seed.as_mut_ptr().cast(), seed.len()) };
        Ok(())
    }
    fn disable(&mut self) {
        unsafe { esp_idf_sys::bootloader_random_disable() };
    }
}

/// Must run once before ADC or RF initialization; failures never allow a later hardware retry.
pub(crate) fn initialize() -> Result<(), EntropyError> {
    OWNER.initialize(&mut EspStartupEntropy)
}

/// Returns cryptographic bytes without hardware access or waiting for another owner.
pub(crate) fn fill(bytes: &mut [u8]) -> Result<(), EntropyError> {
    OWNER.fill(bytes)
}
