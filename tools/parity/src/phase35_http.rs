use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub(crate) const PHASE35_HTTP_SCHEMA: &str = "phase35-http-boundary-v2";

const MAX_TCP_CONNECT_MILLIS: u64 = 5_000;
const REQUEST_TIMEOUT_MILLIS: u64 = 10_000;
const REQUEST_TIMEOUT_OBSERVATION_GRACE_MILLIS: u64 = 1_000;
const MAX_OBSERVED_TOTAL_MILLIS: u64 =
    REQUEST_TIMEOUT_MILLIS + REQUEST_TIMEOUT_OBSERVATION_GRACE_MILLIS;
const MAX_REQUEST_BYTES: u64 = 65_536;
const MAX_RESPONSE_HEADER_COUNT: u64 = 1_024;
const MAX_RESPONSE_HEADER_BYTES: u64 = 65_536;
const MAX_RESPONSE_BODY_BYTES: u64 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SchemeCategory {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TlsVerification {
    NotApplicable,
    Failed,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundedBytes(u64);

impl BoundedBytes {
    fn parse(
        value: u64,
        maximum: u64,
        field: &'static str,
    ) -> Result<Self, Phase35HttpDiagnosticError> {
        if value > maximum {
            return Err(Phase35HttpDiagnosticError::OutOfBounds(field));
        }
        Ok(Self(value))
    }

    const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundedMillis(u64);

impl BoundedMillis {
    fn parse(
        value: u64,
        maximum: u64,
        field: &'static str,
    ) -> Result<Self, Phase35HttpDiagnosticError> {
        if value > maximum {
            return Err(Phase35HttpDiagnosticError::OutOfBounds(field));
        }
        Ok(Self(value))
    }

    const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TransportOutcome {
    Complete,
    TcpConnectionFailure,
    TlsHandshakeFailure,
    RequestSendFailure,
    ResponseTimeout,
    ReceiveFailed,
    ResponseOverLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResponseStatus(u16);

impl ResponseStatus {
    fn parse(value: u16) -> Result<Self, Phase35HttpDiagnosticError> {
        if value != 0 && !(100..=599).contains(&value) {
            return Err(Phase35HttpDiagnosticError::OutOfBounds("response_status"));
        }
        Ok(Self(value))
    }

    const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ResponseStatusClass {
    Missing,
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
}

impl ResponseStatusClass {
    const fn from_status(status: ResponseStatus) -> Self {
        match status.get() {
            0 => Self::Missing,
            100..=199 => Self::Informational,
            200..=299 => Self::Success,
            300..=399 => Self::Redirection,
            400..=499 => Self::ClientError,
            _ => Self::ServerError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HttpObservation {
    scheme: SchemeCategory,
    transport_outcome: TransportOutcome,
    tcp_connect_millis: BoundedMillis,
    tls_handshake_millis: BoundedMillis,
    request_send_complete_millis: BoundedMillis,
    request_bytes: BoundedBytes,
    response_status: ResponseStatus,
    response_header_count: BoundedBytes,
    response_header_bytes: BoundedBytes,
    response_body_bytes: BoundedBytes,
    total_millis: BoundedMillis,
    first_byte_millis: BoundedMillis,
    tls_verification: TlsVerification,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawHttpMetrics {
    pub(crate) scheme_category: SchemeCategory,
    pub(crate) transport_outcome: TransportOutcome,
    pub(crate) tcp_connect_millis: u64,
    pub(crate) tls_handshake_millis: u64,
    pub(crate) request_send_complete_millis: u64,
    pub(crate) request_bytes: u64,
    pub(crate) response_status: u16,
    pub(crate) response_header_count: u64,
    pub(crate) response_header_bytes: u64,
    pub(crate) response_body_bytes: u64,
    pub(crate) total_millis: u64,
    pub(crate) first_byte_millis: u64,
    pub(crate) tls_verification: TlsVerification,
}

impl RawHttpMetrics {
    pub(crate) const fn empty(
        scheme_category: SchemeCategory,
        transport_outcome: TransportOutcome,
        total_millis: u64,
    ) -> Self {
        Self {
            scheme_category,
            transport_outcome,
            tcp_connect_millis: 0,
            tls_handshake_millis: 0,
            request_send_complete_millis: 0,
            request_bytes: 0,
            response_status: 0,
            response_header_count: 0,
            response_header_bytes: 0,
            response_body_bytes: 0,
            total_millis,
            first_byte_millis: 0,
            tls_verification: match scheme_category {
                SchemeCategory::Http => TlsVerification::NotApplicable,
                SchemeCategory::Https => TlsVerification::Failed,
            },
        }
    }
}

impl HttpObservation {
    fn parse(metrics_json: &[u8], body: &[u8]) -> Result<Self, Phase35HttpDiagnosticError> {
        let raw: RawHttpMetrics = serde_json::from_slice(metrics_json)
            .map_err(|_| Phase35HttpDiagnosticError::MalformedMetrics)?;
        let observation = Self {
            scheme: raw.scheme_category,
            transport_outcome: raw.transport_outcome,
            tcp_connect_millis: BoundedMillis::parse(
                raw.tcp_connect_millis,
                MAX_TCP_CONNECT_MILLIS,
                "tcp_connect_millis",
            )?,
            tls_handshake_millis: BoundedMillis::parse(
                raw.tls_handshake_millis,
                MAX_OBSERVED_TOTAL_MILLIS,
                "tls_handshake_millis",
            )?,
            request_send_complete_millis: BoundedMillis::parse(
                raw.request_send_complete_millis,
                MAX_OBSERVED_TOTAL_MILLIS,
                "request_send_complete_millis",
            )?,
            request_bytes: BoundedBytes::parse(
                raw.request_bytes,
                MAX_REQUEST_BYTES,
                "request_bytes",
            )?,
            response_status: ResponseStatus::parse(raw.response_status)?,
            response_header_count: BoundedBytes::parse(
                raw.response_header_count,
                MAX_RESPONSE_HEADER_COUNT,
                "response_header_count",
            )?,
            response_header_bytes: BoundedBytes::parse(
                raw.response_header_bytes,
                MAX_RESPONSE_HEADER_BYTES,
                "response_header_bytes",
            )?,
            response_body_bytes: BoundedBytes::parse(
                raw.response_body_bytes,
                MAX_RESPONSE_BODY_BYTES,
                "response_body_bytes",
            )?,
            total_millis: BoundedMillis::parse(
                raw.total_millis,
                MAX_OBSERVED_TOTAL_MILLIS,
                "total_millis",
            )?,
            first_byte_millis: BoundedMillis::parse(
                raw.first_byte_millis,
                MAX_OBSERVED_TOTAL_MILLIS,
                "first_byte_millis",
            )?,
            tls_verification: raw.tls_verification,
        };
        observation.validate(body)?;
        Ok(observation)
    }

    fn validate(self, body: &[u8]) -> Result<(), Phase35HttpDiagnosticError> {
        let body_length = u64::try_from(body.len())
            .map_err(|_| Phase35HttpDiagnosticError::OutOfBounds("response_body_bytes"))?;
        if body_length != self.response_body_bytes.get() {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "response_body_bytes",
            ));
        }
        if self.first_byte_millis.get() > self.total_millis.get()
            || self.tcp_connect_millis.get() > self.total_millis.get()
            || self.tls_handshake_millis.get() > self.total_millis.get()
            || self.request_send_complete_millis.get() > self.total_millis.get()
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent("timing_order"));
        }

        match (self.scheme, self.tls_verification) {
            (SchemeCategory::Http, TlsVerification::NotApplicable)
                if self.tls_handshake_millis.get() == 0 => {}
            (SchemeCategory::Https, TlsVerification::Failed)
                if self.tls_handshake_millis.get() == 0 => {}
            (SchemeCategory::Https, TlsVerification::Verified)
                if self.tls_handshake_millis.get() > 0 => {}
            _ => return Err(Phase35HttpDiagnosticError::Inconsistent("tls_state")),
        }

        let header_count_present = self.response_header_count.get() > 0;
        let header_bytes_present = self.response_header_bytes.get() > 0;
        if header_count_present != header_bytes_present {
            return Err(Phase35HttpDiagnosticError::Inconsistent("response_headers"));
        }

        let tcp_connected = self.tcp_connect_millis.get() > 0;
        match self.transport_outcome {
            TransportOutcome::TcpConnectionFailure if !tcp_connected => {}
            TransportOutcome::TlsHandshakeFailure
                if self.scheme == SchemeCategory::Https
                    && tcp_connected
                    && self.tls_handshake_millis.get() == 0 => {}
            TransportOutcome::RequestSendFailure if tcp_connected => {}
            TransportOutcome::Complete
            | TransportOutcome::ResponseTimeout
            | TransportOutcome::ReceiveFailed
            | TransportOutcome::ResponseOverLimit => {}
            _ => {
                return Err(Phase35HttpDiagnosticError::Inconsistent(
                    "transport_outcome",
                ));
            }
        }
        if !tcp_connected
            && (self.request_bytes.get() > 0
                || self.request_send_complete_millis.get() > 0
                || self.response_status.get() > 0
                || header_count_present
                || self.response_body_bytes.get() > 0
                || self.first_byte_millis.get() > 0)
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent("tcp_connection"));
        }
        if self.request_send_complete_millis.get() == 0
            && !matches!(
                self.transport_outcome,
                TransportOutcome::TcpConnectionFailure
                    | TransportOutcome::TlsHandshakeFailure
                    | TransportOutcome::RequestSendFailure
            )
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "request_completion_missing",
            ));
        }
        if self.request_send_complete_millis.get() > 0
            && matches!(
                self.transport_outcome,
                TransportOutcome::TcpConnectionFailure
                    | TransportOutcome::TlsHandshakeFailure
                    | TransportOutcome::RequestSendFailure
            )
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "request_completion_outcome",
            ));
        }
        if self.request_send_complete_millis.get() > 0 && self.request_bytes.get() == 0 {
            return Err(Phase35HttpDiagnosticError::Inconsistent("request_bytes"));
        }
        if !self.request_transmission_complete()
            && (self.response_status.get() > 0
                || header_count_present
                || self.response_body_bytes.get() > 0
                || self.first_byte_millis.get() > 0)
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "request_transmission",
            ));
        }
        if self.response_status.get() == 0
            && (header_count_present
                || self.response_body_bytes.get() > 0
                || self.first_byte_millis.get() > 0)
        {
            return Err(Phase35HttpDiagnosticError::Inconsistent("response_status"));
        }
        if !header_count_present && self.response_body_bytes.get() > 0 {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "response_body_without_headers",
            ));
        }
        if self.response_body_bytes.get() > 0 && self.first_byte_millis.get() == 0 {
            return Err(Phase35HttpDiagnosticError::Inconsistent(
                "response_body_without_first_byte",
            ));
        }
        Ok(())
    }

    const fn request_transmission_complete(self) -> bool {
        self.request_send_complete_millis.get() > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HttpTerminalCategory {
    TcpConnectionFailure,
    TlsHandshakeFailure,
    RequestTransmissionIncomplete,
    ResponseStatusMissing,
    ResponseHeadersMissing,
    NonSuccessResponseStatus,
    ResponseBodyMissing,
    ResponseBodyIncompleteOrOverLimit,
    InvalidJson,
    InvalidHostnameSchema,
    Ready,
}

impl HttpTerminalCategory {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TcpConnectionFailure => "tcp_connection_failure",
            Self::TlsHandshakeFailure => "tls_handshake_failure",
            Self::RequestTransmissionIncomplete => "request_transmission_incomplete",
            Self::ResponseStatusMissing => "response_status_missing",
            Self::ResponseHeadersMissing => "response_headers_missing",
            Self::NonSuccessResponseStatus => "non_success_response_status",
            Self::ResponseBodyMissing => "response_body_missing",
            Self::ResponseBodyIncompleteOrOverLimit => "response_body_incomplete_or_over_limit",
            Self::InvalidJson => "invalid_json",
            Self::InvalidHostnameSchema => "invalid_hostname_schema",
            Self::Ready => "ready",
        }
    }
}

