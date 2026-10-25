//! The Gamma 601 hash check's limits, judges and USB record (ADR-0041 item 9, phase F3.2).
//!
//! Only the test-only `gamma601-hash-check` firmware runs the check. After the chip check's
//! power-on and detection, it initializes the BM1370 at [`FREQUENCY_MHZ`], requires a plausible
//! diode reading, sends one known job and verifies every result against it, then holds the reset
//! and turns the output off. Every reason is a closed label.

mod window;

pub use crate::gamma601_chip_check::{ChipCheckRecord, ChipCount};
pub use crate::gamma601_vout_check::{CheckResult, ReplayRecord, ReplayTurn};
pub use window::{HashExtremes, HashReadings, HashSample, HashStop, HashWindowJudge};

use crate::tps546::OffState;

/// The first hashing run's frequency (owner decision, 2026-10-24), below upstream's 525 MHz.
pub const FREQUENCY_MHZ: u16 = 300;
pub const TARGET_MV: u16 = 1_150;
/// The init starts at most this long after the chip detection returns.
pub const INIT_START_CAP_MS: u64 = 1_000;
/// Hashing starts at most this long after the init starts: the 4 s ramp to 300 MHz, upstream's
/// 2 s stabilization, the diode gate and slack.
pub const HASH_START_CAP_MS: u64 = 10_000;
/// The output is never on longer than this, from the start of the power-on sequence.
pub const ON_TIME_CAP_MS: u64 = 45_000;
/// The hashing window's length; it ends early at [`VERIFIED_TARGET`] verified results.
pub const WINDOW_MS: u32 = 20_000;
pub const VERIFIED_TARGET: u32 = 8;
/// The pass rule's least number of verified results.
pub const VERIFIED_MIN: u32 = 3;
/// The diode gate's plausible range and its two readings' largest disagreement.
pub const DIODE_MIN_C: f64 = 10.0;
pub const DIODE_MAX_C: f64 = 65.0;
pub const DIODE_AGREEMENT_C: f64 = 1.0;
/// How many distinct results the duplicate check remembers.
const SEEN_CAPACITY: usize = 64;

/// Whether `now_ms` is more than `cap_ms` after `since_ms`.
#[must_use]
pub const fn late(since_ms: u64, now_ms: u64, cap_ms: u64) -> bool {
    now_ms.saturating_sub(since_ms) > cap_ms
}

/// The diode gate: two readings, each plausible, that agree. Returns the higher one. F3.1's
/// unpowered 127.75 C fails it.
#[must_use]
pub fn diode_gate(maybe_first_c: Option<f64>, maybe_second_c: Option<f64>) -> Option<f64> {
    let (first, second) = maybe_first_c.zip(maybe_second_c)?;
    let plausible = |celsius: f64| (DIODE_MIN_C..=DIODE_MAX_C).contains(&celsius);
    (plausible(first) && plausible(second) && (first - second).abs() <= DIODE_AGREEMENT_C)
        .then_some(first.max(second))
}

/// One result's verification against the known job.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerifiedResult {
    pub nonce: u32,
    pub version_bits: u32,
    pub difficulty: f64,
    pub meets_ticket: bool,
    /// The known answer: the genesis nonce with no version bits rolled.
    pub genesis: bool,
}

/// The window's results.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct HashTally {
    pub verified: u32,
    pub invalid: u32,
    pub duplicate: u32,
    /// Frames the executor discarded before verification, such as a stale job id.
    pub discarded: u32,
    pub best_difficulty: f64,
    pub genesis_hit: bool,
    seen: Vec<(u32, u32)>,
}

