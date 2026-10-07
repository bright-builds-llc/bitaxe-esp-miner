//! Signed hardware profile and one-shot soak allowance with independent durable accounting
//! (ADR-0033).
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROFILE: &str = "worker-soak-allowance-v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
/// Work stays admitted for 600,000 active ms; the budget adds the 19,050 ms upstream-default pre-reset shutdown tail.
pub const SOAK_WORK_GATE_MS: u64 = 600_000;
pub const SOAK_MAXIMUM_ACTIVE_MS: u64 = 619_050;

/// Signed mining hardware profile. Absent on a grant means `Conservative`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HardwareProfile {
    #[default]
    Conservative,
    UpstreamDefault,
}

/// Field order is canonical because authorization hashing serializes directly.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SoakAllowance {
    id: String,
    maximum_active_milliseconds: u64,
    ordinal: u32,
    schema: String,
}
impl SoakAllowance {
    #[must_use]
    pub fn validate(&self) -> bool {
        self.schema == PROFILE
            && crate::serial::canonical_nonce(&self.id, 16)
            && self.ordinal > 0
            && self.maximum_active_milliseconds == SOAK_MAXIMUM_ACTIVE_MS
    }
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
    #[must_use]
    pub const fn maximum_active_milliseconds(&self) -> u64 {
        self.maximum_active_milliseconds
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Reservation {
    allowance: SoakAllowance,
    complete: bool,
}

/// Stored apart from the legacy campaign and the qualification ledger, so neither changes shape.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SoakLedger {
    highest_ordinal: u32,
    total_charged_ms: u64,
    maybe_last: Option<Reservation>,
}

#[derive(Debug, Error)]
#[error("soak allowance admission rejected")]
pub struct SoakRejected;

impl SoakLedger {
    /// Every soak charges its whole allowance, so the total is exact.
    pub fn validate(&self) -> Result<(), SoakRejected> {
        let expected = u64::from(self.highest_ordinal)
            .checked_mul(SOAK_MAXIMUM_ACTIVE_MS)
            .ok_or(SoakRejected)?;
        if self.total_charged_ms > MAX_SAFE_INTEGER || self.total_charged_ms != expected {
            return Err(SoakRejected);
        }
        match &self.maybe_last {
            None if self.highest_ordinal == 0 => Ok(()),
            Some(last)
                if last.allowance.validate() && last.allowance.ordinal == self.highest_ordinal =>
            {
                Ok(())
            }
            _ => Err(SoakRejected),
        }
    }

    /// Reserves the entire allowance before preparation, rejecting gaps, replays and a pending soak.
    pub fn reserve(&self, allowance: &SoakAllowance) -> Result<Self, SoakRejected> {
        self.validate()?;
        if !allowance.validate()
            || self.pending()
            || u64::from(allowance.ordinal) != self.next_ordinal()
        {
            return Err(SoakRejected);
        }
        let next = Self {
            highest_ordinal: allowance.ordinal,
            total_charged_ms: self
                .total_charged_ms
                .checked_add(allowance.maximum_active_milliseconds)
                .ok_or(SoakRejected)?,
            maybe_last: Some(Reservation {
                allowance: allowance.clone(),
                complete: false,
            }),
        };
        next.validate()?;
        Ok(next)
    }

    /// Finalization is idempotent and never reduces the charge.
    pub fn finish(&self) -> Result<Self, SoakRejected> {
        self.validate()?;
        let mut next = self.clone();
        if let Some(last) = next.maybe_last.as_mut() {
            last.complete = true;
        }
        Ok(next)
    }

    #[must_use]
    pub fn pending(&self) -> bool {
        self.maybe_last.as_ref().is_some_and(|last| !last.complete)
    }
    #[must_use]
    pub const fn next_ordinal(&self) -> u64 {
        self.highest_ordinal as u64 + 1
    }
    #[must_use]
    pub const fn total_charged_ms(&self) -> u64 {
        self.total_charged_ms
    }
    #[must_use]
    pub fn last_completed_ordinal(&self) -> u32 {
        if self.pending() {
            self.highest_ordinal.saturating_sub(1)
        } else {
            self.highest_ordinal
        }
    }
    #[must_use]
    pub fn maybe_last_allowance(&self) -> Option<&SoakAllowance> {
        self.maybe_last.as_ref().map(|last| &last.allowance)
    }
}

#[cfg(test)]
#[path = "soak/tests.rs"]
mod tests;
