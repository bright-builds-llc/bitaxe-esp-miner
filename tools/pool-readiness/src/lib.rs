//! Protected, bounded owner-pool Stratum V1 readiness evidence.

use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::Command,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use bitaxe_stratum::{
    jsonrpc::StratumRequestId,
    v1::messages::{parse_server_message, StratumV1ClientMessage, StratumV1ServerMessage},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const EXPECTED_ATTEMPT_ORDINAL: u8 = 5;
const EXPECTED_SAMPLES: u8 = 3;
const EXPECTED_SAMPLE_TIMEOUT: Duration = Duration::from_secs(15);
const EXPECTED_SAMPLE_DELAY: Duration = Duration::from_secs(2);
const MAX_SERVER_BYTES: usize = 64 * 1024;
const MAX_SERVER_MESSAGES: usize = 256;
const REFERENCE_PATH: &str = "reference/esp-miner";
const REPORT_NAME: &str = "readiness-result.json";
const REPORT_SCHEMA: &str = "bitaxe-pool-readiness-evidence-v1";

#[derive(Debug, Clone)]
pub struct ReadinessOptions {
    pub private_root: PathBuf,
    pub pool_credentials: PathBuf,
    pub attempt_ordinal: u8,
    pub samples: u8,
    pub sample_timeout: Duration,
    pub sample_delay: Duration,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessCategory {
    Ready,
    ContractInvalid,
    SourceUnclean,
    CredentialInvalid,
    PrivateRootInvalid,
    ResolutionFailed,
    ConnectionFailed,
    TransportFailed,
    Timeout,
    ProtocolInvalid,
    ConfigureRejected,
    SubscribeRejected,
    AuthorizeRejected,
    InputLimitExceeded,
}

impl ReadinessCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ContractInvalid => "contract_invalid",
            Self::SourceUnclean => "source_unclean",
            Self::CredentialInvalid => "credential_invalid",
            Self::PrivateRootInvalid => "private_root_invalid",
            Self::ResolutionFailed => "resolution_failed",
            Self::ConnectionFailed => "connection_failed",
            Self::TransportFailed => "transport_failed",
            Self::Timeout => "timeout",
            Self::ProtocolInvalid => "protocol_invalid",
            Self::ConfigureRejected => "configure_rejected",
            Self::SubscribeRejected => "subscribe_rejected",
            Self::AuthorizeRejected => "authorize_rejected",
            Self::InputLimitExceeded => "input_limit_exceeded",
        }
    }
}

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
#[error("pool readiness failed closed: {category:?}")]
pub struct ReadinessError {
    category: ReadinessCategory,
}

impl ReadinessError {
    const fn new(category: ReadinessCategory) -> Self {
        Self { category }
    }

