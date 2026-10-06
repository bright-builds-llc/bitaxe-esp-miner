//! Two-thread soak observer: HTTP polling and the WebSocket stream never share a loop, so a slow
//! HTTP exchange cannot delay WebSocket reads (the legacy campaign's single loop could).
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bitaxe_api::SystemInfoWire;
use serde_json::json;

use super::sample::{PoolSettings, SoakSample, SoakTransport};
use crate::continuity::{apply_live_frame, ReconnectBackoff};
use crate::{PlainWebSocket, StrictHttpClient, WebSocketRead};

const HTTP_POLL_INTERVAL: Duration = Duration::from_secs(2);
const HTTP_DEADLINE: Duration = Duration::from_secs(3);
const WEBSOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const WEBSOCKET_IO_TIMEOUT: Duration = Duration::from_millis(250);
const IDLE: Duration = Duration::from_millis(50);

pub(super) enum Event {
    Sample(SoakTransport, SoakSample),
    Connected { reconnect: bool },
    Closed,
    Failed(&'static str),
    ConnectFailed,
    HttpFailed,
    ProjectionInvalid(SoakTransport),
}

pub(super) struct Stamped {
    host_unix_ms: u64,
    event: Event,
}

pub(super) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

fn send(events: &Sender<Stamped>, event: Event) -> bool {
    events
        .send(Stamped {
            host_unix_ms: now_unix_ms(),
            event,
        })
        .is_ok()
}

fn poll_http(
    client: &StrictHttpClient,
    events: &Sender<Stamped>,
    maybe_initial_pool: &mut Option<PoolSettings>,
) {
    let maybe_wire = client
        .get_system_info(Instant::now() + HTTP_DEADLINE)
        .ok()
        .and_then(|observation| {
            observation
                .maybe_http_response()
                .filter(|response| response.status() == 200)
                .map(|response| serde_json::from_slice::<SystemInfoWire>(response.body()))
        });
    let event = match maybe_wire {
        None => Event::HttpFailed,
        Some(Err(_)) => Event::ProjectionInvalid(SoakTransport::Http),
        Some(Ok(wire)) => {
            let pool = PoolSettings::from_wire(&wire);
            let initial = maybe_initial_pool.get_or_insert_with(|| PoolSettings::from_wire(&wire));
            let matches = *initial == pool;
            Event::Sample(
                SoakTransport::Http,
                SoakSample::from_wire(&wire, Some(matches)),
            )
        }
    };
    send(events, event);
}

pub(super) fn http_thread(origin: &str, stop: &AtomicBool, events: &Sender<Stamped>) {
    let Ok(client) = StrictHttpClient::new(origin) else {
        send(events, Event::Failed("http_origin"));
        return;
    };
    let mut maybe_initial_pool = None;
    let mut next = Instant::now();
    while !stop.load(Ordering::Acquire) {
        if Instant::now() >= next {
            next += HTTP_POLL_INTERVAL;
            poll_http(&client, events, &mut maybe_initial_pool);
        }
        thread::sleep(IDLE);
    }
}

pub(super) fn websocket_thread(origin: &str, stop: &AtomicBool, events: &Sender<Stamped>) {
    let mut maybe_socket: Option<PlainWebSocket> = None;
    let mut projection = None;
    let mut connected_once = false;
    let mut backoff = ReconnectBackoff::new();
    let mut next_attempt = Instant::now();
    while !stop.load(Ordering::Acquire) {
        let Some(socket) = maybe_socket.as_mut() else {
            if Instant::now() < next_attempt {
                thread::sleep(IDLE);
                continue;
            }
            match PlainWebSocket::connect(
                origin,
                "/api/ws/live",
                WEBSOCKET_CONNECT_TIMEOUT,
                WEBSOCKET_IO_TIMEOUT,
            ) {
                Ok(socket) => {
                    send(
                        events,
                        Event::Connected {
                            reconnect: connected_once,
                        },
                    );
                    connected_once = true;
                    backoff.reset();
                    projection = None;
                    maybe_socket = Some(socket);
                }
                Err(_) => {
                    send(events, Event::ConnectFailed);
                    next_attempt = Instant::now() + backoff.take_delay();
                }
            }
            continue;
        };
        let event = match socket.read() {
            Ok(WebSocketRead::Timeout) => continue,
            Ok(WebSocketRead::Text(bytes)) => match apply_live_frame(&bytes, &mut projection) {
                Some(wire) => {
                    Event::Sample(SoakTransport::Websocket, SoakSample::from_wire(&wire, None))
                }
                None => Event::ProjectionInvalid(SoakTransport::Websocket),
            },
            Ok(WebSocketRead::Closed) => Event::Closed,
            Err(_) => Event::Failed("websocket_read"),
        };
        if matches!(event, Event::Closed | Event::Failed(_)) {
            maybe_socket = None;
            projection = None;
            next_attempt = Instant::now() + backoff.take_delay();
        }
        send(events, event);
    }
    if let Some(socket) = maybe_socket.as_mut() {
        socket.close();
    }
}

fn journal_line(stamped: &Stamped) -> serde_json::Value {
    let (transport, event, maybe_sample, maybe_detail) = match &stamped.event {
        Event::Sample(transport, sample) => (Some(*transport), "sample", Some(sample), None),
        Event::Connected { reconnect } => (
            Some(SoakTransport::Websocket),
            "connected",
            None,
            Some(json!({"reconnect": reconnect})),
        ),
        Event::Closed => (Some(SoakTransport::Websocket), "closed", None, None),
        Event::Failed(reason) => (None, "failed", None, Some(json!({"reason": reason}))),
        Event::ConnectFailed => (Some(SoakTransport::Websocket), "connect_failed", None, None),
        Event::HttpFailed => (Some(SoakTransport::Http), "http_failed", None, None),
        Event::ProjectionInvalid(transport) => (Some(*transport), "projection_invalid", None, None),
    };
    json!({"schema": super::JOURNAL_SCHEMA, "hostUnixMs": stamped.host_unix_ms, "transport": transport,
        "event": event, "sample": maybe_sample, "detail": maybe_detail})
}

/// Writes every event until `should_stop` or the lifetime bound, then stops both threads.
pub(super) fn run_threads(
    origin: Arc<zeroize::Zeroizing<String>>,
    output: &mut impl Write,
    lifetime: Duration,
    mut should_stop: impl FnMut() -> Result<bool, &'static str>,
) -> Result<&'static str, &'static str> {
    let stop = Arc::new(AtomicBool::new(false));
    let (sender, receiver): (Sender<Stamped>, Receiver<Stamped>) = mpsc::channel();
    let handles = [
        http_thread as fn(&str, &AtomicBool, &Sender<Stamped>),
        websocket_thread,
    ]
    .map(|body| {
        let (stop, sender, origin) = (Arc::clone(&stop), sender.clone(), Arc::clone(&origin));
        thread::spawn(move || body(&origin, &stop, &sender))
    });
    drop(sender);
    let started = Instant::now();
    let outcome = loop {
        match should_stop() {
            Ok(true) => break Ok("requested"),
            Ok(false) => {}
            Err(reason) => break Err(reason),
        }
        if started.elapsed() >= lifetime {
            break Err("deadline");
        }
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(stamped) => {
                if write_line(output, &journal_line(&stamped)).is_err() {
                    break Err("output_failed");
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break Err("observers_exited"),
        }
    };
    stop.store(true, Ordering::Release);
    let panicked = handles
        .into_iter()
        .map(thread::JoinHandle::join)
        .any(|joined| joined.is_err());
    for stamped in receiver.try_iter() {
        write_line(output, &journal_line(&stamped)).map_err(|_| "output_failed")?;
    }
    if panicked {
        return Err("observer_panicked");
    }
    outcome
}

pub(super) fn write_line(
    output: &mut impl Write,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")?;
    output.flush()
}
