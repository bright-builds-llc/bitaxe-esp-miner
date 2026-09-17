import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { proof, protectedPath } from "../str005-noise-serial/files.mjs";
import { restoredBaseline } from "./journal.mjs";
import { check, object, sha256, uint } from "./values.mjs";

const STOP_STAGES = ["not_started", "stop_dispatch", "reduce_frequency_and_reset_nonce", "hold_reset_low",
  "disable_core_voltage", "disable_asic", "fan_full", "cooling_proof", "fan_paused"];
const COUNTERS = ["work_dispatched", "nonce_work_correlations", "submitted", "accepted", "rejected", "active_ms", "generation_elapsed_ms"];

function sensors(q, maximumTemperature) {
  check(q.watchdog_alive && !q.mine_on_boot && q.voltage_fresh && q.power_fresh && q.temperature_fresh && q.fan_fresh &&
    q.voltage_volts >= 4.5 && q.voltage_volts <= 5.5 && q.power_watts >= 0 && q.power_watts <= 15 &&
    q.chip_temp_celsius <= maximumTemperature && q.fan_rpm > 0, "v2_safety_health");
}

/** Fault timestamps must be the retained native atoms, not browser/event observation times. */
export function judgeShareSafety(context, rows, devices, execution, accounting) {
  const record = devices.at(-1).record, fault = execution.fault;
  const bound = rows.filter(row => row.state.qualification?.attempt?.ordinal === 18);
  const q = accounting.after.state.qualification;
  check(bound.length >= 2 && q?.generation === record.workerGeneration && q.generation > 0, "v2_safety_generation");
  let maybePrior, previousStage = 0;
  for (const row of bound) {
    const state = row.state, current = state.qualification;
    check(current.generation === q.generation && current.attempt.purpose === "normal" && current.attempt.maximum_active_ms === 180000 &&
      current.attempt.reserved_ms === 180000 && current.budget_reserved_ms === 240000 && current.budget_complete &&
      current.active_limit_ms === 180000 && current.shutdown_budget_ms === 15550 && !state.ownerResourceFailure,
    "v2_safety_allowance");
    if (maybePrior) check(COUNTERS.every(key => current[key] >= maybePrior[key]) && state.renewalsConfirmed >= maybePrior.renewalsConfirmed,
      "v2_counter_reset");
    if (maybePrior?.gate_closed_ms !== null && maybePrior?.gate_closed_ms !== undefined) check(current.gate_closed_ms === maybePrior.gate_closed_ms &&
      current.last_valid_heartbeat_ms === maybePrior.last_valid_heartbeat_ms && current.work_dispatched === maybePrior.work_dispatched,
    "v2_activity_after_native_revocation");
    if (maybePrior?.shutdown_started_ms !== null && maybePrior?.shutdown_started_ms !== undefined)
      check(current.shutdown_started_ms === maybePrior.shutdown_started_ms, "v2_shutdown_atom_changed");
    const stage = STOP_STAGES.indexOf(current.safe_stop_stage);
    check(stage >= previousStage, "v2_shutdown_order"); previousStage = stage;
    if (state.running && !current.safe_stop_complete) {
      sensors(current, 75);
      check(current.chip_temp_celsius < 75, "v2_safety_health");
      check(current.owner_resources?.phase === "active" && current.owner_resources.generation === q.generation &&
        current.owner_resources.stack_free_bytes >= 4096, "v2_active_owner_resources");
    }
    const expectedLoss = row.atHostMs > fault.suppressionRequestedAtHostMs && state.heartbeatSuppressed && !state.running &&
      state.serialFailureCategory === "liveness_lost" && (state.failure === undefined || state.failure === "window_control_failed");
    check((state.failure === undefined && state.serialFailureCategory === undefined) || expectedLoss, "v2_unexpected_browser_failure");
    maybePrior = { ...current, renewalsConfirmed: state.renewalsConfirmed };
  }
  restoredBaseline(accounting.after.state, context); sensors(q, 45);
  check(q.safe_stop_complete && q.safe_stop_stage === "fan_paused" && q.attempt.complete && q.revocation_reason === "heartbeat_timeout" &&
    q.owner_resources?.phase === "shutdown_complete" && q.owner_resources.generation === q.generation && q.owner_resources.stack_free_bytes >= 4096 &&
    q.active_ms > 0 && q.active_ms <= 180000 && q.active_ms <= q.generation_elapsed_ms && q.work_dispatched > 0 &&
    q.nonce_work_correlations > 0 && q.accepted > 0 && q.submitted >= q.accepted + q.rejected, "v2_safe_stop_incomplete");
  for (const [atom, absolute] of [["last_valid_heartbeat_ms", "lastValidHeartbeatAtDeviceUs"], ["gate_closed_ms", "gateClosedAtDeviceUs"],
    ["shutdown_started_ms", "shutdownStartedAtDeviceUs"]]) {
    check(q[atom] !== null && fault[absolute] % 1000 === 0 && Math.floor(fault[absolute] / 1000) % 0x100000000 === q[atom] &&
      fault[absolute] >= record.admittedAtDeviceUs && fault[absolute] <= record.observedAtUs, "v2_safety_atom_join");
  }
  const cooled = record.events.filter(event => event.kind === "cooled");
  check(cooled.length === 1 && cooled[0].atDeviceUs >= fault.shutdownStartedAtDeviceUs &&
    cooled[0].atDeviceUs - fault.shutdownStartedAtDeviceUs <= 120000000, "v2_cooling_deadline");
  const restored = rows[execution.restoration.observedSequence - 1];
  check(restored.atHostMs >= execution.faultConfirmed.confirmedAtHostMs + 145000, "v2_restoration_wait");
  const checkpoint = accounting.after.state.authorizationRecovery;
  check(checkpoint?.matched === true && checkpoint.generation === q.generation &&
    !bound.some(row => row.state.authorizationRecovery?.matched === false), "v2_authorization_checkpoint");
  const cut = rows.find(row => row.atHostMs >= fault.suppressionRequestedAtHostMs && row.state.heartbeatSuppressed &&
    row.state.qualification?.generation === q.generation);
  check(cut && bound.filter(row => row.sequence > cut.sequence).every(row => row.state.renewalsConfirmed === cut.state.renewalsConfirmed),
    "v2_renewal_after_fault");
  return { workDispatched: q.work_dispatched, submitted: q.submitted, accepted: q.accepted,
    renewals: accounting.after.state.renewalsConfirmed, activeMs: q.active_ms,
    heartbeatToRevocationMs: (q.gate_closed_ms - q.last_valid_heartbeat_ms) >>> 0,
    heartbeatToShutdownMs: (q.shutdown_started_ms - q.last_valid_heartbeat_ms) >>> 0 };
}

