use std::num::NonZeroU64;

/// Supported origin schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

/// Terminal transport label derived from an [`ExchangeState`].
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

/// TLS verification label derived from the established transport boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVerification {
    NotApplicable,
    Failed,
    Verified,
}

/// One completed exchange represented by exactly one valid terminal state.
#[derive(Debug)]
pub struct ExchangeObservation {
    state: ExchangeState,
    total_millis: u64,
}

/// Closed set of terminal boundaries for an HTTP exchange.
#[derive(Debug)]
pub enum ExchangeState {
    TcpConnectionFailed {
        scheme: Scheme,
    },
    TlsHandshakeFailed {
        tcp_connect_millis: NonZeroU64,
    },
    RequestSendFailed {
        transport: EstablishedTransport,
        bytes_written: u64,
    },
    ResponseRead {
        transport: EstablishedTransport,
        request: CompletedRequest,
        response: ResponseRead,
    },
}

/// Successfully established plain or TLS transport with nonzero timings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstablishedTransport {
    Plain {
        tcp_connect_millis: NonZeroU64,
    },
    Tls {
        tcp_connect_millis: NonZeroU64,
        tls_handshake_millis: NonZeroU64,
    },
}

/// Request proven fully written and flushed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedRequest {
    bytes_written: NonZeroU64,
    completed_millis: NonZeroU64,
}

/// Read-only request projection across every terminal exchange state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestProgress {
    NotStarted,
    Incomplete {
        bytes_written: u64,
    },
    Complete {
        bytes_written: NonZeroU64,
        completed_millis: NonZeroU64,
    },
}

/// Result of reading a response after a completed request.
#[derive(Debug)]
pub struct ResponseRead {
    outcome: ResponseReadOutcome,
    maybe_first_byte_millis: Option<NonZeroU64>,
    maybe_http_response: Option<HttpResponse>,
}

/// Closed set of response-read outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseReadOutcome {
    Complete,
    Timeout,
    ReceiveFailed,
    OverLimit,
}

/// Parsed and status-validated HTTP/1.1 response.
#[derive(Debug)]
pub struct HttpResponse {
    status: u16,
    header_count: u64,
    headers: Vec<u8>,
    body: Vec<u8>,
}

impl ExchangeObservation {
    /// Returns the exact terminal state.
    pub fn state(&self) -> &ExchangeState {
        &self.state
    }

    /// Returns the origin scheme derived from the terminal state.
    pub fn scheme(&self) -> Scheme {
        match &self.state {
            ExchangeState::TcpConnectionFailed { scheme } => *scheme,
            ExchangeState::TlsHandshakeFailed { .. } => Scheme::Https,
            ExchangeState::RequestSendFailed { transport, .. }
            | ExchangeState::ResponseRead { transport, .. } => transport.scheme(),
        }
    }

    /// Returns the terminal transport label.
    pub fn transport_outcome(&self) -> TransportOutcome {
        match &self.state {
            ExchangeState::TcpConnectionFailed { .. } => TransportOutcome::TcpConnectionFailure,
            ExchangeState::TlsHandshakeFailed { .. } => TransportOutcome::TlsHandshakeFailure,
            ExchangeState::RequestSendFailed { .. } => TransportOutcome::RequestSendFailure,
            ExchangeState::ResponseRead { response, .. } => match response.outcome() {
                ResponseReadOutcome::Complete => TransportOutcome::Complete,
                ResponseReadOutcome::Timeout => TransportOutcome::ResponseTimeout,
                ResponseReadOutcome::ReceiveFailed => TransportOutcome::ReceiveFailed,
                ResponseReadOutcome::OverLimit => TransportOutcome::ResponseOverLimit,
            },
        }
    }

