//! The new fixture is software-only until the independent Serial job is qualified.
//! No historical fixture mode or evidence schema is changed here.
mod inventory;
mod io;
mod model;
#[cfg(test)]
mod tests;

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, TcpListener};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_stratum::v2::frame::FrameHeader;
use noise_sv2::Responder;
use rand::{rngs::OsRng, RngCore};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde::Serialize;
use zeroize::Zeroizing;

use self::model::{Cause, Terminal};
use super::Args;

// Polling consumers must never see an artifact while its JSON is still being written.
fn write_serial_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let pending = path.with_extension("json.pending");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&pending)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);
    fs::hard_link(&pending, path)?;
    fs::remove_file(pending)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ready<'a> {
    schema: &'static str,
    attempt_id: &'a str,
    listen_ipv4: String,
    listen_port: u16,
    authority_public_key: String,
}

fn validate(args: &Args) -> Result<(&str, IpAddr)> {
    let Some(attempt) = args.attempt_id.as_deref() else {
        bail!("noise_serial_attempt_missing");
    };
    let decoded = URL_SAFE_NO_PAD.decode(attempt)?;
    if decoded.len() != 16 || URL_SAFE_NO_PAD.encode(decoded) != attempt {
        bail!("noise_serial_attempt_invalid");
    }
    let Some(IpAddr::V4(expected)) = args.expected_peer_address else {
        bail!("noise_serial_peer_invalid");
    };
    let IpAddr::V4(listen) = args.listen_address.ip() else {
        bail!("noise_serial_listener_invalid");
    };
    if !listen.is_private()
        || !expected.is_private()
        || args.listen_address.port() != 0
        || args.accept_timeout_seconds != 120
        || args.read_timeout_seconds != Some(10)
        || args.lifetime_seconds != Some(150)
        || args.session_timeout_seconds != 180
    {
        bail!("noise_serial_bounds");
    }
    let Some(parent) = args.private_root.parent() else {
        bail!("noise_serial_parent_missing");
    };
    if !args.private_root.is_absolute()
        || fs::canonicalize(parent)? != parent
        || fs::symlink_metadata(parent)?.permissions().mode() & 0o777 != 0o700
        || fs::symlink_metadata(&args.private_root).is_ok()
    {
        bail!("noise_serial_private_root");
    }
    Ok((attempt, IpAddr::V4(expected)))
}

/// An independent host process deadline also covers opaque cryptographic calls.
/// It emits only a closed failure code; no fabricated terminal receipt is written.
fn lifetime_guard(deadline: Instant) -> mpsc::Sender<()> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if matches!(
            receiver.recv_timeout(remaining),
            Err(mpsc::RecvTimeoutError::Timeout)
        ) {
            eprintln!("noise_serial_lifetime_expired");
            std::process::exit(1);
        }
    });
    sender
}

fn generate_serial_authority() -> Result<(Zeroizing<[u8; 32]>, [u8; 32])> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    for _ in 0..16 {
        let mut private = Zeroizing::new([0; 32]);
        rng.fill_bytes(private.as_mut());
        let Ok(mut secret) = SecretKey::from_slice(private.as_ref()) else {
            continue;
        };
        let keypair = Keypair::from_secret_key(&secp, &secret);
        secret.non_secure_erase();
        return Ok((private, keypair.x_only_public_key().0.serialize()));
    }
    bail!("noise_serial_authority_failed")
}

pub(super) fn run(args: &Args) -> Result<()> {
    let (attempt, expected) = validate(args)?;
    let startup_deadline = Instant::now() + Duration::from_secs(5);
    let startup_guard = lifetime_guard(startup_deadline);
    DirBuilder::new().mode(0o700).create(&args.private_root)?;
    let listener = TcpListener::bind(args.listen_address)?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?;
    let (secret, public) = generate_serial_authority()?;
    let ready = Ready {
        schema: "noise-serial-fixture-ready-v1",
        attempt_id: attempt,
        listen_ipv4: address.ip().to_string(),
        listen_port: address.port(),
        authority_public_key: URL_SAFE_NO_PAD.encode(public),
    };
    let began = Instant::now();
    if began >= startup_deadline {
        bail!("noise_serial_startup_expired");
    }
    let _deadline_guard = lifetime_guard(began + Duration::from_secs(150));
    write_serial_json(&args.private_root.join("ready.json"), &ready)?;
    if Instant::now() >= startup_deadline {
        bail!("noise_serial_startup_expired");
    }
    drop(startup_guard);
    let mut receipt = Terminal::new(attempt.to_owned());
    let result = exchange(
        listener,
        expected,
        &secret,
        &public,
        began,
        Limits::production(),
        &mut receipt,
    );
    receipt.elapsed_ms = began.elapsed().as_millis().try_into()?;
    receipt.outcome = if result.is_ok() {
        "accepted"
    } else {
        "rejected"
    };
    receipt.failure = result.err();
    // Every exchange-local stream has dropped before receipt publication.
    receipt.socket_closed = receipt.expected_peer_connection_count > 0;
    write_serial_json(&args.private_root.join("terminal.json"), &receipt)?;
    if receipt.failure.is_some() {
        bail!("noise_serial_fixture_rejected");
    }
    Ok(())
}

