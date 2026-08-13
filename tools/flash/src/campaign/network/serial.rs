use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use bitaxe_api::{ExpectedRuntimeAttestationIdentity, RuntimeBootAttestation};

use super::super::markers::{
    CampaignStateMarker, CampaignStatusMarker, ResumablePauseSafeStopMarker,
};
use super::super::*;
use super::model::{SharedSerialState, TrustedNetworkTarget, REQUIRED_WINDOWS, WINDOW_MILLIS};

const MAX_PENDING_SERIAL_BYTES: usize = 65_536;
const MAX_ATTESTATION_SAMPLES: usize = 8;
const RUNTIME_ORIGIN_PREFIX: &str = "runtime_origin ";

#[derive(Clone)]
struct OriginCandidate {
    session: String,
    boot_ordinal: u64,
    origin: String,
}

pub(super) struct NetworkSerialTracker {
    expected: ExpectedRuntimeAttestationIdentity,
    pending: Vec<u8>,
    attestations: Vec<RuntimeBootAttestation>,
    origins: Vec<OriginCandidate>,
    maybe_last_attestation_uptime_ms: Option<u64>,
    maybe_previous_active_ms: Option<u64>,
    malformed: bool,
}

impl NetworkSerialTracker {
    pub(super) fn new(expected: ExpectedRuntimeAttestationIdentity) -> Self {
        Self {
            expected,
            pending: Vec::new(),
            attestations: Vec::new(),
            origins: Vec::new(),
            maybe_last_attestation_uptime_ms: None,
            maybe_previous_active_ms: None,
            malformed: false,
        }
    }

    pub(super) fn observe(&mut self, bytes: &[u8], shared: &Arc<Mutex<SharedSerialState>>) {
        self.pending.extend_from_slice(bytes);
        while let Some(newline) = self.pending.iter().position(|byte| *byte == b'\n') {
            let mut line = self.pending.drain(..=newline).collect::<Vec<_>>();
            line.pop();
            self.process_line(&line, shared);
        }
        if self.pending.len() > MAX_PENDING_SERIAL_BYTES {
            self.pending.clear();
            self.malformed = true;
            fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
        }
    }

    pub(super) fn finish(&mut self, shared: &Arc<Mutex<SharedSerialState>>) {
        if !self.pending.is_empty() {
            let line = std::mem::take(&mut self.pending);
            self.process_line(&line, shared);
        }
        if self.maybe_trusted_target().is_none() {
            fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
        }
    }

    pub(super) fn maybe_trusted_target(&self) -> Option<TrustedNetworkTarget> {
        if self.malformed || self.attestations.len() < 2 || self.origins.is_empty() {
            return None;
        }
        let first = self.attestations.first()?;
        if first.firmware_commit() != self.expected.firmware_commit
            || first.reference_commit() != self.expected.reference_commit
            || first.app_elf_sha256() != self.expected.app_elf_sha256
        {
            return None;
        }
        let mut previous_uptime = None;
        for attestation in &self.attestations {
            if attestation.session() != first.session()
                || attestation.boot_ordinal() != first.boot_ordinal()
                || attestation.firmware_commit() != self.expected.firmware_commit
                || attestation.reference_commit() != self.expected.reference_commit
                || attestation.app_elf_sha256() != self.expected.app_elf_sha256
                || previous_uptime.is_some_and(|uptime| attestation.uptime_ms() <= uptime)
            {
                return None;
            }
            previous_uptime = Some(attestation.uptime_ms());
        }
        if self.origins.iter().any(|origin| {
            origin.session != first.session() || origin.boot_ordinal != first.boot_ordinal()
        }) {
            return None;
        }
        let unique: BTreeSet<_> = self.origins.iter().map(|origin| &origin.origin).collect();
        if unique.len() != 1 {
            return None;
        }
        Some(TrustedNetworkTarget {
            origin: self.origins.first()?.origin.clone(),
            boot_session: first.session().to_owned(),
            boot_ordinal: first.boot_ordinal(),
            expected: self.expected.clone(),
        })
    }

    fn process_line(&mut self, line: &[u8], shared: &Arc<Mutex<SharedSerialState>>) {
        let Ok(line) = std::str::from_utf8(line) else {
            return;
        };
        if line.contains(bitaxe_api::RUNTIME_BOOT_ATTESTATION_MARKER) {
            match RuntimeBootAttestation::parse(line) {
                Ok(attestation) if self.attestation_is_consistent(&attestation) => {
                    self.maybe_last_attestation_uptime_ms = Some(attestation.uptime_ms());
                    if self.attestations.len() < MAX_ATTESTATION_SAMPLES {
                        self.attestations.push(attestation);
                    }
                }
                Ok(_) => {
                    self.malformed = true;
                    fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
                }
                Err(_) => {
                    self.malformed = true;
                    fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
                }
            }
        }
        if let Some(index) = line.find(RUNTIME_ORIGIN_PREFIX) {
            match parse_origin(&line[index + RUNTIME_ORIGIN_PREFIX.len()..]) {
                Some(origin) => {
                    if !self.origin_matches_attested_session(&origin) {
                        self.malformed = true;
                        fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
                        return;
                    }
                    self.origins.push(origin);
                    if unique_origin_count(&self.origins) > 1 {
                        self.malformed = true;
                        fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
                    }
                }
                None => {
                    self.malformed = true;
                    fail_shared(shared, CampaignTerminalCategory::NetworkTargetUnavailable);
                }
            }
        }
        if let Some(index) = line.find(CAMPAIGN_MARKER_PREFIX) {
            let payload = &line[index + CAMPAIGN_MARKER_PREFIX.len()..];
            let Ok(marker) = serde_json::from_str::<CampaignStatusMarker>(payload) else {
                return;
            };
            if marker.schema != CAMPAIGN_MARKER_SCHEMA {
                return;
            }
            self.observe_marker(&marker, shared);
        }
    }

