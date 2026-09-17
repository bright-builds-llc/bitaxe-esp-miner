use super::Scope;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Idle,
    Admitted,
    Running,
    Terminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Accepted,
    Rejected,
    Expired,
    Cancelled,
    Incomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Admitted,
    Preparing,
    Connected,
    Authenticated,
    Setup,
    Channel,
    Job,
    Target,
    WorkReady,
    SocketClosed,
    WorkerQuiescent,
    AsicDispatch,
    Nonce,
    Submission,
    Accepted,
    Revoked,
    Shutdown,
    Cooled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    Admission,
    Clock,
    Allocation,
    Authority,
    Timeout,
    Eof,
    Extra,
    Authentication,
    Protocol,
    ChannelMismatch,
    JobMismatch,
    InvalidNonce,
    RejectedShare,
    Safety,
    Cleanup,
    Evidence,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(usize)]
pub enum Operation {
    InitiatorConstruction,
    ActOneConstruction,
    Connect,
    ActOneWrite,
    ActTwoRead,
    ActTwoAuthentication,
    FrameEncrypt,
    FrameWrite,
    FrameRead,
    HeaderDecrypt,
    PayloadDecrypt,
    SocketClose,
    WorkerJoin,
}
impl Operation {
    pub const ALL: [Self; 13] = [
        Self::InitiatorConstruction,
        Self::ActOneConstruction,
        Self::Connect,
        Self::ActOneWrite,
        Self::ActTwoRead,
        Self::ActTwoAuthentication,
        Self::FrameEncrypt,
        Self::FrameWrite,
        Self::FrameRead,
        Self::HeaderDecrypt,
        Self::PayloadDecrypt,
        Self::SocketClose,
        Self::WorkerJoin,
    ];
}
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketTuple {
    pub local_ipv4: String,
    pub local_port: u16,
    pub remote_ipv4: String,
    pub remote_port: u16,
}
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentObservation {
    pub boot_ordinal: u64,
    pub worker_generation: u64,
    pub serial_transport_epoch: u64,
    #[serde(rename = "observedAtUs")]
    pub maybe_observed_at_us: Option<u64>,
    pub clock_valid: bool,
    #[serde(rename = "stationIpv4")]
    pub maybe_station_ipv4: Option<String>,
    pub wifi_connected: bool,
    #[serde(rename = "socket")]
    pub maybe_socket: Option<SocketTuple>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedConnection {
    pub observed_at_us: u64,
    pub boot_ordinal: u64,
    pub worker_generation: u64,
    pub serial_transport_epoch: u64,
    pub pool_session_generation: u64,
    pub pool_transport_epoch: u64,
    pub socket: SocketTuple,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2Status {
    pub schema: &'static str,
    pub scope: Scope,
    pub state: State,
    pub observation: CurrentObservation,
    #[serde(rename = "connection")]
    pub maybe_connection: Option<RetainedConnection>,
    #[serde(rename = "record")]
    pub maybe_record: Option<DeviceRecord>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub sequence: u64,
    #[serde(rename = "atDeviceUs")]
    pub maybe_at_device_us: Option<u64>,
    pub kind: Stage,
    #[serde(rename = "channelId")]
    pub maybe_channel_id: Option<u32>,
    #[serde(rename = "jobId")]
    pub maybe_job_id: Option<u32>,
    #[serde(rename = "submissionSequence")]
    pub maybe_submission_sequence: Option<u32>,
    #[serde(rename = "payloadSha256")]
    pub maybe_payload_sha256: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub operation: Operation,
    pub count: u64,
    pub failed_count: u64,
    #[serde(rename = "maxDurationUs")]
    pub maybe_max_duration_us: Option<u64>,
    #[serde(rename = "totalDurationUs")]
    pub maybe_total_duration_us: Option<u64>,
    #[serde(rename = "firstStartedAtDeviceUs")]
    pub maybe_first_started_at_device_us: Option<u64>,
    #[serde(rename = "lastFinishedAtDeviceUs")]
    pub maybe_last_finished_at_device_us: Option<u64>,
    #[serde(rename = "inFlightStartedAtDeviceUs")]
    pub maybe_in_flight_started_at_device_us: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareFact {
    pub dispatch_sequence: u64,
    pub asic_job_id: u8,
    pub work_fields_sha256: String,
    pub dispatched_at_device_us: u64,
    pub nonce_at_device_us: u64,
    #[serde(rename = "writeStartedAtDeviceUs")]
    pub maybe_write_started_at_device_us: Option<u64>,
    #[serde(rename = "writeCompletedAtDeviceUs")]
    pub maybe_write_completed_at_device_us: Option<u64>,
    pub nonce: u32,
    pub version_bits: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    pub channel_id: u32,
    pub job_id: u32,
    pub submission_sequence: u32,
    pub ntime: u32,
    pub version: u32,
    #[serde(rename = "ackAtDeviceUs")]
    pub maybe_ack_at_device_us: Option<u64>,
    #[serde(rename = "ackLastSequence")]
    pub maybe_ack_last_sequence: Option<u32>,
    #[serde(rename = "ackAcceptedCount")]
    pub maybe_ack_accepted_count: Option<u32>,
    #[serde(rename = "ackSharesSum")]
    pub maybe_ack_shares_sum: Option<u64>,
    #[serde(rename = "matchedSubmitCount")]
    pub maybe_matched_submit_count: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub stage: Stage,
    pub category: FailureCategory,
    #[serde(rename = "atDeviceUs")]
    pub maybe_at_device_us: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    pub socket_closed: bool,
    pub worker_quiescent: bool,
    pub fence_retained: bool,
    #[serde(rename = "socketClosedAtUs")]
    pub maybe_socket_closed_at_us: Option<u64>,
    #[serde(rename = "workerQuiescentAtUs")]
    pub maybe_worker_quiescent_at_us: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub schema: &'static str,
    pub scope: Scope,
    pub attempt_id: String,
    pub boot_ordinal: u64,
    pub worker_generation: u64,
    #[serde(rename = "poolSessionGeneration")]
    pub maybe_pool_session_generation: Option<u64>,
    #[serde(rename = "poolTransportEpoch")]
    pub maybe_pool_transport_epoch: Option<u64>,
    pub serial_transport_epoch: u64,
    #[serde(rename = "jobCommitment")]
    pub maybe_job_commitment: Option<String>,
    #[serde(rename = "observedAtUs")]
    pub maybe_observed_at_us: Option<u64>,
    pub state: State,
    pub admitted_at_device_us: u64,
    #[serde(rename = "authorityDeadlineDeviceUs")]
    pub maybe_authority_deadline_device_us: Option<u64>,
    #[serde(rename = "observationDeadlineDeviceUs")]
    pub maybe_observation_deadline_device_us: Option<u64>,
    #[serde(rename = "terminalAtDeviceUs")]
    pub maybe_terminal_at_device_us: Option<u64>,
    #[serde(rename = "outcome")]
    pub maybe_outcome: Option<Outcome>,
    pub events: Vec<Event>,
    pub timings: Vec<Timing>,
    pub share_facts: Vec<ShareFact>,
    #[serde(rename = "firstFailure")]
    pub maybe_first_failure: Option<Failure>,
    pub secondary_failures: Vec<Failure>,
    pub resources: Resources,
}

impl core::fmt::Debug for SocketTuple {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SocketTuple([redacted])")
    }
}

impl core::fmt::Debug for CurrentObservation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CurrentObservation([redacted])")
    }
}

impl core::fmt::Debug for RetainedConnection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RetainedConnection([redacted])")
    }
}

impl core::fmt::Debug for V2Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("V2Status([redacted])")
    }
}