struct Limits {
    accept: Duration,
    read: Duration,
    lifetime: Duration,
    eof: Duration,
}
impl Limits {
    fn production() -> Self {
        Self {
            accept: Duration::from_secs(120),
            read: Duration::from_secs(10),
            lifetime: Duration::from_secs(150),
            eof: Duration::from_secs(5),
        }
    }
}

fn exchange(
    listener: TcpListener,
    expected: IpAddr,
    secret: &[u8; 32],
    public: &[u8; 32],
    began: Instant,
    limits: Limits,
    receipt: &mut Terminal,
) -> Result<(), Cause> {
    let deadline = began + limits.lifetime;
    let (mut stream, act_one) = inventory::select(
        &listener,
        expected,
        began + limits.accept,
        deadline,
        limits.read,
        receipt,
    )?;
    let mut rng = OsRng;
    let mut responder = Responder::from_authority_kp_with_rng(
        public,
        secret,
        Duration::from_secs(u32::MAX.into()),
        &mut rng,
    )
    .map_err(|_| Cause::new("act_two_received", "authentication", "malformed"))?;
    let (act_two, mut codec) = responder
        .step_1_with_now_rng(act_one, 0, &mut rng)
        .map_err(|_| Cause::new("act_two_received", "authentication", "malformed"))?;
    io::write_all(
        &mut stream,
        &act_two,
        deadline.min(Instant::now() + Duration::from_secs(2)),
        &mut receipt.act_two_bytes_written,
    )?;
    let mut proof = Zeroizing::new(vec![0; 22]);
    io::read_exact(
        &mut stream,
        &mut proof,
        deadline.min(Instant::now() + limits.read),
        &mut receipt.proof_bytes_received,
        "proof_received",
    )?;
    codec
        .decrypt(&mut *proof)
        .map_err(|_| Cause::new("proof_received", "proof", "malformed"))?;
    let header = FrameHeader::parse(&proof)
        .map_err(|_| Cause::new("proof_received", "proof", "malformed"))?;
    if header.extension_type != 0xffff || header.message_type != 0xff || header.payload_len != 0 {
        return Err(Cause::new("proof_received", "proof", "malformed"));
    }
    receipt.encrypted_proof_exact = true;
    let eof_deadline = deadline.min(Instant::now() + limits.eof);
    loop {
        if Instant::now() >= eof_deadline {
            return Err(Cause::new("proof_received", "read", "timeout"));
        }
        let mut extra = [0; 1];
        match stream.read(&mut extra) {
            Ok(0) => {
                receipt.peer_closed = true;
                break;
            }
            Ok(count) => {
                receipt.extra_bytes_received += count;
                return Err(Cause::new("proof_received", "proof", "extra"));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                io::pause(eof_deadline, "proof_received")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Cause::new("proof_received", "read", "io")),
        }
    }
    if Instant::now() >= eof_deadline {
        return Err(Cause::new("proof_received", "read", "timeout"));
    }
    inventory::reject_later_peers(&listener, expected, receipt)?;
    // The observed candidate interval ends at this final nonblocking inventory
    // check; dropping the listener admits no subsequent fixture session.
    drop(listener);
    match stream.shutdown(Shutdown::Both) {
        Ok(()) => {}
        // macOS reports an already fully closed peer as ENOTCONN after EOF.
        Err(error) if error.kind() == std::io::ErrorKind::NotConnected && receipt.peer_closed => {}
        Err(_) => return Err(Cause::new("cleanup", "cleanup", "io")),
    }
    drop(stream);
    if Instant::now() >= deadline {
        return Err(Cause::new("cleanup", "cleanup", "timeout"));
    }
    Ok(())
}