    #[must_use]
    pub const fn category(self) -> ReadinessCategory {
        self.category
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct PoolReadinessReport {
    pub schema_version: String,
    pub attempt_ordinal: u8,
    pub source_commit: String,
    pub reference_commit: String,
    pub pool_config: String,
    pub protocol: String,
    pub samples_required: u8,
    pub samples_completed: u8,
    pub ready_samples: u8,
    pub consecutive_ready: bool,
    pub configure_succeeded: bool,
    pub subscribe_succeeded: bool,
    pub authorize_succeeded: bool,
    pub shares_submitted: bool,
    pub sample_timeout_seconds: u64,
    pub sample_delay_seconds: u64,
    pub max_server_bytes: usize,
    pub max_server_messages: usize,
    pub endpoint_redacted: bool,
    pub credentials_redacted: bool,
    pub bounded: bool,
    pub terminal_category: ReadinessCategory,
}

pub enum ReadinessDisposition {
    Ready(PoolReadinessReport),
    Unavailable(PoolReadinessReport),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PoolCredentialFile {
    #[serde(rename = "poolURL")]
    pool_url: String,
    #[serde(rename = "poolPort")]
    pool_port: u16,
    #[serde(rename = "poolUser")]
    pool_user: String,
    #[serde(rename = "poolPassword")]
    pool_password: String,
}

struct PoolCredentials {
    host: String,
    port: u16,
    username: String,
    password: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct SessionProgress {
    configure: bool,
    subscribe: bool,
    authorize: bool,
}

pub fn execute(options: ReadinessOptions) -> Result<ReadinessDisposition, ReadinessError> {
    validate_contract(&options)?;
    let workspace_root = workspace_root()?;
    let source_commit = source_identity(&workspace_root)?;
    let reference_commit = reference_identity(&workspace_root)?;
    let credential_path = admitted_path(&workspace_root, &options.pool_credentials)?;
    ensure_ignored(
        &workspace_root,
        &credential_path,
        ReadinessCategory::CredentialInvalid,
    )?;
    let credentials = read_credentials(&credential_path)?;
    let private_root = admitted_absent_root(&workspace_root, &options.private_root)?;
    ensure_ignored(
        &workspace_root,
        &private_root,
        ReadinessCategory::PrivateRootInvalid,
    )?;
    create_private_root(&private_root)?;

    let mut samples_completed = 0_u8;
    let mut ready_samples = 0_u8;
    let mut aggregate = SessionProgress::default();
    let mut terminal_category = ReadinessCategory::Ready;

    for sample_index in 0..options.samples {
        match probe_session(&credentials, options.sample_timeout) {
            Ok(progress) => {
                samples_completed = samples_completed.saturating_add(1);
                ready_samples = ready_samples.saturating_add(1);
                aggregate.configure |= progress.configure;
                aggregate.subscribe |= progress.subscribe;
                aggregate.authorize |= progress.authorize;
            }
            Err(error) => {
                samples_completed = samples_completed.saturating_add(1);
                terminal_category = error.category();
                break;
            }
        }
        if sample_index + 1 < options.samples {
            thread::sleep(options.sample_delay);
        }
    }

    let ready = terminal_category == ReadinessCategory::Ready
        && ready_samples == options.samples
        && aggregate.configure
        && aggregate.subscribe
        && aggregate.authorize;
    if !ready && terminal_category == ReadinessCategory::Ready {
        terminal_category = ReadinessCategory::ProtocolInvalid;
    }
    let report = PoolReadinessReport {
        schema_version: REPORT_SCHEMA.to_owned(),
        attempt_ordinal: options.attempt_ordinal,
        source_commit,
        reference_commit,
        pool_config: "local-owner-supplied".to_owned(),
        protocol: "stratum_v1_configure_subscribe_authorize".to_owned(),
        samples_required: options.samples,
        samples_completed,
        ready_samples,
        consecutive_ready: ready,
        configure_succeeded: ready && aggregate.configure,
        subscribe_succeeded: ready && aggregate.subscribe,
        authorize_succeeded: ready && aggregate.authorize,
        shares_submitted: false,
        sample_timeout_seconds: options.sample_timeout.as_secs(),
        sample_delay_seconds: options.sample_delay.as_secs(),
        max_server_bytes: MAX_SERVER_BYTES,
        max_server_messages: MAX_SERVER_MESSAGES,
        endpoint_redacted: true,
        credentials_redacted: true,
        bounded: true,
        terminal_category,
    };
    write_private_report(&private_root, &report)?;

    if ready {
        Ok(ReadinessDisposition::Ready(report))
    } else {
        Ok(ReadinessDisposition::Unavailable(report))
    }
}

fn validate_contract(options: &ReadinessOptions) -> Result<(), ReadinessError> {
    if options.attempt_ordinal != EXPECTED_ATTEMPT_ORDINAL
        || options.samples != EXPECTED_SAMPLES
        || options.sample_timeout != EXPECTED_SAMPLE_TIMEOUT
        || options.sample_delay != EXPECTED_SAMPLE_DELAY
    {
        return Err(ReadinessError::new(ReadinessCategory::ContractInvalid));
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf, ReadinessError> {
    let candidate = std::env::var_os("BUILD_WORKSPACE_DIRECTORY")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| ReadinessError::new(ReadinessCategory::SourceUnclean))?;
    candidate
        .canonicalize()
        .map_err(|_| ReadinessError::new(ReadinessCategory::SourceUnclean))
}

fn source_identity(workspace_root: &Path) -> Result<String, ReadinessError> {
    if !git_output(workspace_root, &["status", "--porcelain"])?.is_empty() {
        return Err(ReadinessError::new(ReadinessCategory::SourceUnclean));
    }
    let source = git_output(workspace_root, &["rev-parse", "HEAD"])?;
    if !is_commit(&source) {
        return Err(ReadinessError::new(ReadinessCategory::SourceUnclean));
    }
    Ok(source)
}

fn reference_identity(workspace_root: &Path) -> Result<String, ReadinessError> {
    let reference_root = workspace_root.join(REFERENCE_PATH);
    if !git_output(&reference_root, &["status", "--porcelain"])?.is_empty() {
        return Err(ReadinessError::new(ReadinessCategory::SourceUnclean));
    }
    let reference = git_output(&reference_root, &["rev-parse", "HEAD"])?;
    if !is_commit(&reference) {
        return Err(ReadinessError::new(ReadinessCategory::SourceUnclean));
    }
    Ok(reference)
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, ReadinessError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|_| ReadinessError::new(ReadinessCategory::SourceUnclean))?;
    if !output.status.success() {
        return Err(ReadinessError::new(ReadinessCategory::SourceUnclean));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|_| ReadinessError::new(ReadinessCategory::SourceUnclean))
}

fn admitted_path(workspace_root: &Path, relative: &Path) -> Result<PathBuf, ReadinessError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ReadinessError::new(ReadinessCategory::CredentialInvalid));
    }
    let candidate = workspace_root.join(relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|_| ReadinessError::new(ReadinessCategory::CredentialInvalid))?;
    if !canonical.starts_with(workspace_root) || !canonical.is_file() {
        return Err(ReadinessError::new(ReadinessCategory::CredentialInvalid));
    }
    Ok(canonical)
}

fn admitted_absent_root(workspace_root: &Path, relative: &Path) -> Result<PathBuf, ReadinessError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ReadinessError::new(ReadinessCategory::PrivateRootInvalid));
    }
    let candidate = workspace_root.join(relative);
    if candidate.exists() || !candidate.starts_with(workspace_root) {
        return Err(ReadinessError::new(ReadinessCategory::PrivateRootInvalid));
    }
    Ok(candidate)
}