    fn attestation_is_consistent(&self, attestation: &RuntimeBootAttestation) -> bool {
        if attestation.firmware_commit() != self.expected.firmware_commit
            || attestation.reference_commit() != self.expected.reference_commit
            || attestation.app_elf_sha256() != self.expected.app_elf_sha256
            || self
                .maybe_last_attestation_uptime_ms
                .is_some_and(|uptime| attestation.uptime_ms() <= uptime)
        {
            return false;
        }
        self.attestations.first().is_none_or(|first| {
            attestation.session() == first.session()
                && attestation.boot_ordinal() == first.boot_ordinal()
        })
    }

    fn origin_matches_attested_session(&self, origin: &OriginCandidate) -> bool {
        self.attestations.first().is_none_or(|first| {
            origin.session == first.session() && origin.boot_ordinal == first.boot_ordinal()
        })
    }

    fn observe_marker(
        &mut self,
        marker: &CampaignStatusMarker,
        shared: &Arc<Mutex<SharedSerialState>>,
    ) {
        let Ok(mut state) = shared.lock() else {
            return;
        };
        state.latest_active_ms = state.latest_active_ms.max(marker.active_ms);
        state.active = marker.campaign_state == CampaignStateMarker::Active;
        let Ok(confirmed) =
            resumable_pause_safe_stop_confirmation(marker.stage, marker.resumable_pause_safe_stop)
        else {
            state
                .maybe_failure
                .get_or_insert(CampaignTerminalCategory::MarkerInvalid);
            return;
        };
        state.resumable_pause_safe_stop_confirmed = confirmed;
        if matches!(
            marker.campaign_state,
            CampaignStateMarker::Active | CampaignStateMarker::SafeStopping
        ) {
            if let Some(previous) = self.maybe_previous_active_ms {
                state.maximum_active_marker_gap_ms = state
                    .maximum_active_marker_gap_ms
                    .max(marker.active_ms.saturating_sub(previous));
            }
            self.maybe_previous_active_ms = Some(marker.active_ms);
        }
        if marker.campaign_state == CampaignStateMarker::Active && marker.active_ms < 600_000 {
            let index = usize::try_from(marker.active_ms / WINDOW_MILLIS)
                .unwrap_or(REQUIRED_WINDOWS - 1)
                .min(REQUIRED_WINDOWS - 1);
            state.serial_windows[index].observe(marker.asic_bridge.poll_request_count);
        }
        if marker.campaign_state == CampaignStateMarker::Consumed {
            state.terminal_consumed = true;
            state.terminal_pool_persisted = marker.pool_config_persisted;
        }
    }
}

fn resumable_pause_safe_stop_confirmation(
    stage: MiningCampaignStage,
    status: ResumablePauseSafeStopMarker,
) -> Result<bool, ()> {
    if stage != MiningCampaignStage::CommandEffects
        && status != ResumablePauseSafeStopMarker::NotRequired
    {
        return Err(());
    }
    Ok(status == ResumablePauseSafeStopMarker::Confirmed)
}

fn parse_origin(fields: &str) -> Option<OriginCandidate> {
    let fields: Vec<_> = fields.split_whitespace().collect();
    if fields.len() != 4 || fields[3] != "redacted=true" {
        return None;
    }
    let session = fields[0].strip_prefix("session=")?;
    if session.len() != 32
        || !session
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let boot_ordinal = fields[1].strip_prefix("boot_ordinal=")?.parse().ok()?;
    let origin = fields[2].strip_prefix("device_url=")?;
    let authority = origin.strip_prefix("http://")?;
    if authority.is_empty()
        || authority.chars().any(|character| {
            character.is_whitespace() || matches!(character, '/' | '@' | '?' | '#')
        })
    {
        return None;
    }
    Some(OriginCandidate {
        session: session.to_owned(),
        boot_ordinal,
        origin: origin.to_owned(),
    })
}

fn unique_origin_count(origins: &[OriginCandidate]) -> usize {
    origins
        .iter()
        .map(|origin| &origin.origin)
        .collect::<BTreeSet<_>>()
        .len()
}

fn fail_shared(shared: &Arc<Mutex<SharedSerialState>>, category: CampaignTerminalCategory) {
    if let Ok(mut state) = shared.lock() {
        state.maybe_failure.get_or_insert(category);
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::markers::ResumablePauseSafeStopMarker;
    use super::super::super::MiningCampaignStage;
    use super::resumable_pause_safe_stop_confirmation;

    #[test]
    fn command_effects_safe_stop_markers_set_and_clear_confirmation() {
        // Arrange / Act
        let pending = resumable_pause_safe_stop_confirmation(
            MiningCampaignStage::CommandEffects,
            ResumablePauseSafeStopMarker::Pending,
        );
        let confirmed = resumable_pause_safe_stop_confirmation(
            MiningCampaignStage::CommandEffects,
            ResumablePauseSafeStopMarker::Confirmed,
        );
        let cleared = resumable_pause_safe_stop_confirmation(
            MiningCampaignStage::CommandEffects,
            ResumablePauseSafeStopMarker::NotRequired,
        );

        // Assert
        assert_eq!(pending, Ok(false));
        assert_eq!(confirmed, Ok(true));
        assert_eq!(cleared, Ok(false));
    }

    #[test]
    fn non_command_effects_safe_stop_confirmation_is_rejected() {
        // Arrange / Act
        let result = resumable_pause_safe_stop_confirmation(
            MiningCampaignStage::Observation,
            ResumablePauseSafeStopMarker::Confirmed,
        );

        // Assert
        assert_eq!(result, Err(()));
    }
}