impl Serialize for HttpTerminalCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase35HttpProjection {
    pub(crate) schema_version: &'static str,
    pub(crate) tcp_connected: bool,
    pub(crate) tls_applicable: bool,
    pub(crate) tls_established: bool,
    pub(crate) tls_verified: bool,
    pub(crate) request_transmission_complete: bool,
    pub(crate) response_status_received: bool,
    pub(crate) response_headers_received: bool,
    pub(crate) response_body_received: bool,
    pub(crate) response_body_complete: bool,
    pub(crate) json_parsed: bool,
    pub(crate) hostname_schema_valid: bool,
    pub(crate) transport_outcome: TransportOutcome,
    pub(crate) request_send_complete_millis: u64,
    pub(crate) request_bytes: u64,
    pub(crate) response_header_count: u64,
    pub(crate) response_header_bytes: u64,
    pub(crate) response_body_bytes: u64,
    pub(crate) tcp_connect_millis: u64,
    pub(crate) tls_handshake_millis: u64,
    pub(crate) first_byte_millis: u64,
    pub(crate) total_millis: u64,
    response_status_class: ResponseStatusClass,
    pub(crate) terminal_category: HttpTerminalCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassifiedPhase35Http {
    pub(crate) terminal_category: HttpTerminalCategory,
    pub(crate) projection: Phase35HttpProjection,
    pub(crate) maybe_hostname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum Phase35HttpDiagnosticError {
    #[error("HTTP diagnostic metrics are malformed")]
    MalformedMetrics,
    #[error("HTTP diagnostic field is out of bounds: {0}")]
    OutOfBounds(&'static str),
    #[error("HTTP diagnostic facts are inconsistent: {0}")]
    Inconsistent(&'static str),
}

pub(crate) fn classify_phase35_http(
    metrics_json: &[u8],
    body: &[u8],
) -> Result<ClassifiedPhase35Http, Phase35HttpDiagnosticError> {
    let observation = HttpObservation::parse(metrics_json, body)?;
    let maybe_json = serde_json::from_slice::<Value>(body).ok();
    let maybe_hostname = maybe_json
        .as_ref()
        .and_then(|document| document.get("hostname"))
        .and_then(Value::as_str)
        .filter(|hostname| valid_hostname(hostname))
        .map(str::to_owned);
    let terminal_category =
        classify_terminal(observation, maybe_json.is_some(), maybe_hostname.is_some());
    let json_parsed = matches!(
        terminal_category,
        HttpTerminalCategory::InvalidHostnameSchema | HttpTerminalCategory::Ready
    );
    let hostname_schema_valid = terminal_category == HttpTerminalCategory::Ready;
    let projection = projection(
        observation,
        terminal_category,
        json_parsed,
        hostname_schema_valid,
    );

    Ok(ClassifiedPhase35Http {
        terminal_category,
        projection,
        maybe_hostname: if hostname_schema_valid {
            maybe_hostname
        } else {
            None
        },
    })
}

fn classify_terminal(
    observation: HttpObservation,
    json_parsed: bool,
    hostname_schema_valid: bool,
) -> HttpTerminalCategory {
    if observation.tcp_connect_millis.get() == 0 {
        return HttpTerminalCategory::TcpConnectionFailure;
    }
    if observation.scheme == SchemeCategory::Https && observation.tls_handshake_millis.get() == 0 {
        return HttpTerminalCategory::TlsHandshakeFailure;
    }
    if !observation.request_transmission_complete() {
        return HttpTerminalCategory::RequestTransmissionIncomplete;
    }
    if observation.response_status.get() == 0 {
        return HttpTerminalCategory::ResponseStatusMissing;
    }
    if observation.response_header_count.get() == 0 {
        return HttpTerminalCategory::ResponseHeadersMissing;
    }
    if !matches!(observation.response_status.get(), 200..=299) {
        return HttpTerminalCategory::NonSuccessResponseStatus;
    }
    if observation.response_body_bytes.get() == 0 {
        return HttpTerminalCategory::ResponseBodyMissing;
    }
    if observation.transport_outcome != TransportOutcome::Complete {
        return HttpTerminalCategory::ResponseBodyIncompleteOrOverLimit;
    }
    if !json_parsed {
        return HttpTerminalCategory::InvalidJson;
    }
    if !hostname_schema_valid {
        return HttpTerminalCategory::InvalidHostnameSchema;
    }
    HttpTerminalCategory::Ready
}

fn projection(
    observation: HttpObservation,
    terminal_category: HttpTerminalCategory,
    json_parsed: bool,
    hostname_schema_valid: bool,
) -> Phase35HttpProjection {
    let tls_applicable = observation.scheme == SchemeCategory::Https;
    let response_body_received = observation.response_body_bytes.get() > 0;
    Phase35HttpProjection {
        schema_version: PHASE35_HTTP_SCHEMA,
        tcp_connected: observation.tcp_connect_millis.get() > 0,
        tls_applicable,
        tls_established: tls_applicable && observation.tls_handshake_millis.get() > 0,
        tls_verified: tls_applicable && observation.tls_verification == TlsVerification::Verified,
        request_transmission_complete: observation.request_transmission_complete(),
        response_status_received: observation.response_status.get() > 0,
        response_headers_received: observation.response_header_count.get() > 0,
        response_body_received,
        response_body_complete: response_body_received
            && observation.transport_outcome == TransportOutcome::Complete,
        json_parsed,
        hostname_schema_valid,
        transport_outcome: observation.transport_outcome,
        request_send_complete_millis: observation.request_send_complete_millis.get(),
        request_bytes: observation.request_bytes.get(),
        response_header_count: observation.response_header_count.get(),
        response_header_bytes: observation.response_header_bytes.get(),
        response_body_bytes: observation.response_body_bytes.get(),
        tcp_connect_millis: observation.tcp_connect_millis.get(),
        tls_handshake_millis: observation.tls_handshake_millis.get(),
        first_byte_millis: observation.first_byte_millis.get(),
        total_millis: observation.total_millis.get(),
        response_status_class: ResponseStatusClass::from_status(observation.response_status),
        terminal_category,
    }
}

fn valid_hostname(hostname: &str) -> bool {
    if hostname.is_empty() || hostname.len() > 253 || !hostname.is_ascii() {
        return false;
    }
    hostname.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

#[cfg(test)]
mod tests;
