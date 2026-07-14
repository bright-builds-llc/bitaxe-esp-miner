const I2C_BUS_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/i2c_bus.rs");
const DISPLAY_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/display_adapter.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");
const INA260_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/ina260.rs");
const EMC2101_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/safety_adapter/emc2101.rs");

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
