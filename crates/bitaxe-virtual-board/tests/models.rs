use bitaxe_virtual_board::{
    peripherals::Peripherals,
    scheduler::DeterministicScheduler,
    storage::{PersistentStore, ResetReason},
    transport::VirtualTransport,
};
use bitaxe_virtual_board::{BoardConfig, BoardEvent, Dynamics, ModelError, VirtualBoard};

#[test]
fn equal_timestamp_events_preserve_insertion_order() {
    // Arrange
    let mut scheduler = DeterministicScheduler::new(1);
    scheduler.schedule(10, "first").expect("event");
    scheduler.schedule(10, "second").expect("event");
    // Act
    let events = scheduler.advance_to(10).expect("advance");
    // Assert
    assert_eq!(events, [(10, "first"), (10, "second")]);
}
#[test]
fn seeded_scheduler_replays_identically() {
    // Arrange
    let mut first = DeterministicScheduler::new(91);
    let mut second = DeterministicScheduler::new(91);
    // Act
    for value in 0..50 {
        first.schedule_jittered(10, 50, value).expect("event");
        second.schedule_jittered(10, 50, value).expect("event");
    }
    // Assert
    assert_eq!(
        first.advance_to(60).expect("advance"),
        second.advance_to(60).expect("advance")
    );
}
#[test]
fn clock_regression_is_rejected() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("valid");
    board.advance_to(100).expect("advance");
    // Act
    let result = board.advance_to(99);
    // Assert
    assert_eq!(result, Err(ModelError::Invalid("clock regression")));
}
#[test]
fn dynamics_independent_of_host_advance_granularity() {
    // Arrange
    let mut first = Peripherals::new(Dynamics::default()).expect("valid");
    first.fan_percent = 100;
    first.power_enabled = true;
    first.target_core_mv = 1200;
    let mut second = first.clone();
    // Act
    first.advance_to(600, false).expect("advance");
    for step in 1..=600 {
        second.advance_to(step, false).expect("advance");
    }
    // Assert
    assert_eq!(first.sample, second.sample);
}
#[test]
fn sensor_publication_stall_keeps_original_timestamp() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("valid");
    board
        .scheduler
        .schedule(150, BoardEvent::SensorPublicationBlocked(true))
        .expect("event");
    // Act
    board.advance_to(1000).expect("advance");
    // Assert
    assert_eq!(board.peripherals.sample.sampled_at_ms, 100);
}
#[test]
fn stalled_fan_cannot_generate_nonzero_rpm() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("valid");
    board.peripherals.fan_percent = 100;
    board.peripherals.fan_stalled = true;
    // Act
    board.advance_to(1000).expect("advance");
    // Assert
    assert_eq!(board.peripherals.sample.fan_rpm, 0);
}
#[test]
fn i2c_claims_serialize_transactions() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("valid");
    board.i2c_read(0x4c, 0xfd, 1).expect("first transaction");
    // Act
    let result = board.i2c_read(0x40, 0xfe, 2);
    // Assert
    assert_eq!(result, Err(ModelError::Busy { until_ms: 1 }));
}
#[test]
fn independent_register_ids_match_reference() {
    // Arrange
    let devices = Peripherals::new(Dynamics::default()).expect("valid");
    // Act
    let ids = (
        devices.read_register(0x40, 0xfe, 2),
        devices.read_register(0x4c, 0xfd, 1),
    );
    // Assert
    assert_eq!(ids, (Ok(vec![0x54, 0x49]), Ok(vec![0x16])));
}
#[test]
fn ina260_wire_units_match_reference() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics::default()).expect("valid");
    devices.power_enabled = true;
    devices.advance_to(100, true).expect("advance");
    // Act
    let facts = (
        devices.read_register(0x40, 0x01, 2),
        devices.read_register(0x40, 0x02, 2),
        devices.read_register(0x40, 0x03, 2),
    );
    // Assert
    assert_eq!(
        facts,
        (
            Ok(1920_u16.to_be_bytes().to_vec()),
            Ok(4000_u16.to_be_bytes().to_vec()),
            Ok(1200_u16.to_be_bytes().to_vec())
        )
    );
}
#[test]
fn unsupported_i2c_operation_does_not_succeed() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics::default()).expect("valid");
    // Act
    let result = devices.write_register(0x4c, 0xff, &[0]);
    // Assert
    assert_eq!(result, Err(ModelError::Unsupported("I2C register write")));
}
#[test]
fn display_tracks_pixels_and_rejects_unknown_commands() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics::default()).expect("valid");
    devices
        .display_write(0, &[0xaf, 0xb1, 0x02])
        .expect("commands");
    // Act
    devices.display_write(0x40, &[0xa5]).expect("pixels");
    // Assert
    assert!(devices.display_enabled);
    assert_eq!(devices.display[130], 0xa5);
    assert_eq!(
        devices.display_write(0, &[0xee]),
        Err(ModelError::Unsupported("SSD1306 command"))
    );
}
#[test]
fn persistence_failure_leaves_entire_transaction_unchanged() {
    // Arrange
    let mut storage = PersistentStore::default();
    storage.reject_writes = true;
    // Act
    let result = storage.transaction(&[("setting".to_owned(), vec![1])]);
    // Assert
    assert_eq!(result, Err(ModelError::Persistence));
    assert_eq!(storage.read("setting"), None);
}
#[test]
fn reset_preserves_charge_identity_and_replay() {
    // Arrange
    let mut storage = PersistentStore::default();
    storage.admit_replay(7).expect("fresh");
    let ordinal = storage.reserve_charge(180000).expect("charge");
    let identity = storage.read("device_identity").expect("identity").to_vec();
    // Act
    storage.reset(ResetReason::Panic).expect("reset");
    // Assert
    assert_eq!(storage.read("device_identity"), Some(identity.as_slice()));
    assert_eq!(
        (
            storage.charged_ms,
            storage.maybe_pending_ordinal,
            storage.replay_high_water
        ),
        (180000, Some(ordinal), 7)
    );
}
#[test]
fn pending_accounting_prevents_second_effect_reservation() {
    // Arrange
    let mut storage = PersistentStore::default();
    storage.reserve_charge(100).expect("first charge");
    // Act
    let result = storage.reserve_charge(100);
    // Assert
    assert_eq!(result, Err(ModelError::Invalid("pending accounting")));
    assert_eq!(storage.charged_ms, 100);
}
#[test]
fn replay_state_is_monotonic() {
    // Arrange
    let mut storage = PersistentStore::default();
    storage.admit_replay(8).expect("fresh");
    // Act
    let result = storage.admit_replay(8);
    // Assert
    assert_eq!(result, Err(ModelError::Invalid("replayed sequence")));
}
#[test]
fn nor_write_requires_erase_before_setting_bits() {
    // Arrange
    let mut storage = PersistentStore::default();
    storage
        .write_partition("coredump", 0, &[0])
        .expect("NOR write");
    // Act
    let result = storage.write_partition("coredump", 0, &[0xff]);
    // Assert
    assert_eq!(result, Err(ModelError::Invalid("NOR zero to one")));
}
#[test]
fn failed_close_preserves_resource_and_cannot_claim_release() {
    // Arrange
    let mut transport = VirtualTransport::new(100);
    transport.connect().expect("connect");
    transport.reject_close = true;
    // Act
    let result = transport.close();
    // Assert
    assert!(result.is_err());
    assert!(transport.connected);
}
#[test]
fn bounded_transport_does_not_return_late_record_early() {
    // Arrange
    let mut transport = VirtualTransport::new(4);
    transport.connect().expect("connect");
    transport.latency_ms = 100;
    transport.send(0, &[1, 2, 3, 4]).expect("send");
    // Act
    let early = transport.receive(99);
    let saturation = transport.send(99, &[5]);
    let ready = transport.receive(100);
    // Assert
    assert_eq!(early, Ok(None));
    assert!(saturation.is_err());
    assert_eq!(ready, Ok(Some(vec![1, 2, 3, 4])));
}

