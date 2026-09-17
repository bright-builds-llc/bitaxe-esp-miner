const STARTUP: &str = include_str!("startup.rs");
const MAIN: &str = include_str!("main.rs");
const OWNER: &str = include_str!("stratum_v2_session.rs");
const TRANSPORT: &str = include_str!("stratum_v2_session/transport.rs");
const RETIRED_TRANSPORT: &str = include_str!("stratum_v2_session/retired_noise_diagnostic.rs");
const DIAGNOSTIC: &str = include_str!("stratum_v2_noise_diagnostic.rs");
const DIAGNOSTIC_ADMISSION: &str = include_str!("settings_adapter/noise_diagnostic.rs");
const TCP_DIAGNOSTIC: &str = include_str!("stratum_v2_tcp_payload_diagnostic.rs");
const TCP_DIAGNOSTIC_ADMISSION: &str = include_str!("settings_adapter/tcp_payload_diagnostic.rs");
const V1_OWNER: &str = include_str!("production_mining_session.rs");
const SETTINGS: &str = include_str!("settings_adapter/stratum_v2.rs");

#[path = "stratum_v2_tcp_payload_replay.rs"]
mod stratum_v2_tcp_payload_replay;

#[test]
fn startup_selects_the_shared_owner_without_linking_retired_boot_v2() {
    // Arrange
    let selector=STARTUP.find("configured_protocol_plan").expect("validated settings");
    let owner=STARTUP.find("production_mining_session::start").expect("ordinary owner");
    let fan=STARTUP.find("fan_controller_runtime::start").expect("fan owner");
    // Act / Assert
    assert!(selector<owner && owner<fan);
    assert_eq!(STARTUP.matches("production_mining_session::start").count(),1);
    assert!(!STARTUP.contains("stratum_v2_session::start"));
    assert!(!MAIN.contains("mod stratum_v2_session;"));
}

#[test]
fn retired_boot_noise_cannot_be_selected_or_linked_by_current_startup() {
    // Arrange / Act / Assert
    assert!(!STARTUP.contains("load_noise_diagnostic_admission"));
    assert!(!STARTUP.contains("stratum_v2_noise_diagnostic::start"));
    assert!(!MAIN.contains("mod stratum_v2_noise_diagnostic;"));
    assert!(!TRANSPORT.contains("run_noise_diagnostic"));
    assert!(RETIRED_TRANSPORT.contains("run_noise_diagnostic"));
    let wifi = STARTUP
        .find("prepare_network_services(maybe_modem)")
        .expect("Wi-Fi first");
    let metadata = STARTUP
        .find("noise_serial_runtime::prepare()")
        .expect("metadata init");
    let runtime = STARTUP
        .find("start_runtime_services(startup_diagnostics)")
        .expect("ordinary owners");
    assert!(wifi < metadata && metadata < runtime);
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
        assert!(
            !DIAGNOSTIC.contains(fragment),
            "forbidden owner fragment {fragment}"
        );
    }
    assert!(DIAGNOSTIC.contains("run_noise_diagnostic"));
    assert!(DIAGNOSTIC.contains("mining_started\\\":false"));
    assert!(DIAGNOSTIC_ADMISSION.contains("sv2diagkind"));
    assert!(DIAGNOSTIC_ADMISSION.contains("erase_admission_tuple"));
}

#[test]
fn noise_auth_owner_replays_connection_send_and_exact_proof_evidence() {
    // Arrange / Act / Assert
    assert!(DIAGNOSTIC_ADMISSION.contains("noise_auth_v1"));
    assert!(DIAGNOSTIC.contains("DiagnosticTranscript"));
    assert!(DIAGNOSTIC.contains("replay_deadline_ms"));
    assert!(DIAGNOSTIC.contains("transcript.replay()"));
    assert!(DIAGNOSTIC.contains("stratum_v2_noise_connection_private="));
    assert!(RETIRED_TRANSPORT.contains(".local_addr()"));
    assert!(RETIRED_TRANSPORT.contains(".set_nodelay(true)"));
    assert!(RETIRED_TRANSPORT.contains(".flush()"));
    assert!(RETIRED_TRANSPORT.contains("DIAGNOSTIC_PROOF_EXTENSION"));
    assert!(RETIRED_TRANSPORT.contains("DIAGNOSTIC_PROOF_MESSAGE"));
    assert!(RETIRED_TRANSPORT.contains("0xffff"));
    assert!(RETIRED_TRANSPORT.contains("0xff"));
}

