//! First-wins detail for a generation revoked as `unsafe_observation`.
//! Diagnostic evidence only; it never grants or restores authority.

use std::sync::atomic::{AtomicU32, Ordering};

use bitaxe_api::{SafetyFact, SafetyFactState, SafetyVerdict};

use super::{GenerationGate, RevocationReason, WorkerGeneration, ACTIVE, FLAGS};

/// Which device-local check revoked the active generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UnsafeTrigger {
    /// A published snapshot failed the mining-safety predicate.
    UnsafeSample = 1,
    /// The fan read zero RPM after the generation's fan proof.
    ZeroFan = 2,
    /// No safe snapshot arrived for over one second after the fan proof.
    NoSafeSample = 3,
}

impl UnsafeTrigger {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::UnsafeSample => "unsafe_sample",
            Self::ZeroFan => "zero_fan",
            Self::NoSafeSample => "no_safe_sample",
        }
    }

    const fn maybe_from_code(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::UnsafeSample),
            2 => Some(Self::ZeroFan),
            3 => Some(Self::NoSafeSample),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsafeObservationDetail {
    pub generation: u32,
    pub trigger: UnsafeTrigger,
    /// The first failing fact for `UnsafeSample`; absent when the caller had none.
    pub maybe_verdict: Option<SafetyVerdict>,
    /// Milliseconds since the last safe snapshot, saturated to `u32`.
    pub since_safe_ms: u32,
    pub closed_ms: u32,
}

impl UnsafeObservationDetail {
    /// Closed, redaction-safe diagnostic line; it carries no authority.
    #[must_use]
    pub fn marker(&self) -> String {
        let optional = |maybe: Option<String>| maybe.unwrap_or_else(|| "unavailable".to_owned());
        let (fact, state, value, age) = self.maybe_verdict.map_or(
            (
                "none",
                "none",
                "unavailable".to_owned(),
                "unavailable".to_owned(),
            ),
            |verdict| {
                (
                    verdict.fact.label(),
                    verdict.state.label(),
                    optional(verdict.maybe_value_milli.map(|value| value.to_string())),
                    optional(verdict.maybe_age_ms.map(|age| age.to_string())),
                )
            },
        );
        format!(
            "worker_revocation_detail schema=v1 generation={} reason=unsafe_observation trigger={} fact={fact} state={state} value_milli={value} age_ms={age} since_safe_ms={} closed_ms={} redacted=true",
            self.generation,
            self.trigger.label(),
            self.since_safe_ms,
            self.closed_ms,
        )
    }
}

// Packed word: trigger bits 0..2, fact bits 2..5, state bits 5..8,
// value-present bit 8, age-present bit 9.
const VALUE_PRESENT: u32 = 1 << 8;
const AGE_PRESENT: u32 = 1 << 9;

pub(super) struct UnsafeDetailCell {
    /// Logical generation plus one; zero means no detail.
    generation: AtomicU32,
    packed: AtomicU32,
    value_milli: AtomicU32,
    age_ms: AtomicU32,
    since_safe_ms: AtomicU32,
    closed_ms: AtomicU32,
}

impl UnsafeDetailCell {
    pub(super) const fn new() -> Self {
        Self {
            generation: AtomicU32::new(0),
            packed: AtomicU32::new(0),
            value_milli: AtomicU32::new(0),
            age_ms: AtomicU32::new(0),
            since_safe_ms: AtomicU32::new(0),
            closed_ms: AtomicU32::new(0),
        }
    }

    /// Called only by the call that won the generation's revocation, so the
    /// first unsafe cause is never replaced by a later one.
    pub(super) fn record(&self, detail: UnsafeObservationDetail) {
        let mut packed = detail.trigger as u32;
        let maybe_value = detail
            .maybe_verdict
            .and_then(|verdict| verdict.maybe_value_milli);
        let maybe_age = detail
            .maybe_verdict
            .and_then(|verdict| verdict.maybe_age_ms);
        if let Some(verdict) = detail.maybe_verdict {
            packed |= (verdict.fact as u32) << 2 | (verdict.state as u32) << 5;
        }
        if maybe_value.is_some() {
            packed |= VALUE_PRESENT;
        }
        if maybe_age.is_some() {
            packed |= AGE_PRESENT;
        }
        // Readers see either the previous generation's complete detail or none
        // while this one is written; the generation word publishes it.
        self.generation.store(0, Ordering::Release);
        self.value_milli.store(
            saturate_i32(maybe_value.unwrap_or(0)) as u32,
            Ordering::Release,
        );
        self.age_ms.store(
            maybe_age.map_or(0, |age| u32::try_from(age).unwrap_or(u32::MAX)),
            Ordering::Release,
        );
        self.packed.store(packed, Ordering::Release);
        self.since_safe_ms
            .store(detail.since_safe_ms, Ordering::Release);
        self.closed_ms.store(detail.closed_ms, Ordering::Release);
        self.generation
            .store(detail.generation.saturating_add(1), Ordering::Release);
    }

