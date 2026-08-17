export const sourceFragments = new Map<string, readonly string[]>([
  ["crates/bitaxe-core/src/hashrate.rs", [
    "const HASHRATE_REGISTER_UNIT_HASHES: f64 = 1_048_576.0;",
    "const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;",
    "const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;",
  ]],
  ["crates/bitaxe-core/src/runtime_health.rs", [
    "Some(\"snapshot_retry_exhausted\")",
    "if maybe_previous.is_some_and(|previous| !latest.is_valid_after(previous)) {",
    "let Some(age_millis) = now_millis.checked_sub(observed_at_millis) else {",
  ]],
  ["crates/bitaxe-core/src/runtime_health/wait.rs", [
    "pub enum TaskWatchdogReadOutcome {",
    "pub enum TaskWatchdogWaitState {",
    "pub const fn state_at(self, current_monotonic_millis: u64)",
  ]],
  ["crates/bitaxe-stratum/src/v1/state.rs", ["pub hashrate_inputs: HashrateInputs"]],
  ["crates/bitaxe-stratum/src/v1/production_session/campaign.rs", [
    "Self::Conservative => (400, 1_100, 100)",
    "core_voltage_mv: i64,",
  ]],
  ["crates/bitaxe-api/src/mining.rs", [
    "hash_rate: hashrate.current_ghs,",
    "hashrate_monitor: HashrateMonitorWire {",
  ]],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub bus_voltage_volts: Observation<f64>,",
    "let min_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);",
    "(min_input_voltage..=max_input_voltage).contains(&bus_voltage_volts)",
  ]],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "hashRate")]',
    '#[serde(rename = "hashrateMonitor")]',
  ]],
  ["crates/bitaxe-api/src/wire/runtime_health.rs", [
    'rename = "taskWatchdogReadOutcome"',
    "task_watchdog_read_outcome: snapshot",
    '#[serde(rename = "taskWatchdogWaitState", default = "invalid_wait_state")]',
    "task_watchdog_wait_state: snapshot.task_watchdog_wait_state().as_str().to_owned(),",
  ]],
  ["firmware/bitaxe/src/production_mining_session/hashrate.rs", [
    "const HASHRATE_CADENCE_MS: u64 = 1_000;",
    "const BM1366_HASH_DOMAIN_COUNT: usize = 4;",
  ]],
  ["firmware/bitaxe/src/production_mining_session/asic_worker.rs", [
    "request_hashrate_monitor_register_reads_tx()",
    "emit(AsicWorkerEvent::RegisterRead {",
  ]],
  ["firmware/bitaxe/src/runtime_snapshot.rs", ["publish_hashrate_snapshot"]],
  ["firmware/bitaxe/src/runtime_health_adapter.rs", [
    "let task_watchdog = crate::task_watchdog_observation::coherent_observation();",
    "let current_monotonic_millis = crate::runtime_uptime::millis();",
  ]],
  ["firmware/bitaxe/src/production_mining_session/owner_loop.rs", [
    "if let Err(error) = adapter.publish_campaign_status",
    "record_owner_phase(TaskWatchdogOwnerPhase::ServicingHashrate)",
  ]],
  ["firmware/bitaxe/src/production_mining_session/campaign_status/publication.rs", [
    "CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS: u64 = 1_000",
    "pub(crate) struct CampaignStatusPublicationSchedule {",
  ]],
  ["firmware/bitaxe/src/task_watchdog_observation.rs", [
    "const COHERENT_READ_ATTEMPTS: usize = 8;",
    "TaskWatchdogReadOutcome::HistoryPoisoned",
    "TaskWatchdogReadOutcome::RetryExhausted",
    "publication_sequence: AtomicU32,",
    "pub(crate) fn coherent_observation()",
  ]],
  ["firmware/bitaxe/sdkconfig.defaults", [
    "CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5",
  ]],
  ["crates/bitaxe-safety/src/power.rs", [
    "pub const INPUT_VOLTAGE_NOMINAL_VOLTS: f64 = 5.0;",
    "pub const INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;",
  ]],
]);

export const referenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/tasks/hashrate_monitor_task.c", [
    "#define HASHRATE_UNIT 0x100000uLL",
    "#define POLL_RATE 1000",
    "#define HASHRATE_1M_SIZE (60000 / POLL_RATE)",
    "void update_hash_counter(measurement_t * measurement, uint32_t value, uint64_t time_us)",
    "ASIC_read_registers(GLOBAL_STATE);",
  ]],
  ["reference/esp-miner/components/stratum/utils.c", [
    "#define HASH_CNT_LSB 0x100000000uLL",
    "float hashCounterToGhs(uint64_t duration_us, uint32_t counter)",
  ]],
  ["reference/esp-miner/main/device_config.h", [
    ".default_voltage_mv = 1200,",
    "FAMILY_ULTRA       = { .id = ULTRA,       .name = \"Ultra\",      .asic = ASIC_BM1366,   .asic_count = 1, .max_power =  25, .power_offset = 5,  .nominal_voltage = 5,",
  ]],
  ["reference/esp-miner/main/tasks/power_management_task.c", [
    "uint16_t voltage = nvs_config_get_u16(NVS_CONFIG_ASIC_VOLTAGE);",
    "VCORE_set_voltage(GLOBAL_STATE, (double) voltage / 1000.0);",
  ]],
  ["reference/esp-miner/main/tasks/protocol_coordinator.c", [
    'xTaskCreateWithCaps(stratum_v1_task, "stratum v1", 8192, (void *)gs, 5,',
  ]],
]);
