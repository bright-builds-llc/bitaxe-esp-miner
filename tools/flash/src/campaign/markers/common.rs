use super::*;

pub(super) fn expected_profile_marker(admission: CampaignAdmission) -> CampaignProfileMarker {
    match admission.maybe_profile {
        Some(MiningCampaignProfile::Conservative) => CampaignProfileMarker::Conservative,
        Some(MiningCampaignProfile::UpstreamDefault) => CampaignProfileMarker::UpstreamDefault,
        None => CampaignProfileMarker::None,
    }
}