#[test]
fn display_horizontal_addressing_wraps_within_declared_window() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics::default()).expect("valid");
    devices
        .display_write(0, &[0x20, 0, 0x21, 3, 4, 0x22, 1, 2])
        .expect("horizontal window");
    // Act
    devices
        .display_write(0x40, &[1, 2, 3, 4, 5])
        .expect("pixels");
    // Assert
    assert_eq!(
        (
            devices.display[131],
            devices.display[132],
            devices.display[259],
            devices.display[260]
        ),
        (5, 2, 3, 4)
    );
}
#[test]
fn display_initialization_retains_configuration_parameters() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics::default()).expect("valid");
    // Act
    devices
        .display_write(
            0,
            &[
                0xae, 0xd5, 0x80, 0xa8, 63, 0xd3, 0, 0x40, 0x8d, 0x14, 0x20, 0, 0xa1, 0xc8, 0xda,
                0x12, 0x81, 0x7f, 0xd9, 0xf1, 0xdb, 0x40, 0xa4, 0xa6, 0xaf,
            ],
        )
        .expect("SSD1306 initialization");
    // Assert
    assert!(devices.display_enabled);
    assert_eq!(devices.display_configuration.get(&0x81), Some(&vec![0x7f]));
}

#[test]
fn submillisecond_thermal_fraction_is_retained_over_time() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics {
        heating_millicelsius_per_second: 500,
        ..Dynamics::default()
    })
    .expect("valid");
    devices.power_enabled = true;
    devices.reset_asserted = false;
    // Act
    devices.advance_to(1000, true).expect("advance");
    // Assert
    assert_eq!(devices.sample.temperature_millicelsius, 25500);
}

