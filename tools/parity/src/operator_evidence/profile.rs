use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
pub(crate) enum OperatorEvidenceProfile {
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperatorEvidenceRootEntry {
    Slot(OperatorEvidenceSlot),
    EvidenceContract,
}

impl OperatorEvidenceRootEntry {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Slot(slot) => slot.file_name(),
            Self::EvidenceContract => "evidence-contract.md",
        }
    }

    pub(crate) const fn is_directory(self) -> bool {
        false
    }
}

impl OperatorEvidenceProfile {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 1] = [Self::Release];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "release",
        }
    }

    pub(crate) const fn descriptor(self) -> OperatorEvidenceProfileDescriptor {
        OperatorEvidenceProfileDescriptor { profile: self }
    }
}

impl fmt::Display for OperatorEvidenceProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for OperatorEvidenceProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "release" => Ok(Self::Release),
            _ => Err(format!("unknown operator evidence profile {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OperatorEvidenceSlot {
    Package,
    Detector,
    BoardInfo,
    Command,
    Log,
    Api,
    Websocket,
    ShareOutcome,
    RedactionReview,
    SafeStop,
    Conclusion,
}

impl OperatorEvidenceSlot {
    pub(crate) const ALL: [Self; 11] = [
        Self::Package,
        Self::Detector,
        Self::BoardInfo,
        Self::Command,
        Self::Log,
        Self::Api,
        Self::Websocket,
        Self::ShareOutcome,
        Self::RedactionReview,
        Self::SafeStop,
        Self::Conclusion,
    ];

    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            Self::Package => "package.md",
            Self::Detector => "detector.md",
            Self::BoardInfo => "board-info.md",
            Self::Command => "command.md",
            Self::Log => "log.md",
            Self::Api => "api.md",
            Self::Websocket => "websocket.md",
            Self::ShareOutcome => "share-outcome.md",
            Self::RedactionReview => "redaction-review.md",
            Self::SafeStop => "safe-stop.md",
            Self::Conclusion => "conclusion.md",
        }
    }

    pub(crate) const fn slot_name(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Detector => "detector",
            Self::BoardInfo => "board-info",
            Self::Command => "command",
            Self::Log => "log",
            Self::Api => "api",
            Self::Websocket => "websocket",
            Self::ShareOutcome => "share-outcome",
            Self::RedactionReview => "redaction-review",
            Self::SafeStop => "safe-stop",
            Self::Conclusion => "conclusion",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceDisposition {
    Observed,
    CrossLinked,
    Blocked,
    Deferred,
}

impl EvidenceDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::CrossLinked => "cross_linked",
            Self::Blocked => "blocked",
            Self::Deferred => "deferred",
        }
    }
}

impl FromStr for EvidenceDisposition {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "observed" => Ok(Self::Observed),
            "cross_linked" => Ok(Self::CrossLinked),
            "blocked" => Ok(Self::Blocked),
            "deferred" => Ok(Self::Deferred),
            _ => Err(format!("unknown evidence disposition {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShareOutcome {
    Accepted,
    Rejected,
    LiveSubmitResponseObserved,
    BlockedSafePrerequisite,
}

impl ShareOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::LiveSubmitResponseObserved => "live_submit_response_observed",
            Self::BlockedSafePrerequisite => "blocked_safe_prerequisite",
        }
    }
}

impl FromStr for ShareOutcome {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "live_submit_response_observed" => Ok(Self::LiveSubmitResponseObserved),
            "blocked_safe_prerequisite" => Ok(Self::BlockedSafePrerequisite),
            _ => Err(format!("unknown share outcome {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OperatorEvidenceProfileDescriptor {
    profile: OperatorEvidenceProfile,
}

impl OperatorEvidenceProfileDescriptor {
    pub(crate) const fn slots(self) -> [OperatorEvidenceSlot; 11] {
        OperatorEvidenceSlot::ALL
    }

    pub(crate) fn root_entries(self) -> Vec<OperatorEvidenceRootEntry> {
        let mut entries = self
            .slots()
            .into_iter()
            .map(OperatorEvidenceRootEntry::Slot)
            .collect::<Vec<_>>();
        match self.profile {
            OperatorEvidenceProfile::Release => {
                entries.push(OperatorEvidenceRootEntry::EvidenceContract);
            }
        }
        entries
    }

    pub(crate) const fn allows_disposition(
        self,
        _slot: OperatorEvidenceSlot,
        disposition: EvidenceDisposition,
    ) -> bool {
        match self.profile {
            OperatorEvidenceProfile::Release => {
                !matches!(disposition, EvidenceDisposition::CrossLinked)
            }
        }
    }

    pub(crate) const fn requires_observation(self, _slot: OperatorEvidenceSlot) -> bool {
        match self.profile {
            OperatorEvidenceProfile::Release => false,
        }
    }

    pub(crate) const fn generated_provenance_required(
        self,
        disposition: EvidenceDisposition,
    ) -> bool {
        matches!(
            disposition,
            EvidenceDisposition::CrossLinked
                | EvidenceDisposition::Blocked
                | EvidenceDisposition::Deferred
        )
    }
}
