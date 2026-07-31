#[path = "safety_adapter/request_queue.rs"]
mod safety_request_queue;

const OPERATOR_SENSOR_RUNTIME_SOURCE: &str = include_str!("operator_sensor_runtime.rs");
const SAFETY_ADAPTER_SOURCE: &str = include_str!("safety_adapter.rs");
const OBSERVATION_STORE_SOURCE: &str = include_str!("safety_adapter/observation_store.rs");
const I2C_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus.rs");
const EMC2101_SOURCE: &str = include_str!("safety_adapter/emc2101.rs");
const DS4432U_SOURCE: &str = include_str!("safety_adapter/ds4432u.rs");
const MINING_ACTUATION_ADAPTER_SOURCE: &str = include_str!("mining_actuation_adapter.rs");
const PRODUCTION_SESSION_SOURCE: &str = include_str!("production_mining_session.rs");
const PRODUCTION_TRANSPORT_SOURCE: &str =
    include_str!("production_mining_session/transport.rs");
const PRODUCTION_ASIC_WORKER_SOURCE: &str =
    include_str!("production_mining_session/asic_worker.rs");
const SETTINGS_ADAPTER_SOURCE: &str = include_str!("settings_adapter.rs");
const SETTINGS_PRODUCTION_SOURCE: &str = include_str!("settings_adapter/production.rs");

#[test]
fn operator_sensor_runtime_is_the_single_normal_acquisition_caller() {
    // Arrange
    let required_calls = [
        "safety_adapter::read_power_acquisition(&mut owner)",
        "safety_adapter::read_asic_temperature_acquisition(&mut owner)",
        "safety_adapter::read_tachometer_acquisition(&mut owner)",
    ];

    // Act / Assert
    for required_call in required_calls {
        assert_eq!(
            OPERATOR_SENSOR_RUNTIME_SOURCE
                .matches(required_call)
                .count(),
            1,
            "expected exactly one owner call for {required_call}"
        );
        assert!(!PRODUCTION_SESSION_SOURCE.contains(required_call));
    }
}

#[test]
fn raw_sensor_bus_capability_is_private_to_the_safety_facade() {
    // Arrange
    let expected_facade_reads = 3;

    // Act / Assert
    assert_eq!(
        SAFETY_ADAPTER_SOURCE.matches("owner.sensors()").count(),
        expected_facade_reads
    );
    assert!(I2C_BUS_SOURCE.contains("pub(super) fn sensors"));
    assert!(!OPERATOR_SENSOR_RUNTIME_SOURCE.contains("owner.sensors()"));
    assert!(!PRODUCTION_SESSION_SOURCE.contains("owner.sensors()"));
}

#[test]
fn operator_runtime_is_the_only_shared_i2c_actuation_consumer() {
    // Arrange
    let service_call = "safety_adapter::service_next_safety_actuation_request(";

    // Act / Assert
    assert_eq!(
        OPERATOR_SENSOR_RUNTIME_SOURCE.matches(service_call).count(),
        1
    );
    assert_eq!(
        SAFETY_ADAPTER_SOURCE.matches("owner.actuators()").count(),
        1
    );
    assert!(I2C_BUS_SOURCE.contains("pub(super) fn actuators"));
    assert!(!OPERATOR_SENSOR_RUNTIME_SOURCE.contains("owner.actuators()"));
    assert!(!PRODUCTION_SESSION_SOURCE.contains("owner.actuators()"));
}

#[test]
fn raw_actuator_primitives_remain_inside_the_safety_adapter() {
    // Arrange
    let raw_primitives = [
        "I2cDriver",
        "write_emc2101",
        "write_ds4432u",
        "Emc2101WriteRegister",
        "Ds4432uWriteRegister",
        "0x4a",
        "0x4c",
        "0xf8",
    ];

    // Act / Assert
    for primitive in raw_primitives {
        assert!(
            I2C_BUS_SOURCE.contains(primitive)
                || EMC2101_SOURCE.contains(primitive)
                || DS4432U_SOURCE.contains(primitive),
            "expected a safety adapter owner for {primitive}"
        );
        assert!(
            !OPERATOR_SENSOR_RUNTIME_SOURCE.contains(primitive),
            "operator runtime must not expose {primitive}"
        );
        assert!(
            !PRODUCTION_SESSION_SOURCE.contains(primitive),
            "production session must not expose {primitive}"
        );
        assert!(
            !MINING_ACTUATION_ADAPTER_SOURCE.contains(primitive),
            "mining collaborator must use semantic commands, not {primitive}"
        );
    }
}

