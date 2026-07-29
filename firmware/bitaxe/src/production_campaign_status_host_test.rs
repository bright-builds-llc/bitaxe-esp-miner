mod settings_adapter {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum MiningCampaignStage {
        Observation,
        LiveShare,
        Soak,
    }

    impl MiningCampaignStage {
        pub(crate) const fn label(self) -> &'static str {
            match self {
                Self::Observation => "observation",
                Self::LiveShare => "live-share",
                Self::Soak => "soak",
            }
        }
    }
}

#[path = "production_mining_session/campaign_status.rs"]
mod campaign_status;
