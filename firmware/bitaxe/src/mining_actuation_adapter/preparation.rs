//! Hardware preparation, reporting progress so the caller can feed its task watchdog.

use super::{MiningActuationAdapterError, Ultra205MiningActuationAdapter};
use crate::mining_actuation::{
    execute_preparation_with_progress, PreparationExecutionFailure, PreparationProgress,
};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

impl Ultra205MiningActuationAdapter {
    /// Prepares `profile`, reporting each step, each heartbeat within it, and each step of
    /// a failure's rollback (the frequency ramp alone takes about 7 s).
    pub fn prepare(
        &mut self,
        profile: MiningHardwareProfile,
        progress: &mut dyn FnMut(PreparationProgress),
    ) -> Result<(), PreparationExecutionFailure<MiningActuationAdapterError>> {
        self.maybe_requested_profile = Some(profile);
        // A profile from an earlier preparation must not choose this rollback's ramp.
        self.maybe_asic_profile = None;
        self.fan_proof_confirmed = false;
        execute_preparation_with_progress(self, profile, progress)
    }
}