/** Revalidate the actual passive stream and separately timed host close witness. */
export async function judgeObserver(root, context, rows, devices, confirmed) {
  const claim = (await proof(root, "observer-start.claim.json")).value, stop = (await proof(root, "observer-stop.json")).value;
  const hash = sha256(JSON.stringify(context)), first = devices[0].record;
  object(claim, ["schema", "contextSha256", "atHostMs", "sourceObservedAtDeviceUs", "bootOrdinal", "workerGeneration", "stateSequence", "binarySha256", "lifetimeMs"]);
  object(stop, ["schema", "contextSha256", "requestedAtHostMs", "completedAtHostMs", "tailMs", "requestedCleanupMs"]);
  for (const key of ["atHostMs", "sourceObservedAtDeviceUs", "bootOrdinal", "workerGeneration", "stateSequence", "lifetimeMs"]) uint(claim[key]);
  for (const key of ["requestedAtHostMs", "completedAtHostMs", "tailMs", "requestedCleanupMs"]) uint(stop[key]);
  check(claim.schema === "str005-v2-observer-claim-v1" && claim.contextSha256 === hash && claim.binarySha256 === context.cadence_observer.sha256 &&
    claim.lifetimeMs === 360000 && claim.bootOrdinal === first.bootOrdinal && claim.workerGeneration === first.workerGeneration &&
    rows[claim.stateSequence - 1]?.atHostMs <= claim.atHostMs && claim.atHostMs <= devices[0].atHostMs, "v2_observer_start_join");
  check(stop.schema === "str005-v2-observer-stop-v1" && stop.contextSha256 === hash && stop.tailMs >= 5000 &&
    stop.tailMs === stop.requestedAtHostMs - confirmed.confirmedAtHostMs && stop.requestedCleanupMs >= 0 && stop.requestedCleanupMs <= 5000 &&
    stop.requestedCleanupMs === stop.completedAtHostMs - stop.requestedAtHostMs, "v2_observer_stop_join");
  const result = (await proof(root, "cadence-observer-result.json")).value;
  object(result, ["schema", "connected", "closed", "exitCode", "reason", "cleanupComplete", "startedAtUnixMs", "connectedAtUnixMs", "closedAtUnixMs",
    "messageCount", "totalBytes", "eventCount", "journalSha256"]);
  for (const key of ["startedAtUnixMs", "connectedAtUnixMs", "closedAtUnixMs", "messageCount", "totalBytes", "eventCount"]) uint(result[key]);
  const path = resolve(root, "cadence-observer.jsonl"); await protectedPath(path);
  const data = await readFile(path); check(data.length <= 1048576 && data.at(-1) === 10, "v2_observer_journal_bound");
  const entries = data.toString("utf8").trimEnd().split("\n").map(line => JSON.parse(line));
  check(result.schema === "worker-cadence-observer-result-v1" && result.connected === true && result.closed === true && result.exitCode === 0 &&
    result.reason === "requested" && result.cleanupComplete === true && result.journalSha256 === sha256(data) &&
    entries.length === result.eventCount && entries.length >= 3 && entries.length <= 2048, "v2_observer_result");
  let previous = { elapsedMs: 0, messageCount: 0, totalBytes: 0 }, previousObserved = result.startedAtUnixMs;
  for (const [index, row] of entries.entries()) {
    object(row, ["sequence", "observedAtUnixMs", "event"]); uint(row.observedAtUnixMs);
    const e = row.event;
    object(e, ["schema", "event", "elapsedMs", "messageCount", "totalBytes", "byteCount", "reason"]);
    for (const key of ["elapsedMs", "messageCount", "totalBytes", "byteCount"]) uint(e[key]);
    check(row.sequence === index + 1 && row.observedAtUnixMs >= previousObserved && e.schema === "cpu0-cadence-observer-v1" &&
      e.elapsedMs >= previous.elapsedMs && e.elapsedMs <= 360000, "v2_observer_event_order");
    if (index === 0) check(e.event === "connected" && e.messageCount === 0 && e.totalBytes === 0 && e.byteCount === 0 && e.reason === null,
      "v2_observer_connected");
    else if (index === entries.length - 1) check(e.event === "closed" && e.reason === "requested" && e.byteCount === 0 &&
      e.messageCount === previous.messageCount && e.totalBytes === previous.totalBytes, "v2_observer_closed");
    else check(e.event === "arrival" && e.reason === null && e.byteCount > 0 && e.byteCount <= 65536 &&
      e.messageCount === previous.messageCount + 1 && e.totalBytes === previous.totalBytes + e.byteCount, "v2_observer_arrival");
    previous = e; previousObserved = row.observedAtUnixMs;
  }
  check(result.connectedAtUnixMs === entries[0].observedAtUnixMs && result.closedAtUnixMs >= previousObserved &&
    result.messageCount === previous.messageCount && result.totalBytes === previous.totalBytes, "v2_observer_summary_join");
  return { messages: result.messageCount, tailMs: stop.tailMs, cleanupMs: stop.requestedCleanupMs };
}
