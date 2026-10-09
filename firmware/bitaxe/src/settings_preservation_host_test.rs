//! Host behavior checks over the production public-settings fingerprint model and the
//! device-local pool-configuration continuity model.
#[path = "settings_adapter/preservation/model.rs"]
mod model;
#[path = "settings_adapter/preservation/pool_configuration.rs"]
mod pool_configuration;

const POOL_SOURCE: &str = include_str!("settings_adapter/preservation/pool_configuration.rs");
const PRESERVATION_SOURCE: &str = include_str!("settings_adapter/preservation.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

fn production_part(source: &str) -> &str {
    source
        .split("#[cfg(test)]")
        .next()
        .expect("production source")
}

#[test]
fn pool_digest_has_no_serialization_logging_or_byte_accessor() {
    // Arrange
    let production = production_part(POOL_SOURCE);

    // Act / Assert
    for forbidden in [
        "log::",
        "println!",
        "format!",
        "serde",
        "Serialize",
        "derive(Debug",
        "derive(Clone",
        "-> [u8; 32]",
        "-> &[u8",
        "pub(crate) struct PoolConfigurationDigest(pub",
    ] {
        assert!(!production.contains(forbidden), "{forbidden}");
    }
    assert!(production.contains("PoolConfigurationDigest([private])"));
}

#[test]
fn preservation_shell_logs_only_fixed_markers_and_reports_only_the_boolean() {
    // Arrange
    let log_lines: Vec<&str> = PRESERVATION_SOURCE
        .lines()
        .filter(|line| line.contains("log::"))
        .collect();

    // Act / Assert
    assert_eq!(log_lines.len(), 2);
    for line in log_lines {
        assert!(!line.contains('{'), "log line interpolates a value: {line}");
    }
    assert_eq!(
        PRESERVATION_SOURCE
            .matches("BOOT_POOL_CONFIGURATION.set(")
            .count(),
        1
    );
    assert!(PRESERVATION_SOURCE.contains("pool_configuration::unchanged_since_boot("));
    assert!(!PRESERVATION_SOURCE.contains("serde"));
}

#[test]
fn boot_capture_precedes_every_lease_capable_owner() {
    // Arrange
    let position = |needle: &str| {
        STARTUP_SOURCE
            .find(needle)
            .unwrap_or_else(|| panic!("startup source lacks {needle}"))
    };

    // Act
    let capture = position("settings_adapter::preservation::capture_boot_pool_configuration()");

    // Assert
    assert_eq!(
        STARTUP_SOURCE
            .matches("capture_boot_pool_configuration")
            .count(),
        1
    );
    assert!(position("settings_adapter::initialize_default_nvs_partition()?") < capture);
    assert!(capture < position("fn initialize_hardware("));
    let run_startup = position("fn run_startup(");
    let identity_call = run_startup
        + STARTUP_SOURCE[run_startup..]
            .find("initialize_boot_identity_and_settings()?")
            .expect("identity call");
    for later_owner in [
        "start_runtime_services(startup_diagnostics)?",
        "start_storage_and_http()",
        "start_network_services(maybe_wifi)",
        "start_deferred_usb_runtime(",
    ] {
        let owner = run_startup
            + STARTUP_SOURCE[run_startup..]
                .find(later_owner)
                .expect("later owner");
        assert!(identity_call < owner, "{later_owner}");
    }
}
