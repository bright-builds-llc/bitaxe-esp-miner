mod model;
mod observer;
mod serial;
mod validation;

#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use bitaxe_api::ExpectedRuntimeAttestationIdentity;

use super::*;
pub(crate) use model::CampaignNetworkEvidence;
use model::SharedSerialState;
use observer::observe_network;
use serial::NetworkSerialTracker;

pub(crate) struct CampaignNetworkCoordinator {
    tracker: NetworkSerialTracker,
    shared: Arc<Mutex<SharedSerialState>>,
    maybe_worker: Option<JoinHandle<CampaignNetworkEvidence>>,
    enabled: bool,
}

impl CampaignNetworkCoordinator {
    pub(crate) fn new(
        admission: CampaignAdmission,
        expected: ExpectedRuntimeAttestationIdentity,
    ) -> Self {
        Self {
            tracker: NetworkSerialTracker::new(expected),
            shared: Arc::new(Mutex::new(SharedSerialState::default())),
            maybe_worker: None,
            enabled: admission.stage == MiningCampaignStage::Soak,
        }
    }

    pub(crate) fn observe_serial_chunk(&mut self, bytes: &[u8]) {
        if !self.enabled {
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
        self.maybe_worker = Some(std::thread::spawn(move || observe_network(target, shared)));
    }

    pub(crate) fn finish(mut self) -> CampaignNetworkEvidence {
        if !self.enabled {
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
