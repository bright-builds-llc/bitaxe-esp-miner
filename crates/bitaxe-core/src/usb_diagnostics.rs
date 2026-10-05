//! Closed boot diagnostic allowlist for the single Serial/JTAG writer.
mod internal_heap;
mod storage_http;
pub use internal_heap::{
    internal_heap_sample_marker, InternalHeapSample, INTERNAL_HEAP_SAMPLE_INTERVAL_MS,
};
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
        Some("core_dump_store_receipt") => valid_core_dump_store(fields),
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

fn valid_core_dump_store(mut fields: std::str::Split<'_, char>) -> bool {
    if fields.next() != Some("schema=v1")
        || !matches!(
            fields.next(),
            Some("origin=current_boot" | "origin=previous_boot")
        )
    {
        return false;
    }
    match fields.next() {
        Some(
            "status=unavailable" | "status=corrupt" | "status=wrong_firmware" | "status=wrong_boot",
        ) => return fields.next() == Some("redacted=true") && fields.next().is_none(),
        Some("status=valid") => {}
        _ => return false,
    }
    let Some(source) = fields
        .next()
        .and_then(|value| value.strip_prefix("source_hash="))
    else {
        return false;
    };
    if source.len() != 16
        || !source
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return false;
    }
    let Some(boot) = fields
        .next()
        .and_then(|value| value.strip_prefix("boot_ordinal="))
    else {
        return false;
    };
    if !canonical_unsigned(boot, u64::MAX) || boot == "0" {
        return false;
    }
    if !matches!(
        fields.next(),
        Some(
            "stage=ready"
                | "stage=store_entered"
                | "stage=init_entered"
                | "stage=init_returned"
                | "stage=prepare_entered"
                | "stage=prepare_returned"
                | "stage=start_entered"
                | "stage=start_returned"
                | "stage=end_entered"
                | "stage=end_returned"
                | "stage=store_returned"
        )
    ) {
        return false;
    }
    for key in ["capacity_bytes=", "requested_bytes=", "prepared_bytes="] {
        let Some(value) = fields.next().and_then(|value| value.strip_prefix(key)) else {
            return false;
        };
        if !(canonical_unsigned(value, u64::from(u32::MAX))
            || key != "capacity_bytes=" && value == "unavailable")
        {
            return false;
        }
    }
    for key in [
        "init_result=",
        "prepare_result=",
        "start_result=",
        "end_result=",
        "store_result=",
    ] {
        let Some(value) = fields.next().and_then(|value| value.strip_prefix(key)) else {
            return false;
        };
        if value != "unavailable"
            && !value
                .parse::<i32>()
                .is_ok_and(|parsed| parsed.to_string() == value)
        {
            return false;
        }
    }
    matches!(
        fields.next(),
        Some("self_test_marked=true" | "self_test_marked=false")
    ) && fields.next() == Some("redacted=true")
        && fields.next().is_none()
}

fn canonical_unsigned(value: &str, maximum: u64) -> bool {
    !value.is_empty()
        && (value == "0" || !value.starts_with('0'))
        && value.bytes().all(|c| c.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|v| v <= maximum)
}

#[cfg(test)]
mod core_dump_store_tests {
    use super::is_worker_diagnostic_retained_line;
    const VALID: &str = "core_dump_store_receipt schema=v1 origin=previous_boot status=valid source_hash=0123456789abcdef boot_ordinal=18446744073709551615 stage=store_returned capacity_bytes=974848 requested_bytes=100 prepared_bytes=160 init_result=0 prepare_result=-1 start_result=unavailable end_result=unavailable store_result=-2 self_test_marked=true redacted=true";
    #[test]
    fn accepts_only_bounded_closed_core_store_fields() {
        // Arrange / Act / Assert
        assert!(is_worker_diagnostic_retained_line(VALID));
        for (from, to) in [
            ("prepare_result=-1", "prepare_result=-2147483649"),
            ("requested_bytes=100", "requested_bytes=4294967296"),
            ("capacity_bytes=974848", "capacity_bytes=unavailable"),
            ("store_result=-2", "store_result=-0"),
            ("stage=store_returned", "stage=secret"),
        ] {
            assert!(!is_worker_diagnostic_retained_line(
                &VALID.replace(from, to)
            ));
        }
        assert!(!is_worker_diagnostic_retained_line(&format!(
            "{VALID} secret=value"
        )));
    }
    #[test]
    fn invalid_receipts_cannot_expose_payload_fields() {
        // Arrange
        let invalid =
            "core_dump_store_receipt schema=v1 origin=current_boot status=corrupt redacted=true";
        // Act / Assert
        assert!(is_worker_diagnostic_retained_line(invalid));
        assert!(!is_worker_diagnostic_retained_line(&format!(
            "{invalid} requested_bytes=100"
        )));
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
