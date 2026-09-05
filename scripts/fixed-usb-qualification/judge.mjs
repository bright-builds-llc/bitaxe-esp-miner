import { canonicalBase64, exactObject, requireCondition, WINDOW_MS } from "./contract.mjs";

const STAGES = ["not_started", "stop_dispatch", "reduce_frequency_and_reset_nonce", "hold_reset_low",
  "disable_core_voltage", "disable_asic", "fan_full", "cooling_proof", "fan_paused"];
const STATUSES = ["unconfigured", "configured", "ready", "window_loaded", "running", "stopping", "baseline_confirmed",
  "closing", "closed", "failed", "restoration_unconfirmed", "disconnected"];
const FAILURES = ["configuration_failed", "connect_failed", "prepare_failed", "start_failed", "load_failed", "stop_failed",
  "close_failed", "probe_failed", "suppress_failed", "local_input_invalid", "window_control_failed", "cleanup_failed"];
const COUNTS = ["generation", "active_ms", "generation_elapsed_ms", "budget_reserved_ms", "submitted", "accepted", "rejected",
  "nonce_work_correlations", "work_dispatched", "last_valid_heartbeat_ms"];
const BOOLEANS = ["budget_complete", "safe_stop_complete", "voltage_fresh", "power_fresh", "temperature_fresh", "fan_fresh", "watchdog_alive", "mine_on_boot"];
const NUMBERS = ["voltage_volts", "power_watts", "chip_temp_celsius", "fan_rpm"];
export const u32 = (value) => Number.isInteger(value) && value >= 0 && value <= 0xffffffff;

export function validateQualification(value) {
  exactObject(value, ["schema", ...COUNTS, ...BOOLEANS, ...NUMBERS, "gate_closed_ms", "shutdown_started_ms", "safe_stop_stage", "revocation_reason",
    "active_limit_ms", "shutdown_budget_ms", "work_gate_remaining_ms"]);
  requireCondition(value.schema === "worker-qualification-v1" && COUNTS.every((key) => u32(value[key])) &&
    BOOLEANS.every((key) => typeof value[key] === "boolean") && value.budget_reserved_ms <= 240000 &&
    STAGES.includes(value.safe_stop_stage), "qualification_shape");
  for (const key of ["gate_closed_ms", "shutdown_started_ms"]) requireCondition(value[key] === null || u32(value[key]), "timing_shape");
  requireCondition(u32(value.shutdown_budget_ms) && ["active_limit_ms", "work_gate_remaining_ms"].every((key) => value[key] === null || u32(value[key])), "budget_timing_shape");
  requireCondition(["none", "heartbeat_timeout", "lease_or_budget_expired", "restoration_requested", "unsafe_observation", "link_closed", "control_failed"].includes(value.revocation_reason), "revocation_reason_shape");
  for (const key of NUMBERS) requireCondition(value[key] === null || (typeof value[key] === "number" && Number.isFinite(value[key])), "numeric_shape");
  requireCondition(value.fan_rpm === null || (Number.isInteger(value.fan_rpm) && value.fan_rpm >= 0 && value.fan_rpm <= 65535), "rpm_shape");
  for (const [numeric, flag] of [["voltage_volts", "voltage_fresh"], ["power_watts", "power_fresh"],
    ["chip_temp_celsius", "temperature_fresh"], ["fan_rpm", "fan_fresh"]]) {
    requireCondition(value[flag] === (value[numeric] !== null), "freshness_shape");
  }
  return value;
}

export function validatePreservation(value) {
  exactObject(value, ["schema", "baseline_id", "settings_match", "authorization_high_water_match", "device_identity_match", "mine_on_boot"]);
  requireCondition(value.schema === "worker-preservation-continuity-v1" && canonicalBase64(value.baseline_id, 16) &&
    ["settings_match", "authorization_high_water_match", "device_identity_match"].every((key) => typeof value[key] === "boolean") &&
    typeof value.mine_on_boot === "boolean", "preservation_shape");
  return value;
}

export function validateState(value, context) {
  exactObject(value, ["schema", "gateCommit", "status", "connected", "running", "heartbeatSuppressed", "renewalsConfirmed", "deviceRestorationConfirmed", "deviceLeaseInactive", "serialOwnershipReleased"],
    ["expectedFirmwareSourceCommit", "expectedAppElfSha256", "qualification", "preservation", "probe", "failure", "admissionFailureStage"]);
  requireCondition(value.schema === "worker-serial-acceptance-v1" && value.gateCommit === context.gate_commit &&
    value.expectedFirmwareSourceCommit === context.firmware_commit && value.expectedAppElfSha256 === context.app_elf_sha256 &&
    STATUSES.includes(value.status) && u32(value.renewalsConfirmed) && value.renewalsConfirmed <= 16 &&
    ["connected", "running", "heartbeatSuppressed", "deviceRestorationConfirmed", "deviceLeaseInactive", "serialOwnershipReleased"].every((key) => typeof value[key] === "boolean"), "browser_state_identity");
  if (value.admissionFailureStage !== undefined) requireCondition(["ownership", "permission", "device_filter", "scope", "opening", "hello", "manifest_identity", "capability", "possession", "baseline", "continuity", "cleanup"].includes(value.admissionFailureStage), "admission_stage_shape");
  if (value.failure !== undefined) requireCondition(FAILURES.includes(value.failure), "browser_failure_shape");
  if (value.qualification !== undefined) validateQualification(value.qualification);
  if (value.preservation !== undefined) validatePreservation(value.preservation);
  if (value.probe !== undefined) {
    exactObject(value.probe, ["paddingBytes", "requestPayloadBytes", "responsePayloadBytes"]);
    requireCondition(Object.values(value.probe).every((count) => u32(count) && count <= 65536), "probe_shape");
  }
  return value;
}

