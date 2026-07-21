use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use thiserror::Error;

use crate::phase35_http::{RawHttpMetrics, SchemeCategory, TlsVerification, TransportOutcome};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RESPONSE_HEADER_BYTES: usize = 65_536;
const MAX_RESPONSE_BODY_BYTES: usize = 65_536;
const READ_CHUNK_BYTES: usize = 8_192;

#[derive(Debug)]
pub(crate) struct ProbeResult {
    pub(crate) metrics: RawHttpMetrics,
    pub(crate) headers: Vec<u8>,
    pub(crate) body: Vec<u8>,
}

#[derive(Debug, Error)]
pub(crate) enum Phase35HttpProbeError {
    #[error("Phase 35 HTTP probe origin is invalid")]
    InvalidOrigin,
    #[error("Phase 35 HTTP probe could not resolve the private origin")]
    ResolutionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Origin {
    scheme: SchemeCategory,
    authority: String,
    host: String,
    port: u16,
}

enum ProbeStream {
    Plain(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl ProbeStream {
    fn set_read_timeout(&self, timeout: Duration) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.set_read_timeout(Some(timeout)),
            Self::Tls(stream) => stream.sock.set_read_timeout(Some(timeout)),
        }
    }
}

impl Read for ProbeStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for ProbeStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buffer),
            Self::Tls(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

pub(crate) fn probe_phase35_http(url: &str) -> Result<ProbeResult, Phase35HttpProbeError> {
    probe_phase35_http_with_timeouts(url, CONNECT_TIMEOUT, TOTAL_TIMEOUT)
}

fn probe_phase35_http_with_timeouts(
    url: &str,
    connect_timeout: Duration,
    total_timeout: Duration,
) -> Result<ProbeResult, Phase35HttpProbeError> {
    let origin = Origin::parse(url)?;
    let started = Instant::now();
    let maybe_socket = connect(&origin, connect_timeout);
    let Ok(socket) = maybe_socket else {
        return Ok(empty_result(
            origin.scheme,
            TransportOutcome::TcpConnectionFailure,
            elapsed_millis(started),
        ));
    };
    let tcp_connect_millis = elapsed_millis(started);
    let remaining = remaining_duration(started, total_timeout);
    if remaining.is_zero() {
        return Ok(connected_result(
            origin.scheme,
            TransportOutcome::RequestSendFailure,
            tcp_connect_millis,
            0,
            elapsed_millis(started),
        ));
    }
    socket
        .set_read_timeout(Some(remaining))
        .map_err(|_| Phase35HttpProbeError::ResolutionFailed)?;
    socket
        .set_write_timeout(Some(remaining))
        .map_err(|_| Phase35HttpProbeError::ResolutionFailed)?;

    let (mut stream, tls_handshake_millis, tls_verification) = match origin.scheme {
        SchemeCategory::Http => (
            ProbeStream::Plain(socket),
            0,
            TlsVerification::NotApplicable,
        ),
        SchemeCategory::Https => match tls_stream(&origin, socket, started, total_timeout) {
            Ok((stream, handshake_completed_millis)) => (
                ProbeStream::Tls(Box::new(stream)),
                handshake_completed_millis
                    .saturating_sub(tcp_connect_millis)
                    .max(1),
                TlsVerification::Verified,
            ),
            Err(()) => {
                return Ok(connected_tls_failure_result(
                    tcp_connect_millis,
                    elapsed_millis(started),
                ));
            }
        },
    };

    let request = request_bytes(&origin);
    let send = send_request(&mut stream, &request);
    if !send.complete {
        let mut result = connected_result_with_tls(
            origin.scheme,
            TransportOutcome::RequestSendFailure,
            tcp_connect_millis,
            tls_handshake_millis,
            tls_verification,
            elapsed_millis(started),
        );
        result.metrics.request_bytes = send.bytes_written;
        return Ok(result);
    }
    let request_send_complete_millis = elapsed_millis(started);
    let request_byte_count = send.bytes_written;

    let mut wire_response = Vec::new();
    let mut first_byte_millis = 0;
    let mut transport_outcome = TransportOutcome::Complete;
    loop {
        let remaining = remaining_duration(started, total_timeout);
        if remaining.is_zero() {
            transport_outcome = TransportOutcome::ResponseTimeout;
            break;
        }
        if stream.set_read_timeout(remaining).is_err() {
            transport_outcome = TransportOutcome::ReceiveFailed;
            break;
        }
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read_bytes) => {
                if first_byte_millis == 0 {
                    first_byte_millis = elapsed_millis(started);
                }
                wire_response.extend_from_slice(&buffer[..read_bytes]);
                if response_exceeds_limit(&wire_response) {
                    transport_outcome = TransportOutcome::ResponseOverLimit;
                    break;
                }
                if response_is_complete(&wire_response) {
                    break;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                transport_outcome = TransportOutcome::ResponseTimeout;
                break;
            }
            Err(_) => {
                transport_outcome = TransportOutcome::ReceiveFailed;
                break;
            }
        }
    }

    let parsed = ParsedResponse::parse(&wire_response);
    let (headers, body, response_status, response_header_count) = match parsed {
        Some(parsed) => (
            parsed.headers,
            parsed.body,
            parsed.status,
            parsed.header_count,
        ),
        None => (Vec::new(), Vec::new(), 0, 0),
    };
    let response_header_bytes = u64::try_from(headers.len()).unwrap_or(u64::MAX);
    let response_body_bytes = u64::try_from(body.len()).unwrap_or(u64::MAX);

    Ok(ProbeResult {
        metrics: RawHttpMetrics {
            scheme_category: origin.scheme,
            transport_outcome,
            tcp_connect_millis,
            tls_handshake_millis,
            request_send_complete_millis,
            request_bytes: request_byte_count,
            response_status,
            response_header_count,
            response_header_bytes,
            response_body_bytes,
            total_millis: elapsed_millis(started),
            first_byte_millis,
            tls_verification,
        },
        headers,
        body,
    })
}

