const STARTUP: &str = include_str!("startup.rs");
const OWNER: &str = include_str!("stratum_v2_session.rs");
const TRANSPORT: &str = include_str!("stratum_v2_session/transport.rs");
const DIAGNOSTIC: &str = include_str!("stratum_v2_noise_diagnostic.rs");
const DIAGNOSTIC_ADMISSION: &str = include_str!("settings_adapter/noise_diagnostic.rs");
const V1_OWNER: &str = include_str!("production_mining_session.rs");
const SETTINGS: &str = include_str!("settings_adapter/stratum_v2.rs");

#[test]
fn startup_selects_exactly_one_protocol_owner_before_fan_controller_start() {
    // Arrange
    let selector = STARTUP
        .find("configured_protocol_plan")
        .expect("protocol selector");
    let v2_start = STARTUP
        .find("stratum_v2_session::start")
        .expect("V2 owner start");
    let v1_start = STARTUP
        .find("production_mining_session::start")
        .expect("V1 owner start");
    let fan_start = STARTUP
        .find("fan_controller_runtime::start")
        .expect("fan controller start");

    // Act / Assert
    assert!(selector < v2_start);
    assert!(selector < v1_start);
    assert!(v2_start < fan_start);
    assert!(v1_start < fan_start);
    assert_eq!(STARTUP.matches("stratum_v2_session::start").count(), 1);
    assert_eq!(STARTUP.matches("production_mining_session::start").count(), 1);
}

#[test]
fn diagnostic_owner_precedes_other_owners_and_suppresses_the_fan() {
    // Arrange
    let diagnostic_admission = STARTUP
        .find("load_noise_diagnostic_admission")
        .expect("diagnostic admission");
    let diagnostic_start = STARTUP
        .find("stratum_v2_noise_diagnostic::start")
        .expect("diagnostic start");
    let self_test_start = STARTUP
        .find("self_test_runtime::start")
        .expect("self-test start");
    let production_start = STARTUP
        .find("production_mining_session::start")
        .expect("production start");
    let fan_start = STARTUP
        .find("fan_controller_runtime::start")
        .expect("fan start");

    // Act / Assert
    assert!(diagnostic_admission < diagnostic_start);
    assert!(diagnostic_start < self_test_start);
    assert!(diagnostic_start < production_start);
    assert!(diagnostic_start < fan_start);
    assert!(STARTUP.contains("if let Some(admission) = maybe_noise_diagnostic_admission"));
    assert!(STARTUP.contains("} else if let Some(admission) = maybe_self_test_admission"));
}

#[test]
fn diagnostic_owner_cannot_reach_hardware_or_mining_adapters() {
    // Arrange
    let forbidden = [
        "asic_adapter",
        "mining_actuation",
        "production_mining_session",
        "fan_controller",
        "core_voltage",
        "V2Session",
    ];

    // Act / Assert
    for fragment in forbidden {
        assert!(!DIAGNOSTIC.contains(fragment), "forbidden owner fragment {fragment}");
    }
    assert!(DIAGNOSTIC.contains("run_noise_diagnostic"));
    assert!(DIAGNOSTIC.contains("mining_started\\\":false"));
    assert!(DIAGNOSTIC_ADMISSION.contains("sv2diagkind"));
    assert!(DIAGNOSTIC_ADMISSION.contains("erase_admission_tuple"));
}

#[test]
fn v2_owner_reuses_single_asic_actuation_watchdog_and_safe_stop_paths() {
    // Arrange / Act / Assert
    assert!(OWNER.contains("Ultra205MiningActuationAdapter::new"));
    assert!(OWNER.contains("ProductionAsicExecutor::new"));
    assert!(OWNER.contains("ProductionTaskWatchdog::subscribe"));
    assert!(OWNER.contains("HardwareSafeStopPurpose::Terminal"));
    assert!(OWNER.contains("block_production_dispatch"));
    assert!(OWNER.contains("MiningCampaignStage::StratumV2"));
    assert!(!V1_OWNER.contains("stratum_v2_session::start"));
}

#[test]
fn v2_transport_and_settings_diagnostics_are_value_free() {
    // Arrange
    let forbidden_output = ["println!", "endpoint_host={", "user_identity={", "authority={"];

    // Act / Assert
    for source in [OWNER, TRANSPORT, SETTINGS] {
        for fragment in forbidden_output {
            assert!(!source.contains(fragment), "forbidden output fragment {fragment}");
        }
    }
    assert!(TRANSPORT.contains("TransportCommand::Send(redacted)"));
    assert!(SETTINGS.contains(".field(\"session\", &\"redacted\")"));
}

#[test]
fn production_and_diagnostic_prepare_noise_before_connecting() {
    // Arrange
    let production_start = TRANSPORT.find("fn connect_and_run(").expect("production start");
    let diagnostic_start = TRANSPORT
        .find("pub(crate) fn run_noise_diagnostic(")
        .expect("diagnostic start");
    let encrypted_loop = TRANSPORT
        .find("fn run_encrypted_loop(")
        .expect("encrypted loop");
    let production = &TRANSPORT[production_start..diagnostic_start];
    let diagnostic = &TRANSPORT[diagnostic_start..encrypted_loop];

    // Act / Assert
    assert!(
        production.find("NoiseInitiator::prepare").expect("production preparation")
            < production.find("connect_first(&addresses)").expect("production connect")
    );
    assert!(
        diagnostic
            .find("NoiseInitiator::prepare_with_observer")
            .expect("diagnostic preparation")
            < diagnostic
                .find("connect_first(&addresses)")
                .expect("diagnostic connect")
    );
}
