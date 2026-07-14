const I2C_BUS_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/i2c_bus.rs");
const DISPLAY_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/display_adapter.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");
const INA260_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/ina260.rs");
const EMC2101_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/emc2101.rs");
const OPERATOR_RUNTIME_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/operator_sensor_runtime.rs");
const OBSERVATION_STORE_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/safety_adapter/observation_store.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");

#[test]
fn phase32_i2c_source_guard_uses_one_bounded_repeated_start_owner() {
    // Arrange
    let expected_timeout = "TickType::new_millis(I2C_TRANSACTION_TIMEOUT_MS).ticks()";

    // Act / Assert
    assert!(I2C_BUS_SOURCE.contains(expected_timeout));
    assert!(I2C_BUS_SOURCE.contains(".write_read("));
    assert_eq!(I2C_BUS_SOURCE.matches("I2cDriver::new").count(), 1);
    assert_eq!(MAIN_SOURCE.matches("BitaxeI2cBus::new").count(), 1);
    assert!(!DISPLAY_SOURCE.contains("I2cDriver::new"));
    assert!(!DISPLAY_SOURCE.contains("BLOCK"));
}

#[test]
fn phase32_i2c_source_guard_display_borrows_and_returns_the_shared_bus() {
    // Arrange / Act / Assert
    assert!(DISPLAY_SOURCE.contains("bus: &mut BitaxeI2cBus<'_>"));
    assert!(DISPLAY_SOURCE.contains("bus.startup_display()"));
    assert!(!DISPLAY_SOURCE.contains("bus: BitaxeI2cBus<'_>"));
}

#[test]
fn phase32_i2c_source_guard_closes_periodic_register_sets() {
    // Arrange
    let ina260_registers = ["Current => 0x01", "BusVoltage => 0x02", "Power => 0x03"];
    let emc2101_registers = [
        "ExternalTemperatureMsb => 0x01",
        "ExternalTemperatureLsb => 0x10",
        "TachometerLsb => 0x46",
        "TachometerMsb => 0x47",
    ];

    // Act / Assert
    for register in ina260_registers.into_iter().chain(emc2101_registers) {
        assert!(I2C_BUS_SOURCE.contains(register), "missing {register}");
    }
    let normal_owner = I2C_BUS_SOURCE
        .split_once("pub(super) struct ActiveI2cBus")
        .expect("active capability delimiter")
        .0;
    assert!(I2C_BUS_SOURCE.contains("struct ReadOnlySensorBus"));
    assert!(!normal_owner.contains("fn write_register"));
    assert!(I2C_BUS_SOURCE.contains("Phase27ActiveI2cToken"));
}

#[test]
fn phase32_sensor_source_guard_normal_readers_are_typed_and_read_only() {
    // Arrange
    let ina260_normal = INA260_SOURCE
        .split_once("pub fn read_sample")
        .expect("legacy INA260 reader delimiter")
        .0;
    let emc2101_normal = EMC2101_SOURCE
        .split_once("pub fn init")
        .expect("legacy EMC2101 active delimiter")
        .0;
    let forbidden = [
        "write_register(",
        "set_fan_duty_percent(",
        "ds4432u",
        "set_core_voltage",
        "hold_reset",
        "reset_pulse",
        "asic_adapter",
        "mining_",
        "fault_stimulus",
        "self_test",
        "credential",
        "uart",
        "gpio",
        "direct_uart",
        "ota_",
        "phase28",
    ];

    // Act / Assert
    assert!(ina260_normal.contains("AcquisitionOutcome<Ina260RawSample>"));
    assert!(ina260_normal.contains("decode_ina260(current, bus_voltage, power)"));
    assert!(emc2101_normal.contains("AcquisitionOutcome<f64>"));
    assert!(emc2101_normal.contains("AcquisitionOutcome<u16>"));
    for token in forbidden {
        assert!(
            !ina260_normal.to_ascii_lowercase().contains(token),
            "INA260 normal reader contains prohibited token {token}"
        );
        assert!(
            !emc2101_normal.to_ascii_lowercase().contains(token),
            "EMC2101 normal reader contains prohibited token {token}"
        );
    }
}

