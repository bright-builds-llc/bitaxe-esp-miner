use serde::Serialize;
use std::time::Instant;
#[derive(Clone, Copy, Debug)]
pub(super) struct Error {
    pub stage: &'static str,
    pub category: &'static str,
}
impl Error {
    pub fn new(stage: &'static str, category: &'static str) -> Self {
        Self { stage, category }
    }
}
pub(super) type Result<T> = std::result::Result<T, Error>;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Failure {
    pub stage: &'static str,
    pub category: &'static str,
    pub at_fixture_us: Option<u64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Event {
    pub sequence: u32,
    pub at_fixture_us: u64,
    pub kind: &'static str,
    pub payload_sha256: Option<String>,
    pub submission_sequence: Option<u32>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Events {
    pub connection_id: String,
    pub events: Vec<Event>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Terminal {
    pub instance_id: String,
    pub connection_id: Option<String>,
    pub outcome: &'static str,
    pub first_failure: Option<Failure>,
    pub elapsed_ms: u64,
    pub received_shares: u32,
    pub accepted_shares: u32,
    pub rejected_shares: u32,
    pub duplicate_shares: u32,
    pub peer_closed: bool,
    pub socket_closed: bool,
    pub listener_closed: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConnectionFacts {
    pub instance_id: String,
    pub connection_id: Option<String>,
    pub expected_peer_count: u32,
    pub unexpected_peer_count: u32,
    pub candidate_overflow: bool,
    pub expected_peer_match: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Submission {
    pub channel_id: u32,
    pub sequence_number: u32,
    pub job_id: u32,
    pub nonce: u32,
    pub ntime: u32,
    pub version: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Share {
    pub submission: Submission,
    pub received_at_fixture_us: u64,
    pub header_sha256d: String,
    pub target_valid: bool,
    pub write_started_at_fixture_us: Option<u64>,
    pub write_completed_at_fixture_us: Option<u64>,
    pub accepted_count: Option<u32>,
    pub shares_sum: Option<u64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Shares {
    pub connection_id: String,
    pub shares: Vec<Share>,
}
pub(super) struct Evidence {
    pub began: Instant,
    pub events: Events,
    pub terminal: Terminal,
    pub shares: Shares,
}
impl Evidence {
    pub fn new(instance: String, connection: String, began: Instant) -> Self {
        Self {
            began,
            events: Events {
                connection_id: connection.clone(),
                events: Vec::new(),
            },
            shares: Shares {
                connection_id: connection,
                shares: Vec::new(),
            },
            terminal: Terminal {
                instance_id: instance,
                connection_id: None,
                outcome: "unverified",
                first_failure: None,
                elapsed_ms: 0,
                received_shares: 0,
                accepted_shares: 0,
                rejected_shares: 0,
                duplicate_shares: 0,
                peer_closed: false,
                socket_closed: false,
                listener_closed: false,
            },
        }
    }
    pub fn micros(&self) -> u64 {
        self.began
            .elapsed()
            .as_micros()
            .try_into()
            .expect("bounded fixture duration")
    }
    pub fn event(
        &mut self,
        kind: &'static str,
        payload: Option<String>,
        sequence: Option<u32>,
    ) -> Result<()> {
        if self.events.events.len() >= 1024 {
            return Err(Error::new(kind, "evidence"));
        }
        self.events.events.push(Event {
            sequence: self.events.events.len() as u32 + 1,
            at_fixture_us: self.micros(),
            kind,
            payload_sha256: payload,
            submission_sequence: sequence,
        });
        Ok(())
    }
    pub fn fail(&mut self, error: Error) {
        if self.terminal.first_failure.is_none() {
            self.terminal.first_failure = Some(Failure {
                stage: error.stage,
                category: error.category,
                at_fixture_us: Some(self.micros()),
            });
        }
    }
}
