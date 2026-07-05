//! Fake BM1366 UART transcript harness.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/asic_common.c:count_asic_chips`
//! - `reference/esp-miner/components/asic/asic_common.c:receive_work`
//! - parity checklist rows `ASIC-002`, `ASIC-004`, `ASIC-005`, and `ASIC-008`

use crate::Bm1366ProtocolFault;

use super::{
    chip_detect::parse_chip_id_response,
    observation::{AsicInitStatus, Bm1366Observation},
    packet::FrameBytes,
    result::{
        parse_bm1366_result_frame, Bm1366ParsedResult, Bm1366ValidJobIds, BM1366_RESULT_FRAME_LEN,
    },
};

use super::command::{Bm1366AdapterAction, Bm1366Command, ADAPTER_TIMEOUT_MS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeUartTranscript {
    events: Vec<FakeUartEvent>,
    expected_chips: u8,
    valid_jobs: Bm1366ValidJobIds,
    address_interval: u16,
}

impl FakeUartTranscript {
    #[must_use]
    pub fn new(events: Vec<FakeUartEvent>) -> Self {
        Self {
            events,
            expected_chips: 1,
            valid_jobs: Bm1366ValidJobIds::empty(),
            address_interval: 1,
        }
    }

    #[must_use]
    pub fn with_expected_chips(events: Vec<FakeUartEvent>, expected_chips: u8) -> Self {
        Self {
            expected_chips,
            ..Self::new(events)
        }
    }

    #[must_use]
    pub fn with_result_context(
        events: Vec<FakeUartEvent>,
        valid_jobs: Bm1366ValidJobIds,
        address_interval: u16,
    ) -> Self {
        Self {
            events,
            expected_chips: 1,
            valid_jobs,
            address_interval,
        }
    }

    #[must_use]
    pub fn run_read_chip_id(self) -> TranscriptOutcome {
        let mut outcome = TranscriptOutcome::new();
        let command = Bm1366Command::ReadChipId;
        outcome.commands.push(command);

        let Ok(frame) = command.frame_bytes() else {
            outcome.fail_closed(
                Bm1366ProtocolFault::PreflightMissing {
                    reason: "read_chip_id_frame_encode_failed",
                },
                "read_chip_id_frame_encode_failed",
            );
            return outcome;
        };

        let mut events = self.events.into_iter().peekable();
        verify_expected_write(&mut events, &frame, &mut outcome);
        if outcome.status == TranscriptStatus::FailClosed {
            return outcome;
        }

        outcome.actions.push(Bm1366AdapterAction::WriteFrame(frame));
        outcome
            .actions
            .push(Bm1366AdapterAction::read_result_frame());

        let mut detected_chips = 0_u8;
        for event in events {
            match event {
                FakeUartEvent::ReadBytes(bytes) => match parse_chip_id_response(&bytes) {
                    Ok(observation) => {
                        detected_chips = detected_chips.saturating_add(1);
                        outcome.observations.push(observation);
                    }
                    Err(fault) => {
                        outcome.fail_closed(fault, "chip_id_response_fault");
                        break;
                    }
                },
                FakeUartEvent::Timeout => {
                    outcome.fail_closed(
                        Bm1366ProtocolFault::Timeout {
                            timeout_ms: ADAPTER_TIMEOUT_MS,
                        },
                        "chip_id_timeout",
                    );
                    break;
                }
                FakeUartEvent::PartialRead(bytes) => {
                    outcome.actions.push(Bm1366AdapterAction::ClearRx);
                    outcome.fail_closed(
                        Bm1366ProtocolFault::InvalidLength {
                            expected: BM1366_RESULT_FRAME_LEN,
                            actual: bytes.len(),
                        },
                        "chip_id_partial_read",
                    );
                    break;
                }
                FakeUartEvent::SplitRead { first, second } => {
                    let mut combined = first;
                    combined.extend(second);
                    match parse_chip_id_response(&combined) {
                        Ok(observation) => {
                            detected_chips = detected_chips.saturating_add(1);
                            outcome.observations.push(observation);
                        }
                        Err(fault) => {
                            outcome.fail_closed(fault, "chip_id_split_read_fault");
                            break;
                        }
                    }
                }
                FakeUartEvent::MalformedPreamble(bytes) | FakeUartEvent::BadCrc(bytes) => {
                    outcome.actions.push(Bm1366AdapterAction::ClearRx);
                    let fault = parse_chip_id_response(&bytes).err().unwrap_or(
                        Bm1366ProtocolFault::PreflightMissing {
                            reason: "malformed_event_was_valid",
                        },
                    );
                    outcome.fail_closed(fault, "chip_id_malformed_read");
                    break;
                }
                FakeUartEvent::ExpectWrite(_) => {
                    outcome.fail_closed(
                        Bm1366ProtocolFault::TranscriptWriteMismatch,
                        "unexpected_write_event",
                    );
                    break;
                }
            }
        }

        if outcome.status != TranscriptStatus::FailClosed && detected_chips != self.expected_chips {
            outcome.fail_closed(
                Bm1366ProtocolFault::ChipCountMismatch {
                    expected: self.expected_chips,
                    actual: detected_chips,
                },
                "chip_count_mismatch",
            );
        }

        if outcome.status != TranscriptStatus::FailClosed {
            outcome.actions.push(Bm1366AdapterAction::PublishStatus(
                AsicInitStatus::ChipDetectedNoMining {
                    chips: detected_chips,
                },
            ));
        }

        outcome
    }

    #[must_use]
    pub fn run_result_read(self) -> TranscriptOutcome {
        let mut outcome = TranscriptOutcome::new();
        outcome
            .actions
            .push(Bm1366AdapterAction::read_result_frame());

        for event in self.events {
            match event {
                FakeUartEvent::ReadBytes(bytes) => {
                    if !parse_result_bytes(
                        &mut outcome,
                        &bytes,
                        &self.valid_jobs,
                        self.address_interval,
                    ) {
                        break;
                    }
                }
                FakeUartEvent::MalformedPreamble(bytes) | FakeUartEvent::BadCrc(bytes) => {
                    outcome.actions.push(Bm1366AdapterAction::ClearRx);
                    if !parse_result_bytes(
                        &mut outcome,
                        &bytes,
                        &self.valid_jobs,
                        self.address_interval,
                    ) {
                        break;
                    }
                }
                FakeUartEvent::Timeout => {
                    outcome.fail_closed(
                        Bm1366ProtocolFault::Timeout {
                            timeout_ms: ADAPTER_TIMEOUT_MS,
                        },
                        "result_timeout",
                    );
                    break;
                }
                FakeUartEvent::PartialRead(bytes) => {
                    outcome.actions.push(Bm1366AdapterAction::ClearRx);
                    outcome.fail_closed(
                        Bm1366ProtocolFault::InvalidLength {
                            expected: BM1366_RESULT_FRAME_LEN,
                            actual: bytes.len(),
                        },
                        "result_partial_read",
                    );
                    break;
                }
                FakeUartEvent::SplitRead { first, second } => {
                    let mut combined = first;
                    combined.extend(second);
                    if !parse_result_bytes(
                        &mut outcome,
                        &combined,
                        &self.valid_jobs,
                        self.address_interval,
                    ) {
                        break;
                    }
                }
                FakeUartEvent::ExpectWrite(_) => {
                    outcome.fail_closed(
                        Bm1366ProtocolFault::TranscriptWriteMismatch,
                        "unexpected_write_event",
                    );
                    break;
                }
            }
        }

        outcome
    }
}

fn parse_result_bytes(
    outcome: &mut TranscriptOutcome,
    bytes: &[u8],
    valid_jobs: &Bm1366ValidJobIds,
    address_interval: u16,
) -> bool {
    match parse_bm1366_result_frame(bytes, valid_jobs, address_interval) {
        Ok(parsed) => {
            outcome.push_parsed_result(parsed);
            true
        }
        Err(fault) => {
            outcome.fail_closed(fault, "result_frame_fault");
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeUartEvent {
    ExpectWrite(FrameBytes),
    ReadBytes(Vec<u8>),
    Timeout,
    PartialRead(Vec<u8>),
    SplitRead { first: Vec<u8>, second: Vec<u8> },
    MalformedPreamble(Vec<u8>),
    BadCrc(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptOutcome {
    commands: Vec<Bm1366Command>,
    actions: Vec<Bm1366AdapterAction>,
    observations: Vec<Bm1366Observation>,
    status: TranscriptStatus,
}

impl TranscriptOutcome {
    #[must_use]
    pub const fn status(&self) -> TranscriptStatus {
        self.status
    }

    #[must_use]
    pub fn commands(&self) -> &[Bm1366Command] {
        &self.commands
    }

    #[must_use]
    pub fn actions(&self) -> &[Bm1366AdapterAction] {
        &self.actions
    }

    #[must_use]
    pub fn observations(&self) -> &[Bm1366Observation] {
        &self.observations
    }

    fn new() -> Self {
        Self {
            commands: Vec::new(),
            actions: Vec::new(),
            observations: Vec::new(),
            status: TranscriptStatus::Complete,
        }
    }

    fn push_parsed_result(&mut self, parsed: Bm1366ParsedResult) {
        match parsed {
            Bm1366ParsedResult::JobNonce(result) => {
                self.observations.push(Bm1366Observation::JobNonce(result));
            }
            Bm1366ParsedResult::RegisterRead(read) => {
                self.observations
                    .push(Bm1366Observation::RegisterRead(read));
            }
        }
    }

    fn fail_closed(&mut self, fault: Bm1366ProtocolFault, reason: &'static str) {
        self.status = TranscriptStatus::FailClosed;
        self.observations
            .push(Bm1366Observation::ProtocolFault(fault));
        self.actions.push(Bm1366AdapterAction::PublishStatus(
            AsicInitStatus::FailClosed { reason },
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptStatus {
    Complete,
    FailClosed,
}

fn verify_expected_write<I>(
    events: &mut std::iter::Peekable<I>,
    actual_frame: &FrameBytes,
    outcome: &mut TranscriptOutcome,
) where
    I: Iterator<Item = FakeUartEvent>,
{
    let Some(FakeUartEvent::ExpectWrite(_)) = events.peek() else {
        return;
    };

    let Some(FakeUartEvent::ExpectWrite(expected_frame)) = events.next() else {
        return;
    };

    if expected_frame != *actual_frame {
        outcome.fail_closed(
            Bm1366ProtocolFault::TranscriptWriteMismatch,
            "transcript_write_mismatch",
        );
    }
}
