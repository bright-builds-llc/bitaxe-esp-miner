//! Closed storage and HTTP startup diagnostics, independent of platform error text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
/// Exact startup operation that failed.
pub enum StorageHttpPhase {
    SpiffsRegister = 1,
    SpiffsInfo,
    HttpNetif,
    HttpDeferredWorker,
    HttpServer,
    HttpRoutes,
    HttpTelemetryWorker,
}
impl StorageHttpPhase {
    /// Stable, nonsecret phase token.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SpiffsRegister => "spiffs_register",
            Self::SpiffsInfo => "spiffs_info",
            Self::HttpNetif => "http_netif",
            Self::HttpDeferredWorker => "http_deferred_worker",
            Self::HttpServer => "http_server",
            Self::HttpRoutes => "http_routes",
            Self::HttpTelemetryWorker => "http_telemetry_worker",
        }
    }
    /// Decodes only valid retained atomic phase values.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::SpiffsRegister),
            2 => Some(Self::SpiffsInfo),
            3 => Some(Self::HttpNetif),
            4 => Some(Self::HttpDeferredWorker),
            5 => Some(Self::HttpServer),
            6 => Some(Self::HttpRoutes),
            7 => Some(Self::HttpTelemetryWorker),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
/// Closed platform error category without arbitrary text.
pub enum StorageHttpError {
    NoMemory = 1,
    NotFound,
    InvalidState,
    InvalidArgument,
    MountFailed,
    HandlerExists,
    HandlersFull,
    HttpTask,
    SocketSetup,
    Timeout,
    IoError,
    Other,
}
impl StorageHttpError {
    /// Stable, nonsecret error token.
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoMemory => "no_memory",
            Self::NotFound => "not_found",
            Self::InvalidState => "invalid_state",
            Self::InvalidArgument => "invalid_argument",
            Self::MountFailed => "mount_failed",
            Self::HandlerExists => "handler_exists",
            Self::HandlersFull => "handlers_full",
            Self::HttpTask => "http_task",
            Self::SocketSetup => "socket_setup",
            Self::Timeout => "timeout",
            Self::IoError => "io_error",
            Self::Other => "other",
        }
    }
    /// Decodes only valid retained atomic error values.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::NoMemory),
            2 => Some(Self::NotFound),
            3 => Some(Self::InvalidState),
            4 => Some(Self::InvalidArgument),
            5 => Some(Self::MountFailed),
            6 => Some(Self::HandlerExists),
            7 => Some(Self::HandlersFull),
            8 => Some(Self::HttpTask),
            9 => Some(Self::SocketSetup),
            10 => Some(Self::Timeout),
            11 => Some(Self::IoError),
            12 => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// First storage/HTTP startup failure for one boot.
pub struct StorageHttpFailure {
    /// Operation boundary.
    pub phase: StorageHttpPhase,
    /// Typed error class.
    pub error: StorageHttpError,
}
impl StorageHttpFailure {
    /// Renders typed fields only; paths and arbitrary error messages are not accepted.
    pub fn marker(self) -> String {
        format!(
            "storage_http_failure schema=v1 phase={} error={} redacted=true",
            self.phase.label(),
            self.error.label()
        )
    }
    /// Parses one complete, closed record without establishing device authority.
    pub fn parse(line: &str) -> Option<Self> {
        let mut fields = line.split(' ');
        if fields.next()? != "storage_http_failure" || fields.next()? != "schema=v1" {
            return None;
        }
        let phase = fields.next()?.strip_prefix("phase=")?;
        let phase = (1..=7)
            .filter_map(StorageHttpPhase::from_code)
            .find(|item| item.label() == phase)?;
        let error = fields.next()?.strip_prefix("error=")?;
        let error = (1..=12)
            .filter_map(StorageHttpError::from_code)
            .find(|item| item.label() == error)?;
        (fields.next()? == "redacted=true" && fields.next().is_none())
            .then_some(Self { phase, error })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Completed outcomes of the filesystem and HTTP initialization attempts.
pub struct StorageHttpOutcome {
    /// Mount and filesystem-info acquisition both succeeded.
    pub spiffs_available: bool,
    /// Server, handlers, and required workers all initialized successfully.
    pub http_ready: bool,
}
impl StorageHttpOutcome {
    /// Both flags describe completed startup attempts, never pending work.
    pub fn marker(self) -> String {
        format!(
            "storage_http_status schema=v1 spiffs_available={} http_ready={} redacted=true",
            self.spiffs_available, self.http_ready
        )
    }
    /// Parses one complete outcome without admitting an application or its health.
    pub fn parse(line: &str) -> Option<Self> {
        let mut fields = line.split(' ');
        if fields.next()? != "storage_http_status" || fields.next()? != "schema=v1" {
            return None;
        }
        let spiffs_available = closed_bool(fields.next()?.strip_prefix("spiffs_available=")?)?;
        let http_ready = closed_bool(fields.next()?.strip_prefix("http_ready=")?)?;
        (fields.next()? == "redacted=true" && fields.next().is_none()).then_some(Self {
            spiffs_available,
            http_ready,
        })
    }
}
fn closed_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_typed_failure_round_trips_without_additional_fields() {
        for phase in (1..=7).filter_map(StorageHttpPhase::from_code) {
            for error in (1..=12).filter_map(StorageHttpError::from_code) {
                let record = StorageHttpFailure { phase, error };
                assert_eq!(StorageHttpFailure::parse(&record.marker()), Some(record));
                assert!(
                    StorageHttpFailure::parse(&format!("{} path=private", record.marker()))
                        .is_none()
                );
            }
        }
    }
    #[test]
    fn outcomes_distinguish_spiffs_failure_from_http_failure() {
        for spiffs_available in [false, true] {
            for http_ready in [false, true] {
                let record = StorageHttpOutcome {
                    spiffs_available,
                    http_ready,
                };
                assert_eq!(StorageHttpOutcome::parse(&record.marker()), Some(record));
            }
        }
        assert!(StorageHttpOutcome::parse(
            "storage_http_status schema=v1 spiffs_available=private http_ready=true redacted=true"
        )
        .is_none());
    }
}
