#[path = "safety_adapter/request_queue.rs"]
mod safety_request_queue;

const OPERATOR_SENSOR_RUNTIME_SOURCE: &str = include_str!("operator_sensor_runtime.rs");
const FAN_CONTROLLER_PLAN_SOURCE: &str = include_str!("fan_controller_plan.rs");
const FAN_CONTROLLER_RUNTIME_SOURCE: &str = include_str!("fan_controller_runtime.rs");
const SAFETY_ADAPTER_SOURCE: &str = include_str!("safety_adapter.rs");
const SAFETY_WATCHDOG_SOURCE: &str = include_str!("safety_adapter/watchdog.rs");
const ADC_SOURCE: &str = include_str!("safety_adapter/adc.rs");
const OBSERVATION_STORE_SOURCE: &str = include_str!("safety_adapter/observation_store.rs");
const I2C_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus.rs");
const I2C_RETRY_SOURCE: &str = include_str!("safety_adapter/i2c_retry.rs");
const EMC2101_SOURCE: &str = include_str!("safety_adapter/emc2101.rs");
const DS4432U_SOURCE: &str = include_str!("safety_adapter/ds4432u.rs");
const MINING_ACTUATION_ADAPTER_SOURCE: &str = include_str!("mining_actuation_adapter.rs");
const PRODUCTION_ASIC_SOURCE: &str = include_str!("asic_adapter/production.rs");
const PRODUCTION_SESSION_SOURCE: &str = include_str!("production_mining_session.rs");
const PRODUCTION_OWNER_LOOP_SOURCE: &str =
    include_str!("production_mining_session/owner_loop.rs");
const PRODUCTION_NOTIFICATIONS_SOURCE: &str =
    include_str!("production_mining_session/notifications.rs");
const PENDING_OBSERVATION_SOURCE: &str =
    include_str!("production_mining_session/pending_observation.rs");
const PRODUCTION_TRANSPORT_SOURCE: &str =
    include_str!("production_mining_session/transport.rs");
const PRODUCTION_ASIC_WORKER_SOURCE: &str =
    include_str!("production_mining_session/asic_worker.rs");
const SETTINGS_ADAPTER_SOURCE: &str = include_str!("settings_adapter.rs");
const SETTINGS_PRODUCTION_SOURCE: &str = include_str!("settings_adapter/production.rs");
const THERMAL_FAULT_SETTINGS_SOURCE: &str =
    include_str!("settings_adapter/thermal_fault_stimulus.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn runtime_owner_startup_and_notification_order_is_explicit() {
    // Arrange
    let safety = STARTUP_SOURCE
        .find("safety_adapter::start_safety_supervisor();")
        .expect("startup must start the safety supervisor");
    let production = STARTUP_SOURCE
        .find("production_mining_session::start()")
        .expect("startup must start the production owner");
    let fan_controller = STARTUP_SOURCE
        .find("fan_controller_runtime::start()")
        .expect("startup must start the fan controller after the production owner");
    let network = STARTUP_SOURCE
        .find("wifi_adapter::start_wifi(modem)")
        .expect("startup must start the network owner");
    let network_wakeup = STARTUP_SOURCE
        .find("ProductionSessionWakeup::NetworkChanged")
        .expect("startup must notify the production owner after network admission");

    // Act / Assert
    assert!(safety < production);
    assert!(production < fan_controller);
    assert!(production < network);
    assert!(network < network_wakeup);
    assert_eq!(STARTUP_SOURCE.matches("operator_sensor_runtime::start(").count(), 1);
    assert_eq!(STARTUP_SOURCE.matches("production_mining_session::start()").count(), 1);
    assert_eq!(STARTUP_SOURCE.matches("start_safety_supervisor();").count(), 1);
    assert_eq!(
        STARTUP_SOURCE
            .matches("fan_controller_runtime::start()")
            .count(),
        1
    );
}