fn connect(origin: &Origin, timeout: Duration) -> io::Result<TcpStream> {
    let addresses = (origin.host.as_str(), origin.port).to_socket_addrs()?;
    let mut last_error = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address")))
}

fn tls_stream(
    origin: &Origin,
    mut socket: TcpStream,
    started: Instant,
    total_timeout: Duration,
) -> Result<(StreamOwned<ClientConnection, TcpStream>, u64), ()> {
    let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = ServerName::try_from(origin.host.clone()).map_err(|_| ())?;
    let mut connection = ClientConnection::new(Arc::new(config), server_name).map_err(|_| ())?;
    while connection.is_handshaking() {
        let remaining = remaining_duration(started, total_timeout);
        if remaining.is_zero() {
            return Err(());
        }
        socket.set_read_timeout(Some(remaining)).map_err(|_| ())?;
        socket.set_write_timeout(Some(remaining)).map_err(|_| ())?;
        connection.complete_io(&mut socket).map_err(|_| ())?;
    }
    Ok((
        StreamOwned::new(connection, socket),
        elapsed_millis(started),
    ))
}

fn request_bytes(origin: &Origin) -> Vec<u8> {
    format!(
        "GET /api/system/info HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        origin.authority
    )
    .into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SendObservation {
    bytes_written: u64,
    complete: bool,
}

fn send_request(writer: &mut impl Write, request: &[u8]) -> SendObservation {
    let mut offset = 0;
    while offset < request.len() {
        match writer.write(&request[offset..]) {
            Ok(0) | Err(_) => {
                return SendObservation {
                    bytes_written: u64::try_from(offset).unwrap_or(u64::MAX),
                    complete: false,
                };
            }
            Ok(written) => offset = offset.saturating_add(written),
        }
    }
    let complete = writer.flush().is_ok();
    SendObservation {
        bytes_written: u64::try_from(offset).unwrap_or(u64::MAX),
        complete,
    }
}

fn empty_result(
    scheme: SchemeCategory,
    outcome: TransportOutcome,
    total_millis: u64,
) -> ProbeResult {
    ProbeResult {
        metrics: RawHttpMetrics::empty(scheme, outcome, total_millis),
        headers: Vec::new(),
        body: Vec::new(),
    }
}

fn connected_result(
    scheme: SchemeCategory,
    outcome: TransportOutcome,
    tcp_connect_millis: u64,
    tls_handshake_millis: u64,
    total_millis: u64,
) -> ProbeResult {
    connected_result_with_tls(
        scheme,
        outcome,
        tcp_connect_millis,
        tls_handshake_millis,
        TlsVerification::NotApplicable,
        total_millis,
    )
}

fn connected_result_with_tls(
    scheme: SchemeCategory,
    outcome: TransportOutcome,
    tcp_connect_millis: u64,
    tls_handshake_millis: u64,
    tls_verification: TlsVerification,
    total_millis: u64,
) -> ProbeResult {
    let mut result = empty_result(scheme, outcome, total_millis);
    result.metrics.tcp_connect_millis = tcp_connect_millis;
    result.metrics.tls_handshake_millis = tls_handshake_millis;
    result.metrics.tls_verification = tls_verification;
    result
}

fn connected_tls_failure_result(tcp_connect_millis: u64, total_millis: u64) -> ProbeResult {
    let mut result = empty_result(
        SchemeCategory::Https,
        TransportOutcome::TlsHandshakeFailure,
        total_millis,
    );
    result.metrics.tcp_connect_millis = tcp_connect_millis;
    result.metrics.tls_verification = TlsVerification::Failed;
    result
}

fn elapsed_millis(started: Instant) -> u64 {
    let elapsed = started.elapsed();
    if elapsed.is_zero() {
        return 0;
    }
    u64::try_from(elapsed.as_millis())
        .unwrap_or(u64::MAX)
        .max(1)
}

fn remaining_duration(started: Instant, total: Duration) -> Duration {
    total.saturating_sub(started.elapsed())
}

fn header_end(response: &[u8]) -> Option<usize> {
    response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn response_exceeds_limit(response: &[u8]) -> bool {
    match header_end(response) {
        Some(end) => {
            end > MAX_RESPONSE_HEADER_BYTES || response.len() - end > MAX_RESPONSE_BODY_BYTES
        }
        None => response.len() > MAX_RESPONSE_HEADER_BYTES,
    }
}

fn response_is_complete(response: &[u8]) -> bool {
    let Some(end) = header_end(response) else {
        return false;
    };
    let Ok(header_text) = std::str::from_utf8(&response[..end]) else {
        return false;
    };
    let body = &response[end..];
    if let Some(length) = content_length(header_text) {
        return body.len() >= length;
    }
    if is_chunked(header_text) {
        return decode_chunked(body).is_some();
    }
    false
}

struct ParsedResponse {
    status: u16,
    header_count: u64,
    headers: Vec<u8>,
    body: Vec<u8>,
}

impl ParsedResponse {
    fn parse(response: &[u8]) -> Option<Self> {
        let end = header_end(response)?;
        let header_text = std::str::from_utf8(&response[..end]).ok()?;
        let mut lines = header_text.split("\r\n");
        let status_line = lines.next()?;
        let mut status_parts = status_line.split_ascii_whitespace();
        let protocol = status_parts.next()?;
        let status = status_parts.next()?.parse::<u16>().ok()?;
        if protocol != "HTTP/1.1" || !(100..=599).contains(&status) {
            return None;
        }
        let header_count = lines
            .filter(|line| !line.is_empty() && line.split_once(':').is_some())
            .count();
        if header_count == 0 {
            return Some(Self {
                status,
                header_count: 0,
                headers: Vec::new(),
                body: Vec::new(),
            });
        }
        let raw_body = &response[end..];
        let body = if is_chunked(header_text) {
            decode_chunked(raw_body)?
        } else if let Some(length) = content_length(header_text) {
            if raw_body.len() < length {
                raw_body.to_vec()
            } else {
                raw_body[..length].to_vec()
            }
        } else {
            raw_body.to_vec()
        };
        Some(Self {
            status,
            header_count: u64::try_from(header_count).ok()?,
            headers: response[..end].to_vec(),
            body,
        })
    }
}

fn content_length(headers: &str) -> Option<usize> {
    headers.split("\r\n").find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })
}

