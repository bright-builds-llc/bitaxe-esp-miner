mod settings_adapter {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum MiningCampaignStage {
        Observation,
        LiveShare,
        Soak,
        #[allow(dead_code)]
        JobTransition,
        CommandEffects,
    }

    impl MiningCampaignStage {
        pub(crate) const fn label(self) -> &'static str {
            match self {
                Self::Observation => "observation",
                Self::LiveShare => "live-share",
                Self::Soak => "soak",
                Self::JobTransition => "job-transition",
                Self::CommandEffects => "command-effects",
            }
        }
    }
}

#[path = "operator_sensor_diagnostics.rs"]
mod operator_sensor_diagnostics;

#[path = "production_mining_session/readiness_trace.rs"]
#[allow(dead_code)]
mod readiness_trace;

#[path = "production_mining_session/campaign_status.rs"]
mod campaign_status;
