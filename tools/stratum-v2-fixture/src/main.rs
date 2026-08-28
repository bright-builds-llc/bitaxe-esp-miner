use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};
use bitaxe_stratum::v2::authority::encode_authority_public_key;
use bitaxe_stratum::v2::frame::{Frame, FrameHeader, FRAME_HEADER_LEN};
use bitaxe_stratum::v2::messages::{
    ClientMessage, NewMiningJob, OpenStandardMiningChannelSuccess, SetNewPrevHash,
    SetupConnectionSuccess, SubmitSharesStandard, SubmitSharesSuccess,
};
use bitaxe_stratum::v2::work::V2MiningWork;
use clap::{Parser, ValueEnum};
use noise_sv2::{NoiseCodec, Responder, AEAD_MAC_LEN, ELLSWIFT_ENCODING_SIZE};
use rand::{rngs::OsRng, RngCore};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde::Serialize;

mod tcp_payload;

use tcp_payload::read_tcp_payload;

const ENCRYPTED_HEADER_LEN: usize = FRAME_HEADER_LEN + AEAD_MAC_LEN;
const CHANNEL_ID: u32 = 1;
const JOB_ID: u32 = 1;
const VERSION: u32 = 0x2000_0000;
const NBITS: u32 = 0x207f_ffff;
const FIXTURE_CERTIFICATE_VALIDITY: Duration = Duration::from_secs(u32::MAX as u64);

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    private_root: PathBuf,
    #[arg(long, default_value = "0.0.0.0:0")]
    listen_address: SocketAddr,
    #[arg(long, default_value_t = 120)]
    accept_timeout_seconds: u64,
    #[arg(long, default_value_t = 180)]
    session_timeout_seconds: u64,
    #[arg(long, value_enum, default_value_t = FixtureMode::Pool)]
    mode: FixtureMode,
    #[arg(long)]
    expected_peer_address: Option<IpAddr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum FixtureMode {
    Pool,
    HandshakeOnly,
    TcpPayload,
}

#[derive(Serialize)]
struct ReadyDocument {
    schema_version: &'static str,
    port: u16,
    authority_public_key: String,
}

#[derive(Serialize)]
struct ResultDocument {
    schema_version: &'static str,
    status: &'static str,
    noise_authenticated: bool,
    client_authenticated: bool,
    setup_accepted: bool,
    channel_opened: bool,
    job_sent: bool,
    share_received: bool,
    share_target_valid: bool,
    response_sent: bool,
    elapsed_millis: u64,
}

#[derive(Default, Serialize)]
struct FixtureProgress {
    listener_ready: bool,
    connection_accepted: bool,
    peer_matched: bool,
    unexpected_peer_count: u16,
    act_one_bytes_received: u16,
    act_one_read_category: &'static str,
    accept_to_first_byte_millis: Option<u32>,
    act_one_read_millis: u32,
    act_one_received: bool,
    payload_bytes_received: u16,
    payload_read_category: &'static str,
    payload_digest_match: bool,
    extra_bytes_received: u16,
    receipt_ack_sent: bool,
    responder_created: bool,
    act_two_created: bool,
    act_two_sent: bool,
    client_authenticated: bool,
    noise_authenticated: bool,
    setup_accepted: bool,
    channel_opened: bool,
    job_sent: bool,
    share_received: bool,
    response_sent: bool,
}

#[derive(Serialize)]
struct TerminalDocument<'a> {
    schema_version: &'static str,
    status: &'a str,
    terminal_category: &'a str,
    mode: FixtureMode,
    progress: &'a FixtureProgress,
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate_bounds(&args)?;
    create_private_root(&args.private_root)?;
    let mut progress = FixtureProgress::default();
    let result = run_fixture(&args, &mut progress);
    let accepted = result.is_ok();
    write_private_json(
        &args.private_root.join("terminal.json"),
        &TerminalDocument {
            schema_version: "bitaxe-stratum-v2-fixture-terminal-v1",
            status: if accepted { "accepted" } else { "failed" },
            terminal_category: fixture_terminal_category(&progress, args.mode),
            mode: args.mode,
            progress: &progress,
        },
    )?;
    result
}