#[test]
fn only_high_level_actuation_requests_cross_into_the_mining_collaborator() {
    // Arrange / Act / Assert
    assert!(SAFETY_ADAPTER_SOURCE.contains("pub(crate) fn request_safety_actuation("));
    assert!(SAFETY_ADAPTER_SOURCE.contains("pub(crate) fn queue_safety_actuation("));
    assert!(SAFETY_ADAPTER_SOURCE.contains("pub(crate) fn safety_actuation_available()"));
    assert!(!PRODUCTION_SESSION_SOURCE.contains("RuntimeI2cOwner"));
    assert!(!PRODUCTION_SESSION_SOURCE.contains("SafetyActuationOwnerInbox"));
}

#[test]
fn fan_preparation_never_waits_for_an_actuation_reply() {
    // Arrange
    let start = MINING_ACTUATION_ADAPTER_SOURCE
        .find("fn set_fan_full")
        .expect("fan preparation function");
    let end = MINING_ACTUATION_ADAPTER_SOURCE[start..]
        .find("fn wait_for_post_command_fan_proof")
        .map(|offset| start + offset)
        .expect("fan proof function");
    let set_fan_full = &MINING_ACTUATION_ADAPTER_SOURCE[start..end];
    let snapshot_start = OBSERVATION_STORE_SOURCE
        .find("pub(crate) fn observation_snapshot")
        .expect("observation snapshot function");
    let snapshot_end = OBSERVATION_STORE_SOURCE[snapshot_start..]
        .find("pub(crate) fn replace_observations_from_producer")
        .map(|offset| snapshot_start + offset)
        .expect("observation producer function");
    let observation_snapshot = &OBSERVATION_STORE_SOURCE[snapshot_start..snapshot_end];

    // Act / Assert
    assert!(set_fan_full.contains("queue_safety_actuation"));
    assert!(!set_fan_full.contains("request_safety("));
    assert!(observation_snapshot.contains("store().try_lock()"));
    assert!(!observation_snapshot.contains("store().lock()"));
}

#[test]
fn unsupported_ultra205_vr_truth_is_projected_but_not_required_for_mining() {
    // Arrange / Act / Assert
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains(
        "AcquisitionOutcome::Unavailable(UnavailableReason::UnsupportedOnBoard)"
    ));
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains("vr_temp_celsius: project_observation("));
    assert!(PRODUCTION_SESSION_SOURCE.contains("observations.is_ultra_205_mining_safe_at(now())"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("self.mining_actuation.prepare(profile)"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("safety_prerequisites_fresh"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("actuation_qualified"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("ProductionSessionEffect::DispatchAsic"));
}

#[test]
fn production_owner_uses_typed_workers_without_owning_raw_io() {
    // Arrange
    let owner_forbidden = ["TcpStream", "write_all", "EspNvs", "stratumurl", "stratumpass"];

    // Act / Assert
    for primitive in owner_forbidden {
        assert!(!PRODUCTION_SESSION_SOURCE.contains(primitive));
    }
    assert!(PRODUCTION_TRANSPORT_SOURCE.contains("TcpStream"));
    assert!(PRODUCTION_TRANSPORT_SOURCE.contains("PoolTransportEvent"));
    assert!(PRODUCTION_ASIC_WORKER_SOURCE.contains("ProductionAsicExecutor"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("OwnerInboxMessage::Transport"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("OwnerInboxMessage::Asic"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("self.transports.request_close"));
}

#[test]
fn pool_secrets_are_owned_only_by_the_lazy_settings_reader() {
    // Arrange
    let secret_keys = ["stratumurl", "stratumuser", "stratumpass"];

    // Act / Assert
    for key in secret_keys {
        assert!(SETTINGS_PRODUCTION_SOURCE.contains(key));
        assert!(!PRODUCTION_SESSION_SOURCE.contains(key));
        assert!(!PRODUCTION_TRANSPORT_SOURCE.contains(key));
        assert!(!PRODUCTION_ASIC_WORKER_SOURCE.contains(key));
    }
    assert!(SETTINGS_ADAPTER_SOURCE.contains("mod production;"));
    assert!(SETTINGS_PRODUCTION_SOURCE.contains("read_production_pool_set"));
    assert!(PRODUCTION_SESSION_SOURCE
        .contains("ProductionSessionEffect::ReadPoolConfiguration"));
}
