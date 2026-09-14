import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { CADENCE_PHASES, validateCadencePolicy } from "./cadence-contract.mjs";
import { requireCadenceDiagnostics, requireCadencePhase, validateCadenceProbe } from "./cadence-evidence.mjs";
import { digest, exactObject, fileDigest, protectedPath, QualificationError, readJson, requireCondition } from "./contract.mjs";
import { requireOwnerResources } from "./iterative-judge.mjs";

/** Re-read immutable phase inputs; no caller-authored pass flag replaces numeric checks. */
export async function requireCadenceEvidence(root, context, records) {
  try { return await readCadenceEvidence(root, context, records); }
  catch (error) {
    if (error instanceof QualificationError) throw error;
    if (error.code === "ENOENT") throw new QualificationError("cadence_evidence_missing");
    throw error;
  }
}

async function readCadenceEvidence(root, context, records) {
  const evidence = [], phaseRecords = [];
  let previousPhase;
  for (const name of CADENCE_PHASES) {
    const path = resolve(root, `cadence-${name}.json`);
    await protectedPath(path);
    const record = await readJson(path);
    requireCadenceDiagnostics(context, record?.review);
    exactObject(record, ["schema", "context_sha256", "phase", "arm", "started_sequence", "finished_sequence", "started_at_unix_ms", "finished_at_unix_ms", "collected_at_unix_ms", "measurement_end_sequence", "review", "observer_connected"]);
    requireCondition(record.schema === "worker-cadence-phase-v1" && record.context_sha256 === digest(JSON.stringify(context)) &&
      record.phase === name && record.observer_connected === true && Number.isInteger(record.started_sequence) &&
      record.started_sequence > 0 && record.finished_sequence > record.started_sequence, "cadence_phase_binding");
    const before = records.find(value => value.sequence === record.started_sequence)?.state;
    const after = records.find(value => value.sequence === record.finished_sequence)?.state;
    requireCondition(before?.connected && !before.running && before.deviceLeaseInactive && after?.connected &&
      !after.running && after.deviceLeaseInactive && isDeepStrictEqual(after.cadence?.review, record.review), "cadence_phase_journal_binding");
    exactObject(record.arm, ["schema", "phase", "armedAtUs", "generation"]);
    const phase = requireCadencePhase(record.review, name);
    requireCondition(record.arm.schema === "worker-telemetry-cadence-arm-v1" && record.arm.phase === name &&
      record.arm.armedAtUs === phase.armedAtUs && record.arm.generation === phase.generation, "cadence_arm_binding");
    requireCondition(Number.isSafeInteger(record.finished_sequence) &&
      Number.isSafeInteger(record.started_at_unix_ms) && record.started_at_unix_ms >= 0 &&
      Number.isSafeInteger(record.finished_at_unix_ms) && record.finished_at_unix_ms >= record.started_at_unix_ms &&
      Number.isSafeInteger(record.collected_at_unix_ms) && record.collected_at_unix_ms >= record.finished_at_unix_ms &&
      Number.isSafeInteger(record.measurement_end_sequence) && record.measurement_end_sequence >= record.started_sequence &&
      record.measurement_end_sequence <= record.finished_sequence && records.some(value => value.sequence === record.measurement_end_sequence),
      "cadence_phase_clock");
    if (name !== "mining") requireCondition(record.measurement_end_sequence === record.finished_sequence &&
      record.finished_at_unix_ms === record.collected_at_unix_ms, "cadence_nonmining_collection");
    if (previousPhase) requireCondition(previousPhase.record.finished_sequence <= record.started_sequence &&
      previousPhase.record.finished_at_unix_ms <= record.started_at_unix_ms && previousPhase.summary.endedAtUs <= phase.startedAtUs,
      "cadence_phase_chronology");
    previousPhase = { record, summary: phase }; phaseRecords.push(record);
    evidence.push({ phase: name, sha256: await fileDigest(path), summary: phase, measurement_end_sequence: record.measurement_end_sequence });
  }
  const measurementPath = resolve(root, "cadence-mining-measurement-end.json");
  await protectedPath(measurementPath);
  const measurementBytes = await readFile(measurementPath);
  requireCondition(measurementBytes.length > 0 && measurementBytes.length <= 4096, "cadence_measurement_witness_bound");
  const measurement = JSON.parse(measurementBytes.toString("utf8"));
  exactObject(measurement, ["context_sha256", "sequence", "observedAtUnixMs"]);
  requireCondition(measurement.context_sha256 === digest(JSON.stringify(context)) &&
    measurement.sequence === phaseRecords[2].measurement_end_sequence && measurement.observedAtUnixMs === phaseRecords[2].finished_at_unix_ms,
    "cadence_measurement_witness_binding");
  evidence[2].measurement_end_sha256 = digest(measurementBytes);
  const final = await readJson(resolve(root, "cadence-mining.json"));
  for (const entry of evidence) requireCondition(isDeepStrictEqual(entry.summary, requireCadencePhase(final.review, entry.phase)), "cadence_frozen_summary_changed");
  const probesPath = resolve(root, "cadence-probes.jsonl");
  await protectedPath(probesPath);
  const bytes = await readFile(probesPath);
  requireCondition(bytes.length <= 16384 && bytes.at(-1) === 10, "cadence_probe_bound");
  const probes = bytes.toString("utf8").trim().split("\n").map(line => JSON.parse(line));
  requireCondition(probes.length === 12, "cadence_probe_count");
  let previous;
  for (const probe of probes) previous = validateCadenceProbe(probe, previous);
  const witnessesPath = resolve(root, "cadence-probe-witnesses.jsonl");
  await protectedPath(witnessesPath);
  const witnessBytes = await readFile(witnessesPath);
  requireCondition(witnessBytes.length > 0 && witnessBytes.length <= 16384 && witnessBytes.at(-1) === 10, "cadence_probe_witness_bound");
  const witnesses = witnessBytes.toString("utf8").trim().split("\n").map(line => JSON.parse(line));
  requireCondition(witnesses.length === 12, "cadence_probe_witness_count");
  const usbPhase = phaseRecords[1]; let lastWitness;
  for (const [index, witness] of witnesses.entries()) {
    exactObject(witness, ["ordinal", "observedAtUnixMs", "sequence"]);
    requireCondition(witness.ordinal === probes[index].ordinal && Number.isSafeInteger(witness.observedAtUnixMs) &&
      witness.observedAtUnixMs >= usbPhase.started_at_unix_ms && witness.observedAtUnixMs <= usbPhase.finished_at_unix_ms &&
      Number.isSafeInteger(witness.sequence) && witness.sequence >= usbPhase.started_sequence && witness.sequence <= usbPhase.finished_sequence &&
      (!lastWitness || (witness.observedAtUnixMs >= lastWitness.observedAtUnixMs && witness.sequence >= lastWitness.sequence)), "cadence_probe_witness_binding");
    lastWitness = witness;
  }
  evidence.push({ phase: "usb_probes", sha256: digest(bytes), witness_sha256: digest(witnessBytes), count: probes.length });
  const observerPath = resolve(root, "cadence-observer-result.json");
  await protectedPath(observerPath);
  const observer = await readJson(observerPath);
  exactObject(observer, ["schema", "connected", "closed", "exitCode", "reason", "cleanupComplete", "startedAtUnixMs", "connectedAtUnixMs", "closedAtUnixMs", "messageCount", "totalBytes", "eventCount", "journalSha256"]);
  requireCondition(observer.schema === "worker-cadence-observer-result-v1" && observer.connected === true &&
    observer.closed === true && observer.exitCode === 0 && observer.reason === "requested" && observer.cleanupComplete === true &&
    [observer.startedAtUnixMs, observer.connectedAtUnixMs, observer.closedAtUnixMs].every(value => Number.isSafeInteger(value) && value >= 0) &&
    observer.connectedAtUnixMs >= observer.startedAtUnixMs &&
    observer.closedAtUnixMs >= observer.connectedAtUnixMs && observer.closedAtUnixMs - observer.startedAtUnixMs <= 366000,
    "cadence_observer_failed");
  requireCondition(phaseRecords.every(phase => observer.connectedAtUnixMs <= phase.started_at_unix_ms &&
    observer.closedAtUnixMs >= phase.finished_at_unix_ms) &&
    observer.closedAtUnixMs - phaseRecords[2].finished_at_unix_ms >= 5000, "cadence_observer_phase_coverage");
  const observerJournal = resolve(root, "cadence-observer.jsonl");
  await protectedPath(observerJournal);
  const observerBytes = await readFile(observerJournal);
  requireCondition(observerBytes.length > 0 && observerBytes.length <= 1048576 && observerBytes.at(-1) === 10 &&
    digest(observerBytes) === observer.journalSha256, "cadence_observer_journal");
  const events = observerBytes.toString("utf8").trim().split("\n").map(line => JSON.parse(line));
  requireCondition(events.length === observer.eventCount && events.length >= 3 && events.length <= 2048, "cadence_observer_journal");
  let lastTime = observer.startedAtUnixMs, lastElapsed = 0, messages = 0, total = 0;
  for (const [index, row] of events.entries()) {
    exactObject(row, ["sequence", "observedAtUnixMs", "event"]);
    const e = row.event;
    exactObject(e, ["schema", "event", "elapsedMs", "messageCount", "totalBytes", "byteCount", "reason"]);
    requireCondition(row.sequence === index + 1 && Number.isSafeInteger(row.observedAtUnixMs) && row.observedAtUnixMs >= lastTime &&
      e.schema === "cpu0-cadence-observer-v1" && Number.isSafeInteger(e.elapsedMs) && e.elapsedMs >= lastElapsed && e.elapsedMs <= 360000,
      "cadence_observer_journal");
    if (index === 0) requireCondition(e.event === "connected" && e.reason === null && e.byteCount === 0, "cadence_observer_journal");
    else if (index === events.length - 1) requireCondition(e.event === "closed" && e.reason === "requested" && e.byteCount === 0, "cadence_observer_journal");
    else {
      requireCondition(e.event === "arrival" && e.reason === null && Number.isSafeInteger(e.byteCount) && e.byteCount > 0 && e.byteCount <= 65536,
        "cadence_observer_journal");
      messages += 1; total += e.byteCount;
    }
    requireCondition(e.messageCount === messages && e.totalBytes === total, "cadence_observer_journal");
    lastTime = row.observedAtUnixMs; lastElapsed = e.elapsedMs;
  }
  requireCondition(messages === observer.messageCount && total === observer.totalBytes &&
    events[0].observedAtUnixMs === observer.connectedAtUnixMs &&
    events.at(-1).observedAtUnixMs - phaseRecords[2].finished_at_unix_ms >= 5000 &&
    events.at(-1).observedAtUnixMs <= observer.closedAtUnixMs && observer.closedAtUnixMs - events.at(-1).observedAtUnixMs <= 6000,
    "cadence_observer_journal");
  evidence.push({ phase: "observer", sha256: await fileDigest(observerPath), result: observer });
  return evidence;
}