fn run_fixture(args: &Args, progress: &mut FixtureProgress) -> Result<()> {
    let listener = TcpListener::bind(args.listen_address).context("bind fixture listener")?;
    listener
        .set_nonblocking(true)
        .context("set listener nonblocking")?;
    let local_port = listener
        .local_addr()
        .context("read listener address")?
        .port();
    let (authority_private, authority_public) = generate_authority_keypair()?;
    write_private_json(
        &args.private_root.join("ready.json"),
        &ReadyDocument {
            schema_version: "bitaxe-stratum-v2-fixture-ready-v1",
            port: local_port,
            authority_public_key: encode_authority_public_key(authority_public),
        },
    )?;
    progress.listener_ready = true;
    println!("stratum_v2_fixture=ready");
    let (mut stream, unexpected_peer_count) = accept_one(
        &listener,
        Duration::from_secs(args.accept_timeout_seconds),
        args.expected_peer_address,
    )?;
    progress.connection_accepted = true;
    progress.peer_matched = true;
    progress.unexpected_peer_count = unexpected_peer_count;
    let accepted_at = Instant::now();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .context("set read timeout")?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .context("set write timeout")?;
    if args.mode == FixtureMode::TcpPayload {
        read_tcp_payload(&mut stream, progress)?;
        println!("stratum_v2_fixture=accepted");
        return Ok(());
    }
    let started = Instant::now();
    let mut codec = respond_noise(
        &mut stream,
        authority_private,
        authority_public,
        progress,
        accepted_at,
    )?;
    if args.mode == FixtureMode::HandshakeOnly {
        read_client_proof(&mut stream, &mut codec)?;
        progress.client_authenticated = true;
        progress.noise_authenticated = true;
        println!("stratum_v2_fixture=accepted");
        return Ok(());
    }
    let result = run_pool_session(
        &mut stream,
        &mut codec,
        started,
        Duration::from_secs(args.session_timeout_seconds),
        progress,
    )?;
    write_private_json(&args.private_root.join("result.json"), &result)?;
    println!("stratum_v2_fixture=accepted");
    Ok(())
}

fn fixture_terminal_category(progress: &FixtureProgress, mode: FixtureMode) -> &'static str {
    if !progress.listener_ready {
        "listener"
    } else if !progress.connection_accepted {
        "accept"
    } else if mode == FixtureMode::TcpPayload && !progress.payload_digest_match {
        "payload_read"
    } else if mode == FixtureMode::TcpPayload && !progress.receipt_ack_sent {
        "receipt_ack"
    } else if mode == FixtureMode::TcpPayload {
        "accepted"
    } else if mode != FixtureMode::TcpPayload && !progress.act_one_received {
        "act_one_read"
    } else if !progress.responder_created {
        "responder"
    } else if !progress.act_two_created {
        "act_two_create"
    } else if !progress.act_two_sent {
        "act_two_write"
    } else if !progress.client_authenticated {
        "client_authentication"
    } else if matches!(mode, FixtureMode::HandshakeOnly | FixtureMode::TcpPayload) {
        "accepted"
    } else if !progress.setup_accepted {
        "setup"
    } else if !progress.channel_opened {
        "channel"
    } else if !progress.job_sent {
        "job"
    } else if !progress.share_received {
        "share"
    } else if !progress.response_sent {
        "response"
    } else {
        "accepted"
    }
}

fn validate_bounds(args: &Args) -> Result<()> {
    if args.accept_timeout_seconds == 0
        || args.accept_timeout_seconds > 300
        || args.session_timeout_seconds == 0
        || args.session_timeout_seconds > 300
    {
        bail!("fixture timeouts must be within 1..=300 seconds");
    }
    Ok(())
}

fn create_private_root(path: &Path) -> Result<()> {
    if path.exists() {
        bail!("private fixture root already exists");
    }
    fs::create_dir(path).context("create private fixture root")?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).context("protect fixture root")
}

fn write_private_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .context("create private fixture artifact")?;
    serde_json::to_writer_pretty(&mut file, value).context("write fixture artifact")?;
    file.write_all(b"\n").context("finish fixture artifact")
}

fn generate_authority_keypair() -> Result<([u8; 32], [u8; 32])> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    for _ in 0..16 {
        let mut private = [0; 32];
        rng.fill_bytes(&mut private);
        let Ok(secret) = SecretKey::from_slice(&private) else {
            continue;
        };
        let keypair = Keypair::from_secret_key(&secp, &secret);
        return Ok((private, keypair.x_only_public_key().0.serialize()));
    }
    bail!("authority key generation failed")
}

fn accept_one(
    listener: &TcpListener,
    timeout: Duration,
    maybe_expected_peer: Option<IpAddr>,
) -> Result<(TcpStream, u16)> {
    let deadline = Instant::now() + timeout;
    let mut unexpected_peer_count = 0_u16;
    loop {
        match listener.accept() {
            Ok((stream, peer))
                if maybe_expected_peer.is_none_or(|expected| peer.ip() == expected) =>
            {
                return Ok((stream, unexpected_peer_count));
            }
            Ok((_stream, _peer)) => {
                unexpected_peer_count = unexpected_peer_count.saturating_add(1);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    bail!("fixture accept deadline elapsed");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error).context("accept fixture client"),
        }
    }
}