#[test]
fn fan_controller_uses_the_pure_plan_and_typed_owner_queue_only() {
    // Arrange / Act / Assert
    assert!(FAN_CONTROLLER_PLAN_SOURCE.contains("FanControlDecision::from_inputs"));
    assert!(FAN_CONTROLLER_PLAN_SOURCE.contains("FAN_CONTROLLER_CADENCE_MS"));
    assert!(FAN_CONTROLLER_PLAN_SOURCE.contains("hardware_control_not_qualified"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("request_safety_actuation"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("SafetyActuationCommand::SetFanDuty"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("fan_controller_actuation_qualified"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("current_settings_snapshot"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("observation_snapshot"));
    assert!(FAN_CONTROLLER_RUNTIME_SOURCE.contains("record_apply_failure"));
    assert!(!FAN_CONTROLLER_RUNTIME_SOURCE.contains("RuntimeI2cOwner"));
    assert!(!FAN_CONTROLLER_RUNTIME_SOURCE.contains("write_emc2101"));
    assert!(!FAN_CONTROLLER_RUNTIME_SOURCE.contains("Emc2101WriteRegister"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("FAN_CONTROLLER_ACTUATION_QUALIFIED"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("MiningCampaignState::Active"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("production_handle_available"));
    assert!(PRODUCTION_SESSION_SOURCE.contains(
        "FAN_CONTROLLER_ACTUATION_QUALIFIED.store(false, Ordering::Release)"
    ));
}

#[test]
fn runtime_owners_use_bounded_shared_cadence_and_queue_contracts() {
    // Arrange / Act / Assert
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains(
        "PeriodicDeadline::new(started_at_ms, SENSOR_SWEEP_CADENCE_MS)"
    ));
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains("PRODUCER_THREAD_PRIORITY: u32 = 10"));
    assert_eq!(
        OPERATOR_SENSOR_RUNTIME_SOURCE
            .matches("vTaskPrioritySet(core::ptr::null_mut(), PRODUCER_THREAD_PRIORITY)")
            .count(),
        1
    );
    assert!(!PRODUCTION_ASIC_WORKER_SOURCE.contains("vTaskPrioritySet"));
    assert!(OPERATOR_SENSOR_RUNTIME_SOURCE.contains("advance.missed_slots()"));
    assert!(!OPERATOR_SENSOR_RUNTIME_SOURCE.contains("fn next_future_deadline"));
    assert!(SAFETY_WATCHDOG_SOURCE.contains(
        "PeriodicDeadline::new(current_monotonic_millis(), SAFETY_SUPERVISOR_CADENCE_MS)"
    ));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE
        .contains("PeriodicDeadline::new(0, PRODUCTION_REREAD_CADENCE_MS)"));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("readiness_schedule.is_due(schedule_now_ms)"));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("let now_ms = crate::runtime_uptime::millis()"));
    assert!(PRODUCTION_SESSION_SOURCE.contains("mpsc::sync_channel(NOTIFICATION_CAPACITY)"));
    assert!(PRODUCTION_ASIC_WORKER_SOURCE.contains("mpsc::sync_channel(COMMAND_CAPACITY)"));
    assert!(PRODUCTION_ASIC_WORKER_SOURCE.contains(
        "executor.try_read_production_result(&valid_jobs, slice_ms)"
    ));
    assert!(PRODUCTION_ASIC_WORKER_SOURCE.contains(
        "emit(AsicWorkerEvent::Result { generation, result })"
    ));
    assert!(PRODUCTION_SESSION_SOURCE.contains("ProductionSessionEvent::AsicResult"));
}

#[test]
fn operator_sensor_runtime_is_the_single_normal_acquisition_caller() {
    // Arrange
    let required_calls = [
        "safety_adapter::read_power_acquisition(owner, budget)",
        "safety_adapter::read_asic_temperature_acquisition(owner, budget)",
        "safety_adapter::read_tachometer_acquisition(owner, budget)",
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
fn thermal_fault_stimulus_is_consumed_once_before_the_sensor_owner_starts() {
    // Arrange
    let load = STARTUP_SOURCE
        .find("settings_adapter::load_thermal_fault_stimulus()")
        .expect("startup must consume the private stimulus tuple");
    let start = STARTUP_SOURCE
        .find("operator_sensor_runtime::start(")
        .expect("startup must start the single sensor owner");

    // Act / Assert
    assert!(load < start);
    assert_eq!(
        OPERATOR_SENSOR_RUNTIME_SOURCE
            .matches("stimulus.step(prior, actual)")
            .count(),
        1
    );
    let erase = THERMAL_FAULT_SETTINGS_SOURCE
        .find("erase_tuple(&writable)?")
        .expect("the tuple must be erased before admission");
    let confirm = THERMAL_FAULT_SETTINGS_SOURCE
        .find("confirm_erased(partition)?")
        .expect("tuple absence must be confirmed before admission");
    let validate = THERMAL_FAULT_SETTINGS_SOURCE
        .find("let kind = kind?.ok_or_else")
        .expect("the consumed tuple must be validated after erasure");
    assert!(erase < confirm && confirm < validate);
}

#[test]
fn core_voltage_adc_has_one_semantic_producer_and_exact_ultra205_configuration() {
    // Arrange
    let producer_call = "safety_adapter::read_core_voltage_acquisition";

    // Act / Assert
    assert_eq!(OPERATOR_SENSOR_RUNTIME_SOURCE.matches(producer_call).count(), 1);
    assert_eq!(SAFETY_ADAPTER_SOURCE.matches("adc.read_millivolts()").count(), 1);
    assert!(ADC_SOURCE.contains("ADCCH1"));
    assert!(ADC_SOURCE.contains("Gpio2"));
    assert!(ADC_SOURCE.contains("attenuation::DB_12"));
    assert!(ADC_SOURCE.contains("resolution: Resolution::new()"));
    assert!(ADC_SOURCE.contains("calibration: Calibration::Curve"));
    assert_eq!(
        STARTUP_SOURCE
            .matches("Ultra205CoreVoltageAdc::new")
            .count(),
        1
    );
    assert!(!PRODUCTION_SESSION_SOURCE.contains(producer_call));
}

#[test]
fn raw_sensor_bus_capability_is_private_to_the_safety_facade() {
    // Arrange
    let expected_facade_reads = 3;

    // Act / Assert
    assert_eq!(
        SAFETY_ADAPTER_SOURCE
            .matches("owner.sensors(budget)")
            .count(),
        expected_facade_reads
    );
    assert!(I2C_BUS_SOURCE.contains("pub(super) fn sensors"));
    assert!(!OPERATOR_SENSOR_RUNTIME_SOURCE.contains("owner.sensors()"));
    assert!(!PRODUCTION_SESSION_SOURCE.contains("owner.sensors()"));
}

#[test]
fn every_runtime_i2c_capability_shares_the_sensor_publication_deadline() {
    // Arrange
    let expected_runtime_transfer_shapes = 3;

    // Act / Assert
    assert_eq!(I2C_BUS_SOURCE.matches("retry_driver_transfer(||").count(), 1);
    assert_eq!(
        I2C_BUS_SOURCE
            .matches("retry_runtime_driver_transfer(")
            .count(),
        expected_runtime_transfer_shapes
    );
    assert!(I2C_BUS_SOURCE.contains("retry_runtime_transfer"));
    assert!(I2C_BUS_SOURCE.contains("pub(crate) fn display<'bus, 'budget>"));
    assert!(I2C_BUS_SOURCE.contains("pub(super) fn sensors<'bus, 'budget>"));
    assert!(I2C_BUS_SOURCE.contains("pub(super) fn actuators<'bus, 'budget>"));
    assert!(I2C_RETRY_SOURCE.contains("I2C_TRANSACTION_TIMEOUT_MS: u64 = 500"));
    assert!(I2C_RETRY_SOURCE.contains("I2C_RETRY_COUNT: usize = 3"));
    assert!(I2C_RETRY_SOURCE.contains("I2C_RETRY_DELAY_MS: u32 = 10"));
    assert!(!I2C_BUS_SOURCE.contains("I2C_TRANSACTION_TIMEOUT_MS: u64 = 50"));
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
        SAFETY_ADAPTER_SOURCE
            .matches("owner.actuators(budget)")
            .count(),
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
    // Act / Assert
    assert!(set_fan_full.contains("queue_safety_actuation"));
    assert!(!set_fan_full.contains("request_safety("));
}

#[test]
fn observation_publication_releases_storage_before_owner_wakeup() {
    // Arrange
    let producer_start = OBSERVATION_STORE_SOURCE
        .find("pub(crate) fn replace_observations_from_producer")
        .expect("observation producer function");
    let producer = &OBSERVATION_STORE_SOURCE[producer_start..];

    // Act
    let replace = producer.find("store.replace(observations);").expect("replace");
    let release = producer.find("drop(store);").expect("release");
    let wakeup = producer
        .find("production_mining_session::notify")
        .expect("owner wakeup");

    // Assert
    assert!(replace < release);
    assert!(release < wakeup);
}

#[test]
fn a_full_owner_queue_cannot_discard_the_fresh_observation_wakeup() {
    // Arrange / Act / Assert
    assert!(PRODUCTION_NOTIFICATIONS_SOURCE.contains("OBSERVATIONS_CHANGED_PENDING"));
    assert!(PRODUCTION_NOTIFICATIONS_SOURCE.contains("OBSERVATIONS_CHANGED_PENDING.mark()"));
    assert!(PENDING_OBSERVATION_SOURCE.contains("swap(false, Ordering::AcqRel)"));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("take_pending_observations_changed()"));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("ProductionSessionWakeup::ObservationsChanged"));
}

#[test]
fn production_safe_stop_binds_the_typed_pause_purpose_without_sensor_waiting() {
    // Arrange / Act / Assert
    assert!(PRODUCTION_SESSION_SOURCE.contains(
        "ProductionSessionEffect::SafeStopHardware { lease_id, purpose }"
    ));
    assert!(PRODUCTION_SESSION_SOURCE
        .contains(".safe_stop(purpose, &mut safe_stop_progress)"));
    assert!(MINING_ACTUATION_ADAPTER_SOURCE
        .contains("execute_safe_stop_with_progress(self, purpose, progress)"));
    assert!(MINING_ACTUATION_ADAPTER_SOURCE
        .contains("self.wait_for_cooling_proof_with_progress(progress)"));
}

#[test]
fn long_frequency_shutdown_reports_progress_before_each_typed_asic_action() {
    // Arrange
    let progress = PRODUCTION_ASIC_SOURCE
        .find("progress();")
        .expect("ASIC action progress boundary");
    let execute = PRODUCTION_ASIC_SOURCE
        .find("match super::interpret_action(action, uart, reset)")
        .expect("ASIC action execution");

    // Act / Assert
    assert!(progress < execute);
    assert!(MINING_ACTUATION_ADAPTER_SOURCE
        .contains("execute_safe_shutdown_actions_with_progress"));
    assert!(MINING_ACTUATION_ADAPTER_SOURCE.contains("let mut action_progress = || progress(step)"));
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