fn ensure_ignored(
    workspace_root: &Path,
    candidate: &Path,
    category: ReadinessCategory,
) -> Result<(), ReadinessError> {
    let status = Command::new("git")
        .args(["check-ignore", "--quiet", "--"])
        .arg(candidate)
        .current_dir(workspace_root)
        .status()
        .map_err(|_| ReadinessError::new(category))?;
    if !status.success() {
        return Err(ReadinessError::new(category));
    }
    Ok(())
}

fn create_private_root(private_root: &Path) -> Result<(), ReadinessError> {
    fs::create_dir(private_root)
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))?;
    fs::set_permissions(private_root, fs::Permissions::from_mode(0o700))
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))
}

fn read_credentials(path: &Path) -> Result<PoolCredentials, ReadinessError> {
    let document = fs::read_to_string(path)
        .map_err(|_| ReadinessError::new(ReadinessCategory::CredentialInvalid))?;
    let file: PoolCredentialFile = serde_json::from_str(&document)
        .map_err(|_| ReadinessError::new(ReadinessCategory::CredentialInvalid))?;
    let host = normalized_host(&file.pool_url)?;
    if file.pool_port == 0 || file.pool_user.is_empty() || file.pool_user.len() > 255 {
        return Err(ReadinessError::new(ReadinessCategory::CredentialInvalid));
    }
    Ok(PoolCredentials {
        host,
        port: file.pool_port,
        username: file.pool_user,
        password: file.pool_password,
    })
}

