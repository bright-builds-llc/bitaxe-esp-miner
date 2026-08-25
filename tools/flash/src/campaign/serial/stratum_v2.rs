use super::*;

impl CampaignSerialAnalyzer {
    pub(super) fn v2_assessment_succeeded(&self) -> bool {
        self.admission.stage == MiningCampaignStage::StratumV2 && self.stratum_v2.assess().is_ok()
    }

    pub(super) fn process_v2_runtime(&mut self, payload: &[u8]) {
        if self.admission.stage != MiningCampaignStage::StratumV2 {
            return;
        }
        let Ok(text) = std::str::from_utf8(payload) else {
            self.fail_v2_marker();
            return;
        };
        let Ok(marker) = serde_json::from_str::<StratumV2RuntimeMarker>(text) else {
            self.fail_v2_marker();
            return;
        };
        if marker.schema != STRATUM_V2_RUNTIME_SCHEMA
            || self.stratum_v2.observe_runtime(marker).is_err()
        {
            self.fail_v2_marker();
        }
    }

    pub(super) fn process_v2_terminal(&mut self, payload: &[u8]) {
        if self.admission.stage != MiningCampaignStage::StratumV2 {
            return;
        }
        let Ok(text) = std::str::from_utf8(payload) else {
            self.fail_v2_marker();
            return;
        };
        let Ok(marker) = serde_json::from_str::<StratumV2TerminalMarker>(text) else {
            self.fail_v2_marker();
            return;
        };
        if marker.schema != STRATUM_V2_TERMINAL_SCHEMA {
            self.fail_v2_marker();
            return;
        }
        self.stratum_v2.terminal = Some(marker);
    }

    fn fail_v2_marker(&mut self) {
        self.maybe_failure
            .get_or_insert(CampaignTerminalCategory::MarkerInvalid);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StratumV2RuntimeStage {
    HardwarePrepared,
    ChannelReady,
    WorkDispatched,
    TargetUpdated,
    ShareAccepted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct StratumV2RuntimeMarker {
    schema: String,
    stage: StratumV2RuntimeStage,
    sequence: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StratumV2TerminalKind {
    Accepted,
    Asic,
    CampaignContract,
    CampaignMismatch,
    Deadline,
    HardwarePreparation,
    PoolConfiguration,
    Preflight,
    Protocol,
    Safety,
    Session,
    ShareRejected,
    Transport,
    TransportClosed,
    TransportQueue,
    TransportWorker,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StratumV2TerminalDetail {
    NotApplicable,
    Resolve,
    Connect,
    Configure,
    Handshake,
    Write,
    Read,
    Frame,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct StratumV2TerminalMarker {
    schema: String,
    category: StratumV2TerminalKind,
    detail: StratumV2TerminalDetail,
    accepted: bool,
    safe_stop_complete: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct StratumV2Aggregate {
    runtime_count: u64,
    maybe_last_sequence: Option<u64>,
    hardware_prepared: bool,
    channel_ready: bool,
    work_dispatched: bool,
    share_accepted: bool,
    terminal: Option<StratumV2TerminalMarker>,
}

impl StratumV2Aggregate {
    fn observe_runtime(&mut self, marker: StratumV2RuntimeMarker) -> Result<(), ()> {
        if marker.sequence == 0
            || self
                .maybe_last_sequence
                .is_some_and(|previous| marker.sequence <= previous)
        {
            return Err(());
        }
        self.runtime_count = self.runtime_count.saturating_add(1);
        self.maybe_last_sequence = Some(marker.sequence);
        match marker.stage {
            StratumV2RuntimeStage::HardwarePrepared => self.hardware_prepared = true,
            StratumV2RuntimeStage::ChannelReady => self.channel_ready = true,
            StratumV2RuntimeStage::WorkDispatched => self.work_dispatched = true,
            StratumV2RuntimeStage::TargetUpdated => {}
            StratumV2RuntimeStage::ShareAccepted => self.share_accepted = true,
        }
        Ok(())
    }

    pub(crate) const fn marker_count(&self) -> u64 {
        self.runtime_count + if self.terminal.is_some() { 1 } else { 0 }
    }

    pub(crate) fn assess(&self) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
        let Some(terminal) = self.terminal.as_ref() else {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::MarkerMissing,
            ));
        };
        let detail_valid = match terminal.category {
            StratumV2TerminalKind::Transport => {
                terminal.detail != StratumV2TerminalDetail::NotApplicable
            }
            _ => terminal.detail == StratumV2TerminalDetail::NotApplicable,
        };
        if !detail_valid {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::MarkerInvalid,
            ));
        }
        if !terminal.safe_stop_complete {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::SafeStopUnconfirmed,
            ));
        }
        if terminal.category != StratumV2TerminalKind::Accepted || !terminal.accepted {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::SubmitResponseMissing,
            ));
        }
        if !(self.hardware_prepared
            && self.channel_ready
            && self.work_dispatched
            && self.share_accepted)
        {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::ObservationContractIncomplete,
            ));
        }
        Ok(CampaignTerminalCategory::StratumV2Accepted)
    }
}