    pub(super) fn maybe_read(&self) -> Option<UnsafeObservationDetail> {
        let published = self.generation.load(Ordering::Acquire);
        if published == 0 {
            return None;
        }
        let packed = self.packed.load(Ordering::Acquire);
        let value_milli = self.value_milli.load(Ordering::Acquire) as i32;
        let age_ms = self.age_ms.load(Ordering::Acquire);
        let since_safe_ms = self.since_safe_ms.load(Ordering::Acquire);
        let closed_ms = self.closed_ms.load(Ordering::Acquire);
        if self.generation.load(Ordering::Acquire) != published {
            return None;
        }
        let trigger = UnsafeTrigger::maybe_from_code(packed & 0b11)?;
        let maybe_fact = SafetyFact::maybe_from_code(((packed >> 2) & 0b111) as u8);
        let maybe_state = SafetyFactState::maybe_from_code(((packed >> 5) & 0b111) as u8);
        let maybe_verdict = maybe_fact
            .zip(maybe_state)
            .map(|(fact, state)| SafetyVerdict {
                fact,
                state,
                maybe_value_milli: (packed & VALUE_PRESENT != 0).then_some(i64::from(value_milli)),
                maybe_age_ms: (packed & AGE_PRESENT != 0).then_some(u64::from(age_ms)),
            });
        Some(UnsafeObservationDetail {
            generation: published - 1,
            trigger,
            maybe_verdict,
            since_safe_ms,
            closed_ms,
        })
    }
}

impl GenerationGate {
    pub fn check_safety(&self, safe: bool, nonzero_fan: bool, now_ms: u64) {
        self.check_safety_detail(safe, None, nonzero_fan, now_ms);
    }

    /// Like `check_safety`, recording the first failing fact if this revokes.
    pub fn check_safety_verdict(
        &self,
        maybe_verdict: Option<SafetyVerdict>,
        nonzero_fan: bool,
        now_ms: u64,
    ) {
        self.check_safety_detail(maybe_verdict.is_none(), maybe_verdict, nonzero_fan, now_ms);
    }

    fn check_safety_detail(
        &self,
        safe: bool,
        maybe_verdict: Option<SafetyVerdict>,
        nonzero_fan: bool,
        now_ms: u64,
    ) {
        let state = self.state.load(Ordering::Acquire);
        if state & FLAGS != ACTIVE {
            return;
        }
        let generation = WorkerGeneration(state & !FLAGS);
        if !safe {
            self.revoke_unsafe_at(
                generation,
                now_ms,
                UnsafeTrigger::UnsafeSample,
                maybe_verdict,
            );
            return;
        }
        if self.fan_proof_generation.load(Ordering::Acquire) == generation.0 && !nonzero_fan {
            self.revoke_unsafe_at(generation, now_ms, UnsafeTrigger::ZeroFan, None);
            return;
        }
        self.last_safety_ms.store(now_ms as u32, Ordering::Release);
    }

    pub(super) fn revoke_unsafe_at(
        &self,
        generation: WorkerGeneration,
        now_ms: u64,
        trigger: UnsafeTrigger,
        maybe_verdict: Option<SafetyVerdict>,
    ) {
        if !self.revoke_reason_at(generation, now_ms, RevocationReason::UnsafeObservation) {
            return;
        }
        let since_safe_ms =
            (now_ms as u32).wrapping_sub(self.last_safety_ms.load(Ordering::Acquire));
        self.unsafe_detail.record(UnsafeObservationDetail {
            generation: generation.raw(),
            trigger,
            maybe_verdict,
            since_safe_ms,
            closed_ms: now_ms as u32,
        });
    }

    /// The most recent generation's first unsafe-observation detail, if any.
    pub fn maybe_unsafe_detail(&self) -> Option<UnsafeObservationDetail> {
        self.unsafe_detail.maybe_read()
    }
}

#[allow(clippy::cast_possible_truncation)]
fn saturate_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
#[path = "unsafe_detail_tests.rs"]
mod tests;
