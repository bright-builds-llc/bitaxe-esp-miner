use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct Cause {
    pub stage: &'static str,
    pub category: &'static str,
    pub detail: &'static str,
}

impl Cause {
    pub fn new(stage: &'static str, category: &'static str, detail: &'static str) -> Self {
        Self {
            stage,
            category,
            detail,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CandidateReceipt {
    pub remote_port: u16,
    pub act_one_bytes: u16,
    pub read_outcome: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Terminal {
    pub schema: &'static str,
    pub attempt_id: String,
    pub outcome: &'static str,
    pub failure: Option<Cause>,
    pub elapsed_ms: u64,
    pub expected_peer_connection_count: u32,
    pub unexpected_peer_count: u32,
    pub candidate_overflow: bool,
    pub selected_index: Option<usize>,
    pub candidates: Vec<CandidateReceipt>,
    pub act_two_bytes_written: usize,
    pub proof_bytes_received: usize,
    pub extra_bytes_received: usize,
    pub encrypted_proof_exact: bool,
    pub peer_closed: bool,
    pub socket_closed: bool,
}

impl Terminal {
    pub fn new(attempt_id: String) -> Self {
        Self {
            schema: "noise-serial-fixture-terminal-v1",
            attempt_id,
            outcome: "incomplete",
            failure: None,
            elapsed_ms: 0,
            expected_peer_connection_count: 0,
            unexpected_peer_count: 0,
            candidate_overflow: false,
            selected_index: None,
            candidates: Vec::new(),
            act_two_bytes_written: 0,
            proof_bytes_received: 0,
            extra_bytes_received: 0,
            encrypted_proof_exact: false,
            peer_closed: false,
            socket_closed: false,
        }
    }
}
