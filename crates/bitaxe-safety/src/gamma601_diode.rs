//! The Gamma 601's chip temperature by power stage (plan F3.3 section 1).
//!
//! The BM1370's thermal diode reads 127.75 C (raw `0x3FE`, which the decoder accepts) whenever
//! the chip is not initialized. Upstream reads it only after its ASIC init
//! (`reference/esp-miner/main/thermal/thermal.c:74-90`). The producer tracks the stage and
//! replaces the raw outcome before publishing, so an unpowered or uninitialized chip is a typed
//! unavailable reading rather than a false overheat.

use crate::gamma601_hash_check::late;
use crate::observation::{FaultReason, UnavailableReason};
use crate::sensor_acquisition::AcquisitionOutcome;

/// What the BM1370's diode reads without its register-0x54 configuration.
pub const BM1370_DIODE_UNCONFIGURED_C: f64 = 127.75;

/// The longest the chip may stay powered but uninitialized before the producer faults.
pub const POWERED_UNINITIALIZED_CAP_MS: u64 = 20_000;

/// The chip's power stage, as far as its diode is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiodeStage {
    Unpowered,
    PoweredUninitialized,
    Initialized,
}

/// The stage after one sweep.
///
/// - A confirmed off is [`DiodeStage::Unpowered`].
/// - Until then, [`DiodeStage::Initialized`] stays latched.
/// - With the regulator on, this power-on's init makes it initialized; otherwise it is
///   powered but uninitialized.
/// - With the regulator off but the off not yet confirmed (the gap between the off command
///   and its confirmation), a powered stage holds: the rail may still be up, so the chip is
///   not called unpowered before the off is proven. Only an already unpowered stage stays so.
#[must_use]
pub const fn next_stage(
    prior: DiodeStage,
    off_confirmed: bool,
    regulator_on: bool,
    init_matches_generation: bool,
) -> DiodeStage {
    if off_confirmed {
        return DiodeStage::Unpowered;
    }
    if matches!(prior, DiodeStage::Initialized) {
        return DiodeStage::Initialized;
    }
    if regulator_on {
        return if init_matches_generation {
            DiodeStage::Initialized
        } else {
            DiodeStage::PoweredUninitialized
        };
    }
    prior
}

/// The chip temperature to publish for `stage`. Unpowered and uninitialized chips have no
/// reading. An initialized chip's exact 127.75 C means it lost its diode configuration, which
/// is a fault; every other outcome, read failures included, passes through.
#[must_use]
pub fn chip_temperature_outcome(
    stage: DiodeStage,
    regulator_commanded_on: bool,
    raw: AcquisitionOutcome<f64>,
) -> AcquisitionOutcome<f64> {
    match stage {
        DiodeStage::Unpowered => AcquisitionOutcome::Unavailable(UnavailableReason::AsicUnpowered),
        DiodeStage::PoweredUninitialized => {
            AcquisitionOutcome::Unavailable(UnavailableReason::AsicNotInitialized)
        }
        DiodeStage::Initialized => match raw {
            // Commanded off but not yet confirmed off: the chip is losing its configuration, so
            // its unconfigured reading is not a fault. It is not unpowered either until the off
            // is confirmed, so no cooling rule counts it. Every other reading still counts.
            AcquisitionOutcome::Success(celsius)
                if celsius == BM1370_DIODE_UNCONFIGURED_C && !regulator_commanded_on =>
            {
                AcquisitionOutcome::Unavailable(UnavailableReason::AsicNotInitialized)
            }
            AcquisitionOutcome::Success(celsius) if celsius == BM1370_DIODE_UNCONFIGURED_C => {
                AcquisitionOutcome::InvalidSample
            }
            other => other,
        },
    }
}

/// Whether the BM1370 draws no hashing load, so the regulator's IOUT offset may read negative
/// (`gamma601_sensors::power_outcome`): the output confirmed off, the regulator commanded off
/// (work is already blocked; the current is decaying), or the chip powered but not yet
/// initialized (held in reset). Only an initialized chip with the regulator commanded on is
/// loaded, and only its negative reading is invalid.
#[must_use]
pub fn chip_unloaded(stage: DiodeStage, off_confirmed: bool, regulator_commanded_on: bool) -> bool {
    off_confirmed || !regulator_commanded_on || stage != DiodeStage::Initialized
}

/// How long the chip has stayed powered but uninitialized.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UninitializedWindow {
    maybe_since_ms: Option<u64>,
}

impl UninitializedWindow {
    /// One sweep: the clock starts when the stage enters [`DiodeStage::PoweredUninitialized`]
    /// and clears on any other stage. Faults once the stage has stayed there more than
    /// [`POWERED_UNINITIALIZED_CAP_MS`].
    pub fn observe(&mut self, stage: DiodeStage, now_ms: u64) -> Option<FaultReason> {
        if stage != DiodeStage::PoweredUninitialized {
            self.maybe_since_ms = None;
            return None;
        }
        let since_ms = *self.maybe_since_ms.get_or_insert(now_ms);
        late(since_ms, now_ms, POWERED_UNINITIALIZED_CAP_MS)
            .then_some(FaultReason::Bm1370InitOverdue)
    }
}

#[cfg(test)]
mod tests;
