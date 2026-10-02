//! Runs only Noise act-two authentication on a joined helper whose stack lives in PSRAM.
//!
//! Certificate verification needs about 10.6 KB below the completion call, which does
//! not fit the 12 KiB pool workers with the 2,048-byte margin. Internal RAM cannot grow
//! those workers, so the opaque crypto moves to a short-lived helper. The helper never
//! touches flash or disables the cache, which PSRAM stacks require. Socket I/O and every
//! other step stay on the worker.
use bitaxe_stratum::v2::noise::{
    diagnostic::Failure, NoiseCompletionFailure, NoiseInitiator, NoiseTransport, ACT_TWO_LEN,
};
use esp_idf_sys as sys;

/// Helper stack; `scripts/audit-device-noise-stack.mjs` checks the measured descent.
pub(crate) const COMPLETION_STACK_BYTES: usize = 16 * 1024;
// Measured completion descent is 10,624 bytes plus the helper entry; keep the margin.
const _: () = assert!(COMPLETION_STACK_BYTES >= 10_624 + 512 + 2_048);
const HELPER_NAME: &core::ffi::CStr = c"noise-complete";

/// Applies PSRAM stack caps to pthreads created by this thread until dropped.
struct PsramStackConfig {
    previous: sys::esp_pthread_cfg_t,
}

impl PsramStackConfig {
    fn apply() -> Option<Self> {
        let mut previous = unsafe { sys::esp_pthread_get_default_config() };
        let found = unsafe { sys::esp_pthread_get_cfg(&mut previous) };
        if found != sys::ESP_OK && found != sys::ESP_ERR_NOT_FOUND {
            return None;
        }
        if found == sys::ESP_ERR_NOT_FOUND {
            previous = unsafe { sys::esp_pthread_get_default_config() };
        }
        let mut config = unsafe { sys::esp_pthread_get_default_config() };
        config.stack_size = COMPLETION_STACK_BYTES;
        config.prio = unsafe { sys::uxTaskPriorityGet(std::ptr::null_mut()) } as usize;
        config.inherit_cfg = false;
        config.thread_name = HELPER_NAME.as_ptr();
        config.stack_alloc_caps = sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT;
        (unsafe { sys::esp_pthread_set_cfg(&config) } == sys::ESP_OK).then_some(Self { previous })
    }
}

impl Drop for PsramStackConfig {
    fn drop(&mut self) {
        // Restoring cannot fail for a configuration that was previously accepted.
        unsafe { sys::esp_pthread_set_cfg(&self.previous) };
    }
}

/// Completes into `slot` on the helper and joins it; the outcome is unchanged.
pub(crate) fn authenticate_on_psram_stack(
    initiator: NoiseInitiator,
    act_two: &[u8; ACT_TWO_LEN],
    unix_time_seconds: u32,
    slot: &mut Vec<NoiseTransport>,
) -> Result<(), Failure> {
    let response = *act_two;
    let mut owned = std::mem::take(slot);
    let config = PsramStackConfig::apply().ok_or(Failure::Allocation)?;
    let spawned = std::thread::Builder::new()
        .stack_size(COMPLETION_STACK_BYTES)
        .spawn(move || {
            let outcome = complete_in_helper(initiator, &response, unix_time_seconds, &mut owned);
            (outcome, owned)
        });
    drop(config);
    let (outcome, owned) = spawned
        .map_err(|_| Failure::Allocation)?
        .join()
        .map_err(|_| Failure::Authentication(NoiseCompletionFailure::Other))?;
    *slot = owned;
    outcome.map_err(Failure::Authentication)
}

/// The native audit roots the helper's crypto descent here.
#[inline(never)]
fn complete_in_helper(
    initiator: NoiseInitiator,
    act_two: &[u8; ACT_TWO_LEN],
    unix_time_seconds: u32,
    slot: &mut Vec<NoiseTransport>,
) -> Result<(), NoiseCompletionFailure> {
    initiator.complete_diagnostic_into(act_two, unix_time_seconds, slot)
}