    /// Returns the TLS verification label implied by the reached boundary.
    pub fn tls_verification(&self) -> TlsVerification {
        match &self.state {
            ExchangeState::TcpConnectionFailed {
                scheme: Scheme::Http,
            }
            | ExchangeState::RequestSendFailed {
                transport: EstablishedTransport::Plain { .. },
                ..
            }
            | ExchangeState::ResponseRead {
                transport: EstablishedTransport::Plain { .. },
                ..
            } => TlsVerification::NotApplicable,
            ExchangeState::TcpConnectionFailed {
                scheme: Scheme::Https,
            }
            | ExchangeState::TlsHandshakeFailed { .. } => TlsVerification::Failed,
            ExchangeState::RequestSendFailed {
                transport: EstablishedTransport::Tls { .. },
                ..
            }
            | ExchangeState::ResponseRead {
                transport: EstablishedTransport::Tls { .. },
                ..
            } => TlsVerification::Verified,
        }
    }

    /// Returns the TCP timing only when a connection was established.
    pub fn maybe_tcp_connect_millis(&self) -> Option<NonZeroU64> {
        match &self.state {
            ExchangeState::TcpConnectionFailed { .. } => None,
            ExchangeState::TlsHandshakeFailed { tcp_connect_millis } => Some(*tcp_connect_millis),
            ExchangeState::RequestSendFailed { transport, .. }
            | ExchangeState::ResponseRead { transport, .. } => Some(transport.tcp_connect_millis()),
        }
    }

    /// Returns the TLS timing only when a verified TLS transport was established.
    pub fn maybe_tls_handshake_millis(&self) -> Option<NonZeroU64> {
        match &self.state {
            ExchangeState::RequestSendFailed { transport, .. }
            | ExchangeState::ResponseRead { transport, .. } => {
                transport.maybe_tls_handshake_millis()
            }
            ExchangeState::TcpConnectionFailed { .. }
            | ExchangeState::TlsHandshakeFailed { .. } => None,
        }
    }

    /// Returns typed request progress for the reached terminal boundary.
    pub fn request_progress(&self) -> RequestProgress {
        match &self.state {
            ExchangeState::TcpConnectionFailed { .. }
            | ExchangeState::TlsHandshakeFailed { .. } => RequestProgress::NotStarted,
            ExchangeState::RequestSendFailed { bytes_written, .. } => RequestProgress::Incomplete {
                bytes_written: *bytes_written,
            },
            ExchangeState::ResponseRead { request, .. } => request.progress(),
        }
    }

    /// Returns response-read details only after a completed request.
    pub fn maybe_response_read(&self) -> Option<&ResponseRead> {
        match &self.state {
            ExchangeState::ResponseRead { response, .. } => Some(response),
            ExchangeState::TcpConnectionFailed { .. }
            | ExchangeState::TlsHandshakeFailed { .. }
            | ExchangeState::RequestSendFailed { .. } => None,
        }
    }

    /// Returns a parsed response when the received bytes formed valid HTTP.
    pub fn maybe_http_response(&self) -> Option<&HttpResponse> {
        self.maybe_response_read()
            .and_then(ResponseRead::maybe_http_response)
    }

    /// Returns total elapsed time, including immediate failures that may record zero.
    pub fn total_millis(&self) -> u64 {
        self.total_millis
    }

    pub(crate) fn tcp_connection_failed(scheme: Scheme, total_millis: u64) -> Self {
        Self {
            state: ExchangeState::TcpConnectionFailed { scheme },
            total_millis,
        }
    }

    pub(crate) fn tls_handshake_failed(tcp_connect_millis: NonZeroU64, total_millis: u64) -> Self {
        Self {
            state: ExchangeState::TlsHandshakeFailed { tcp_connect_millis },
            total_millis,
        }
    }

    pub(crate) fn request_send_failed(
        transport: EstablishedTransport,
        bytes_written: u64,
        total_millis: u64,
    ) -> Self {
        Self {
            state: ExchangeState::RequestSendFailed {
                transport,
                bytes_written,
            },
            total_millis,
        }
    }

    pub(crate) fn response_read(
        transport: EstablishedTransport,
        request: CompletedRequest,
        response: ResponseRead,
        total_millis: u64,
    ) -> Self {
        Self {
            state: ExchangeState::ResponseRead {
                transport,
                request,
                response,
            },
            total_millis,
        }
    }
}