#[test]
fn phase32_runtime_source_guard_moves_one_post_display_owner_into_one_producer() {
    // Arrange
    let display = MAIN_SOURCE
        .find("render_startup_debug_text")
        .expect("normal startup display call");
    let producer = MAIN_SOURCE
        .find("operator_sensor_runtime::start(bus.into_read_only())")
        .expect("normal sensor producer handoff");

    // Act / Assert
    assert_eq!(MAIN_SOURCE.matches("BitaxeI2cBus::new").count(), 1);
    assert!(display < producer);
    assert!(MAIN_SOURCE.contains("match display_adapter::render_startup_debug_text"));
    assert!(I2C_BUS_SOURCE.contains("struct ReadOnlySensorOwner"));
    assert!(OPERATOR_RUNTIME_SOURCE.contains("ReadOnlySensorOwner<'static>"));
    assert!(!OPERATOR_RUNTIME_SOURCE.contains("BitaxeI2cBus"));
    assert!(!OPERATOR_RUNTIME_SOURCE.contains("startup_display"));
    assert!(OPERATOR_RUNTIME_SOURCE.contains("SENSOR_SWEEP_CADENCE_MS: u64 = 500"));
    assert!(OPERATOR_RUNTIME_SOURCE.contains("thread::Builder::new()"));
}

#[test]
fn phase32_runtime_source_guard_attempts_all_sources_before_store_replacement() {
    // Arrange
    let power = OPERATOR_RUNTIME_SOURCE
        .find("read_power_acquisition")
        .expect("power acquisition");
    let temperature = OPERATOR_RUNTIME_SOURCE
        .find("read_temperature_acquisition")
        .expect("temperature acquisition");
    let tachometer = OPERATOR_RUNTIME_SOURCE
        .find("read_tachometer_acquisition")
        .expect("tachometer acquisition");
    let replacement = OPERATOR_RUNTIME_SOURCE
        .find("replace_observations_from_producer")
        .expect("complete observation replacement");

    // Act / Assert
    assert!(power < temperature && temperature < tachometer && tachometer < replacement);
    assert!(OPERATOR_RUNTIME_SOURCE.contains("ProducerSequences::default()"));
    assert!(OPERATOR_RUNTIME_SOURCE.contains("next_future_deadline"));
    assert!(!OPERATOR_RUNTIME_SOURCE.contains("retry"));
}

#[test]
fn phase32_runtime_source_guard_projects_exact_facts_without_active_effects() {
    // Arrange
    let required = [
        "power_watts: project_observation(",
        "bus_voltage_volts: project_observation(",
        "current_amps: project_observation(",
        "chip_temp_celsius: project_observation(",
        "fan_rpm: project_observation(",
        "vr_temp_celsius: bitaxe_safety::observation::Observation::unavailable(",
    ];
    let forbidden = [
        "active_for_phase27",
        "write_register",
        "set_fan_duty_percent",
        "set_core_voltage",
        "hold_reset",
        "reset_pulse",
        "asic_adapter",
        "mining_",
        "credential",
        "uart",
        "gpio",
        "ota_",
        "phase28",
    ];

    // Act / Assert
    for projection in required {
        assert!(OPERATOR_RUNTIME_SOURCE.contains(projection));
    }
    for token in forbidden {
        assert!(
            !OPERATOR_RUNTIME_SOURCE.to_ascii_lowercase().contains(token),
            "normal producer contains prohibited token {token}"
        );
    }
}

#[test]
fn phase32_consumer_source_guard_keeps_firmware_snapshot_reads_clone_only() {
    // Arrange
    let forbidden = ["i2c", "read_acquisition", "sensor", "write_register"];

    // Act / Assert
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains("observation_snapshot()"));
    assert_eq!(
        RUNTIME_SNAPSHOT_SOURCE
            .matches("observation_snapshot()")
            .count(),
        1
    );
    for token in forbidden {
        assert!(
            !OBSERVATION_STORE_SOURCE
                .to_ascii_lowercase()
                .contains(token),
            "observation store contains producer token {token}"
        );
        assert!(
            !RUNTIME_SNAPSHOT_SOURCE.to_ascii_lowercase().contains(token),
            "runtime snapshot contains producer token {token}"
        );
    }
}