#[test]
fn tcp_payload_owner_cannot_reach_noise_or_hardware() {
    // Arrange
    let tcp_admission = STARTUP
        .find("load_tcp_payload_diagnostic_admission")
        .expect("TCP diagnostic admission");
    let tcp_start = STARTUP
        .find("stratum_v2_tcp_payload_diagnostic::start")
        .expect("TCP diagnostic start");
    // Act / Assert
    assert!(tcp_admission < tcp_start);
    for forbidden in [
        "NoiseInitiator",
        "V2Session",
        "asic_adapter",
        "mining_actuation",
        "production_mining_session",
        "fan_controller",
        "core_voltage",
    ] {
        assert!(!TCP_DIAGNOSTIC.contains(forbidden));
    }
    assert!(TCP_DIAGNOSTIC.contains("stream.write_all(&PAYLOAD)"));
    let payload_write = TCP_DIAGNOSTIC
        .find("stream.write_all(&PAYLOAD)")
        .expect("fixed payload write");
    let write_shutdown = TCP_DIAGNOSTIC
        .find(".shutdown(Shutdown::Write)")
        .expect("write half-close");
    let receipt_read = TCP_DIAGNOSTIC
        .find("stream.read_exact(&mut receipt)")
        .expect("receipt read");
    assert!(TCP_DIAGNOSTIC.contains("stream.set_nodelay(true)"));
    assert!(TCP_DIAGNOSTIC.contains(".local_addr()"));
    assert!(TCP_DIAGNOSTIC.contains("stratum_v2_tcp_connection_private="));
    assert!(TCP_DIAGNOSTIC.contains("reported_bytes_written"));
    assert!(payload_write < write_shutdown);
    assert!(write_shutdown < receipt_read);
    for category in [
        "shutdown_would_block",
        "shutdown_not_connected",
        "shutdown_out_of_memory",
        "shutdown_invalid_input",
        "shutdown_unsupported",
        "shutdown_other",
    ] {
        assert!(TCP_DIAGNOSTIC.contains(category));
    }
    assert!(MAIN.contains("mod stratum_v2_tcp_payload_replay;"));
    assert!(TCP_DIAGNOSTIC.contains("replay_deadline_ms"));
    assert!(TCP_DIAGNOSTIC.contains("transcript.replay()"));
    assert!(TCP_DIAGNOSTIC.contains("receipt_acknowledged"));
    assert!(TCP_DIAGNOSTIC.contains("noise_started\\\":false"));
    assert!(TCP_DIAGNOSTIC_ADMISSION.contains("tcpdiagkind"));
    assert!(TCP_DIAGNOSTIC_ADMISSION.contains("erase_admission_tuple"));
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
    let forbidden_output = [
        "println!",
        "endpoint_host={",
        "user_identity={",
        "authority={",
    ];

    // Act / Assert
    for source in [OWNER, TRANSPORT, SETTINGS] {
        for fragment in forbidden_output {
            assert!(
                !source.contains(fragment),
                "forbidden output fragment {fragment}"
            );
        }
    }
    assert!(TRANSPORT.contains("TransportCommand::Send(redacted)"));
    assert!(SETTINGS.contains(".field(\"session\", &\"redacted\")"));
}

#[test]
fn production_and_retained_diagnostic_prepare_noise_before_connecting() {
    // Arrange
    let production_start = TRANSPORT
        .find("fn connect_and_run(")
        .expect("production start");
    let production_end = TRANSPORT
        .find("fn run_encrypted_loop(")
        .expect("encrypted loop");
    let production = &TRANSPORT[production_start..production_end];
    // Act / Assert
    assert!(
        production
            .find("NoiseInitiator::prepare")
            .expect("preparation")
            < production
                .find("connect_first(&addresses)")
                .expect("connect")
    );
    assert!(
        RETIRED_TRANSPORT
            .find("NoiseInitiator::prepare_with_observer")
            .expect("retained preparation")
            < RETIRED_TRANSPORT
                .find("connect_first(&addresses)")
                .expect("retained connect")
    );
}