#[test]
fn internal_emc_raw_byte_roundtrips_real_ultra205_driver_offset() {
    use bitaxe_safety::sensor_acquisition::{
        apply_ultra205_emc2101_temperature_offset, decode_emc2101_internal_temperature,
    };
    // Arrange
    let mut devices = Peripherals::new(Dynamics {
        ambient_millicelsius: 45000,
        ..Dynamics::default()
    })
    .expect("valid");
    devices.advance_to(100, false).expect("publication");
    // Act
    let raw = devices.read_register(0x4c, 0x00, 1).expect("internal raw")[0];
    let observed = apply_ultra205_emc2101_temperature_offset(
        decode_emc2101_internal_temperature(raw).expect("driver decode"),
    )
    .expect("driver calibration");
    // Assert
    assert_eq!(raw, 40);
    assert_eq!(observed, 45.0);
    assert_eq!(devices.sample.temperature_millicelsius, 45000);
}
#[test]
fn external_emc_raw_domain_is_unchanged_by_internal_calibration() {
    use bitaxe_safety::sensor_acquisition::decode_emc2101_external_temperature;
    // Arrange
    let mut devices = Peripherals::new(Dynamics {
        ambient_millicelsius: 45125,
        ..Dynamics::default()
    })
    .expect("valid");
    devices.advance_to(100, false).expect("publication");
    // Act
    let msb = devices.read_register(0x4c, 0x01, 1).expect("external MSB")[0];
    let lsb = devices.read_register(0x4c, 0x10, 1).expect("external LSB")[0];
    // Assert
    assert_eq!(decode_emc2101_external_temperature([msb, lsb]), Ok(45.125));
}
#[test]
fn reboot_retains_old_sample_epoch_but_marks_it_unavailable() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.advance_to(100).expect("sample");
    let before = board.peripherals.sample;
    // Act
    board.reset(ResetReason::Software).expect("reset");
    // Assert
    assert_eq!(board.peripherals.sample, before);
    assert_eq!(board.storage.boot_ordinal, 2);
    assert!(!board.peripherals.sample_valid);
    assert_eq!(
        board.peripherals.read_register(0x4c, 0x00, 1),
        Err(ModelError::Unavailable("sensor publication unavailable"))
    );
}
#[test]
fn first_new_publication_is_bound_to_new_boot() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.advance_to(100).expect("sample");
    board.reset(ResetReason::Software).expect("reset");
    // Act
    board.advance_to(200).expect("new sample");
    // Assert
    assert!(board.peripherals.sample_valid);
    assert_eq!(board.peripherals.sample.boot_ordinal, 2);
    assert_eq!(board.peripherals.sample.sampled_at_ms, 200);
}
#[test]
fn blocked_publication_after_reboot_cannot_retell_old_sample_as_fresh() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.advance_to(100).expect("sample");
    board.reset(ResetReason::Panic).expect("reset");
    board.peripherals.sensor_publication_blocked = true;
    // Act
    board.advance_to(1000).expect("time");
    // Assert
    assert!(!board.peripherals.sample_valid);
    assert_eq!(board.peripherals.sample.boot_ordinal, 1);
    assert_eq!(board.peripherals.sample.sampled_at_ms, 100);
}
#[test]
fn reboot_destroys_both_endpoints_and_queued_records_even_if_close_rejects() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.control.connect().expect("control");
    board.network.connect().expect("network");
    board.control.send(0, &[1]).expect("queue");
    board.network.send(0, &[2]).expect("queue");
    board.control.reject_close = true;
    // Act
    board.reset(ResetReason::Panic).expect("reset");
    // Assert
    assert!(!board.control.connected);
    assert!(!board.network.connected);
    assert_eq!(board.control.queued_bytes(), 0);
    assert_eq!(board.network.queued_bytes(), 0);
    board.control.connect().expect("new control");
    assert_eq!(board.control.receive(0), Ok(None));
}
#[test]
fn reboot_preserves_physical_fan_and_thermal_inertia() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig {
        dynamics: Dynamics {
            heating_millicelsius_per_second: 10000,
            cooling_millicelsius_per_second: 1000,
            ..Dynamics::default()
        },
        ..BoardConfig::default()
    })
    .expect("board");
    board.peripherals.fan_percent = 100;
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.maybe_last_payload = Some([0; 82]);
    board.advance_to(1000).expect("fan");
    let before = board.peripherals.sample;
    // Act
    board.reset(ResetReason::Software).expect("reset");
    board.advance_to(1100).expect("new observation");
    // Assert
    assert_eq!(board.peripherals.sample.fan_rpm, before.fan_rpm);
    assert!(before.temperature_millicelsius > 25000);
    assert_eq!(
        board.peripherals.sample.temperature_millicelsius,
        before.temperature_millicelsius - 100
    );
    assert!(board.peripherals.sample.fan_rpm > 0);
}

#[test]
fn unrepresentable_thermal_curve_inputs_are_rejected_before_board_construction() {
    // Arrange
    let dynamics = Dynamics {
        cooling_millicelsius_per_second: i32::MAX,
        ..Dynamics::default()
    };
    // Act
    let result = Peripherals::new(dynamics);
    // Assert
    assert!(matches!(result, Err(ModelError::Invalid("dynamics range"))));
}
#[test]
fn supported_maximum_thermal_rate_does_not_overflow_fan_multiplication() {
    // Arrange
    let mut devices = Peripherals::new(Dynamics {
        cooling_millicelsius_per_second: 1_000_000,
        ..Dynamics::default()
    })
    .expect("supportedrate");
    devices.fan_percent = 100;
    // Act
    devices.advance_to(100, false).expect("wide arithmetic");
    // Assert
    assert_eq!(devices.sample.temperature_millicelsius, 25000);
}
