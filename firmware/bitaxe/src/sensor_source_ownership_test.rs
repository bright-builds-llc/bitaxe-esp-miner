const OPERATOR_SENSOR_RUNTIME_SOURCE: &str = include_str!("operator_sensor_runtime.rs");
const SAFETY_ADAPTER_SOURCE: &str = include_str!("safety_adapter.rs");
const I2C_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus.rs");
const PRODUCTION_SESSION_SOURCE: &str = include_str!("production_mining_session.rs");

#[test]
fn operator_sensor_runtime_is_the_single_normal_acquisition_caller() {
    // Arrange
    let required_calls = [
        "safety_adapter::read_power_acquisition(&mut owner)",
        "safety_adapter::read_asic_temperature_acquisition(&mut owner)",
        "safety_adapter::read_vr_temperature_acquisition(&mut owner)",
        "safety_adapter::read_tachometer_acquisition(&mut owner)",
    ];

    // Act / Assert
    for required_call in required_calls {
        assert_eq!(
            OPERATOR_SENSOR_RUNTIME_SOURCE.matches(required_call).count(),
            1,
            "expected exactly one owner call for {required_call}"
        );
        assert!(!PRODUCTION_SESSION_SOURCE.contains(required_call));
    }
}

#[test]
fn raw_sensor_bus_capability_is_private_to_the_safety_facade() {
    // Arrange
    let expected_facade_reads = 4;

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
fn vr_truth_is_projected_and_used_by_the_closed_mining_safety_verdict() {
    // Arrange / Act / Assert
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains("vr_temp_celsius: project_observation("));
    assert!(!OPERATOR_SENSOR_RUNTIME_SOURCE.contains(
        "vr_temp_celsius: bitaxe_safety::observation::Observation::unavailable("
    ));
    assert!(PRODUCTION_SESSION_SOURCE.contains("observations.is_ultra_205_mining_safe_at(now())"));
    assert!(PRODUCTION_SESSION_SOURCE.contains(
        "Self::maybe_reject_safety_gated_effect(None, \"hardware_prepare\")"
    ));
    assert!(PRODUCTION_SESSION_SOURCE
        .contains("Self::maybe_reject_safety_gated_effect(None, \"asic_dispatch\")"));
}
