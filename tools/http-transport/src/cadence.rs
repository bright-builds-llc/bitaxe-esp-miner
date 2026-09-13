//! Passive, metadata-only observer for one freshly admitted telemetry endpoint.
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{PlainWebSocket, WebSocketRead};
mod input;
#[cfg(test)]
mod tests;

const READ_TIMEOUT: Duration = Duration::from_millis(250);
// Leave room for the final bounded read and close within the 360-second contract.
const OBSERVATION_LIMIT: Duration = Duration::from_secs(359);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Handoff<'a> {
    schema: &'a str,
    ipv4: &'a str,
    port: u16,
    observed_unix_ms: u64,
    expires_unix_ms: u64,
    binding_verified: bool,
}

fn origin(bytes: &[u8], now: u64) -> Result<Zeroizing<String>, &'static str> {
    let handoff: Handoff<'_> = serde_json::from_slice(bytes).map_err(|_| "invalid_input")?;
    let ip: std::net::Ipv4Addr = handoff.ipv4.parse().map_err(|_| "invalid_input")?;
    if handoff.schema != "cpu0-cadence-observer-input-v1"
        || !handoff.binding_verified
        || !ip.is_private()
        || handoff.port == 0
        || handoff.observed_unix_ms > now
        || now.saturating_sub(handoff.observed_unix_ms) > 5000
        || handoff.expires_unix_ms <= now
        || handoff
            .expires_unix_ms
            .saturating_sub(handoff.observed_unix_ms)
            > 5000
    {
        return Err("invalid_input");
    }
    Ok(Zeroizing::new(format!("http://{}:{}", ip, handoff.port)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<'a> {
    event: &'a str,
    #[serde(borrow)]
    data: &'a serde_json::value::RawValue,
}

fn validate_frame(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > 65_536 {
        return Err("invalid_frame");
    }
    let envelope: Envelope<'_> = serde_json::from_slice(bytes).map_err(|_| "invalid_frame")?;
    if envelope.event != "update" || !envelope.data.get().trim_start().starts_with('{') {
        return Err("invalid_frame");
    }
    // IgnoredAny validates the nested JSON without allocating copies of private fields.
    let _: serde::de::IgnoredAny =
        serde_json::from_str(envelope.data.get()).map_err(|_| "invalid_frame")?;
    Ok(())
}

#[derive(Default)]
struct Receipts {
    messages: u64,
    bytes: u64,
}

impl Receipts {
    fn emit(
        &self,
        output: &mut impl Write,
        started: Instant,
        event: &'static str,
        byte_count: usize,
        maybe_reason: Option<&'static str>,
    ) -> Result<(), &'static str> {
        serde_json::to_writer(
            &mut *output,
            &serde_json::json!({
                "schema": "cpu0-cadence-observer-v1", "event": event,
                "elapsedMs": started.elapsed().as_millis() as u64,
                "messageCount": self.messages, "totalBytes": self.bytes,
                "byteCount": byte_count, "reason": maybe_reason,
            }),
        )
        .map_err(|_| "output_failed")?;
        output.write_all(b"\n").map_err(|_| "output_failed")?;
        output.flush().map_err(|_| "output_failed")
    }
}

fn observe(
    socket: &mut PlainWebSocket,
    output: &mut impl Write,
    started: Instant,
    limit: Duration,
    mut stop: impl FnMut() -> Result<bool, &'static str>,
    receipts: &mut Receipts,
) -> Result<&'static str, &'static str> {
    receipts.emit(output, started, "connected", 0, None)?;
    loop {
        if stop()? {
            return Ok("requested");
        }
        if started.elapsed() >= limit {
            return Err("deadline");
        }
        match socket.read().map_err(|_| "transport_failed")? {
            WebSocketRead::Text(bytes) => {
                let bytes = Zeroizing::new(bytes);
                validate_frame(&bytes)?;
                receipts.messages = receipts.messages.checked_add(1).ok_or("counter_overflow")?;
                receipts.bytes = receipts
                    .bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or("counter_overflow")?;
                receipts.emit(output, started, "arrival", bytes.len(), None)?;
            }
            WebSocketRead::Timeout => {}
            WebSocketRead::Closed => return Err("peer_closed"),
        }
    }
}

/// Runs the stdin-controlled observer; returns an exit code and never prints raw errors.
/// Stdin carries a bounded initial handoff followed by `{"op":"stop"}` or EOF.
#[cfg(unix)]
pub fn run() -> i32 {
    let started = Instant::now();
    let mut output = std::io::stdout().lock();
    let mut receipts = Receipts::default();
    let result = (|| {
        if std::env::args_os().count() != 1 {
            return Err("invalid_input");
        }
        let mut input = input::PrivateInput::new(0);
        let handoff = input.initial_line(Instant::now() + Duration::from_secs(5))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "clock_invalid")?
            .as_millis() as u64;
        let endpoint = origin(&handoff, now)?;
        let mut socket = PlainWebSocket::connect(
            &endpoint,
            "/api/ws/live",
            Duration::from_secs(5),
            READ_TIMEOUT,
        )
        .map_err(|_| "connect_failed")?;
        let result = observe(
            &mut socket,
            &mut output,
            started,
            OBSERVATION_LIMIT,
            || input.stop_requested(),
            &mut receipts,
        );
        socket.close();
        result
    })();
    let (event, reason, exit) = match result {
        Ok(reason) => ("closed", reason, 0),
        Err(reason) => ("error", reason, 1),
    };
    if receipts
        .emit(&mut output, started, event, 0, Some(reason))
        .is_err()
    {
        return 1;
    }
    exit
}

#[cfg(not(unix))]
pub fn run() -> i32 {
    1
}