fn is_chunked(headers: &str) -> bool {
    headers.split("\r\n").any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.eq_ignore_ascii_case("transfer-encoding")
            && value
                .split(',')
                .any(|encoding| encoding.trim().eq_ignore_ascii_case("chunked"))
    })
}

fn decode_chunked(mut body: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::new();
    loop {
        let line_end = body.windows(2).position(|window| window == b"\r\n")?;
        let size_text = std::str::from_utf8(&body[..line_end]).ok()?;
        let size = usize::from_str_radix(size_text.split(';').next()?.trim(), 16).ok()?;
        body = &body[line_end + 2..];
        if size == 0 {
            return body.starts_with(b"\r\n").then_some(decoded);
        }
        if size > MAX_RESPONSE_BODY_BYTES.saturating_sub(decoded.len())
            || body.len() < size + 2
            || &body[size..size + 2] != b"\r\n"
        {
            return None;
        }
        decoded.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
}

impl Origin {
    fn parse(url: &str) -> Result<Self, Phase35HttpProbeError> {
        let (scheme, remainder, default_port) = if let Some(remainder) = url.strip_prefix("http://")
        {
            (SchemeCategory::Http, remainder, 80)
        } else if let Some(remainder) = url.strip_prefix("https://") {
            (SchemeCategory::Https, remainder, 443)
        } else {
            return Err(Phase35HttpProbeError::InvalidOrigin);
        };
        let authority = remainder.strip_suffix('/').unwrap_or(remainder);
        if authority.is_empty()
            || authority.contains('/')
            || authority.contains('@')
            || authority.chars().any(char::is_whitespace)
        {
            return Err(Phase35HttpProbeError::InvalidOrigin);
        }
        let (host, port) = parse_authority(authority, default_port)?;
        Ok(Self {
            scheme,
            authority: authority.to_owned(),
            host,
            port,
        })
    }
}

fn parse_authority(
    authority: &str,
    default_port: u16,
) -> Result<(String, u16), Phase35HttpProbeError> {
    if let Some(bracketed) = authority.strip_prefix('[') {
        let Some(close) = bracketed.find(']') else {
            return Err(Phase35HttpProbeError::InvalidOrigin);
        };
        let host = &bracketed[..close];
        let suffix = &bracketed[close + 1..];
        let port = if suffix.is_empty() {
            default_port
        } else {
            suffix
                .strip_prefix(':')
                .ok_or(Phase35HttpProbeError::InvalidOrigin)?
                .parse::<u16>()
                .map_err(|_| Phase35HttpProbeError::InvalidOrigin)?
        };
        return (!host.is_empty() && port > 0)
            .then(|| (host.to_owned(), port))
            .ok_or(Phase35HttpProbeError::InvalidOrigin);
    }
    if authority.matches(':').count() > 1 {
        return Err(Phase35HttpProbeError::InvalidOrigin);
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host,
            port.parse::<u16>()
                .map_err(|_| Phase35HttpProbeError::InvalidOrigin)?,
        ),
        None => (authority, default_port),
    };
    (!host.is_empty() && port > 0)
        .then(|| (host.to_owned(), port))
        .ok_or(Phase35HttpProbeError::InvalidOrigin)
}

#[cfg(test)]
mod tests;
