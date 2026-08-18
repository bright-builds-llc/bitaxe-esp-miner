import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { toolProgram } from "./cli-tools.js";
import type { HashrateMonitorEvidenceOptions } from "./hashrate-monitor-evidence.js";

export const sourceCommit = "a".repeat(40);
export const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
export const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
export const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();
export const validatorProgram = toolProgram(
  workspace,
  "crates/bitaxe-automation-contracts/validate_hashrate_monitor_evidence",
);
const sourceDocuments = new Map<string, string>([
  ["crates/bitaxe-core/src/hashrate.rs", [
    "const HASHRATE_REGISTER_UNIT_HASHES: f64 = 1_048_576.0;",
    "const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;",
    "const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;",
  ].join("\n")],
  ["crates/bitaxe-core/src/runtime_health.rs", [
    "Some(\"snapshot_retry_exhausted\")",
    "if maybe_previous.is_some_and(|previous| !latest.is_valid_after(previous)) {",
    "let Some(age_millis) = now_millis.checked_sub(observed_at_millis) else {",
  ].join("\n")],
  ["crates/bitaxe-core/src/runtime_health/wait.rs", [
    "pub enum TaskWatchdogReadOutcome {",
    "pub enum TaskWatchdogOwnerSubphase {",
    "pub enum TaskWatchdogWaitState {",
    "pub const fn state_at(self, current_monotonic_millis: u64)",
  ].join("\n")],
  ["crates/bitaxe-stratum/src/v1/state.rs", "pub hashrate_inputs: HashrateInputs"],
  ["crates/bitaxe-stratum/src/v1/production_session/campaign.rs", [
    "Self::Conservative => (400, 1_100, 100)",
    "core_voltage_mv: i64,",
  ].join("\n")],
  ["crates/bitaxe-api/src/mining.rs", [
    "hash_rate: hashrate.current_ghs,",
    "hashrate_monitor: HashrateMonitorWire {",
  ].join("\n")],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub bus_voltage_volts: Observation<f64>,",
    "let min_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);",
    "(min_input_voltage..=max_input_voltage).contains(&bus_voltage_volts)",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "hashRate")]',
    '#[serde(rename = "hashrateMonitor")]',
  ].join("\n")],
  ["crates/bitaxe-api/src/wire/runtime_health.rs", [
    'rename = "taskWatchdogReadOutcome"',
    'rename = "taskWatchdogOwnerSubphase"',
    "task_watchdog_read_outcome: snapshot",
    '#[serde(rename = "taskWatchdogWaitState", default = "invalid_wait_state")]',
    "task_watchdog_wait_state: snapshot.task_watchdog_wait_state().as_str().to_owned(),",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/hashrate.rs", [
    "const HASHRATE_CADENCE_MS: u64 = 1_000;",
    "const BM1366_HASH_DOMAIN_COUNT: usize = 4;",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/asic_worker.rs", [
    "request_hashrate_monitor_register_reads_tx()",
    "emit(AsicWorkerEvent::RegisterRead {",
  ].join("\n")],
  ["firmware/bitaxe/src/runtime_snapshot.rs", "publish_hashrate_snapshot"],
  ["firmware/bitaxe/src/runtime_health_adapter.rs", [
    "let task_watchdog = crate::task_watchdog_observation::coherent_observation();",
    "let current_monotonic_millis = crate::runtime_uptime::millis();",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/owner_loop.rs", [
    "if let Err(error) = adapter.publish_campaign_status",
    "OwnerProgressBoundary::EventStarted",
    "task_watchdog.feed_owner_progress(now_millis, subphase);",
    "record_owner_phase(TaskWatchdogOwnerPhase::ServicingHashrate)",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/campaign_status/publication.rs", [
    "CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS: u64 = 1_000",
    "pub(crate) struct CampaignStatusPublicationSchedule {",
  ].join("\n")],
  ["firmware/bitaxe/src/task_watchdog_observation.rs", [
    "state: Mutex<TaskWatchdogObservationState>",
    "TaskWatchdogReadOutcome::HistoryPoisoned",
    "let state = match self.state.lock()",
    "pub(crate) fn coherent_observation()",
  ].join("\n")],
  ["firmware/bitaxe/sdkconfig.defaults", "CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5"],
  ["crates/bitaxe-safety/src/power.rs", [
    "pub const INPUT_VOLTAGE_NOMINAL_VOLTS: f64 = 5.0;",
    "pub const INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;",
  ].join("\n")],
  ["tools/flash/src/campaign/serial.rs", [
    "self.process_panic_line(line, byte_offset);",
    "self.diagnostics.panic_signature = \"unknown\";",
  ].join("\n")],
  ["tools/flash/src/campaign/serial/diagnostics.rs", [
    'const DIAGNOSTICS_SCHEMA: &str = "mining-campaign-serial-diagnostics-v4";',
    "pub(super) panic_signature: &'static str,",
    "pub(super) panic_task_family: &'static str,",
    "pub(super) panic_signature_count: u64,",
  ].join("\n")],
  ["tools/flash/src/campaign/serial/panic.rs", [
    "pub(super) enum PanicSignature {",
    "pub(super) enum PanicTaskFamily {",
    "pub(super) fn classify_panic_line(line: &[u8])",
  ].join("\n")],
  ["tools/flash/src/campaign/network/terminal_settlement.rs", [
    "pub(super) const fn terminal_settlement(",
  ].join("\n")],
  ["tools/flash/src/campaign/network/observer.rs", [
    "TerminalSettlementDecision::RequestSerialClose => request_serial_close(&shared),",
  ].join("\n")],
  ["tools/flash/src/campaign/network/model.rs", [
    "terminal_settlement: self.terminal_settlement,",
  ].join("\n")],
  ["tools/flash/src/campaign/network/model/evidence.rs", [
    "pub(in crate::campaign) final_terminal_consumed: bool,",
  ].join("\n")],
]);

