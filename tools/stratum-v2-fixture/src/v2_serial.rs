//! Bounded fixed-Serial V2 fixture. Runtime pool configuration never enters disk evidence.
mod input;
mod io;
mod job;
mod model;
mod oracle;
mod session;
#[cfg(test)]
mod tests;
mod wire;
use crate::noise_serial::{
    generate_serial_authority, inventory, model::Terminal as Candidates, write_serial_json,
};
use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use input::{Input, Options};
use model::{ConnectionFacts, Error, Evidence};
use rand::{rngs::OsRng, RngCore};
use serde::Serialize;
use std::fs::{DirBuilder, File};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener};
use std::os::unix::fs::DirBuilderExt;
use std::sync::mpsc;
use std::time::{Duration, Instant};
fn nonce() -> String {
    let mut bytes = [0; 16];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
fn watchdog(deadline: Instant) -> mpsc::Sender<()> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if matches!(
            rx.recv_timeout(deadline.saturating_duration_since(Instant::now())),
            Err(mpsc::RecvTimeoutError::Timeout)
        ) {
            eprintln!("v2_serial_lifetime_expired");
            std::process::exit(1);
        }
    });
    tx
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ready<'a> {
    schema: &'static str,
    scope: input::Scope,
    attempt_id: &'a str,
    instance_id: &'a str,
    listen_ipv4: &'a str,
    listen_port: u16,
    authority_public_key: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Connected<'a> {
    schema: &'static str,
    scope: input::Scope,
    attempt_id: &'a str,
    instance_id: &'a str,
    connection_id: &'a str,
    observed_at_fixture_us: u64,
    local_ipv4: String,
    local_port: u16,
    peer_ipv4: String,
    peer_port: u16,
}
/// New mode uses only inherited private pipes; all outward errors are closed at main.
pub(super) fn run(args: &[String]) -> Result<()> {
    let startup = Instant::now() + Duration::from_secs(5);
    let startup_guard = watchdog(startup);
    let options = input::parse_options(args)?;
    // Transfer fd3 first: a missing descriptor must not be filled by a later dup.
    let connection_pipe = input::inherited_pipe(3)?;
    let stdout = input::inherited_pipe(1)?;
    let stdin = input::inherited_pipe(0)?;
    let input = input::read_input(stdin, &options)?;
    DirBuilder::new().mode(0o700).create(&options.root)?;
    let listener = TcpListener::bind((input::ipv4(&input.listen_ipv4)?, 0))?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?;
    let (secret, public) = generate_serial_authority()?;
    let instance = nonce();
    let connection = nonce();
    if Instant::now() >= startup {
        bail!("startup");
    }
    let began = Instant::now();
    let deadline = began + Duration::from_secs(input.scope.lifetime_seconds());
    let lifetime = watchdog(deadline);
    input::write_pipe(
        stdout,
        &Ready {
            schema: "str005-v2-fixture-ready-runtime-v1",
            scope: input.scope,
            attempt_id: &options.attempt,
            instance_id: &instance,
            listen_ipv4: &input.listen_ipv4,
            listen_port: address.port(),
            authority_public_key: URL_SAFE_NO_PAD.encode(public),
        },
    )?;
    if Instant::now() >= startup {
        bail!("startup");
    }
    drop(startup_guard);
    let mut evidence = Evidence::new(instance.clone(), connection.clone(), began);
    let mut candidates = Candidates::new(options.attempt.clone());
    let mut result = exchange(
        Exchange {
            options: &options,
            input: &input,
            secret: &secret,
            public: &public,
            deadline,
        },
        listener,
        connection_pipe,
        &mut evidence,
        &mut candidates,
    );
    if result.is_ok() && Instant::now() >= deadline {
        result = Err(Error::new("listener_closed", "timeout"));
    }
    if let Err(error) = result {
        evidence.fail(error);
    }
    evidence.terminal.elapsed_ms = began.elapsed().as_millis().try_into()?;
    if result.is_ok() {
        evidence.terminal.outcome = "accepted";
    }
    let counts = ConnectionFacts {
        instance_id: instance,
        connection_id: evidence.terminal.connection_id.clone(),
        expected_peer_count: candidates.expected_peer_connection_count,
        unexpected_peer_count: candidates.unexpected_peer_count,
        candidate_overflow: candidates.candidate_overflow,
        expected_peer_match: candidates.expected_peer_connection_count == 1
            && candidates.unexpected_peer_count == 0,
    };
    write_serial_json(&options.root.join("connection-facts.json"), &counts)?;
    write_serial_json(&options.root.join("fixture-events.json"), &evidence.events)?;
    write_serial_json(&options.root.join("shares.json"), &evidence.shares)?;
    write_serial_json(
        &options.root.join("fixture-terminal.json"),
        &evidence.terminal,
    )?;
    drop(lifetime);
    if result.is_err() || Instant::now() >= deadline {
        bail!("exchange");
    }
    Ok(())
}
fn inventory_error(cause: crate::noise_serial::model::Cause) -> Error {
    let category = match cause.detail {
        "timeout" => "timeout",
        "eof" => "eof",
        "extra" => "extra",
        "overflow" => "evidence",
        "peer_conflict" => "authentication",
        _ => "protocol",
    };
    Error::new("setup_received", category)
}
struct Exchange<'a> {
    options: &'a Options,
    input: &'a Input,
    secret: &'a [u8; 32],
    public: &'a [u8; 32],
    deadline: Instant,
}
fn exchange(
    args: Exchange<'_>,
    listener: TcpListener,
    connection_pipe: File,
    evidence: &mut Evidence,
    candidates: &mut Candidates,
) -> model::Result<()> {
    let Exchange {
        options,
        input,
        secret,
        public,
        deadline,
    } = args;
    let expected = IpAddr::V4(
        input::ipv4(&input.expected_peer_ipv4)
            .map_err(|_| Error::new("setup_received", "admission"))?,
    );
    let selected = inventory::select(
        &listener,
        expected,
        (evidence.began + Duration::from_secs(120)).min(deadline),
        deadline,
        Duration::from_secs(10),
        candidates,
    )
    .map_err(inventory_error);
    let result = match selected {
        Ok((mut stream, act_one)) => {
            evidence.terminal.connection_id = Some(evidence.events.connection_id.clone());
            let attempt = (|| {
                let local = stream
                    .local_addr()
                    .map_err(|_| Error::new("setup_received", "protocol"))?;
                let peer = stream
                    .peer_addr()
                    .map_err(|_| Error::new("setup_received", "protocol"))?;
                emit_connection(connection_pipe, options, evidence, local, peer)?;
                let mut codec =
                    session::authenticate(&mut stream, act_one, secret, public, deadline)?;
                session::run(
                    &mut stream,
                    &mut codec,
                    input,
                    &options.root,
                    deadline,
                    evidence,
                )
            })();
            let close = stream.shutdown(Shutdown::Both);
            drop(stream);
            evidence.terminal.socket_closed = true;
            if close.is_err() && !evidence.terminal.peer_closed {
                attempt.and(Err(Error::new("peer_eof", "cleanup")))
            } else {
                attempt
            }
        }
        Err(error) => Err(error),
    };
    let extra =
        inventory::reject_later_peers(&listener, expected, candidates).map_err(inventory_error);
    drop(listener);
    evidence.terminal.listener_closed = true;
    let recorded = evidence.event("listener_closed", None, None);
    result.and(extra).and(recorded)
}
fn emit_connection(
    pipe: File,
    options: &Options,
    evidence: &Evidence,
    local: SocketAddr,
    peer: SocketAddr,
) -> model::Result<()> {
    input::write_pipe(
        pipe,
        &Connected {
            schema: "str005-v2-fixture-connection-runtime-v1",
            scope: options.scope,
            attempt_id: &options.attempt,
            instance_id: &evidence.terminal.instance_id,
            connection_id: &evidence.events.connection_id,
            observed_at_fixture_us: evidence.micros(),
            local_ipv4: local.ip().to_string(),
            local_port: local.port(),
            peer_ipv4: peer.ip().to_string(),
            peer_port: peer.port(),
        },
    )
    .map_err(|_| Error::new("setup_received", "evidence"))
}