impl EstablishedTransport {
    /// Returns the scheme implied by this established transport.
    pub fn scheme(self) -> Scheme {
        match self {
            Self::Plain { .. } => Scheme::Http,
            Self::Tls { .. } => Scheme::Https,
        }
    }

    /// Returns the nonzero TCP connection timing.
    pub fn tcp_connect_millis(self) -> NonZeroU64 {
        match self {
            Self::Plain { tcp_connect_millis }
            | Self::Tls {
                tcp_connect_millis, ..
            } => tcp_connect_millis,
        }
    }

    /// Returns the nonzero TLS timing for TLS transports.
    pub fn maybe_tls_handshake_millis(self) -> Option<NonZeroU64> {
        match self {
            Self::Plain { .. } => None,
            Self::Tls {
                tls_handshake_millis,
                ..
            } => Some(tls_handshake_millis),
        }
    }

    pub(crate) fn plain(tcp_connect_millis: NonZeroU64) -> Self {
        Self::Plain { tcp_connect_millis }
    }

    pub(crate) fn tls(tcp_connect_millis: NonZeroU64, tls_handshake_millis: NonZeroU64) -> Self {
        Self::Tls {
            tcp_connect_millis,
            tls_handshake_millis,
        }
    }
}

impl CompletedRequest {
    /// Returns the nonzero number of flushed request bytes.
    pub fn bytes_written(self) -> NonZeroU64 {
        self.bytes_written
    }

    /// Returns the nonzero elapsed time when request flushing completed.
    pub fn completed_millis(self) -> NonZeroU64 {
        self.completed_millis
    }

    pub(crate) fn new(bytes_written: NonZeroU64, completed_millis: NonZeroU64) -> Self {
        Self {
            bytes_written,
            completed_millis,
        }
    }

    fn progress(self) -> RequestProgress {
        RequestProgress::Complete {
            bytes_written: self.bytes_written,
            completed_millis: self.completed_millis,
        }
    }
}

impl RequestProgress {
    /// Returns bytes written, using zero only when no request started.
    pub fn bytes_written(self) -> u64 {
        match self {
            Self::NotStarted => 0,
            Self::Incomplete { bytes_written } => bytes_written,
            Self::Complete { bytes_written, .. } => bytes_written.get(),
        }
    }

    /// Returns whether the complete request was flushed.
    pub fn is_complete(self) -> bool {
        matches!(self, Self::Complete { .. })
    }

    /// Returns completion timing only for a fully flushed request.
    pub fn maybe_completed_millis(self) -> Option<NonZeroU64> {
        match self {
            Self::Complete {
                completed_millis, ..
            } => Some(completed_millis),
            Self::NotStarted | Self::Incomplete { .. } => None,
        }
    }
}

impl ResponseRead {
    /// Returns the terminal response-read outcome.
    pub fn outcome(&self) -> ResponseReadOutcome {
        self.outcome
    }

    /// Returns first-byte timing only when response bytes arrived.
    pub fn maybe_first_byte_millis(&self) -> Option<NonZeroU64> {
        self.maybe_first_byte_millis
    }

    /// Returns a parsed response when the received bytes formed valid HTTP.
    pub fn maybe_http_response(&self) -> Option<&HttpResponse> {
        self.maybe_http_response.as_ref()
    }

    pub(crate) fn new(
        outcome: ResponseReadOutcome,
        maybe_first_byte_millis: Option<NonZeroU64>,
        maybe_http_response: Option<HttpResponse>,
    ) -> Self {
        Self {
            outcome,
            maybe_first_byte_millis,
            maybe_http_response,
        }
    }
}

impl HttpResponse {
    /// Returns the validated HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Returns the number of parsed header fields.
    pub fn header_count(&self) -> u64 {
        self.header_count
    }

    /// Returns the retained raw response header bytes.
    pub fn headers(&self) -> &[u8] {
        &self.headers
    }

    /// Returns the retained or decoded response body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub(crate) fn maybe_new(
        status: u16,
        header_count: u64,
        headers: Vec<u8>,
        body: Vec<u8>,
    ) -> Option<Self> {
        (100..=599).contains(&status).then_some(Self {
            status,
            header_count,
            headers,
            body,
        })
    }
}