fn respond_noise(
    stream: &mut TcpStream,
    authority_private: [u8; 32],
    authority_public: [u8; 32],
    progress: &mut FixtureProgress,
    accepted_at: Instant,
) -> Result<NoiseCodec> {
    let act_one = read_act_one(stream, progress, accepted_at)?;
    let mut rng = OsRng;
    let mut responder = Responder::from_authority_kp_with_rng(
        &authority_public,
        &authority_private,
        FIXTURE_CERTIFICATE_VALIDITY,
        &mut rng,
    )
    .map_err(|_| anyhow::anyhow!("create Noise responder"))?;
    progress.responder_created = true;
    let (act_two, codec) = responder
        .step_1_with_now_rng(act_one, 0, &mut rng)
        .map_err(|_| anyhow::anyhow!("produce Noise act two"))?;
    progress.act_two_created = true;
    stream.write_all(&act_two).context("write Noise act two")?;
    progress.act_two_sent = true;
    Ok(codec)
}

fn read_act_one(
    stream: &mut TcpStream,
    progress: &mut FixtureProgress,
    accepted_at: Instant,
) -> Result<[u8; ELLSWIFT_ENCODING_SIZE]> {
    let mut act_one = [0; ELLSWIFT_ENCODING_SIZE];
    let mut received = 0_usize;
    let read_started = Instant::now();
    while received < act_one.len() {
        match stream.read(&mut act_one[received..]) {
            Ok(0) => {
                progress.act_one_read_category = "eof";
                progress.act_one_bytes_received = received.try_into().unwrap_or(u16::MAX);
                progress.act_one_read_millis = elapsed_millis(read_started);
                bail!("Noise act one ended before the exact frame length");
            }
            Ok(count) => {
                if received == 0 {
                    progress.accept_to_first_byte_millis = Some(elapsed_millis(accepted_at));
                }
                received += count;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                progress.act_one_read_category = "timeout";
                progress.act_one_bytes_received = received.try_into().unwrap_or(u16::MAX);
                progress.act_one_read_millis = elapsed_millis(read_started);
                bail!("Noise act one read deadline elapsed");
            }
            Err(error) => {
                progress.act_one_read_category = "io";
                progress.act_one_bytes_received = received.try_into().unwrap_or(u16::MAX);
                progress.act_one_read_millis = elapsed_millis(read_started);
                return Err(error).context("read Noise act one");
            }
        }
    }
    progress.act_one_bytes_received = received.try_into().unwrap_or(u16::MAX);
    progress.act_one_read_category = "complete";
    progress.act_one_read_millis = elapsed_millis(read_started);
    progress.act_one_received = true;
    Ok(act_one)
}

fn elapsed_millis(started: Instant) -> u32 {
    started.elapsed().as_millis().try_into().unwrap_or(u32::MAX)
}

fn read_client_proof(stream: &mut TcpStream, codec: &mut NoiseCodec) -> Result<()> {
    let _ = read_noise_frame(stream, codec)?;
    Ok(())
}

