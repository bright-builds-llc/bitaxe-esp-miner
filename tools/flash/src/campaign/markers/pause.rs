use super::MiningCampaignStage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum SafeStopMarker {
    NotRequired,
    Pending,
    Confirmed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum ResumablePauseSafeStopMarker {
    NotRequired,
    Pending,
    Confirmed,
}

impl ResumablePauseSafeStopMarker {
    pub(super) fn is_valid_for(self, stage: MiningCampaignStage) -> bool {
        stage == MiningCampaignStage::CommandEffects || self == Self::NotRequired
    }
}
