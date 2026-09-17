//! Joins actual UART observations to outgoing frames and matched received ACKs.
use super::*;
use bitaxe_asic::bm1366::{production::ProductionWorkPayload, result::Bm1366NonceResult};
use bitaxe_stratum::v2::noise::diagnostic::Failure as IoFailure;
use bitaxe_stratum::{
    v1::production_work::PoolSessionGeneration,
    v2::{
        frame::Frame,
        messages::{ClientMessage, ServerMessage},
        work::V2MiningWork,
    },
};
use sha2::{Digest, Sha256};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn work_ready(work: V2MiningWork, commitment: String) {
    if let Ok(mut data) = DATA.lock() {
        if let Some(record) = data
            .maybe_record
            .as_mut()
            .filter(|r| r.scope() == Scope::Share)
        {
            record.set_job_commitment(commitment);
            record.event(
                Stage::WorkReady,
                now_us(),
                Some(work.channel_id),
                Some(work.job_id),
                None,
                None,
            );
            data.maybe_work = Some(work);
        }
    }
}
pub(crate) fn dispatched(
    generation: PoolSessionGeneration,
    payload: &ProductionWorkPayload,
) -> bool {
    let Ok(mut data) = DATA.lock() else {
        return false;
    };
    let Some(now) = now_us() else {
        if let Some(r) = data.maybe_record.as_mut() {
            r.fail(Stage::AsicDispatch, FailureCategory::Clock, None);
        }
        return false;
    };
    let Some(record) = data.maybe_record.as_ref().filter(|r| {
        r.scope() == Scope::Share && r.pool_binding().is_some_and(|(g, _)| g == generation.raw())
    }) else {
        return false;
    };
    let _binding = record.binding();
    if data.maybe_dispatch.is_some()
        || data.maybe_work.as_ref().is_none_or(|w| {
            w.asic_job_id != payload.job_id()
                || ProductionWorkPayload::new(w.asic_job_id, w.fields)
                    .payload()
                    .bytes()
                    != payload.payload().bytes()
        })
    {
        if let Some(r) = data.maybe_record.as_mut() {
            r.fail(Stage::AsicDispatch, FailureCategory::Evidence, Some(now));
        }
        return false;
    }
    data.maybe_dispatch = Some(Dispatch {
        sequence: 1,
        at_us: now,
        id: payload.job_id().raw(),
        hash: hex(&Sha256::digest(payload.payload().bytes())),
    });
    let Some(work) = data.maybe_work.as_ref() else {
        return false;
    };
    let ids = (work.channel_id, work.job_id);
    if let Some(r) = data.maybe_record.as_mut() {
        r.event(
            Stage::AsicDispatch,
            Some(now),
            Some(ids.0),
            Some(ids.1),
            None,
            None,
        );
    }
    true
}
pub(crate) fn nonce_observed(generation: PoolSessionGeneration, result: Bm1366NonceResult) {
    let Ok(mut data) = DATA.lock() else { return };
    let Some(now) = now_us() else {
        if let Some(r) = data.maybe_record.as_mut() {
            r.fail(Stage::Nonce, FailureCategory::Clock, None);
        }
        return;
    };
    if data.maybe_record.as_ref().is_none_or(|r| {
        r.scope() != Scope::Share || r.pool_binding().is_none_or(|(g, _)| g != generation.raw())
    }) {
        return;
    }
    if data.maybe_dispatch.is_none()
        || data
            .maybe_work
            .as_ref()
            .is_none_or(|w| !w.qualifies(result).unwrap_or(false))
    {
        return;
    }
    if data.pending_nonces.iter().any(|(r, _)| {
        r.job_id.lookup_key() == result.job_id.lookup_key()
            && r.nonce == result.nonce
            && r.version_bits == result.version_bits
    }) || data
        .maybe_record
        .as_ref()
        .is_some_and(|r| r.has_candidate(result.job_id.raw(), result.nonce, result.version_bits))
    {
        return;
    }
    if data.pending_nonces.len() >= 16 || data.pending_nonces.try_reserve(1).is_err() {
        if let Some(r) = data.maybe_record.as_mut() {
            r.fail(Stage::Nonce, FailureCategory::Evidence, Some(now));
        }
        return;
    }
    data.pending_nonces.push((result, now));
    let ids = data.maybe_work.as_ref().map(|w| (w.channel_id, w.job_id));
    if let Some(r) = data.maybe_record.as_mut() {
        r.event(
            Stage::Nonce,
            Some(now),
            ids.map(|i| i.0),
            ids.map(|i| i.1),
            None,
            None,
        );
    }
}
pub(super) fn prepare_submission(frame: &Frame) -> Result<Option<u32>, IoFailure> {
    let message = ClientMessage::decode(frame).map_err(|_| IoFailure::Io)?;
    let ClientMessage::SubmitSharesStandard(submit) = message else {
        return Ok(None);
    };
    let mut data = DATA.lock().map_err(|_| IoFailure::Io)?;
    let dispatch = data.maybe_dispatch.clone().ok_or(IoFailure::Io)?;
    let index = data
        .pending_nonces
        .iter()
        .position(|(n, _)| {
            n.nonce == submit.nonce
                && n.version_bits | bitaxe_stratum::v2::standard::BASE_VERSION == submit.version
        })
        .ok_or(IoFailure::Io)?;
    let (nonce, time) = data.pending_nonces.remove(index);
    let fact = ShareFact {
        dispatch_sequence: dispatch.sequence,
        asic_job_id: dispatch.id,
        work_fields_sha256: dispatch.hash,
        dispatched_at_device_us: dispatch.at_us,
        nonce_at_device_us: time,
        maybe_write_started_at_device_us: None,
        maybe_write_completed_at_device_us: None,
        nonce: nonce.nonce,
        version_bits: nonce.version_bits,
        asic_index: nonce.asic_index,
        core_id: nonce.core_id,
        small_core_id: nonce.small_core_id,
        channel_id: submit.channel_id,
        job_id: submit.job_id,
        submission_sequence: submit.sequence_number,
        ntime: submit.ntime,
        version: submit.version,
        maybe_ack_at_device_us: None,
        maybe_ack_last_sequence: None,
        maybe_ack_accepted_count: None,
        maybe_ack_shares_sum: None,
        maybe_matched_submit_count: None,
    };
    let r = data.maybe_record.as_mut().ok_or(IoFailure::Io)?;
    if !r.add_share(fact) {
        return Err(IoFailure::Extra);
    }

    Ok(Some(submit.sequence_number))
}
pub(crate) fn validated_frame(frame: &Frame) {
    let Ok(message) = ServerMessage::decode(frame) else {
        return;
    };
    let digest = || Some(hex(&Sha256::digest(frame.encode())));
    with_record(|r| match message {
        ServerMessage::SetupConnectionSuccess(_) => {
            r.event(Stage::Setup, now_us(), None, None, None, None)
        }
        ServerMessage::OpenStandardMiningChannelSuccess(v) => r.event(
            Stage::Channel,
            now_us(),
            Some(v.channel_id),
            None,
            None,
            digest(),
        ),
        ServerMessage::NewMiningJob(v) => r.event(
            Stage::Job,
            now_us(),
            Some(v.channel_id),
            Some(v.job_id),
            None,
            digest(),
        ),
        ServerMessage::SetTarget(v) => r.event(
            Stage::Target,
            now_us(),
            Some(v.channel_id),
            None,
            None,
            digest(),
        ),
        // ACK evidence is captured at authenticated receive completion, before
        // the ordinary owner may retire its pool or process this queued event.
        ServerMessage::SubmitSharesSuccess(_) => {}
        _ => {}
    });
}
pub(super) fn received_acknowledgement(frame: &Frame) -> Result<(), IoFailure> {
    use bitaxe_worker_control::v2::ObservedAcknowledgement;
    if frame.header.message_type
        != bitaxe_stratum::v2::messages::MessageType::SubmitSharesSuccess as u8
    {
        return Ok(());
    }
    if frame.header.extension_type != 0x8000 {
        with_record(|r| r.fail(Stage::Accepted, FailureCategory::Protocol, now_us()));
        return Err(IoFailure::Io);
    }
    let ServerMessage::SubmitSharesSuccess(ack) =
        ServerMessage::decode(frame).map_err(|_| IoFailure::Io)?
    else {
        return Err(IoFailure::Io);
    };
    let mut accepted = false;
    with_record(|r| {
        accepted = r.observe_acknowledgement(
            ObservedAcknowledgement {
                channel_id: ack.channel_id,
                last_sequence: ack.last_sequence_number,
                accepted_count: ack.accepted_count,
                shares_sum: ack.shares_sum,
            },
            hex(&Sha256::digest(frame.encode())),
            now_us(),
        );
    });
    if accepted {
        Ok(())
    } else {
        Err(IoFailure::Io)
    }
}
pub(super) fn expected_heartbeat_fault() -> bool {
    let Ok(data) = DATA.lock() else { return false };
    let Some((generation, _)) = data.maybe_binding else {
        return false;
    };
    data.maybe_record
        .as_ref()
        .is_some_and(|r| r.scope() == Scope::Share && r.has_accepted_share())
        && revocation::timing(crate::runtime_uptime::millis()).is_some_and(|t| {
            t.generation == generation.raw()
                && t.revocation_reason == revocation::RevocationReason::HeartbeatTimeout
        })
}