export const okResult = {
  schema: "mining-campaign-result-v16",
  status: "accepted",
  terminal_category: "submit_response_observed",
  stage: "live-share",
  profile: "conservative",
  duration_seconds: 600,
  runtime_identity: "trusted",
  safe_stop: "confirmed",
  usb_cleanup: "ready",
  watchdog_failure: "none",
  watchdog_read_outcome: "stable",
  watchdog_owner_phase: "waiting_inbox",
  watchdog_owner_subphase: "unavailable",
  watchdog_wait_state: "within_deadline",
  runtime_attestation_parse_failure: "none",
  runtime_attestation_parse_failure_counts: {
    missing_marker: 0,
    malformed_token: 0,
    duplicate_field: 0,
    unknown_field: 0,
    missing_field: 0,
    invalid_field: 0,
    incomplete_readiness: 0,
  },
};
export const okDiagnostics = {
  schema: "mining-campaign-serial-diagnostics-v4",
  runtime_attestation_mixed_reset_reason: "none",
  panic_signature: "none",
  panic_task_family: "none",
  panic_signature_count: 0,
};

export type Fixture = {
  readonly root: string;
  readonly planSha256: string;
  readonly options: HashrateMonitorEvidenceOptions;
};

function sha256(document: string): string {
  return createHash("sha256").update(document).digest("hex");
}

async function writeProtected(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

export async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-hashrate-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  for (const [relative, document] of sourceDocuments) {
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${document}\n`);
  }
  const reference = path.join(root, "reference/esp-miner/main/tasks/hashrate_monitor_task.c");
  await mkdir(path.dirname(reference), { recursive: true });
  await writeFile(reference, [
    "#define HASHRATE_UNIT 0x100000uLL",
    "#define POLL_RATE 1000",
    "#define HASHRATE_1M_SIZE (60000 / POLL_RATE)",
    "void update_hash_counter(measurement_t * measurement, uint32_t value, uint64_t time_us)",
    ...Array.from({ length: 7 }, () => "update_hash_counter"),
    "ASIC_read_registers(GLOBAL_STATE);",
  ].join("\n"));
  const counterReference = path.join(root, "reference/esp-miner/components/stratum/utils.c");
  await mkdir(path.dirname(counterReference), { recursive: true });
  await writeFile(counterReference, [
    "#define HASH_CNT_LSB 0x100000000uLL",
    "float hashCounterToGhs(uint64_t duration_us, uint32_t counter)",
  ].join("\n"));
  const deviceReference = path.join(root, "reference/esp-miner/main/device_config.h");
  await mkdir(path.dirname(deviceReference), { recursive: true });
  await writeFile(deviceReference, [
    ".default_voltage_mv = 1200,",
    "FAMILY_ULTRA       = { .id = ULTRA,       .name = \"Ultra\",      .asic = ASIC_BM1366,   .asic_count = 1, .max_power =  25, .power_offset = 5,  .nominal_voltage = 5,",
  ].join("\n"));
  const powerReference = path.join(
    root,
    "reference/esp-miner/main/tasks/power_management_task.c",
  );
  await writeFile(powerReference, [
    "uint16_t voltage = nvs_config_get_u16(NVS_CONFIG_ASIC_VOLTAGE);",
    "VCORE_set_voltage(GLOBAL_STATE, (double) voltage / 1000.0);",
  ].join("\n"));
  const coordinatorReference = path.join(
    root,
    "reference/esp-miner/main/tasks/protocol_coordinator.c",
  );
  await writeFile(coordinatorReference, [
    'xTaskCreateWithCaps(stratum_v1_task, "stratum v1", 8192, (void *)gs, 5,',
  ].join("\n"));
  const planRelative = "docs/parity/work-plans/20260818T050654Z-STAT-001/PLAN.md";
  const plan = "- Parity row: `STAT-001`\n- Active task: `task-parity-stat001-hashrate-monitor`\n";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-stat001-hashrate-monitor | fixture",
    `Plan: \`${planRelative}\`.`,
    "Attempt: `attempt-019`.",
  ].join("\n"));
  await writeFile(path.join(root, "TASKS.archive.md"), "# Archived tasks\n");
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  await writeFile(path.join(inputs, "package.json"), JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
  }));
  const wrapper = path.join(root, "scratch/stat001-hashrate-monitor/wrapper-019");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const output of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeProtected(path.join(wrapper, output), "");
  }
  return {
    root,
    planSha256: sha256(plan),
    options: {
      privateRoot: "scratch/stat001-hashrate-monitor/attempt-019",
      packageManifest: "inputs/package.json",
      wifiCredentials: "inputs/wifi.json",
      poolCredentials: "inputs/pool.json",
      detectorOutput: "scratch/stat001-hashrate-monitor/wrapper-019/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json",
      durationSeconds: 600,
      captureTimeoutSeconds: 30,
    },
  };
}
