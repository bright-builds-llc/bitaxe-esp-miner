use super::{
    input::{Input, Scope},
    io,
    job::Job,
    model::{Error, Evidence, Result, Share},
    oracle, wire,
};
use bitaxe_stratum::v2::messages::{SetupConnectionSuccess, SubmitSharesSuccess};
use noise_sv2::{NoiseCodec, Responder};
use rand::rngs::OsRng;
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn authenticate(
    stream: &mut TcpStream,
    act_one: [u8; 64],
    secret: &[u8; 32],
    public: &[u8; 32],
    deadline: Instant,
) -> Result<NoiseCodec> {
    let mut rng = OsRng;
    let mut responder = Responder::from_authority_kp_with_rng(
        public,
        secret,
        Duration::from_secs(u32::MAX.into()),
        &mut rng,
    )
    .map_err(|_| Error::new("setup_received", "authentication"))?;
    let (act_two, codec) = responder
        .step_1_with_now_rng(act_one, 0, &mut rng)
        .map_err(|_| Error::new("setup_received", "authentication"))?;
    io::write(stream, &act_two, deadline, "setup_received")?;
    Ok(codec)
}
pub(super) fn run(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    input: &Input,
    root: &Path,
    deadline: Instant,
    evidence: &mut Evidence,
) -> Result<()> {
    let setup = io::read_frame(stream, codec, deadline, true, "setup_received")?
        .ok_or(Error::new("setup_received", "eof"))?;
    wire::setup(
        &setup,
        stream
            .local_addr()
            .map_err(|_| Error::new("setup_received", "protocol"))?,
    )?;
    drop(setup);
    evidence.event("setup_received", None, None)?;
    let response = SetupConnectionSuccess {
        used_version: 2,
        flags: 0,
    }
    .encode()
    .map_err(|_| Error::new("setup_received", "protocol"))?;
    io::send_frame(stream, codec, &response, deadline, "setup_received")?;
    let open = io::read_frame(stream, codec, deadline, true, "channel_open_received")?
        .ok_or(Error::new("channel_open_received", "eof"))?;
    let request = wire::open(&open, input)?;
    drop(open);
    evidence.event("channel_open_received", None, None)?;
    let job = Job::new(request, evidence.events.connection_id.clone())?;
    crate::noise_serial::write_serial_json(&root.join("job.json"), &job.facts)
        .map_err(|_| Error::new("job_sent", "evidence"))?;
    for (frame, kind) in job.frames.iter().zip([
        "channel_success_sent",
        "job_sent",
        "target_sent",
        "prev_hash_sent",
    ]) {
        io::send_frame(stream, codec, frame, deadline, kind)?;
        evidence.event(kind, Some(oracle::sha256(&frame.encode())), None)?;
    }
    receive_shares(stream, codec, input.scope, &job, root, deadline, evidence)
}
fn receive_shares(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    scope: Scope,
    job: &Job,
    root: &Path,
    deadline: Instant,
    evidence: &mut Evidence,
) -> Result<()> {
    loop {
        let maybe_frame = io::read_frame(stream, codec, deadline, false, "share_received")?;
        let Some(frame) = maybe_frame else {
            evidence.terminal.peer_closed = true;
            evidence.event("peer_eof", None, None)?;
            if scope == Scope::Share && evidence.terminal.accepted_shares == 0 {
                return Err(Error::new("peer_eof", "rejected_share"));
            }
            return Ok(());
        };
        if scope == Scope::Channel {
            return Err(Error::new("share_received", "protocol"));
        }
        if evidence.shares.shares.len() >= 1024 {
            return Err(Error::new("share_received", "evidence"));
        }
        let submission = wire::share(&frame)?;
        evidence.terminal.received_shares += 1;
        if evidence.shares.shares.iter().any(|old| {
            old.submission.sequence_number == submission.sequence_number
                || (old.submission.nonce == submission.nonce
                    && old.submission.version == submission.version)
        }) {
            evidence.terminal.duplicate_shares += 1;
            return Err(Error::new("share_received", "rejected_share"));
        }
        if evidence
            .shares
            .shares
            .last()
            .is_some_and(|old| old.submission.sequence_number >= submission.sequence_number)
        {
            return Err(Error::new("share_received", "protocol"));
        }
        let header_hash = job.validate(&submission)?;
        let valid = oracle::meets_target(&header_hash, &oracle::TARGET);
        evidence.event(
            "share_received",
            Some(oracle::sha256(
                &[frame.header.encode().as_slice(), &frame.payload].concat(),
            )),
            Some(submission.sequence_number),
        )?;
        evidence.shares.shares.push(Share {
            submission,
            received_at_fixture_us: evidence.micros(),
            header_sha256d: oracle::hex(&header_hash),
            target_valid: valid,
            write_started_at_fixture_us: None,
            write_completed_at_fixture_us: None,
            accepted_count: None,
            shares_sum: None,
        });
        if !valid {
            evidence.terminal.rejected_shares += 1;
            return Err(Error::new("share_received", "invalid_nonce"));
        }
        let ack = SubmitSharesSuccess {
            channel_id: job.channel,
            last_sequence_number: submission.sequence_number,
            accepted_count: 1,
            shares_sum: 1024,
        }
        .encode()
        .map_err(|_| Error::new("share_success_sent", "protocol"))?;
        let encoded = io::encrypt_frame(codec, &ack, "share_success_sent")?;
        let began = evidence.micros();
        evidence
            .shares
            .shares
            .last_mut()
            .expect("inserted share")
            .write_started_at_fixture_us = Some(began);
        io::write(stream, &encoded, deadline, "share_success_sent")?;
        let completed = evidence.micros();
        let share = evidence.shares.shares.last_mut().expect("inserted share");
        share.write_completed_at_fixture_us = Some(completed);
        share.accepted_count = Some(1);
        share.shares_sum = Some(1024);
        evidence.terminal.accepted_shares += 1;
        evidence.event(
            "share_success_sent",
            Some(oracle::sha256(&ack.encode())),
            Some(submission.sequence_number),
        )?;
        persist_completed_share(root, evidence)?;
    }
}

/// Publish only completed ACK facts; a failed write cannot create live success evidence.
pub(super) fn persist_completed_share(root: &Path, evidence: &Evidence) -> Result<()> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Completed<'a> {
        connection_id: &'a str,
        shares: [&'a Share; 1],
    }
    let row = evidence
        .shares
        .shares
        .last()
        .ok_or(Error::new("share_success_sent", "evidence"))?;
    let index = evidence.terminal.accepted_shares;
    if index == 0
        || index > 1024
        || !row.target_valid
        || row.accepted_count != Some(1)
        || row.shares_sum != Some(1024)
        || row.write_completed_at_fixture_us.is_none()
        || row.write_started_at_fixture_us.is_none()
        || row.write_completed_at_fixture_us < row.write_started_at_fixture_us
    {
        return Err(Error::new("share_success_sent", "evidence"));
    }
    crate::noise_serial::write_serial_json(
        &root.join(format!("share-{index:04}.json")),
        &Completed {
            connection_id: &evidence.shares.connection_id,
            shares: [row],
        },
    )
    .map_err(|_| Error::new("share_success_sent", "evidence"))
}
