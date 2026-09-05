//! Closed boot diagnostic allowlist for the single Serial/JTAG writer.
mod storage_http;
pub use storage_http::{
    StorageHttpError, StorageHttpFailure, StorageHttpOutcome, StorageHttpPhase,
};

/// Selects only exact closed memory/startup fields, never arbitrary retained log text.
#[must_use]
pub fn is_worker_diagnostic_retained_line(line: &str) -> bool {
    let mut fields = line.split(' ');
    match fields.next() {
        Some("usb_memory_checkpoint") => {
            matches!(
                fields.next(),
                Some(
                    "stage=worker_owner_prepare"
                        | "stage=usb_install"
                        | "stage=usb_installed"
                        | "stage=statistics_start"
                        | "stage=statistics_started"
                        | "stage=wifi_driver_prepare"
                        | "stage=wifi_driver_prepared"
                )
            ) && fields
                .next()
                .is_some_and(|field| decimal_field(field, "free_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "largest_block_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "reserve_bytes="))
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        Some("storage_http_failure") => StorageHttpFailure::parse(line).is_some(),
        Some("storage_http_status") => StorageHttpOutcome::parse(line).is_some(),
        Some("wifi_startup_failure") => valid_network_startup_failure(fields),
        Some("bwg_worker_start_failure") => {
            fields.next() == Some("category=startup_failed")
                && matches!(
                    fields.next(),
                    Some("detail=owner_spawn" | "detail=usb_install" | "detail=control_owner")
                )
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        _ => false,
    }
}

/// Closed Wi-Fi startup phases; none contain network configuration values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkStartupPhase {
    Netif = 1,
    EventLoop,
    Driver,
    ApConfiguration,
    StationConfiguration,
    DriverStart,
    ApNetif,
    StationNetif,
    CaptiveDns,
    OwnerInstall,
    ReconnectSubscription,
    ReconnectSpawn,
}
impl NetworkStartupPhase {
    /// Stable public diagnostic token.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Netif => "netif",
            Self::EventLoop => "event_loop",
            Self::Driver => "driver",
            Self::ApConfiguration => "ap_configuration",
            Self::StationConfiguration => "station_configuration",
            Self::DriverStart => "driver_start",
            Self::ApNetif => "ap_netif",
            Self::StationNetif => "station_netif",
            Self::CaptiveDns => "captive_dns",
            Self::OwnerInstall => "owner_install",
            Self::ReconnectSubscription => "reconnect_subscription",
            Self::ReconnectSpawn => "reconnect_spawn",
        }
    }
    /// Recovers only an existing closed phase from an atomic diagnostic record.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Netif),
            2 => Some(Self::EventLoop),
            3 => Some(Self::Driver),
            4 => Some(Self::ApConfiguration),
            5 => Some(Self::StationConfiguration),
            6 => Some(Self::DriverStart),
            7 => Some(Self::ApNetif),
            8 => Some(Self::StationNetif),
            9 => Some(Self::CaptiveDns),
            10 => Some(Self::OwnerInstall),
            11 => Some(Self::ReconnectSubscription),
            12 => Some(Self::ReconnectSpawn),
            _ => None,
        }
    }
}

/// Typed error classes exposed independently of arbitrary error messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkStartupError {
    NoMemory = 1,
    InvalidState,
    Timeout,
    DriverError,
    IoError,
    OwnerError,
}
impl NetworkStartupError {
    /// Stable public diagnostic token.
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoMemory => "no_memory",
            Self::InvalidState => "invalid_state",
            Self::Timeout => "timeout",
            Self::DriverError => "driver_error",
            Self::IoError => "io_error",
            Self::OwnerError => "owner_error",
        }
    }
    /// Recovers only a closed error from an atomic diagnostic record.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::NoMemory),
            2 => Some(Self::InvalidState),
            3 => Some(Self::Timeout),
            4 => Some(Self::DriverError),
            5 => Some(Self::IoError),
            6 => Some(Self::OwnerError),
            _ => None,
        }
    }
}

/// Emits only closed nonsecret fields from a typed first startup failure.
pub fn network_startup_failure_marker(
    phase: NetworkStartupPhase,
    error: NetworkStartupError,
) -> String {
    format!(
        "wifi_startup_failure schema=v1 phase={} error={} redacted=true",
        phase.label(),
        error.label()
    )
}

fn valid_network_startup_failure(mut fields: std::str::Split<'_, char>) -> bool {
    fields.next() == Some("schema=v1")
        && fields.next().is_some_and(|field| {
            (1..=12)
                .filter_map(NetworkStartupPhase::from_code)
                .any(|phase| field.strip_prefix("phase=") == Some(phase.label()))
        })
        && fields.next().is_some_and(|field| {
            (1..=6)
                .filter_map(NetworkStartupError::from_code)
                .any(|error| field.strip_prefix("error=") == Some(error.label()))
        })
        && fields.next() == Some("redacted=true")
        && fields.next().is_none()
}

fn decimal_field(field: &str, prefix: &str) -> bool {
    field.strip_prefix(prefix).is_some_and(|value| {
        !value.is_empty() && value.len() <= 10 && value.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extra_or_sensitive_fields_never_become_diagnostics() {
        let valid = "usb_memory_checkpoint stage=usb_install free_bytes=100 largest_block_bytes=90 reserve_bytes=98304 redacted=true";
        assert!(is_worker_diagnostic_retained_line(valid));
        assert!(!is_worker_diagnostic_retained_line(&format!(
            "{valid} secret=value"
        )));
        assert!(!is_worker_diagnostic_retained_line(
            "bwg_worker_start_failure category=startup_failed detail=private-url redacted=true"
        ));
    }
    #[test]
    fn network_failures_accept_only_closed_phase_error_pairs() {
        // Arrange / Act / Assert
        for phase in (1..=12).filter_map(NetworkStartupPhase::from_code) {
            for error in (1..=6).filter_map(NetworkStartupError::from_code) {
                let marker = network_startup_failure_marker(phase, error);
                assert!(is_worker_diagnostic_retained_line(&marker));
                assert!(!is_worker_diagnostic_retained_line(&format!(
                    "{marker} private=value"
                )));
            }
        }
        for invalid in [
            "wifi_startup_failure schema=v1 phase=private error=no_memory redacted=true",
            "wifi_startup_failure schema=v1 phase=driver error=private redacted=true",
            "wifi_startup_failure schema=v2 phase=driver error=no_memory redacted=true",
            "wifi_startup_failure schema=v1 phase=driver error=no_memory redacted=false",
        ] {
            assert!(!is_worker_diagnostic_retained_line(invalid));
        }
    }
    #[test]
    fn wifi_constructor_checkpoints_accept_only_the_existing_numeric_shape() {
        // Arrange / Act / Assert
        for stage in ["wifi_driver_prepare", "wifi_driver_prepared"] {
            let line = format!("usb_memory_checkpoint stage={stage} free_bytes=100000 largest_block_bytes=64000 reserve_bytes=98304 redacted=true");
            assert!(is_worker_diagnostic_retained_line(&line));
            assert!(!is_worker_diagnostic_retained_line(&format!(
                "{line} private=value"
            )));
        }
    }
}
