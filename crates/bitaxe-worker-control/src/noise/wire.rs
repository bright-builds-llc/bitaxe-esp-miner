use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::Ipv4Addr;
use zeroize::Zeroize;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoiseStart {
    pub schema: StartSchema,
    pub attempt_id: String,
    pub expected_boot_ordinal: u64,
    pub network_observed_at_us: u64,
    pub fixture_ipv4: String,
    pub fixture_port: u16,
    pub authority_public_key: String,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum StartSchema {
    #[serde(rename = "worker-noise-diagnostic-start-v2")]
    V2,
}
impl Drop for NoiseStart {
    fn drop(&mut self) {
        self.fixture_ipv4.zeroize();
        self.authority_public_key.zeroize();
    }
}
impl NoiseStart {
    pub fn valid(&self) -> bool {
        canonical_bytes::<16>(&self.attempt_id).is_some()
            && canonical_bytes::<32>(&self.authority_public_key).is_some()
            && self.expected_boot_ordinal > 0
            && self.expected_boot_ordinal <= super::MAX_SAFE_INTEGER
            && self.network_observed_at_us <= super::MAX_SAFE_INTEGER
            && self.fixture_port != 0
            && self
                .fixture_ipv4
                .parse::<Ipv4Addr>()
                .is_ok_and(|ip| ip.is_private() && ip.to_string() == self.fixture_ipv4)
    }
    pub fn input_sha256(&self) -> Option<String> {
        let value = serde_json::to_value(self).ok()?;
        let mut canonical = crate::codec::canonical_json(&value).ok()?;
        let digest = Sha256::digest(canonical.as_bytes());
        canonical.zeroize();
        Some(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }
}
pub fn canonical_bytes<const N: usize>(value: &str) -> Option<[u8; N]> {
    let bytes = URL_SAFE_NO_PAD.decode(value).ok()?;
    if URL_SAFE_NO_PAD.encode(&bytes) != value {
        return None;
    }
    bytes.try_into().ok()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoiseQuery {
    pub schema: QuerySchema,
    #[serde(deserialize_with = "required_nullable")]
    pub attempt_id: Option<String>,
}
fn required_nullable<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}
#[derive(Deserialize)]
pub enum QuerySchema {
    #[serde(rename = "worker-noise-diagnostic-query-v1")]
    V1,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseObservation {
    pub boot_ordinal: u64,
    pub worker_generation: u64,
    pub transport_epoch: u64,
    pub observed_at_us: u64,
    pub station_ipv4: Option<String>,
    pub wifi_connected: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseState {
    Idle,
    Admitted,
    Running,
    Cancelling,
    Terminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseStage {
    NoisePrepared,
    TcpConnected,
    ActOneWritten,
    ActTwoReceived,
    AuthorityVerified,
    ProofWritten,
    SocketClosed,
    WorkerQuiescent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureStage {
    NoisePrepared,
    TcpConnected,
    ActOneWritten,
    ActTwoReceived,
    AuthorityVerified,
    ProofWritten,
    SocketClosed,
    WorkerQuiescent,
    Admission,
    FixtureReady,
    CandidateInventory,
    ProofReceived,
    Cleanup,
    Evidence,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseCategory {
    Preparation,
    Connect,
    Write,
    Read,
    Authentication,
    Proof,
    AuthorityLost,
    ClockInvalid,
    Cleanup,
    EvidenceIncomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseDetail {
    Timeout,
    Eof,
    Partial,
    Extra,
    Malformed,
    Io,
    WrongAuthority,
    CertificateTime,
    SessionReplaced,
    HeartbeatExpired,
    CancelRequested,
    ClockDiscontinuity,
    DeliveryAmbiguous,
    ResourceUnreleased,
    IdentityConflict,
    PeerConflict,
    Overflow,
    Missing,
    Stale,
    Duplicate,
    Rng,
    Allocation,
    BeforeEpoch,
    TimeOverflow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseFailure {
    pub stage: FailureStage,
    pub category: NoiseCategory,
    pub detail: NoiseDetail,
    pub at_us: Option<u64>,
}
impl NoiseFailure {
    pub const fn new(
        stage: FailureStage,
        category: NoiseCategory,
        detail: NoiseDetail,
        now: Option<u64>,
    ) -> Self {
        Self {
            stage,
            category,
            detail,
            at_us: now,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseOutcome {
    Accepted,
    Rejected,
    Cancelled,
    Expired,
    Incomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseStageObservation {
    pub stage: NoiseStage,
    pub sequence: u8,
    pub at_us: Option<u64>,
    pub duration_us: Option<u64>,
    pub bytes: Option<u16>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SocketState {
    NotCreated,
    Open,
    Closed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerState {
    NotStarted,
    Running,
    Quiescent,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseRelease {
    pub socket_state: SocketState,
    pub worker_state: WorkerState,
    pub volatile_inputs_disposed: bool,
    pub started_at_us: Option<u64>,
    pub deadline_at_us: Option<u64>,
    pub released_at_us: Option<u64>,
    pub deadline_met: Option<bool>,
    pub failure: Option<NoiseFailure>,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseTerminal {
    pub outcome: NoiseOutcome,
    pub decided_at_us: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseJob {
    pub attempt_id: String,
    pub input_sha256: String,
    pub boot_ordinal: u64,
    pub worker_generation: u64,
    pub transport_epoch: u64,
    pub admitted_at_us: u64,
    pub authority_deadline_us: u64,
    pub local_socket_port: Option<u16>,
    pub stages: Vec<NoiseStageObservation>,
    pub first_failure: Option<NoiseFailure>,
    pub terminal: Option<NoiseTerminal>,
    pub resources: NoiseRelease,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseStatus {
    pub schema: &'static str,
    pub state: NoiseState,
    pub observation: NoiseObservation,
    pub job: Option<NoiseJob>,
}
