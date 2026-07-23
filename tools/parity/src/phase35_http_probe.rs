use std::time::Duration;

use bitaxe_http_transport::{
    Scheme, StrictHttpClient, TlsVerification as SharedTlsVerification,
    TransportOutcome as SharedTransportOutcome,
};
use thiserror::Error;

use crate::phase35_http::{RawHttpMetrics, SchemeCategory, TlsVerification, TransportOutcome};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(10);

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
    #[error("Phase 35 HTTP probe transport setup failed")]
    TransportSetupFailed,
}

pub(crate) fn probe_phase35_http(url: &str) -> Result<ProbeResult, Phase35HttpProbeError> {
    probe_phase35_http_with_timeouts(url, CONNECT_TIMEOUT, TOTAL_TIMEOUT)
}

fn probe_phase35_http_with_timeouts(
    url: &str,
    connect_timeout: Duration,
    total_timeout: Duration,
) -> Result<ProbeResult, Phase35HttpProbeError> {
    let client = StrictHttpClient::new(url).map_err(|_| Phase35HttpProbeError::InvalidOrigin)?;
    let observation = client
        .exchange_with_timeouts("GET", "/api/system/info", connect_timeout, total_timeout)
        .map_err(|_| Phase35HttpProbeError::TransportSetupFailed)?;
    let scheme_category = match observation.scheme {
        Scheme::Http => SchemeCategory::Http,
        Scheme::Https => SchemeCategory::Https,
    };
    let transport_outcome = match observation.transport_outcome {
        SharedTransportOutcome::Complete => TransportOutcome::Complete,
        SharedTransportOutcome::TcpConnectionFailure => TransportOutcome::TcpConnectionFailure,
        SharedTransportOutcome::TlsHandshakeFailure => TransportOutcome::TlsHandshakeFailure,
        SharedTransportOutcome::RequestSendFailure => TransportOutcome::RequestSendFailure,
        SharedTransportOutcome::ResponseTimeout => TransportOutcome::ResponseTimeout,
        SharedTransportOutcome::ReceiveFailed => TransportOutcome::ReceiveFailed,
        SharedTransportOutcome::ResponseOverLimit => TransportOutcome::ResponseOverLimit,
    };
    let tls_verification = match observation.tls_verification {
        SharedTlsVerification::NotApplicable => TlsVerification::NotApplicable,
        SharedTlsVerification::Failed => TlsVerification::Failed,
        SharedTlsVerification::Verified => TlsVerification::Verified,
    };
    let response_header_bytes = u64::try_from(observation.headers.len()).unwrap_or(u64::MAX);
    let response_body_bytes = u64::try_from(observation.body.len()).unwrap_or(u64::MAX);
    Ok(ProbeResult {
        metrics: RawHttpMetrics {
            scheme_category,
            transport_outcome,
            tcp_connect_millis: observation.tcp_connect_millis,
            tls_handshake_millis: observation.tls_handshake_millis,
            request_send_complete_millis: observation.request_send_complete_millis,
            request_bytes: observation.request_bytes_written,
            response_status: observation.response_status,
            response_header_count: observation.response_header_count,
            response_header_bytes,
            response_body_bytes,
            total_millis: observation.total_millis,
            first_byte_millis: observation.first_byte_millis,
            tls_verification,
        },
        headers: observation.headers,
        body: observation.body,
    })
}

#[cfg(test)]
mod tests;
