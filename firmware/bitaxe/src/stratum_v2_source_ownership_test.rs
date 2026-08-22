const STARTUP: &str = include_str!("startup.rs");
const OWNER: &str = include_str!("stratum_v2_session.rs");
const TRANSPORT: &str = include_str!("stratum_v2_session/transport.rs");
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
