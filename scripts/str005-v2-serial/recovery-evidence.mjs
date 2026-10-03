import { validateState } from "../fixed-usb-qualification/judge.mjs";
import { requireExhaustedOriginal, validateLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { parseStatus } from "./device.mjs";
import { parseDeviceRecord } from "./device-record.mjs";
import { boolean, check, nullable, object, uint } from "./values.mjs";
import { SAFETY_CATEGORIES, validateSafetyDiagnostic } from "./safety-diagnostics.mjs";

const stages = ["ledger", "original_budget", "state", "closed", "status", "diagnostics"];
const observationKeys = ["bootOrdinal", "workerGeneration", "serialTransportEpoch", "observedAtUs", "clockValid"];

function validateFinished(value) {
  object(value, ["failures"]);
  check(Array.isArray(value.failures) && value.failures.length <= stages.length &&
    value.failures.every(stage => stages.includes(stage)) && new Set(value.failures).size === value.failures.length,
  "recovery_finished_shape");
}

/** Independently revalidate persisted projections before deriving or sealing any result. */
export function validateRecoveryParts(parts, context) {
  check(parts !== null && typeof parts === "object" && !Array.isArray(parts) &&
    Object.keys(parts).every(stage => [...stages, "finished"].includes(stage)), "recovery_parts_shape");
  for (const [stage, value] of Object.entries(parts)) {
    if (stage === "finished") { validateFinished(value); continue; }
    if (stage === "diagnostics") {
      object(value, ["schema", "observations", "omitted_count", "authoritative"]);
      check(value.schema === "str005-recovery-diagnostics-v1" && value.authoritative === false, "recovery_diagnostics_shape");
      uint(value.omitted_count, 40);
      const projected = projectDiagnostics({ schema: "worker-diagnostic-export-v1", observations: value.observations });
      check(projected.omitted_count === 0 && value.observations.length + value.omitted_count <= 40, "recovery_diagnostics_shape");
      continue;
    }
    if (stage !== "status") { projectRecoveryPart(stage, value, context); continue; }
    object(value, ["schema", "scope", "state", "observation", "record"]);
    check(value.schema === "str005-recovery-status-v1" && value.scope === context.scope &&
      ["idle", "admitted", "running", "terminal"].includes(value.state), "recovery_status_shape");
    const observation = value.observation;
    object(observation, observationKeys);
    for (const key of ["bootOrdinal", "workerGeneration", "serialTransportEpoch"]) uint(observation[key]);
    boolean(observation.clockValid); nullable(observation.observedAtUs, uint);
    check(observation.bootOrdinal > 0 && observation.clockValid === (observation.observedAtUs !== null), "recovery_status_clock");
    if (value.state === "idle") {
      check(value.record === null && observation.clockValid, "recovery_status_idle");
      continue;
    }
    const record = parseDeviceRecord(value.record);
    check(record.scope === context.scope && record.attemptId === context.attemptId && record.state === value.state &&
      record.bootOrdinal === observation.bootOrdinal, "recovery_status_attempt");
    check(observation.clockValid || [record.firstFailure, ...record.secondaryFailures].some(failure => failure?.category === "clock"),
      "recovery_status_clock");
    if (observation.observedAtUs !== null && record.observedAtUs !== null)
      check(record.observedAtUs <= observation.observedAtUs, "recovery_status_clock");
  }
  return parts;
}

/** Retain boot and closed safety rows; unknown categories never cross persistence. */
function projectDiagnostics(value) {
  object(value, ["schema", "observations"]);
  check(value.schema === "worker-diagnostic-export-v1" && Array.isArray(value.observations) && value.observations.length <= 40,
    "recovery_diagnostics_shape");
  const observations = [];
  for (const row of value.observations) {
    check(row !== null && typeof row === "object" && !Array.isArray(row), "recovery_diagnostics_shape");
    if (SAFETY_CATEGORIES.includes(row.category)) { observations.push(validateSafetyDiagnostic(row)); continue; }
    if (row.category !== "boot") continue;
    object(row, ["category", "authoritative", "boot_ordinal", "reset_reason", "uptime_ms"]);
    uint(row.boot_ordinal); uint(row.uptime_ms);
    check(row.authoritative === false && row.boot_ordinal > 0 && ["power_on", "software_cpu", "watchdog", "panic", "brownout", "other"].includes(row.reset_reason),
      "recovery_diagnostics_shape");
    observations.push(structuredClone(row));
  }
  return { schema: "str005-recovery-diagnostics-v1", observations, omitted_count: value.observations.length - observations.length,
    authoritative: false };
}

/** Validate authenticated page results before projecting them into successor evidence. */
export function projectRecoveryPart(stage, value, context) {
  check(stages.includes(stage), "recovery_stage");
  if (stage === "ledger") validateLedger(value);
  else if (stage === "original_budget") requireExhaustedOriginal(value);
  else if (stage === "state" || stage === "closed") validateState(value, context);
  else if (stage === "diagnostics") return projectDiagnostics(value);
  else {
    const status = parseStatus(value);
    check(status.scope === context.scope && (status.record === null || status.record.attemptId === context.attemptId),
      "recovery_status_attempt");
    return { schema: "str005-recovery-status-v1", scope: status.scope, state: status.state,
      observation: Object.fromEntries(observationKeys.map(key => [key, status.observation[key]])), record: status.record };
  }
  return structuredClone(value);
}

function restored(state) {
  const preservation = state?.preservation;
  const authorization = preservation?.authorization_high_water_match === true ||
    (state?.authorizationRecovery?.matched === true && state.authorizationRecovery.generation > 0 &&
      state.authorizationRecovery.generation === state.qualification?.generation);
  return Boolean(state?.deviceRestorationConfirmed === true && state.deviceBaselineConfirmed === true &&
    state.deviceLeaseInactive === true && state.running === false && state.heartbeatSuppressed === false &&
    preservation?.device_identity_match === true && preservation.settings_match === true && preservation.mine_on_boot === false && authorization);
}

/** Assess already validated parts without inferring a reservation or historical release. */
export function recoveryConclusion(parts) {
  const blockers = [...stages, "finished"].filter(stage => !Object.hasOwn(parts, stage)).map(stage => `recovery_missing_${stage}`);
  if (parts.finished) {
    validateFinished(parts.finished);
    blockers.push(...parts.finished.failures.map(stage => `recovery_collection_failed_${stage}`));
  }
  const accounting = Boolean(parts.ledger && parts.original_budget);
  if (parts.ledger?.pending) blockers.push("recovery_accounting_pending");
  const restoration = restored(parts.state) && parts.state.connected === true &&
    ["ready", "baseline_confirmed"].includes(parts.state.status);
  if (parts.state && !restoration) blockers.push("recovery_restoration_unconfirmed");
  const record = parts.status?.record;
  const resources = Boolean(record?.state === "terminal" && record.resources.socketClosed === true &&
    record.resources.workerQuiescent === true && record.resources.fenceRetained === false);
  if (parts.status && !record) blockers.push("recovery_retained_record_unavailable");
  else if (record && !resources) blockers.push("recovery_device_resources_unreleased");
  const released = Boolean(parts.closed?.status === "closed" && parts.closed.connected === false &&
    parts.closed.running === false && parts.closed.serialOwnershipReleased === true && restored(parts.closed));
  if (parts.closed && !released) blockers.push("recovery_serial_release_unconfirmed");
  if (parts.state && parts.closed && parts.state.preservation?.baseline_id !== parts.closed.preservation?.baseline_id)
    blockers.push("recovery_baseline_changed");
  return { complete: blockers.length === 0, blockers, accounting_measured: accounting,
    device_resources_released: resources, restoration_confirmed: restoration, serial_released: released, parity_promotion: false };
}
