use super::{exchange::Exchange, Fault, ProfileIoEvent};
use crate::{CheckStatus, ProfileCheck, SimulationError};
use bitaxe_asic::bm1366::{
    packet::CommandFrame,
    result::{parse_bm1366_result_frame, Bm1366ParsedResult, Bm1366ValidJobIds},
    work::diagnostic_job_frame,
};
use bitaxe_stratum::v2::{
    frame::Frame,
    messages::{
        ChannelKind, NewMiningJob, OpenStandardMiningChannelSuccess, ServerMessage, SetNewPrevHash,
        SetTarget, SetupConnectionSuccess,
    },
    session::{SessionConfig, SessionEvent, V2Session},
    work::V2MiningWork,
};
use bitaxe_stratum_v2_fixture::{
    functional::{self, JobFacts},
    oracle,
};
use bitaxe_virtual_board::{asic::NonceFixture, BoardConfig, VirtualBoard};
use bitaxe_worker_control::v2::ShareFact;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct EncryptedShareFacts {
    pub profile: String,
    pub clock_kind: String,
    pub job_commitment: String,
    pub header_sha256d: String,
    pub independently_target_valid: bool,
    pub acknowledged_shares: u32,
    pub encrypted_frames_delivered: u32,
    pub resources_released: bool,
    pub share_fact: ShareFact,
}
fn boundary<T, E>(value: Result<T, E>, label: &'static str) -> Result<T, SimulationError> {
    value.map_err(|_| SimulationError::Boundary(label))
}
fn frame_event(event: SessionEvent) -> Result<Frame, SimulationError> {
    let SessionEvent::Outbound(frame) = event else {
        return Err(SimulationError::Boundary("expected_outbound"));
    };
    Ok(frame)
}
fn handle(session: &mut V2Session, frame: &Frame) -> Result<Vec<SessionEvent>, SimulationError> {
    let message = boundary(ServerMessage::decode(frame), "server_decode")?;
    boundary(session.handle(message), "session_handle")
}
fn outbound(events: Vec<SessionEvent>) -> Result<Frame, SimulationError> {
    events
        .into_iter()
        .find_map(|event| {
            if let SessionEvent::Outbound(frame) = event {
                Some(frame)
            } else {
                None
            }
        })
        .ok_or(SimulationError::Boundary("missing_outbound"))
}

pub fn run_encrypted_share_profile(
    seed: u64,
) -> Result<(EncryptedShareFacts, Vec<ProfileCheck>), SimulationError> {
    let facts = run_with_fault(seed, Fault::None)?;
    let checks = vec![
        ProfileCheck {
            id: "generic_encrypted_share".into(),
            status: CheckStatus::Passed,
            detail: "production_noise_v2session_independent_fixture_genesis_nonce_submit_ack"
                .into(),
        },
        ProfileCheck {
            id: "strict_live_profile_share".into(),
            status: CheckStatus::Unsupported,
            detail:
                "public_genesis_vector_version1_nbits1d00ffff_does_not_match_fixed_live_profile"
                    .into(),
        },
        ProfileCheck {
            id: "controller_owned_noise_loop".into(),
            status: CheckStatus::Unsupported,
            detail: "protocol_profile_runs_separately_from_worker_session_transport_owner".into(),
        },
    ];
    Ok((facts, checks))
}