fn normalized_host(value: &str) -> Result<String, ReadinessError> {
    let host = ["stratum+tcp://", "stratum://", "tcp://"]
        .iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(value);
    if host.is_empty()
        || host.len() > 253
        || host.chars().any(char::is_whitespace)
        || host.contains('/')
        || host.contains('\0')
    {
        return Err(ReadinessError::new(ReadinessCategory::CredentialInvalid));
    }
    Ok(host.to_owned())
}

fn probe_session(
    credentials: &PoolCredentials,
    timeout: Duration,
) -> Result<SessionProgress, ReadinessError> {
    let deadline = Instant::now() + timeout;
    let addresses = resolve_addresses(&credentials.host, credentials.port, timeout)?;
    let stream = connect(&addresses, deadline)?;
    stream
        .set_nodelay(true)
        .map_err(|_| ReadinessError::new(ReadinessCategory::TransportFailed))?;
    let mut writer = stream
        .try_clone()
        .map_err(|_| ReadinessError::new(ReadinessCategory::TransportFailed))?;
    let mut reader = BufReader::new(stream);

    write_client_message(
        &mut writer,
        &StratumV1ClientMessage::ConfigureVersionRolling {
            id: StratumRequestId::new(1),
            mask: 0x1fff_e000,
        },
        deadline,
    )?;
    write_client_message(
        &mut writer,
        &StratumV1ClientMessage::subscribe(StratumRequestId::new(2), "readiness", "1"),
        deadline,
    )?;

    let mut progress = SessionProgress::default();
    let mut authorize_sent = false;
    let mut total_bytes = 0_usize;
    let mut message_count = 0_usize;
    while !(progress.configure && progress.subscribe && progress.authorize) {
        let line = read_bounded_line(&mut reader, deadline, &mut total_bytes, &mut message_count)?;
        let text = std::str::from_utf8(&line)
            .map_err(|_| ReadinessError::new(ReadinessCategory::ProtocolInvalid))?;
        let message = parse_server_message(text.trim_end_matches(['\r', '\n']))
            .map_err(|_| ReadinessError::new(ReadinessCategory::ProtocolInvalid))?;
        match message {
            StratumV1ServerMessage::Response(response) => {
                let maybe_id = response.maybe_id.map(StratumRequestId::raw);
                match maybe_id {
                    Some(1) => {
                        if !response.success || response.maybe_version_mask.is_none() {
                            return Err(ReadinessError::new(ReadinessCategory::ConfigureRejected));
                        }
                        progress.configure = true;
                    }
                    Some(2) => {
                        if !response.success || response.maybe_extranonce.is_none() {
                            return Err(ReadinessError::new(ReadinessCategory::SubscribeRejected));
                        }
                        progress.subscribe = true;
                    }
                    Some(3) => {
                        if !response.success {
                            return Err(ReadinessError::new(ReadinessCategory::AuthorizeRejected));
                        }
                        progress.authorize = true;
                    }
                    _ => {}
                }
            }
            StratumV1ServerMessage::ClientReconnect => {
                return Err(ReadinessError::new(ReadinessCategory::TransportFailed));
            }
            StratumV1ServerMessage::Ping { maybe_id: Some(id) } => {
                write_client_message(&mut writer, &StratumV1ClientMessage::Pong { id }, deadline)?;
            }
            StratumV1ServerMessage::ClientGetVersion => {
                write_client_message(
                    &mut writer,
                    &StratumV1ClientMessage::SendVersion {
                        id: StratumRequestId::new(0),
                        version: "bitaxe-readiness/1".to_owned(),
                    },
                    deadline,
                )?;
            }
            _ => {}
        }
        if progress.subscribe && !authorize_sent {
            write_client_message(
                &mut writer,
                &StratumV1ClientMessage::authorize(
                    StratumRequestId::new(3),
                    &credentials.username,
                    &credentials.password,
                ),
                deadline,
            )?;
            authorize_sent = true;
        }
    }
    Ok(progress)
}

