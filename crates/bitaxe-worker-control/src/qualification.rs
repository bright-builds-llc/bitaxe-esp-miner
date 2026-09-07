//! Signed iterative qualification allowances and independent durable accounting.
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROFILE: &str = "worker-qualification-attempt-v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationPurpose {
    Diagnostic,
    Normal,
    ForegroundLoss,
    HeartbeatLoss,
}
impl QualificationPurpose {
    pub const fn maximum_active_ms(self) -> u64 {
        match self {
            Self::Normal => 180_000,
            _ => 30_000,
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Diagnostic => "diagnostic",
            Self::Normal => "normal",
            Self::ForegroundLoss => "foreground_loss",
            Self::HeartbeatLoss => "heartbeat_loss",
        }
    }
}

/// Field order is canonical because authorization hashing serializes directly.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QualificationAttempt {
    id: String,
    maximum_active_milliseconds: u64,
    ordinal: u32,
    purpose: QualificationPurpose,
    schema: String,
}
impl QualificationAttempt {
    pub fn validate(&self) -> bool {
        self.schema == PROFILE
            && crate::serial::canonical_nonce(&self.id, 16)
            && self.ordinal > 0
            && self.maximum_active_milliseconds == self.purpose.maximum_active_ms()
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
    pub const fn purpose(&self) -> QualificationPurpose {
        self.purpose
    }
    pub const fn maximum_active_milliseconds(&self) -> u64 {
        self.maximum_active_milliseconds
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Reservation {
    allowance: QualificationAttempt,
    complete: bool,
}

/// Stored separately from the immutable legacy acceptance campaign.
#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationLedger {
    highest_ordinal: u32,
    total_charged_ms: u64,
    maybe_last: Option<Reservation>,
}
#[derive(Debug, Error)]
#[error("qualification attempt admission rejected")]
pub struct QualificationRejected;
impl QualificationLedger {
    pub fn validate(&self) -> Result<(), QualificationRejected> {
        if self.total_charged_ms > MAX_SAFE_INTEGER || self.total_charged_ms % 30_000 != 0 {
            return Err(QualificationRejected);
        }
        match &self.maybe_last {
            None if self.highest_ordinal == 0 && self.total_charged_ms == 0 => Ok(()),
            Some(last)
                if last.allowance.validate()
                    && last.allowance.ordinal == self.highest_ordinal
                    && self.total_charged_ms >= last.allowance.maximum_active_milliseconds
                    && self.total_charged_ms >= u64::from(self.highest_ordinal) * 30_000
                    && self.total_charged_ms <= u64::from(self.highest_ordinal) * 180_000 =>
            {
                let normal_extra = self.total_charged_ms - u64::from(self.highest_ordinal) * 30_000;
                let normals = normal_extra / 150_000;
                let last_is_normal = last.allowance.purpose == QualificationPurpose::Normal;
                if normal_extra % 150_000 != 0
                    || (last_is_normal && normals == 0)
                    || (!last_is_normal && normals >= u64::from(self.highest_ordinal))
                {
                    return Err(QualificationRejected);
                }
                Ok(())
            }
            _ => Err(QualificationRejected),
        }
    }
    /// Reserves the entire allowance before preparation, rejecting gaps and replays.
    pub fn reserve(&self, allowance: &QualificationAttempt) -> Result<Self, QualificationRejected> {
        self.validate()?;
        if !allowance.validate()
            || self.pending()
            || u64::from(allowance.ordinal) != self.next_ordinal()
        {
            return Err(QualificationRejected);
        }
        let total = self
            .total_charged_ms
            .checked_add(allowance.maximum_active_milliseconds)
            .ok_or(QualificationRejected)?;
        let next = Self {
            highest_ordinal: allowance.ordinal,
            total_charged_ms: total,
            maybe_last: Some(Reservation {
                allowance: allowance.clone(),
                complete: false,
            }),
        };
        next.validate()?;
        Ok(next)
    }
    /// Finalization is idempotent and can never reduce charged duration.
    pub fn finish(&self) -> Result<Self, QualificationRejected> {
        self.validate()?;
        let mut next = self.clone();
        if let Some(last) = next.maybe_last.as_mut() {
            last.complete = true;
        }
        Ok(next)
    }
    pub fn pending(&self) -> bool {
        self.maybe_last.as_ref().is_some_and(|last| !last.complete)
    }
    pub const fn next_ordinal(&self) -> u64 {
        self.highest_ordinal as u64 + 1
    }
    pub const fn total_charged_ms(&self) -> u64 {
        self.total_charged_ms
    }
    pub fn last_completed_ordinal(&self) -> u32 {
        if self.pending() {
            self.highest_ordinal.saturating_sub(1)
        } else {
            self.highest_ordinal
        }
    }
    pub fn maybe_last_allowance(&self) -> Option<&QualificationAttempt> {
        self.maybe_last.as_ref().map(|last| &last.allowance)
    }
}
