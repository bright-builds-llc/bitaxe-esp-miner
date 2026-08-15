mod command_effects;
mod command_evidence;
mod command_witness;
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
use command_effects::observe_command_effects;
pub(crate) use command_effects::{
    respond_identify_checkpoint, IdentifyCheckpointKind, IdentifyCheckpointOutcome,
};
pub(crate) use model::CampaignNetworkEvidence;
use model::SharedSerialState;
use observer::observe_network;
use serial::NetworkSerialTracker;

pub(crate) struct CampaignNetworkCoordinator {
    tracker: NetworkSerialTracker,
    shared: Arc<Mutex<SharedSerialState>>,
    maybe_worker: Option<JoinHandle<CampaignNetworkEvidence>>,
    stage: MiningCampaignStage,
    evidence_root: Utf8PathBuf,
    maybe_target_deadline: Option<Instant>,
}

impl CampaignNetworkCoordinator {
    pub(crate) fn new(
        admission: CampaignAdmission,
        expected: ExpectedRuntimeAttestationIdentity,
        evidence_root: Utf8PathBuf,
    ) -> Self {
        Self {
            tracker: NetworkSerialTracker::new(expected),
            shared: Arc::new(Mutex::new(SharedSerialState::default())),
            maybe_worker: None,
            stage: admission.stage,
            evidence_root,
            maybe_target_deadline: matches!(admission.stage, MiningCampaignStage::CommandEffects)
                .then(|| Instant::now() + Duration::from_secs(admission.duration_seconds)),
        }
    }

    pub(crate) fn observe_serial_chunk(&mut self, bytes: &[u8]) {
        if !matches!(
            self.stage,
            MiningCampaignStage::Soak | MiningCampaignStage::CommandEffects
        ) {
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
        let stage = self.stage;
        let evidence_root = self.evidence_root.clone();
        self.maybe_worker = Some(std::thread::spawn(move || match stage {
            MiningCampaignStage::Soak => observe_network(target, shared),
            MiningCampaignStage::CommandEffects => {
                observe_command_effects(target, shared, &evidence_root)
            }
            _ => CampaignNetworkEvidence::not_required(),
        }));
    }

    pub(crate) fn should_stop(&self) -> bool {
        self.shared.lock().map_or(true, |mut state| {
            if self.maybe_worker.is_none()
                && matches!(self.stage, MiningCampaignStage::CommandEffects)
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

    pub(crate) fn finish(mut self) -> CampaignNetworkEvidence {
        if !matches!(
            self.stage,
            MiningCampaignStage::Soak | MiningCampaignStage::CommandEffects
        ) {
            return CampaignNetworkEvidence::not_required();
        }
        self.tracker.finish(&self.shared);
        if let Ok(mut shared) = self.shared.lock() {
            shared.serial_finished = true;
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

pub(crate) struct CampaignObservationCapture {
    pub(crate) serial: CampaignSerialCapture,
    pub(crate) network: CampaignNetworkEvidence,
}