fn resolve_addresses(
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<Vec<SocketAddr>, ReadinessError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let owned_host = host.to_owned();
    thread::spawn(move || {
        let result = (owned_host.as_str(), port)
            .to_socket_addrs()
            .map(|addresses| addresses.collect::<Vec<_>>());
        let _ = sender.send(result);
    });
    match receiver.recv_timeout(timeout) {
        Ok(Ok(addresses)) if !addresses.is_empty() => Ok(addresses),
        Ok(_) => Err(ReadinessError::new(ReadinessCategory::ResolutionFailed)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            Err(ReadinessError::new(ReadinessCategory::Timeout))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(ReadinessError::new(ReadinessCategory::ResolutionFailed))
        }
    }
}

fn connect(addresses: &[SocketAddr], deadline: Instant) -> Result<TcpStream, ReadinessError> {
    for address in addresses {
        let remaining = remaining(deadline)?;
        if let Ok(stream) = TcpStream::connect_timeout(address, remaining) {
            return Ok(stream);
        }
    }
    Err(ReadinessError::new(ReadinessCategory::ConnectionFailed))
}

fn write_client_message(
    stream: &mut TcpStream,
    message: &StratumV1ClientMessage,
    deadline: Instant,
) -> Result<(), ReadinessError> {
    let line = message
        .to_json_line()
        .map_err(|_| ReadinessError::new(ReadinessCategory::ProtocolInvalid))?;
    stream
        .set_write_timeout(Some(remaining(deadline)?))
        .map_err(|_| ReadinessError::new(ReadinessCategory::TransportFailed))?;
    stream.write_all(line.as_bytes()).map_err(map_io_error)?;
    stream.flush().map_err(map_io_error)
}

fn read_bounded_line(
    reader: &mut BufReader<TcpStream>,
    deadline: Instant,
    total_bytes: &mut usize,
    message_count: &mut usize,
) -> Result<Vec<u8>, ReadinessError> {
    if *message_count >= MAX_SERVER_MESSAGES || *total_bytes >= MAX_SERVER_BYTES {
        return Err(ReadinessError::new(ReadinessCategory::InputLimitExceeded));
    }
    reader
        .get_mut()
        .set_read_timeout(Some(remaining(deadline)?))
        .map_err(|_| ReadinessError::new(ReadinessCategory::TransportFailed))?;
    let available = MAX_SERVER_BYTES - *total_bytes;
    let mut line = Vec::new();
    let count = reader
        .by_ref()
        .take((available + 1) as u64)
        .read_until(b'\n', &mut line)
        .map_err(map_io_error)?;
    if count == 0 {
        return Err(ReadinessError::new(ReadinessCategory::TransportFailed));
    }
    if count > available || !line.ends_with(b"\n") {
        return Err(ReadinessError::new(ReadinessCategory::InputLimitExceeded));
    }
    *total_bytes += count;
    *message_count += 1;
    Ok(line)
}

fn remaining(deadline: Instant) -> Result<Duration, ReadinessError> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| ReadinessError::new(ReadinessCategory::Timeout))
}

fn map_io_error(error: std::io::Error) -> ReadinessError {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            ReadinessError::new(ReadinessCategory::Timeout)
        }
        _ => ReadinessError::new(ReadinessCategory::TransportFailed),
    }
}

fn write_private_report(
    private_root: &Path,
    report: &PoolReadinessReport,
) -> Result<(), ReadinessError> {
    let report_path = private_root.join(REPORT_NAME);
    let temp_path = private_root.join(format!(".{REPORT_NAME}.tmp"));
    let document = serde_json::to_vec_pretty(report)
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp_path)
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))?;
    file.write_all(&document)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))?;
    fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))
        .and_then(|_| fs::rename(&temp_path, &report_path))
        .map_err(|_| ReadinessError::new(ReadinessCategory::PrivateRootInvalid))?;
    Ok(())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
