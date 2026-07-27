//! Shared bounded HTTP/1.1 transport for host-side ESP tooling.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_HEADER_BYTES: usize = 65_536;
const MAX_BODY_BYTES: usize = 65_536;
const READ_CHUNK_BYTES: usize = 8_192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportOutcome {
    Complete,
    TcpConnectionFailure,
    TlsHandshakeFailure,
    RequestSendFailure,
    ResponseTimeout,
    ReceiveFailed,
    ResponseOverLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVerification {
    NotApplicable,
    Failed,
    Verified,
}

#[derive(Debug, Clone)]
struct Origin {
    scheme: Scheme,
    authority: String,
    host: String,
    port: u16,
}

pub struct StrictHttpClient {
    origin: Origin,
}

#[derive(Debug)]
pub struct ExchangeObservation {
    pub scheme: Scheme,
    pub transport_outcome: TransportOutcome,
    pub tcp_connect_millis: u64,
    pub tls_handshake_millis: u64,
    pub request_send_complete_millis: u64,
    pub request_bytes_written: u64,
    pub request_write_complete: bool,
    pub response_received: bool,
    pub response_status: u16,
    pub response_header_count: u64,
    pub headers: Vec<u8>,
    pub body: Vec<u8>,
    pub total_millis: u64,
    pub first_byte_millis: u64,
    pub tls_verification: TlsVerification,
}

enum Transport {
    Plain(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Transport {
    fn set_read_timeout(&self, timeout: Duration) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.set_read_timeout(Some(timeout)),
            Self::Tls(stream) => stream.sock.set_read_timeout(Some(timeout)),
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for Transport {
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

impl StrictHttpClient {
    pub fn new(origin: &str) -> Result<Self> {
        Ok(Self {
            origin: Origin::parse(origin)?,
        })
    }

    pub fn get_system_info(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until("GET", "/api/system/info", deadline, DEFAULT_CONNECT_TIMEOUT)
    }

    pub fn post_restart_once(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until(
            "POST",
            "/api/system/restart",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }

    pub fn exchange_with_timeouts(
        &self,
        method: &str,
        path: &str,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<ExchangeObservation> {
        self.exchange_until(
            method,
            path,
            Instant::now() + total_timeout,
            connect_timeout,
        )
    }

    fn exchange_until(
        &self,
        method: &str,
        path: &str,
        deadline: Instant,
        connect_timeout: Duration,
    ) -> Result<ExchangeObservation> {
        if !matches!(method, "GET" | "POST")
            || !path.starts_with('/')
            || path.contains(char::is_whitespace)
        {
            bail!("strict HTTP request is invalid");
        }
        let started = Instant::now();
        let mut observation = ExchangeObservation::empty(self.origin.scheme);
        let Some(socket) = connect(&self.origin, started, deadline, connect_timeout) else {
            observation.transport_outcome = TransportOutcome::TcpConnectionFailure;
            observation.total_millis = elapsed_millis(started);
            return Ok(observation);
        };
        observation.tcp_connect_millis = elapsed_millis(started);
        let remaining = remaining(deadline);
        if remaining.is_zero() {
            observation.transport_outcome = TransportOutcome::RequestSendFailure;
            observation.total_millis = elapsed_millis(started);
            return Ok(observation);
        }
        socket.set_read_timeout(Some(remaining))?;
        socket.set_write_timeout(Some(remaining))?;

        let Some(mut transport) =
            build_transport(&self.origin, socket, started, deadline, &mut observation)?
        else {
            observation.total_millis = elapsed_millis(started);
            return Ok(observation);
        };
        let request = request_bytes(method, path, &self.origin.authority);
        let send = send_request(&mut transport, &request);
        observation.request_bytes_written = send.bytes_written;
        observation.request_write_complete = send.complete;
        if !send.complete {
            observation.transport_outcome = TransportOutcome::RequestSendFailure;
            observation.total_millis = elapsed_millis(started);
            return Ok(observation);
        }
        observation.request_send_complete_millis = elapsed_millis(started).max(1);

        let response = read_response(&mut transport, started, deadline);
        observation.transport_outcome = response.outcome;
        observation.first_byte_millis = response.first_byte_millis;
        if let Some(parsed) = ParsedResponse::parse(&response.wire) {
            observation.response_received = true;
            observation.response_status = parsed.status;
            observation.response_header_count = parsed.header_count;
            observation.headers = parsed.headers;
            observation.body = parsed.body;
        }
        observation.total_millis = elapsed_millis(started);
        Ok(observation)
    }
}

impl ExchangeObservation {
    fn empty(scheme: Scheme) -> Self {
        Self {
            scheme,
            transport_outcome: TransportOutcome::Complete,
            tcp_connect_millis: 0,
            tls_handshake_millis: 0,
            request_send_complete_millis: 0,
            request_bytes_written: 0,
            request_write_complete: false,
            response_received: false,
            response_status: 0,
            response_header_count: 0,
            headers: Vec::new(),
            body: Vec::new(),
            total_millis: 0,
            first_byte_millis: 0,
            tls_verification: match scheme {
                Scheme::Http => TlsVerification::NotApplicable,
                Scheme::Https => TlsVerification::Failed,
            },
        }
    }
}

fn connect(
    origin: &Origin,
    started: Instant,
    deadline: Instant,
    connect_timeout: Duration,
) -> Option<TcpStream> {
    let addresses = (origin.host.as_str(), origin.port)
        .to_socket_addrs()
        .ok()?
        .collect::<Vec<SocketAddr>>();
    let connect_deadline = (started + connect_timeout).min(deadline);
    for address in addresses {
        let budget = connect_deadline.saturating_duration_since(Instant::now());
        if budget.is_zero() {
            return None;
        }
        if let Ok(stream) = TcpStream::connect_timeout(&address, budget) {
            return Some(stream);
        }
    }
    None
}

fn build_transport(
    origin: &Origin,
    mut socket: TcpStream,
    started: Instant,
    deadline: Instant,
    observation: &mut ExchangeObservation,
) -> Result<Option<Transport>> {
    if origin.scheme == Scheme::Http {
        return Ok(Some(Transport::Plain(socket)));
    }
    let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = match ServerName::try_from(origin.host.clone()) {
        Ok(server_name) => server_name,
        Err(_) => {
            observation.transport_outcome = TransportOutcome::TlsHandshakeFailure;
            return Ok(None);
        }
    };
    let mut connection = match ClientConnection::new(Arc::new(config), server_name) {
        Ok(connection) => connection,
        Err(_) => {
            observation.transport_outcome = TransportOutcome::TlsHandshakeFailure;
            return Ok(None);
        }
    };
    while connection.is_handshaking() {
        let budget = remaining(deadline);
        if budget.is_zero() {
            observation.transport_outcome = TransportOutcome::TlsHandshakeFailure;
            return Ok(None);
        }
        socket.set_read_timeout(Some(budget))?;
        socket.set_write_timeout(Some(budget))?;
        if connection.complete_io(&mut socket).is_err() {
            observation.transport_outcome = TransportOutcome::TlsHandshakeFailure;
            return Ok(None);
        }
    }
    observation.tls_handshake_millis = elapsed_millis(started)
        .saturating_sub(observation.tcp_connect_millis)
        .max(1);
    observation.tls_verification = TlsVerification::Verified;
    Ok(Some(Transport::Tls(Box::new(StreamOwned::new(
        connection, socket,
    )))))
}

fn request_bytes(method: &str, path: &str, authority: &str) -> Vec<u8> {
    format!(
        "{method} {path} HTTP/1.1\r\nHost: {authority}\r\nAccept: application/json\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
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
    SendObservation {
        bytes_written: u64::try_from(offset).unwrap_or(u64::MAX),
        complete: writer.flush().is_ok(),
    }
}

struct ResponseObservation {
    outcome: TransportOutcome,
    first_byte_millis: u64,
    wire: Vec<u8>,
}

fn read_response(
    stream: &mut Transport,
    started: Instant,
    deadline: Instant,
) -> ResponseObservation {
    let mut wire = Vec::new();
    let mut first_byte_millis = 0;
    let mut outcome = TransportOutcome::Complete;
    loop {
        let budget = remaining(deadline);
        if budget.is_zero() {
            outcome = TransportOutcome::ResponseTimeout;
            break;
        }
        if stream.set_read_timeout(budget).is_err() {
            outcome = TransportOutcome::ReceiveFailed;
            break;
        }
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read_bytes) => {
                if first_byte_millis == 0 {
                    first_byte_millis = elapsed_millis(started).max(1);
                }
                wire.extend_from_slice(&buffer[..read_bytes]);
                if response_exceeds_limit(&wire) {
                    outcome = TransportOutcome::ResponseOverLimit;
                    clamp_response_to_limits(&mut wire);
                    break;
                }
                if response_is_complete(&wire) {
                    break;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                outcome = TransportOutcome::ResponseTimeout;
                break;
            }
            Err(_) => {
                outcome = TransportOutcome::ReceiveFailed;
                break;
            }
        }
    }
    ResponseObservation {
        outcome,
        first_byte_millis,
        wire,
    }
}

fn clamp_response_to_limits(response: &mut Vec<u8>) {
    let maximum = header_end(response)
        .map(|end| end.min(MAX_HEADER_BYTES).saturating_add(MAX_BODY_BYTES))
        .unwrap_or(MAX_HEADER_BYTES);
    response.truncate(maximum);
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
        let mut status_parts = lines.next()?.split_ascii_whitespace();
        if status_parts.next()? != "HTTP/1.1" {
            return None;
        }
        let status = status_parts.next()?.parse::<u16>().ok()?;
        if !(100..=599).contains(&status) {
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
            decode_chunked(raw_body).unwrap_or_else(|| raw_body.to_vec())
        } else if let Some(length) = content_length(header_text) {
            raw_body[..raw_body.len().min(length)].to_vec()
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

fn header_end(response: &[u8]) -> Option<usize> {
    response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn response_exceeds_limit(response: &[u8]) -> bool {
    header_end(response).map_or(response.len() > MAX_HEADER_BYTES, |end| {
        end > MAX_HEADER_BYTES || response.len().saturating_sub(end) > MAX_BODY_BYTES
    })
}

fn response_is_complete(response: &[u8]) -> bool {
    let Some(end) = header_end(response) else {
        return false;
    };
    let Ok(headers) = std::str::from_utf8(&response[..end]) else {
        return false;
    };
    if let Some(length) = content_length(headers) {
        return response.len().saturating_sub(end) >= length;
    }
    is_chunked(headers) && decode_chunked(&response[end..]).is_some()
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
        if size > MAX_BODY_BYTES.saturating_sub(decoded.len())
            || body.len() < size + 2
            || &body[size..size + 2] != b"\r\n"
        {
            return None;
        }
        decoded.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
}

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
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

impl Origin {
    fn parse(url: &str) -> Result<Self> {
        let (scheme, remainder, default_port) = if let Some(remainder) = url.strip_prefix("http://")
        {
            (Scheme::Http, remainder, 80)
        } else if let Some(remainder) = url.strip_prefix("https://") {
            (Scheme::Https, remainder, 443)
        } else {
            bail!("strict HTTP origin scheme is invalid");
        };
        let authority = remainder.strip_suffix('/').unwrap_or(remainder);
        if authority.is_empty()
            || authority.contains('/')
            || authority.contains('@')
            || authority.chars().any(char::is_whitespace)
        {
            bail!("strict HTTP origin authority is invalid");
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

fn parse_authority(authority: &str, default_port: u16) -> Result<(String, u16)> {
    if let Some(bracketed) = authority.strip_prefix('[') {
        let Some(close) = bracketed.find(']') else {
            bail!("strict HTTP bracketed authority is invalid");
        };
        let host = &bracketed[..close];
        let suffix = &bracketed[close + 1..];
        let port = if suffix.is_empty() {
            default_port
        } else {
            suffix
                .strip_prefix(':')
                .ok_or_else(|| anyhow::anyhow!("strict HTTP authority port is invalid"))?
                .parse::<u16>()?
        };
        if host.is_empty() || port == 0 {
            bail!("strict HTTP authority is invalid");
        }
        return Ok((host.to_owned(), port));
    }
    if authority.matches(':').count() > 1 {
        bail!("strict HTTP authority is ambiguous");
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>()?),
        None => (authority, default_port),
    };
    if host.is_empty() || port == 0 {
        bail!("strict HTTP authority is invalid");
    }
    Ok((host.to_owned(), port))
}

#[cfg(test)]
mod tests;