impl HashTally {
    /// Counts one verified result: a repeat of a remembered (nonce, version) pair counts as a
    /// duplicate, a hash short of the chip's ticket as invalid.
    pub fn observe(&mut self, result: VerifiedResult) {
        let key = (result.nonce, result.version_bits);
        if self.seen.contains(&key) {
            self.duplicate += 1;
            return;
        }
        if self.seen.len() < SEEN_CAPACITY {
            self.seen.push(key);
        }
        if !result.meets_ticket {
            self.invalid += 1;
            return;
        }
        self.verified += 1;
        self.best_difficulty = self.best_difficulty.max(result.difficulty);
        self.genesis_hit |= result.genesis;
    }

    pub fn discard(&mut self) {
        self.discarded += 1;
    }

    /// At least [`VERIFIED_MIN`] verified results, and invalid ones at most a tenth of all.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.verified >= VERIFIED_MIN && self.invalid * 10 <= self.verified + self.invalid
    }

    #[must_use]
    pub fn target_reached(&self) -> bool {
        self.verified >= VERIFIED_TARGET
    }
}

/// The check's record.
#[derive(Clone, Debug, PartialEq)]
pub struct HashCheckRecord {
    /// The power-on, the detection and the off, as the chip check records them.
    pub power: ChipCheckRecord,
    pub freq_mhz: u16,
    /// The diode gate's reading; `None` when the gate was not reached or failed.
    pub maybe_diode_gate_c: Option<f64>,
    pub extremes: HashExtremes,
    pub tally: HashTally,
    /// From the job's send to the first verified result, and to the window's end.
    pub maybe_first_result_ms: Option<u32>,
    pub maybe_window_ms: Option<u32>,
}

impl HashCheckRecord {
    /// Before the check runs.
    #[must_use]
    pub fn pending() -> Self {
        Self {
            power: ChipCheckRecord::pending(u32::from(TARGET_MV)),
            freq_mhz: FREQUENCY_MHZ,
            maybe_diode_gate_c: None,
            extremes: HashExtremes::default(),
            tally: HashTally::default(),
            maybe_first_result_ms: None,
            maybe_window_ms: None,
        }
    }

    /// The closed USB record.
    #[must_use]
    pub fn usb_marker(&self) -> String {
        let power = &self.power;
        let extremes = &self.extremes;
        let tally = &self.tally;
        format!(
            "usb_gamma601_hash_check schema=v1 result={} reason={} target_mv={} freq_mhz={} chips={} blocker={} diode_before_c={} diode_gate_c={} diode_max_c={} vout_mv={} vin_mv_min={} iout_ma_max={} power_mw_max={} vr_temp_c_max={} verified={} invalid={} duplicate={} discarded={} best_difficulty={} genesis_hit={} first_result_ms={} window_ms={} on_ms={} off={} stack_free={} redacted=true",
            power.result.label(),
            power.reason,
            power.target_mv,
            self.freq_mhz,
            power.chips.label(),
            power.maybe_blocker.unwrap_or("none"),
            celsius(power.maybe_diode_before_c),
            celsius(self.maybe_diode_gate_c),
            celsius(extremes.maybe_diode_max_c),
            unread(power.maybe_vout_mv),
            unread(extremes.maybe_vin_mv_min),
            extremes.iout_ma_max.max(power.iout_ma_max),
            extremes.power_mw_max,
            extremes.vr_temp_c_max.max(power.vr_temp_c_max),
            tally.verified,
            tally.invalid,
            tally.duplicate,
            tally.discarded,
            tally.best_difficulty.floor(),
            tally.genesis_hit,
            unread(self.maybe_first_result_ms),
            unread(self.maybe_window_ms),
            unread(power.maybe_on_ms),
            power.maybe_off.map_or("unread", OffState::label),
            unread(power.maybe_stack_free_bytes),
        )
    }
}

fn celsius(maybe_celsius: Option<f64>) -> String {
    maybe_celsius
        .filter(|celsius| celsius.is_finite())
        .map_or_else(|| "unread".to_owned(), |celsius| format!("{celsius:.1}"))
}

fn unread<T: ToString>(maybe_value: Option<T>) -> String {
    maybe_value.map_or_else(|| "unread".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests;