/// Runs the generic encrypted profile on the controller's already prepared board.
/// The owner supplies the production cancellation/authority check at each boundary.
pub fn run_encrypted_share_on_board(
    seed: u64,
    board: &mut VirtualBoard,
    check: impl FnMut(u64) -> Result<(), SimulationError>,
) -> Result<EncryptedShareFacts, SimulationError> {
    run_encrypted_share_on_board_with_dispatch(seed, board, check, |_| Ok(()))
}
/// The dispatch owner admits the real work write after its frame and nonce
/// fixture are checked; successful construction alone never counts as dispatch.
pub fn run_encrypted_share_on_board_with_dispatch(
    seed: u64,
    board: &mut VirtualBoard,
    check: impl FnMut(u64) -> Result<(), SimulationError>,
    dispatch: impl FnMut(u64) -> Result<(), SimulationError>,
) -> Result<EncryptedShareFacts, SimulationError> {
    run_encrypted_share_on_board_with_io(seed, board, check, dispatch, |_, _| {})
}
/// Correlates completed I/O with the controller owner even if later stages fail.
pub fn run_encrypted_share_on_board_with_io(
    seed: u64,
    board: &mut VirtualBoard,
    mut check: impl FnMut(u64) -> Result<(), SimulationError>,
    mut dispatch: impl FnMut(u64) -> Result<(), SimulationError>,
    mut observe: impl FnMut(ProfileIoEvent, u64),
) -> Result<EncryptedShareFacts, SimulationError> {
    if !board.peripherals.power_enabled
        || board.peripherals.reset_asserted
        || board.asic.frequency_mhz == 0
    {
        return Err(SimulationError::Boundary("controller_board_not_prepared"));
    }
    run_exchange(
        seed,
        Fault::None,
        board,
        &mut check,
        &mut dispatch,
        &mut observe,
    )
}
pub(super) fn run_with_fault(
    seed: u64,
    fault: Fault,
) -> Result<EncryptedShareFacts, SimulationError> {
    let mut board = VirtualBoard::new(BoardConfig {
        seed,
        ..BoardConfig::default()
    })?;
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    let pll = boundary(
        CommandFrame::new(0x51, &[0, 8, 0x50, 194, 2, 0x40]),
        "PLL frame",
    )?;
    board.uart_exchange(pll.bytes())?;
    run_exchange(
        seed,
        fault,
        &mut board,
        &mut |_| Ok(()),
        &mut |_| Ok(()),
        &mut |_, _| {},
    )
}
pub(super) fn run_exchange(
    seed: u64,
    fault: Fault,
    board: &mut VirtualBoard,
    check: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
    dispatch: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
    observe: &mut dyn FnMut(ProfileIoEvent, u64),
) -> Result<EncryptedShareFacts, SimulationError> {
    check(board.now_ms())?;
    let mut exchange = Exchange::new(seed, fault == Fault::WrongAuthority)?;
    if matches!(
        fault,
        Fault::RejectedClose | Fault::InvalidSubmissionAndRejectedClose
    ) {
        exchange.inject_close_rejection();
    }
    let outcome = check(board.now_ms())
        .and_then(|()| exchange_profile(&mut exchange, board, fault, check, dispatch, observe));
    // Natural completion and release are separate: cleanup never replaces failure.
    let cleanup = exchange.close();
    match (outcome, cleanup) {
        (Ok(mut facts), Ok(())) => {
            facts.resources_released = exchange.released();
            if !facts.resources_released {
                return Err(SimulationError::Boundary("encrypted_resources_retained"));
            }
            Ok(facts)
        }
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(original), Err(cleanup)) => Err(SimulationError::Cleanup {
            original: Box::new(original),
            cleanup: Box::new(cleanup),
        }),
    }
}
fn exchange_profile(
    exchange: &mut Exchange,
    board: &mut VirtualBoard,
    fault: Fault,
    check: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
    dispatch: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
    observe: &mut dyn FnMut(ProfileIoEvent, u64),
) -> Result<EncryptedShareFacts, SimulationError> {
    let facts = JobFacts::genesis();
    let mut session = boundary(
        V2Session::new(SessionConfig {
            endpoint_host: functional::SYNTHETIC_ENDPOINT_HOST.into(),
            endpoint_port: functional::SYNTHETIC_ENDPOINT_PORT,
            vendor: "synthetic".into(),
            hardware_version: "205".into(),
            firmware: "virtual-v1".into(),
            device_id: "synthetic".into(),
            user_identity: functional::SYNTHETIC_USER_IDENTITY.into(),
            nominal_hashrate: 1.0,
            channel_kind: ChannelKind::Standard,
            minimum_extranonce_size: 0,
        }),
        "session_new",
    )?;
    let (work, job_commitment) =
        initialize_mining_work(exchange, board, &mut session, &facts, check)?;
    check(board.now_ms())?;
    let (parsed, dispatch_fact) = model_nonce(&work, &facts, board, dispatch, observe)?;
    let nonce_at = virtual_micros(board)?;
    check(board.now_ms())?;
    let submission = frame_event(
        boundary(session.observe_nonce(parsed), "nonce_validation")?
            .ok_or(SimulationError::Boundary("nonce_not_qualified"))?,
    )?;
    let mut submission = submission;
    if matches!(
        fault,
        Fault::InvalidPlainSubmission | Fault::InvalidSubmissionAndRejectedClose
    ) {
        let mut bytes = submission.encode();
        bytes[6 + 8] ^= 1;
        submission = boundary(Frame::parse(&bytes), "mutated_submission_frame")?;
    }
    check(board.now_ms())?;
    let mut encrypted = exchange.encrypt_client(&submission)?;
    let intended_length = encrypted.len();
    check(board.now_ms())?;
    if fault == Fault::TamperedEncryptedSubmission {
        let byte = encrypted
            .last_mut()
            .ok_or(SimulationError::Boundary("empty_submission"))?;
        *byte ^= 1;
    }
    if fault == Fault::TruncatedEncryptedSubmission {
        encrypted.pop();
    }
    check(board.now_ms())?;
    let write_started_at = virtual_micros(board)?;
    let received = exchange.deliver_client_with_write_observer(&encrypted, || {
        if encrypted.len() == intended_length && fault != Fault::IncompleteSubmissionWrite {
            observe(ProfileIoEvent::ShareWriteCompleted, board.now_ms());
        }
    })?;
    let write_completed_at = virtual_micros(board)?;
    check(board.now_ms())?;
    if fault == Fault::ReplayedEncryptedSubmission {
        exchange.deliver_client(&encrypted)?;
        return Err(SimulationError::Boundary("replay_unexpectedly_accepted"));
    }
    let (validated, hash) = boundary(
        facts.validate_submission(&received.encode()),
        "independent_submission",
    )?;
    let mut ack = boundary(facts.acknowledgement(validated), "independent_ack_creation")?;
    if fault == Fault::WrongAcknowledgementChannel {
        ack[6] ^= 1;
    }
    if fault == Fault::InflatedAcknowledgement {
        ack[6 + 8] = 2;
    }
    let encoded_ack = boundary(Frame::parse(&ack), "ack_frame")?;
    check(board.now_ms())?;
    let decrypted_ack = exchange.server_to_client(
        &encoded_ack,
        fault == Fault::TamperedEncryptedAcknowledgement,
    )?;
    check(board.now_ms())?;
    boundary(
        functional::validate_ack(
            &decrypted_ack.encode(),
            validated,
            fault != Fault::IncompleteSubmissionWrite,
        ),
        "independent_ack_correlation",
    )?;
    let ServerMessage::SubmitSharesSuccess(decoded_ack) =
        boundary(ServerMessage::decode(&decrypted_ack), "observed_ack_decode")?
    else {
        return Err(SimulationError::Boundary("observed_ack_type"));
    };
    let ack_at = virtual_micros(board)?;
    let events = handle(&mut session, &decrypted_ack)?;
    let accepted = events
        .iter()
        .find_map(|event| {
            if let SessionEvent::ShareAccepted { accepted_count } = event {
                Some(*accepted_count)
            } else {
                None
            }
        })
        .ok_or(SimulationError::Boundary("production_ack_not_accepted"))?;
    if accepted != 1 {
        return Err(SimulationError::Boundary("production_ack_count"));
    }
    let matched_submit_count = events
        .iter()
        .filter(|event| matches!(event, SessionEvent::ShareAccepted { .. }))
        .count();
    if matched_submit_count != 1 {
        return Err(SimulationError::Boundary("observed_ack_matches"));
    }
    observe(ProfileIoEvent::AcknowledgementValidated, board.now_ms());
    if session.stop() != SessionEvent::Stopped {
        return Err(SimulationError::Boundary("v2_stop"));
    }
    if session.observe_nonce(parsed).is_ok() {
        return Err(SimulationError::Boundary("nonce_after_stop"));
    }
    Ok(EncryptedShareFacts {
        profile: "public_genesis_generic_v2".into(),
        clock_kind: "virtual_model_clock".into(),
        job_commitment,
        header_sha256d: oracle::hex(&hash),
        independently_target_valid: true,
        acknowledged_shares: accepted,
        encrypted_frames_delivered: exchange.delivered_frames,
        resources_released: false,
        share_fact: build_share_fact(
            &work,
            dispatch_fact,
            parsed,
            validated,
            decoded_ack,
            ShareTimes {
                nonce_at,
                write_started_at,
                write_completed_at,
                ack_at,
            },
            matched_submit_count,
        )?,
    })
}
fn initialize_mining_work(
    exchange: &mut Exchange,
    board: &VirtualBoard,
    session: &mut V2Session,
    facts: &JobFacts,
    check: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
) -> Result<(V2MiningWork, String), SimulationError> {
    let setup = frame_event(boundary(session.start(), "session_start")?)?;
    check(board.now_ms())?;
    let setup = exchange.client_to_server(&setup)?;
    check(board.now_ms())?;
    boundary(
        functional::validate_setup(&setup.encode()),
        "independent_setup",
    )?;
    let response = boundary(
        SetupConnectionSuccess {
            used_version: 2,
            flags: 0,
        }
        .encode(),
        "setup_response",
    )?;
    check(board.now_ms())?;
    let response = exchange.server_to_client(&response, false)?;
    check(board.now_ms())?;
    let open = outbound(handle(session, &response)?)?;
    check(board.now_ms())?;
    let open = exchange.client_to_server(&open)?;
    check(board.now_ms())?;
    boundary(
        functional::validate_open(&open.encode()),
        "independent_open",
    )?;
    let frames = [
        boundary(
            OpenStandardMiningChannelSuccess {
                request_id: 1,
                channel_id: facts.channel,
                target: facts.target,
                extranonce_prefix: Vec::new(),
                group_channel_id: 0,
            }
            .encode(),
            "channel",
        )?,
        boundary(
            NewMiningJob {
                channel_id: facts.channel,
                job_id: facts.job,
                maybe_min_ntime: None,
                version: facts.version,
                merkle_root: facts.merkle,
            }
            .encode(),
            "job",
        )?,
        boundary(
            SetTarget {
                channel_id: facts.channel,
                maximum_target: facts.target,
            }
            .encode(),
            "target",
        )?,
        boundary(
            SetNewPrevHash {
                channel_id: facts.channel,
                job_id: facts.job,
                prev_hash: facts.previous,
                min_ntime: facts.ntime,
                nbits: facts.nbits,
            }
            .encode(),
            "previous_hash",
        )?,
    ];
    let mut maybe_work = None;
    let mut commitment = Sha256::new();
    for frame in frames {
        check(board.now_ms())?;
        let received = exchange.server_to_client(&frame, false)?;
        check(board.now_ms())?;
        let encoded = received.encode();
        let length = u32::try_from(encoded.len())
            .map_err(|_| SimulationError::Boundary("job_commitment_frame_size"))?;
        commitment.update(length.to_le_bytes());
        commitment.update(encoded);
        for event in handle(session, &received)? {
            if let SessionEvent::Work(work) = event {
                maybe_work = Some(work);
            }
        }
    }
    let work = maybe_work.ok_or(SimulationError::Boundary("no_production_work"))?;
    Ok((work, oracle::hex(&commitment.finalize())))
}
fn model_nonce(
    work: &V2MiningWork,
    facts: &JobFacts,
    board: &mut VirtualBoard,
    dispatch: &mut dyn FnMut(u64) -> Result<(), SimulationError>,
    observe: &mut dyn FnMut(ProfileIoEvent, u64),
) -> Result<(bitaxe_asic::bm1366::result::Bm1366NonceResult, DispatchFact), SimulationError> {
    let frame = boundary(
        diagnostic_job_frame(work.asic_job_id, work.fields),
        "ASIC production work",
    )?;
    let mut target = facts.target;
    target.reverse();
    board.asic.install_nonce_fixture(NonceFixture {
        work_payload: *frame.payload().bytes(),
        header: facts.header(facts.nonce),
        nonce: facts.nonce,
        target_be: target,
        provenance: "Bitcoin genesis public vector; independent fixture oracle; generic SV2 only"
            .into(),
    })?;
    dispatch(board.now_ms())?;
    let at_us = virtual_micros(board)?;
    let response = board.uart_exchange(frame.bytes())?;
    let dispatch_fact = DispatchFact {
        sequence: board.asic.dispatched_jobs,
        at_us,
        work_fields_sha256: oracle::sha256(frame.payload().bytes()),
    };
    observe(ProfileIoEvent::AsicWriteCompleted, board.now_ms());
    let parsed = boundary(
        parse_bm1366_result_frame(&response, &Bm1366ValidJobIds::single(work.asic_job_id), 256),
        "ASIC result parser",
    )?;
    let Bm1366ParsedResult::JobNonce(result) = parsed else {
        return Err(SimulationError::Boundary("ASIC result not nonce"));
    };
    Ok((result, dispatch_fact))
}