export function judgeWindow(index, records, fault) {
  requireCondition(Number.isInteger(index) && index >= 0 && index < 3 && records.length > 0, "window_records_missing");
  const start = records.find((record) => record.state.running && record.state.qualification?.generation > 0 &&
    !record.state.qualification.safe_stop_complete);
  requireCondition(start !== undefined, "running_device_evidence_missing");
  const generation = start.state.qualification.generation;
  const expectedBudget = WINDOW_MS.slice(0, index + 1).reduce((sum, value) => sum + value, 0);
  const bound = records.filter((record) => record.state.qualification?.generation === generation &&
    record.state.qualification.budget_reserved_ms === expectedBudget);
  requireCondition(bound.length > 0, "campaign_budget_evidence_missing");
  const q = bound.at(-1).state.qualification;
  requireCondition(bound.at(-1).state.deviceRestorationConfirmed && bound.at(-1).state.deviceLeaseInactive, "device_restoration_ack_missing");
  requireCondition(q.safe_stop_complete && !q.mine_on_boot && q.gate_closed_ms !== null && q.shutdown_started_ms !== null,
    "qualified_stop_missing");
  requireCondition(q.safe_stop_stage === "fan_paused" && q.temperature_fresh && q.chip_temp_celsius <= 45 &&
    q.fan_fresh && q.fan_rpm > 0, "terminal_cooling_proof_missing");
  requireCondition(q.active_ms > 0 && q.active_ms <= q.generation_elapsed_ms && q.active_ms <= WINDOW_MS[index] &&
    q.active_limit_ms === WINDOW_MS[index] && q.shutdown_budget_ms === 15550 &&
    q.submitted >= q.accepted + q.rejected && q.work_dispatched > 0 && q.nonce_work_correlations > 0, "mining_evidence_missing");
  if (index === 0) {
    requireCondition(bound.some((record) => record.state.renewalsConfirmed >= 1) && fault === undefined &&
      records.every((record) => record.state.failure === undefined), "foreground_window_incomplete");
  }
  if (index > 0) requireCondition(fault?.window === index && fault.kind === (index === 1 ? "visibility_hidden" : "heartbeats_suppressed"), "fault_observation_missing");
  if (index > 0) {
    const checkpoint = records.find((record) => record.sequence === fault.after_sequence);
    requireCondition(fault.generation === generation && checkpoint?.state.running === true &&
      checkpoint.state.qualification?.generation === generation && checkpoint.state.qualification.gate_closed_ms === null &&
      checkpoint.state.qualification.work_gate_remaining_ms > 3000 &&
      bound.at(-1).sequence > fault.after_sequence && (index !== 2 || checkpoint.state.heartbeatSuppressed), "fault_generation_binding");
  }
  if (index === 2) requireCondition(q.revocation_reason === "heartbeat_timeout", "heartbeat_timeout_not_proven");
  if (index === 2) requireCondition(q.budget_complete === true, "campaign_budget_incomplete");
  if (index === 1) requireCondition(["heartbeat_timeout", "restoration_requested", "link_closed"].includes(q.revocation_reason), "visibility_stop_not_proven");
  const gateDelay = (q.gate_closed_ms - q.last_valid_heartbeat_ms) >>> 0;
  const stopDelay = (q.shutdown_started_ms - q.last_valid_heartbeat_ms) >>> 0;
  requireCondition(gateDelay <= 3000 && stopDelay <= 3000, "revocation_deadline_missed");
  for (const record of bound.filter((entry) => entry.state.running && !entry.state.qualification.safe_stop_complete)) {
    const sample = record.state.qualification;
    requireCondition(sample.voltage_fresh && sample.power_fresh && sample.temperature_fresh && sample.fan_fresh &&
      sample.voltage_volts >= 4.5 && sample.voltage_volts <= 5.5 && sample.power_watts >= 0 && sample.power_watts <= 15 &&
      sample.chip_temp_celsius < 75 && sample.fan_rpm > 0 && sample.watchdog_alive && !sample.mine_on_boot, "safety_evidence_failed");
  }
  return { schema: "fixed-usb-window-report-v1", window: index, generation, active_ms: q.active_ms,
    budget_reserved_ms: q.budget_reserved_ms, accepted: q.accepted, rejected: q.rejected,
    submitted: q.submitted, nonce_work_correlations: q.nonce_work_correlations,
    gate_close_delay_ms: gateDelay, shutdown_start_delay_ms: stopDelay,
    accepted_share_verified: q.accepted > 0,
    unverified_reason: index === 0 && q.accepted === 0 ? "no_accepted_share_within_budget" : null,
    browser_report_accepted: true, hardware_execution_claimed_by_supervisor: false };
}

export function validateCycle(value, context, previous) {
  const flags = ["browser_released", "flash_success", "runtime_identity_match", "cleanup_complete", "device_identity_match", "settings_match", "authorization_high_water_match"];
  exactObject(value, ["schema", "cycle", "firmware_commit", "app_elf_sha256", "baseline_id", ...flags, "probe_request_bytes", "probe_response_bytes", "mine_on_boot"]);
  requireCondition(value.schema === "fixed-usb-cycle-report-v1" && value.cycle === (previous?.cycle ?? 0) + 1 && value.cycle <= 20 &&
    value.firmware_commit === context.firmware_commit && value.app_elf_sha256 === context.app_elf_sha256 &&
    canonicalBase64(value.baseline_id, 16) && flags.every((key) => value[key] === true) &&
    value.probe_request_bytes === 65536 && value.probe_response_bytes === 65536 && value.mine_on_boot === false, "cycle_evidence_failed");
  requireCondition(!previous || previous.baseline_id === value.baseline_id, "cycle_baseline_changed");
  return value;
}
