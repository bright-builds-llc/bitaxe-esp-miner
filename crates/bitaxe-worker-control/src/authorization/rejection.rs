//! Boot-local, metadata-only attribution of Work Lease authorization rejections.
//!
//! The log is RAM-only: it survives logical serial sessions because its owner is
//! the boot-lifetime verifier, and a reboot clears it. It never records key ids,
//! sequences, lease or challenge ids, bindings, or authorization bytes.

use serde::{Serialize, Serializer};

use super::LeaseAuthorizationError;
use crate::{RestorationReason, StateFingerprint};

/// Signed operation whose authorization was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationOperation {
    Start,
    Renew,
}

impl AuthorizationOperation {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Renew => "renew",
        }
    }
}

/// Whether the trusted-key signature was checked and what it proved.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureAttribution {
    Valid,
    Invalid,
    /// Rejected before `verify_strict` ran (malformed, unknown key, changed request).
    NotEvaluated,
}

/// The device's admission context relative to the presented authorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextAttribution {
    /// The signed binding equals the current-generation admission.
    Current,
    /// The request presented no binding equal to the current admission.
    Mismatch,
    /// A current-generation admission exists but is at least 60 s old.
    Expired,
    /// No current-generation admission exists.
    Absent,
}

/// What the read-only durable high-water comparison found.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayGuardAttribution {
    Fresh,
    AtOrBelowDurableHighWater,
    Unavailable,
    NotEvaluated,
}

/// One closed rejection attribution; `ordinal` counts rejections since boot.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRejectionRecord {
    pub ordinal: u32,
    pub operation: AuthorizationOperation,
    pub signature: SignatureAttribution,
    pub context: ContextAttribution,
    pub replay_guard: ReplayGuardAttribution,
    /// The safe stop this rejection itself triggered; `none` when it stopped nothing.
    #[serde(rename = "safeStop", serialize_with = "safe_stop_label")]
    pub maybe_safe_stop: Option<RestorationReason>,
}

fn safe_stop_label<S: Serializer>(
    maybe_reason: &Option<RestorationReason>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(maybe_reason.map_or("none", RestorationReason::category))
}

/// RAM-only rejection attribution plus whether this boot advanced the high-water.
#[derive(Debug, Default)]
pub struct AuthorizationRejectionLog {
    boot_rejections: u32,
    maybe_last: Option<AuthorizationRejectionRecord>,
    // Only the rejection just recorded may receive its safe stop, and only once,
    // so a later disconnect or unrelated stop can never rewrite the record.
    last_awaits_safe_stop: bool,
    high_water_advanced_this_boot: bool,
}

impl AuthorizationRejectionLog {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            boot_rejections: 0,
            maybe_last: None,
            last_awaits_safe_stop: false,
            high_water_advanced_this_boot: false,
        }
    }

    // Kept out of line so rejection bookkeeping never widens the signed-path frames.
    #[cold]
    #[inline(never)]
    pub(crate) fn record(
        &mut self,
        operation: AuthorizationOperation,
        signature: SignatureAttribution,
        context: ContextAttribution,
        replay_guard: ReplayGuardAttribution,
    ) {
        self.boot_rejections = self.boot_rejections.saturating_add(1);
        self.maybe_last = Some(AuthorizationRejectionRecord {
            ordinal: self.boot_rejections,
            operation,
            signature,
            context,
            replay_guard,
            maybe_safe_stop: None,
        });
        self.last_awaits_safe_stop = true;
    }

    /// Records the safe stop the rejection just recorded triggered; a no-op otherwise.
    #[cold]
    #[inline(never)]
    pub(crate) fn attribute_safe_stop(&mut self, reason: RestorationReason) {
        if !std::mem::take(&mut self.last_awaits_safe_stop) {
            return;
        }
        if let Some(last) = self.maybe_last.as_mut() {
            last.maybe_safe_stop = Some(reason);
        }
    }

    pub(crate) fn mark_high_water_advanced(&mut self) {
        self.last_awaits_safe_stop = false;
        self.high_water_advanced_this_boot = true;
    }

    #[must_use]
    pub const fn boot_rejections(&self) -> u32 {
        self.boot_rejections
    }

    #[must_use]
    pub const fn maybe_last(&self) -> Option<AuthorizationRejectionRecord> {
        self.maybe_last
    }

    #[must_use]
    pub const fn high_water_advanced_this_boot(&self) -> bool {
        self.high_water_advanced_this_boot
    }
}

/// Narrow read-only view used by the rejection review; it cannot sign, prove or write.
pub trait AuthorizationRejectionSource {
    fn rejection_log(&self) -> Option<&AuthorizationRejectionLog>;
    fn high_water_fingerprint(&self) -> Result<Option<StateFingerprint>, LeaseAuthorizationError>;
}

/// Exact `worker-authorization-rejection-review-v2` result body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRejectionReview {
    schema: &'static str,
    boot_rejections: u32,
    last: Option<AuthorizationRejectionRecord>,
    high_water: HighWaterReview,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HighWaterReview {
    advanced_this_boot: bool,
    fingerprint_sha256: StateFingerprint,
}

/// Builds the metadata-only review. `Ok(None)` means the source keeps no log.
pub fn authorization_rejection_review(
    source: &impl AuthorizationRejectionSource,
) -> Result<Option<AuthorizationRejectionReview>, LeaseAuthorizationError> {
    let Some(log) = source.rejection_log() else {
        return Ok(None);
    };
    let fingerprint = source
        .high_water_fingerprint()?
        .ok_or(LeaseAuthorizationError::Persistence)?;
    Ok(Some(AuthorizationRejectionReview {
        schema: "worker-authorization-rejection-review-v2",
        boot_rejections: log.boot_rejections(),
        last: log.maybe_last(),
        high_water: HighWaterReview {
            advanced_this_boot: log.high_water_advanced_this_boot(),
            fingerprint_sha256: fingerprint,
        },
    }))
}

#[cfg(test)]
#[path = "rejection_tests.rs"]
mod tests;