struct DispatchFact {
    sequence: u64,
    at_us: u64,
    work_fields_sha256: String,
}
fn virtual_micros(board: &VirtualBoard) -> Result<u64, SimulationError> {
    board
        .now_ms()
        .checked_mul(1000)
        .ok_or(SimulationError::Boundary("virtual_clock_overflow"))
}

struct ShareTimes {
    nonce_at: u64,
    write_started_at: u64,
    write_completed_at: u64,
    ack_at: u64,
}
fn build_share_fact(
    work: &V2MiningWork,
    dispatch_fact: DispatchFact,
    parsed: bitaxe_asic::bm1366::result::Bm1366NonceResult,
    validated: functional::Submission,
    decoded_ack: bitaxe_stratum::v2::messages::SubmitSharesSuccess,
    times: ShareTimes,
    matched_submit_count: usize,
) -> Result<ShareFact, SimulationError> {
    Ok(ShareFact {
        dispatch_sequence: dispatch_fact.sequence,
        asic_job_id: work.asic_job_id.raw(),
        work_fields_sha256: dispatch_fact.work_fields_sha256,
        dispatched_at_device_us: dispatch_fact.at_us,
        nonce_at_device_us: times.nonce_at,
        maybe_write_started_at_device_us: Some(times.write_started_at),
        maybe_write_completed_at_device_us: Some(times.write_completed_at),
        nonce: parsed.nonce,
        version_bits: parsed.version_bits,
        asic_index: parsed.asic_index,
        core_id: parsed.core_id,
        small_core_id: parsed.small_core_id,
        channel_id: validated.channel_id,
        job_id: validated.job_id,
        submission_sequence: validated.sequence,
        ntime: validated.ntime,
        version: validated.version,
        maybe_ack_at_device_us: Some(times.ack_at),
        maybe_ack_last_sequence: Some(decoded_ack.last_sequence_number),
        maybe_ack_accepted_count: Some(decoded_ack.accepted_count),
        maybe_ack_shares_sum: Some(decoded_ack.shares_sum),
        maybe_matched_submit_count: Some(
            u32::try_from(matched_submit_count)
                .map_err(|_| SimulationError::Boundary("observed_ack_matches"))?,
        ),
    })
}
