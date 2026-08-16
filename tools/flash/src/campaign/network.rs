mod command_effects;
mod command_evidence;
mod command_witness;
mod hashrate;
mod model;
mod observer;
mod serial;
mod validation;

#[cfg(test)]
mod test_evidence;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use bitaxe_api::ExpectedRuntimeAttestationIdentity;
use camino::Utf8PathBuf;

use super::*;
use crate::campaign::markers::{CampaignStateMarker, CampaignTerminalReasonMarker};
use command_effects::observe_command_effects;
pub(crate) use command_effects::{
    respond_identify_checkpoint, IdentifyCheckpointKind, IdentifyCheckpointOutcome,
};
pub(crate) use model::CampaignNetworkEvidence;
use model::SharedSerialState;
use observer::observe_network;
use serial::NetworkSerialTracker;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NetworkObservationMode {
    NotRequired,
    Continuity,
    CommandEffects,
}

impl NetworkObservationMode {
    const fn for_stage(stage: MiningCampaignStage) -> Self {
        match stage {
            MiningCampaignStage::Observation | MiningCampaignStage::JobTransition => {
                Self::NotRequired
            }
            MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => Self::Continuity,
            MiningCampaignStage::CommandEffects => Self::CommandEffects,
        }
    }
}

pub(crate) struct CampaignNetworkCoordinator {
    tracker: NetworkSerialTracker,
    shared: Arc<Mutex<SharedSerialState>>,
    maybe_worker: Option<JoinHandle<CampaignNetworkEvidence>>,
    observation_mode: NetworkObservationMode,
    evidence_root: Utf8PathBuf,
    maybe_target_deadline: Option<Instant>,
}

impl CampaignNetworkCoordinator {
    pub(crate) fn new(
        admission: CampaignAdmission,
        expected: ExpectedRuntimeAttestationIdentity,
        evidence_root: Utf8PathBuf,
    ) -> Self {
        let observation_mode = NetworkObservationMode::for_stage(admission.stage);
        Self {
            tracker: NetworkSerialTracker::new(expected),
            shared: Arc::new(Mutex::new(SharedSerialState::default())),
            maybe_worker: None,
            observation_mode,
            evidence_root,
            maybe_target_deadline: matches!(
                observation_mode,
                NetworkObservationMode::CommandEffects
            )
            .then(|| Instant::now() + Duration::from_secs(admission.duration_seconds)),
        }
    }

    pub(crate) fn observe_serial_chunk(&mut self, bytes: &[u8]) {
        if self.observation_mode == NetworkObservationMode::NotRequired {
            return;
        }
        self.tracker.observe(bytes, &self.shared);
        if self.maybe_worker.is_some() {
            return;
        }
        let Some(target) = self.tracker.maybe_trusted_target() else {
            return;
        };
        let shared = Arc::clone(&self.shared);
        let observation_mode = self.observation_mode;
        let evidence_root = self.evidence_root.clone();
        self.maybe_worker = Some(std::thread::spawn(move || match observation_mode {
            NetworkObservationMode::Continuity => observe_network(target, shared),
            NetworkObservationMode::CommandEffects => {
                observe_command_effects(target, shared, &evidence_root)
            }
            NetworkObservationMode::NotRequired => CampaignNetworkEvidence::not_required(),
        }));
    }

    pub(crate) fn should_stop(&self) -> bool {
        self.shared.lock().map_or(true, |mut state| {
            if self.maybe_worker.is_none()
                && self.observation_mode == NetworkObservationMode::CommandEffects
                && (state.maybe_failure.is_some()
                    || self
                        .maybe_target_deadline
                        .is_some_and(|deadline| Instant::now() >= deadline))
            {
                state
                    .maybe_failure
                    .get_or_insert(CampaignTerminalCategory::NetworkTargetUnavailable);
                state.network_stop_requested = true;
            }
            state.network_stop_requested
        })
    }

    pub(crate) fn finish(mut self, serial: &CampaignSerialCapture) -> CampaignNetworkEvidence {
        if self.observation_mode == NetworkObservationMode::NotRequired {
            return CampaignNetworkEvidence::not_required();
        }
        self.tracker.finish(&self.shared);
        if let Ok(mut shared) = self.shared.lock() {
            close_serial_input(&mut shared, terminal_capture_handoff(serial));
            if self.maybe_worker.is_none() && shared.maybe_failure.is_none() {
                shared.maybe_failure = Some(CampaignTerminalCategory::NetworkTargetUnavailable);
            }
        }
        let Some(worker) = self.maybe_worker.take() else {
            return CampaignNetworkEvidence::from_unobserved(&self.shared);
        };
        worker
            .join()
            .unwrap_or_else(|_| CampaignNetworkEvidence::worker_failed(&self.shared))
    }
}

#[derive(Clone, Copy)]
pub(super) struct TerminalCaptureHandoff {
    pub(super) terminal_consumed: bool,
    pub(super) pool_config_persisted: bool,
    pub(super) maybe_failure: Option<CampaignTerminalCategory>,
}

fn close_serial_input(
    shared: &mut SharedSerialState,
    maybe_terminal: Option<TerminalCaptureHandoff>,
) {
    if let Some(terminal) = maybe_terminal {
        if let Some(category) = terminal.maybe_failure {
            shared.maybe_failure.get_or_insert(category);
        }
        if terminal.terminal_consumed
            && shared.terminal_consumed
            && shared.terminal_pool_persisted != terminal.pool_config_persisted
        {
            shared
                .maybe_failure
                .get_or_insert(CampaignTerminalCategory::NetworkCorrelationFailed);
        } else if terminal.terminal_consumed {
            shared.terminal_consumed = true;
            shared.terminal_pool_persisted = terminal.pool_config_persisted;
        }
    }
    // The network worker must see the analyzer's terminal fact before it sees
    // input closure, because HTTP confirmation intentionally follows USB.
    shared.serial_finished = true;
}

pub(super) fn terminal_capture_handoff(
    serial: &CampaignSerialCapture,
) -> Option<TerminalCaptureHandoff> {
    serial.aggregate.terminal.as_ref().and_then(|marker| {
        let terminal_consumed = marker.campaign_state == CampaignStateMarker::Consumed;
        let consumed_reason_without_state = marker.terminal_reason
            == CampaignTerminalReasonMarker::CampaignLeaseConsumed
            && !terminal_consumed;
        (terminal_consumed || consumed_reason_without_state).then_some(TerminalCaptureHandoff {
            terminal_consumed,
            pool_config_persisted: marker.pool_config_persisted,
            maybe_failure: consumed_reason_without_state
                .then_some(CampaignTerminalCategory::TerminalStateUnconfirmed),
        })
    })
}

pub(crate) struct CampaignObservationCapture {
    pub(crate) serial: CampaignSerialCapture,
    pub(crate) network: CampaignNetworkEvidence,
}