fn run_pool_session(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    started: Instant,
    timeout: Duration,
    progress: &mut FixtureProgress,
) -> Result<ResultDocument> {
    let setup = read_client_message(stream, codec, started, timeout)?;
    progress.client_authenticated = true;
    progress.noise_authenticated = true;
    if !matches!(setup, ClientMessage::SetupConnection(_)) {
        bail!("expected setup connection");
    }
    progress.setup_accepted = true;
    write_server_frame(
        stream,
        codec,
        &SetupConnectionSuccess {
            used_version: 2,
            flags: 1,
        }
        .encode()?,
    )?;
    let open = read_client_message(stream, codec, started, timeout)?;
    let ClientMessage::OpenStandardMiningChannel(open) = open else {
        bail!("fixture requires a standard channel");
    };
    progress.channel_opened = true;
    write_server_frame(
        stream,
        codec,
        &OpenStandardMiningChannelSuccess {
            request_id: open.request_id,
            channel_id: CHANNEL_ID,
            target: [0xff; 32],
            extranonce_prefix: Vec::new(),
            group_channel_id: 0,
        }
        .encode()?,
    )?;
    let ntime = unix_time_u32()?;
    let job = NewMiningJob {
        channel_id: CHANNEL_ID,
        job_id: JOB_ID,
        maybe_min_ntime: None,
        version: VERSION,
        merkle_root: [0x11; 32],
    };
    let prev_hash = SetNewPrevHash {
        channel_id: CHANNEL_ID,
        job_id: JOB_ID,
        prev_hash: [0x22; 32],
        min_ntime: ntime,
        nbits: NBITS,
    };
    write_server_frame(stream, codec, &job.encode()?)?;
    write_server_frame(stream, codec, &prev_hash.encode()?)?;
    progress.job_sent = true;
    let submit = read_client_message(stream, codec, started, timeout)?;
    let ClientMessage::SubmitSharesStandard(submit) = submit else {
        bail!("expected standard share submission");
    };
    progress.share_received = true;
    let share_target_valid = validate_share(&job, &prev_hash, submit)?;
    if !share_target_valid {
        bail!("submitted share failed target validation");
    }
    write_server_frame(
        stream,
        codec,
        &SubmitSharesSuccess {
            channel_id: CHANNEL_ID,
            last_sequence_number: submit.sequence_number,
            accepted_count: 1,
            shares_sum: 1,
        }
        .encode()?,
    )?;
    progress.response_sent = true;
    Ok(ResultDocument {
        schema_version: "bitaxe-stratum-v2-fixture-result-v1",
        status: "accepted",
        noise_authenticated: true,
        client_authenticated: true,
        setup_accepted: true,
        channel_opened: true,
        job_sent: true,
        share_received: true,
        share_target_valid,
        response_sent: true,
        elapsed_millis: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
    })
}

fn validate_share(
    job: &NewMiningJob,
    prev_hash: &SetNewPrevHash,
    submit: SubmitSharesStandard,
) -> Result<bool> {
    if submit.channel_id != CHANNEL_ID
        || submit.job_id != JOB_ID
        || submit.ntime != prev_hash.min_ntime
        || submit.version & VERSION != VERSION
    {
        return Ok(false);
    }
    let work = V2MiningWork::standard(job, prev_hash, [0xff; 32], Bm1366JobId::new(0))?;
    work.qualifies(Bm1366NonceResult {
        job_id: Bm1366JobId::new(0),
        nonce: submit.nonce,
        asic_index: 0,
        core_id: 0,
        small_core_id: 0,
        version_bits: submit.version ^ VERSION,
    })
    .map_err(Into::into)
}

fn read_client_message(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    started: Instant,
    timeout: Duration,
) -> Result<ClientMessage> {
    if started.elapsed() >= timeout {
        bail!("fixture session deadline elapsed");
    }
    let frame = read_noise_frame(stream, codec)?;
    ClientMessage::decode(&frame).map_err(Into::into)
}

fn read_noise_frame(stream: &mut TcpStream, codec: &mut NoiseCodec) -> Result<Frame> {
    let mut encrypted_header = vec![0; ENCRYPTED_HEADER_LEN];
    stream
        .read_exact(&mut encrypted_header)
        .context("read encrypted header")?;
    codec
        .decrypt(&mut encrypted_header)
        .map_err(|_| anyhow::anyhow!("decrypt frame header"))?;
    let header = FrameHeader::parse(&encrypted_header)?;
    let mut payload = if header.payload_len == 0 {
        Vec::new()
    } else {
        let mut encrypted = vec![0; header.payload_len + AEAD_MAC_LEN];
        stream
            .read_exact(&mut encrypted)
            .context("read encrypted payload")?;
        codec
            .decrypt(&mut encrypted)
            .map_err(|_| anyhow::anyhow!("decrypt frame payload"))?;
        encrypted
    };
    Ok(Frame::new(
        header.extension_type,
        header.message_type,
        std::mem::take(&mut payload),
    )?)
}

fn write_server_frame(stream: &mut TcpStream, codec: &mut NoiseCodec, frame: &Frame) -> Result<()> {
    let mut header = frame.header.encode().to_vec();
    codec
        .encrypt(&mut header)
        .map_err(|_| anyhow::anyhow!("encrypt frame header"))?;
    stream
        .write_all(&header)
        .context("write encrypted header")?;
    if !frame.payload().is_empty() {
        let mut payload = frame.payload().to_vec();
        codec
            .encrypt(&mut payload)
            .map_err(|_| anyhow::anyhow!("encrypt frame payload"))?;
        stream
            .write_all(&payload)
            .context("write encrypted payload")?;
    }
    Ok(())
}

fn unix_time_u32() -> Result<u32> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time precedes Unix epoch")?
        .as_secs()
        .try_into()
        .context("system time exceeds SV2 certificate range")
}

#[cfg(test)]
mod tests;