export function judgeCadence(context, records, fault, evidence) {
  validateCadencePolicy(context);
  requireCondition(evidence.length === 5 && CADENCE_PHASES.every((name, index) => evidence[index].phase === name), "cadence_evidence_missing");
  const a = context.qualification_attempt;
  const observed = records.filter(value => value.state.qualification?.attempt?.ordinal === a.ordinal);
  const last = observed.at(-1)?.state, q = last?.qualification;
  const mining = evidence[2].summary;
  requireCondition(q && observed.length > 2 && q.generation === mining.generation && q.generation > 0, "cadence_work_generation");
  for (const { state } of observed) {
    const value = state.qualification;
    requireCondition(value.generation === q.generation && value.attempt.purpose === "normal" && value.attempt.maximum_active_ms === 180000 &&
      value.attempt.reserved_ms === 180000 && value.budget_reserved_ms === 240000 && value.budget_complete, "cadence_work_binding");
    if (!state.running || value.safe_stop_complete) continue;
    requireOwnerResources(value, "active", context.owner_stack_minimum_bytes);
    requireCondition(value.voltage_fresh && value.power_fresh && value.temperature_fresh && value.fan_fresh &&
      value.voltage_volts >= 4.5 && value.voltage_volts <= 5.5 && value.power_watts >= 0 && value.power_watts <= 15 &&
      value.chip_temp_celsius < 75 && value.fan_rpm > 0 && value.watchdog_alive && !value.mine_on_boot && !state.ownerResourceFailure,
    "cadence_safety_observation");
  }
  requireOwnerResources(q, "shutdown_complete", context.owner_stack_minimum_bytes);
  requireCondition(last.deviceRestorationConfirmed && last.deviceLeaseInactive && q.safe_stop_complete && q.attempt.complete &&
    q.safe_stop_stage === "fan_paused" && q.temperature_fresh && q.chip_temp_celsius <= 45 && q.fan_fresh && q.fan_rpm > 0 &&
    !q.mine_on_boot && q.active_ms >= 60000 && q.active_ms <= 180000 && q.active_ms <= q.generation_elapsed_ms &&
    q.active_limit_ms === 180000 && q.shutdown_budget_ms === 15550, "cadence_qualified_stop");
  requireCondition(fault?.kind === "heartbeats_suppressed" && fault.generation === q.generation, "cadence_fault_missing");
  const cut = records.find(value => value.sequence === fault.after_sequence)?.state;
  requireCondition(cut?.running && cut.heartbeatSuppressed && cut.cadence?.suppressionRequested &&
    cut.qualification?.generation === q.generation && cut.qualification.gate_closed_ms === null &&
    cut.qualification.work_gate_remaining_ms > 3000 && q.revocation_reason === "heartbeat_timeout" &&
    q.gate_closed_ms !== null && q.shutdown_started_ms !== null, "cadence_fault_binding");
  requireCondition(evidence[2].measurement_end_sequence === fault.after_sequence, "cadence_measurement_fault_binding");
  const endMilliseconds = Math.ceil(mining.endedAtUs / 1000) % 0x100000000;
  const captureToGate = (q.gate_closed_ms - endMilliseconds) >>> 0;
  requireCondition(Number.isSafeInteger(mining.endedAtUs) && mining.endedAtUs >= 0 &&
    Number.isInteger(q.generation_elapsed_ms) && q.generation_elapsed_ms >= 0 && q.generation_elapsed_ms < 0x80000000 &&
    captureToGate <= q.generation_elapsed_ms, "cadence_capture_after_gate");
  const gateDelay = (q.gate_closed_ms - q.last_valid_heartbeat_ms) >>> 0;
  const shutdownDelay = (q.shutdown_started_ms - q.last_valid_heartbeat_ms) >>> 0;
  requireCondition(gateDelay <= 3000 && shutdownDelay <= 3000, "cadence_safety_deadline");
  const start = cut.cadence.firstWorkObservedAtMs;
  requireCondition(records.every(({ sequence, state }) => {
    if (state.failure === undefined && state.serialFailureCategory === undefined) return true;
    const expectedLoss = sequence > fault.after_sequence && state.heartbeatSuppressed === true &&
      state.cadence?.suppressionRequested === true && state.qualification?.generation === q.generation &&
      state.serialFailureCategory === "liveness_lost" && !state.running && !state.ownerResourceFailure;
    return expectedLoss && (state.failure === undefined || state.failure === "window_control_failed");
  }), "cadence_unexpected_browser_failure");
  const checkpoint = cut.authorizationRecovery;
  requireCondition(checkpoint?.generation === q.generation && checkpoint.matched === null &&
    records.some(value => value.sequence > fault.after_sequence && value.state.connected &&
      value.state.authorizationRecovery?.checkpointId === checkpoint.checkpointId &&
      value.state.authorizationRecovery.generation === q.generation && value.state.authorizationRecovery.matched === true) &&
    !records.some(value => value.sequence > fault.after_sequence && value.state.authorizationRecovery?.matched === false),
    "cadence_authorization_preservation");
  requireCondition(Number.isSafeInteger(start) && cut.cadence.latestWork.atMs >= start + 60000 &&
    observed.some(value => value.state.renewalsConfirmed > 0), "cadence_work_duration");
  for (const { state } of observed) {
    if (state.cadence?.latestWork) requireCondition(state.cadence.latestWork.generation === q.generation &&
      state.cadence.firstWorkObservedAtMs === start, "cadence_work_sample_binding");
  }
  let count = 0;
  for (let bucket = 0; bucket < 6; bucket++) {
    const values = observed.filter(({ state }) => state.running && state.cadence?.latestWork &&
      state.cadence.latestWork.atMs >= start + bucket * 10000 && state.cadence.latestWork.atMs < start + (bucket + 1) * 10000);
    const next = Math.max(0, ...values.map(({ state }) => state.cadence.latestWork.workDispatched));
    requireCondition(next > count, "cadence_work_segment_missing"); count = next;
  }
  return { schema: "worker-cadence-judgment-v1", generation: q.generation, phases_verified: 3, usb_probes_verified: 12,
    active_ms: q.active_ms, gate_close_delay_ms: gateDelay, shutdown_start_delay_ms: shutdownDelay,
    purpose: "normal", diagnostic_only: true, accepted_share_required: false,
    accepted_share_verified: q.accepted > 0, observer_verified: true, hardware_execution_claimed_by_supervisor: false };
}
