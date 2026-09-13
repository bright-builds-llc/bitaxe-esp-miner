import assert from "node:assert/strict";
import test from "node:test";
import { requireCadencePhase, validateCadenceBrowser, validateCadenceProbe, validateCadenceReview } from "./cadence-evidence.mjs";

function review() {
  return { schema: "worker-telemetry-cadence-v1", snapshotAvailable: true, droppedObservations: 0, storageBytes: 1024,
    phases: ["idle", "usb", "mining"].map((phase, index) => ({ phase, state: "complete", armedAtUs: 1000000 + index * 70000000,
      startedAtUs: 1000000 + index * 70000000, endedAtUs: 61000000 + index * 70000000, generation: index === 2 ? 2 : 0,
      maxProbeCount: index === 1 ? 12 : 0, firstMaxProbeAtUs: index === 1 ? 71500000 : 0, lastMaxProbeAtUs: index === 1 ? 126500000 : 0,
      intervalCount: 100, maximumIntervalUs: 600000, maximumExecutionUs: 100000, maximumLiveUs: 50000,
      maximumLogsUs: 30000, maximumPruneUs: 20000, cpuMismatchCount: 0, priorityMismatchCount: 0,
      subscriberMismatchCount: 0, projectionCount: 101, unchangedCount: 0, noSubscriberCount: 0,
      projectionFailures: 0, serializationFailures: 0, queueFailures: 0, sendFailures: 0,
      sendsQueued: 101, sendsCompleted: 101, pendingSends: 0, clockFailures: 0,
      intervalBuckets: [0, 100, 0, 0], overflow: false, passed: true })) };
}

test("numeric phase guard accepts exact percentile/time limits", () => {
  // Arrange
  const value = review(); Object.assign(value.phases[0], { intervalBuckets: [0, 95, 5, 0], maximumIntervalUs: 1500000, maximumExecutionUs: 500000 });
  // Act / Assert
  assert.equal(requireCadencePhase(value, "idle").phase, "idle");
});

test("pass flag cannot hide percentile, period, execution or interval-count failure", () => {
  // Arrange / Act / Assert
  for (const change of [{ intervalBuckets: [0, 94, 6, 0] }, { maximumIntervalUs: 1500001 }, { maximumExecutionUs: 500001 },
    { intervalCount: 59, intervalBuckets: [0, 59, 0, 0] }, { intervalBuckets: [0, 99, 0, 0] }, { endedAtUs: 60999999 },
    { maximumLiveUs: 100001 }, { sendsCompleted: 100 }, { overflow: true }, { state: "capturing" }]) {
    const value = review(); Object.assign(value.phases[0], change); assert.throws(() => requireCadencePhase(value, "idle"));
  }
});

test("lost snapshots, dropped observations and safety/publication counters reject capture", () => {
  // Arrange / Act / Assert
  for (const key of ["cpuMismatchCount", "priorityMismatchCount", "subscriberMismatchCount", "projectionFailures", "serializationFailures",
    "queueFailures", "sendFailures", "pendingSends", "clockFailures", "noSubscriberCount"]) {
    const value = review(); value.phases[0][key] = 1; assert.throws(() => requireCadencePhase(value, "idle"));
  }
  for (const change of [{ snapshotAvailable: false }, { droppedObservations: 1 }, { storageBytes: 2049 }]) assert.throws(() => requireCadencePhase({ ...review(), ...change }, "idle"));
});

test("review rejects private extra keys, duplicate phase labels and unsafe integers", () => {
  // Arrange / Act / Assert
  assert.throws(() => validateCadenceReview({ ...review(), endpoint: "private-fixture" }));
  for (const change of [{ phase: "usb" }, { generation: Number.MAX_SAFE_INTEGER + 1 }, { private: "fixture" }]) {
    const value = review(); Object.assign(value.phases[0], change); assert.throws(() => validateCadenceReview(value));
  }
});

test("phase cannot begin before authenticated arming", () => {
  // Arrange
  const value = review(); value.phases[0].armedAtUs += 1;
  // Act / Assert
  assert.throws(() => requireCadencePhase(value, "idle"));
});

test("boundary capture cannot hide arbitrarily long post-deadline tail", () => {
  // Arrange
  const value = review(); value.phases[0].endedAtUs += 60000000;
  // Act / Assert
  assert.throws(() => requireCadencePhase(value, "idle"));
});

test("USB probes require exact sizes, consecutive five-second schedule and no overlap", () => {
  // Arrange
  const first = { ordinal: 1, scheduledAtMs: 1000, startedAtMs: 1000, completedAtMs: 2000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 };
  const second = { ...first, ordinal: 2, scheduledAtMs: 6000, startedAtMs: 6000, completedAtMs: 7000 };
  // Act / Assert
  assert.deepEqual(validateCadenceProbe(first), first); assert.deepEqual(validateCadenceProbe(second, first), second);
  for (const change of [{ ordinal: 3 }, { scheduledAtMs: 6001 }, { completedAtMs: 11001 }, { responsePayloadBytes: 65535 }, { private: "fixture" }]) {
    assert.throws(() => validateCadenceProbe({ ...second, ...change }, first));
  }
  assert.throws(() => validateCadenceProbe(second, { ...first, completedAtMs: 6001 }));
});

test("five-second schedule labels cannot admit twelve probes compressed into a late burst", () => {
  // Arrange
  const probes = Array.from({ length: 12 }, (_, index) => ({ ordinal: index + 1, scheduledAtMs: index * 5000,
    startedAtMs: 58000 + index * 10, completedAtMs: 58010 + index * 10, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }));
  assert.equal(probes.at(-1).scheduledAtMs - probes[0].scheduledAtMs, 55000);
  assert.equal(probes.at(-1).completedAtMs - probes[0].startedAtMs, 120);
  // Act / Assert
  assert.throws(() => {
    let previous;
    for (const probe of probes) previous = validateCadenceProbe(probe, previous);
  }, { code: "cadence_usb_probe" });
});

test("USB probe timing permits one-second start jitter but requires completion before its slot ends", () => {
  // Arrange
  const probe = { ordinal: 1, scheduledAtMs: 5000, startedAtMs: 6000, completedAtMs: 9999, requestPayloadBytes: 65536, responsePayloadBytes: 65536 };
  // Act / Assert
  assert.deepEqual(validateCadenceProbe(probe), probe);
  assert.throws(() => validateCadenceProbe({ ...probe, startedAtMs: 6001 }), { code: "cadence_usb_probe" });
  assert.throws(() => validateCadenceProbe({ ...probe, completedAtMs: 10000 }), { code: "cadence_usb_probe" });
});

test("browser cadence data rejects accidental endpoint export and malformed work", () => {
  // Arrange
  const value = { schema: "worker-cadence-browser-v1", enabled: true, suppressionRequested: false,
    firstWorkObservedAtMs: 0, latestWork: { atMs: 1000, generation: 2, workDispatched: 1 }, review: review() };
  // Act / Assert
  validateCadenceBrowser(value);
  assert.throws(() => validateCadenceBrowser({ ...value, endpoint: "private-fixture" }));
  assert.throws(() => validateCadenceBrowser({ ...value, latestWork: { ...value.latestWork, generation: 0 } }));
});

async function sealedFixture(t) {
  const { chmod, mkdtemp, realpath, rm, writeFile } = await import("node:fs/promises");
  const { tmpdir } = await import("node:os");
  const { resolve } = await import("node:path");
  const { digest, writeNew } = await import("./contract.mjs");
  const root = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-evidence-"))); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  const context = { fixture: true }, value = review(), records = [], phases = [];
  for (const [index, name] of ["idle", "usb", "mining"].entries()) {
    const summary = value.phases[index];
    const phase = { schema: "worker-cadence-phase-v1", context_sha256: digest(JSON.stringify(context)), phase: name,
      arm: { schema: "worker-telemetry-cadence-arm-v1", phase: name, armedAtUs: summary.armedAtUs, generation: summary.generation },
      started_sequence: index * 2 + 1, finished_sequence: index * 2 + 2,
      started_at_unix_ms: 1000 + index * 70000, finished_at_unix_ms: 61000 + index * 70000,
      collected_at_unix_ms: 61000 + index * 70000, measurement_end_sequence: index * 2 + 2, review: structuredClone(value), observer_connected: true };
    phases.push(phase); await writeNew(resolve(root, `cadence-${name}.json`), phase);
    for (let offset = 1; offset <= 2; offset++) records.push({ sequence: index * 2 + offset,
      state: { connected: true, running: false, deviceLeaseInactive: true, cadence: { review: structuredClone(value) } } });
  }
  const measurement = { context_sha256: digest(JSON.stringify(context)), sequence: phases[2].measurement_end_sequence,
    observedAtUnixMs: phases[2].finished_at_unix_ms };
  await writeNew(resolve(root, "cadence-mining-measurement-end.json"), measurement);
  const probes = Array.from({ length: 12 }, (_, index) => ({ ordinal: index + 1, scheduledAtMs: index * 5000,
    startedAtMs: index * 5000, completedAtMs: index * 5000 + 500, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }));
  await writeFile(resolve(root, "cadence-probes.jsonl"), probes.map(item => JSON.stringify(item) + "\n").join(""), { mode: 0o600 });
  const witnesses = probes.map((probe, index) => ({ ordinal: probe.ordinal, observedAtUnixMs: 71500 + index * 5000, sequence: 3 }));
  await writeFile(resolve(root, "cadence-probe-witnesses.jsonl"), witnesses.map(item => JSON.stringify(item) + "\n").join(""), { mode: 0o600 });
  const rows = ["connected", "arrival", "closed"].map((event, index) => ({ sequence: index + 1, observedAtUnixMs: [1000, 100000, 210000][index],
    event: { schema: "cpu0-cadence-observer-v1", event, elapsedMs: [0, 99000, 209000][index], messageCount: index > 0 ? 1 : 0,
      totalBytes: index > 0 ? 10 : 0, byteCount: index === 1 ? 10 : 0, reason: index === 2 ? "requested" : null } }));
  const journal = rows.map(row => JSON.stringify(row) + "\n").join("");
  await writeFile(resolve(root, "cadence-observer.jsonl"), journal, { mode: 0o600 });
  const result = { schema: "worker-cadence-observer-result-v1", connected: true, closed: true, exitCode: 0, reason: "requested", cleanupComplete: true,
    startedAtUnixMs: 1000, connectedAtUnixMs: 1000, closedAtUnixMs: 210000, messageCount: 1, totalBytes: 10, eventCount: 3, journalSha256: digest(journal) };
  await writeNew(resolve(root, "cadence-observer-result.json"), result);
  return { root, context, records, phases, result, rows, witnesses, measurement, resolve,
    writeLines: async (name, values) => writeFile(resolve(root, name), values.map(value => JSON.stringify(value) + "\n").join("")), write: async (name, object) => writeFile(resolve(root, name), JSON.stringify(object)) };
}

test("sealed observer accepts distinct terminal-arrival and actual reap timestamps", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  // Act / Assert
  await f.write("cadence-observer-result.json", { ...f.result, closedAtUnixMs: 210002 });
  assert.equal((await requireCadenceEvidence(f.root, f.context, f.records)).length, 5);
});

test("observer journal tampering, private result keys and incomplete cleanup reject evidence", async t => {
  // Arrange / Act / Assert
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  for (const change of [{ cleanupComplete: false }, { private: "fixture" }, { journalSha256: "0".repeat(64) }, { messageCount: 2 }]) {
    const f = await sealedFixture(t); await f.write("cadence-observer-result.json", { ...f.result, ...change });
    await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records));
  }
});

test("saved phase cannot change its context hash or frozen earlier summary", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  // Act / Assert
  await f.write("cadence-idle.json", { ...f.phases[0], context_sha256: "0".repeat(64) });
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_phase_binding" });
  await f.write("cadence-idle.json", f.phases[0]);
  f.phases[2].review.phases[0].maximumExecutionUs++;
  f.records[5].state.cadence.review = structuredClone(f.phases[2].review);
  await f.write("cadence-mining.json", f.phases[2]);
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_frozen_summary_changed" });
});

test("valid-looking individual phase receipts cannot reverse chronological journal order", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  Object.assign(f.phases[0], { started_sequence: 3, finished_sequence: 4 });
  Object.assign(f.phases[1], { started_sequence: 1, finished_sequence: 2 });
  await f.write("cadence-idle.json", f.phases[0]); await f.write("cadence-usb.json", f.phases[1]);
  // Act / Assert
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records));
});

test("USB witness ordinals, host-time scope and journal scope are mandatory", async t => {
  // Arrange / Act / Assert
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  for (const change of [{ ordinal: 2 }, { observedAtUnixMs: 70999 }, { observedAtUnixMs: 131001 }, { sequence: 2 }, { private: "fixture" }]) {
    const f = await sealedFixture(t); Object.assign(f.witnesses[0], change);
    await f.writeLines("cadence-probe-witnesses.jsonl", f.witnesses);
    await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records));
  }
});

test("observer handshake must precede all phases and connection must survive their completion", async t => {
  // Arrange / Act / Assert
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  for (const change of [{ connectedAtUnixMs: 1001 }, { closedAtUnixMs: 200999 }]) {
    const f = await sealedFixture(t); await f.write("cadence-observer-result.json", { ...f.result, ...change });
    await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_observer_phase_coverage" });
  }
});

test("observer reap cannot precede terminal output or exceed the cleanup bound", async t => {
  // Arrange / Act / Assert
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  for (const closedAtUnixMs of [209999, 216001]) {
    const f = await sealedFixture(t); await f.write("cadence-observer-result.json", { ...f.result, closedAtUnixMs });
    await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_observer_journal" });
  }
});

test("device capture chronology and u32 counter bounds cannot be fabricated", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  const usb = f.phases[1]; usb.review.phases[1].startedAtUs = 1000000; usb.review.phases[1].armedAtUs = 1000000; usb.review.phases[1].endedAtUs = 61000000;
  usb.review.phases[1].firstMaxProbeAtUs = 1500000; usb.review.phases[1].lastMaxProbeAtUs = 56500000;
  usb.arm.armedAtUs = 1000000; f.records[3].state.cadence.review = structuredClone(usb.review); await f.write("cadence-usb.json", usb);
  // Act / Assert
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_phase_chronology" });
  const value = review(); value.phases[0].projectionCount = 0x100000000;
  assert.throws(() => validateCadenceReview(value));
});

function liveFixture() {
  const context = { schema: "fixed-usb-cadence-context-v1", required_no_mining_cycles: 4, owner_stack_minimum_bytes: 4096,
    suggested_difficulty: 1000, qualification_attempt: { ordinal: 16, purpose: "normal", maximumActiveMilliseconds: 180000 },
    cadence_limits: { duration_ms: 60000, minimum_intervals: 60, percent_within_750_ms: 95, maximum_interval_ms: 1500,
      maximum_execution_ms: 500, maximum_storage_bytes: 2048, observer_lifetime_ms: 360000, usb_probes: 12, probe_period_ms: 5000 },
    cadence_observer: { path: "/fixture", sha256: "a".repeat(64) }, cadence_validator_sha256: "b".repeat(64) };
  const q = { generation: 2, budget_reserved_ms: 240000, budget_complete: true, active_ms: 65000, generation_elapsed_ms: 66000,
    active_limit_ms: 180000, shutdown_budget_ms: 15550, work_gate_remaining_ms: 10000, voltage_fresh: true, power_fresh: true,
    temperature_fresh: true, fan_fresh: true, voltage_volts: 5, power_watts: 8, chip_temp_celsius: 35, fan_rpm: 5000,
    watchdog_alive: true, mine_on_boot: false, safe_stop_complete: false, gate_closed_ms: null, shutdown_started_ms: null,
    last_valid_heartbeat_ms: 202000, accepted: 0,
    owner_resources: { schema: "worker-owner-resources-v1", generation: 2, phase: "active", observed_at_ms: "1000",
      heap_free_bytes: 50000, heap_largest_bytes: 12000, stack_free_bytes: 8192 },
    attempt: { ordinal: 16, purpose: "normal", maximum_active_ms: 180000, reserved_ms: 180000, complete: false } };
  const records = Array.from({ length: 7 }, (_, index) => ({ sequence: index + 1, state: { running: true, renewalsConfirmed: 1,
    heartbeatSuppressed: index === 6, qualification: structuredClone(q), cadence: { suppressionRequested: index === 6,
      firstWorkObservedAtMs: 1000, latestWork: { atMs: 1000 + index * 10000, generation: 2, workDispatched: index + 1 } } } }));
  records.push({ sequence: 8, state: { running: false, deviceRestorationConfirmed: true, deviceLeaseInactive: true,
    qualification: { ...q, safe_stop_complete: true, safe_stop_stage: "fan_paused", revocation_reason: "heartbeat_timeout",
      gate_closed_ms: 204800, shutdown_started_ms: 204826, attempt: { ...q.attempt, complete: true },
      owner_resources: { ...q.owner_resources, phase: "shutdown_complete" } } } });
  records[6].state.authorizationRecovery = { generation: 2, checkpointId: "fixture", matched: null };
  records[7].state.connected = true;
  records[7].state.authorizationRecovery = { generation: 2, checkpointId: "fixture", matched: true };
  const evidence = ["idle", "usb", "mining"].map((phase, index) => ({ phase, summary: review().phases[index], measurement_end_sequence: index === 2 ? 7 : index * 2 + 2 }));
  evidence.push({ phase: "usb_probes" }, { phase: "observer" });
  return { context, records, evidence, fault: { kind: "heartbeats_suppressed", generation: 2, after_sequence: 7 } };
}

test("live cadence requires work per ten-second segment and renewals without requiring accepted shares", async () => {
  // Arrange
  const { judgeCadence } = await import("./cadence-judge.mjs"); const f = liveFixture();
  // Act / Assert
  assert.equal(judgeCadence(f.context, f.records, f.fault, f.evidence).accepted_share_required, false);
  const stalled = structuredClone(f.records); stalled[2].state.cadence.latestWork.workDispatched = 2;
  assert.throws(() => judgeCadence(f.context, stalled, f.fault, f.evidence), /cadence_work_segment_missing/u);
  const noRenewal = structuredClone(f.records); for (const record of noRenewal) record.state.renewalsConfirmed = 0;
  assert.throws(() => judgeCadence(f.context, noRenewal, f.fault, f.evidence), /cadence_work_duration/u);
});

test("live cadence cannot count work observations from another generation", async () => {
  // Arrange
  const { judgeCadence } = await import("./cadence-judge.mjs"); const f = liveFixture();
  for (const record of f.records) if (record.state.cadence) record.state.cadence.latestWork.generation = 3;
  // Act / Assert
  assert.throws(() => judgeCadence(f.context, f.records, f.fault, f.evidence));
});

test("USB requires twelve actual device probe preparations within the half-open capture window", () => {
  // Arrange
  const value = review(), usb = value.phases[1];
  // Act / Assert
  requireCadencePhase(value, "usb");
  for (const change of [{ maxProbeCount: 11 }, { maxProbeCount: 13 }, { firstMaxProbeAtUs: usb.startedAtUs - 1 },
    { lastMaxProbeAtUs: usb.firstMaxProbeAtUs - 1 }, { lastMaxProbeAtUs: usb.startedAtUs + 60000000 }]) {
    const changed = structuredClone(value); Object.assign(changed.phases[1], change);
    assert.throws(() => requireCadencePhase(changed, "usb"), /cadence_device_probes/u);
  }
  const nonUsb = review(); nonUsb.phases[0].maxProbeCount = 1;
  assert.throws(() => requireCadencePhase(nonUsb, "idle"), /cadence_device_probes/u);
});

test("host stall followed by twelve late probes cannot substitute for device capture load", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  for (const phase of f.phases) {
    Object.assign(phase.review.phases[1], { maxProbeCount: 0, firstMaxProbeAtUs: 0, lastMaxProbeAtUs: 0 });
    await f.write(`cadence-${phase.phase}.json`, phase);
  }
  for (const record of f.records) Object.assign(record.state.cadence.review.phases[1], { maxProbeCount: 0, firstMaxProbeAtUs: 0, lastMaxProbeAtUs: 0 });
  // Act / Assert: all twelve host receipts and witnesses remain present and otherwise valid.
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_device_probes" });
});

test("intended heartbeat loss accepts only the correlated Gate failure/category pair after intent", async () => {
  // Arrange
  const { judgeCadence } = await import("./cadence-judge.mjs"); const f = liveFixture();
  const loss = structuredClone(f.records[6]); loss.sequence = 8;
  Object.assign(loss.state, { running: false, status: "disconnected", failure: "window_control_failed", serialFailureCategory: "liveness_lost" });
  f.records[7].sequence = 9; f.records.splice(7, 0, loss);
  // Act / Assert
  assert.equal(judgeCadence(f.context, f.records, f.fault, f.evidence).gate_close_delay_ms, 2800);
  const ordinaryLoss = structuredClone(f.records); delete ordinaryLoss[7].state.failure;
  judgeCadence(f.context, ordinaryLoss, f.fault, f.evidence);
  for (const change of [{ failure: "cleanup_failed" }, { serialFailureCategory: "write_failed" }, { heartbeatSuppressed: false },
    { ownerResourceFailure: {} }, { cadence: { ...loss.state.cadence, suppressionRequested: false } }]) {
    const changed = structuredClone(f.records); Object.assign(changed[7].state, change);
    assert.throws(() => judgeCadence(f.context, changed, f.fault, f.evidence), /cadence_unexpected_browser_failure/u);
  }
  const early = structuredClone(f.records); early[7].sequence = 6;
  assert.throws(() => judgeCadence(f.context, early, f.fault, f.evidence), /cadence_unexpected_browser_failure/u);
});

test("frozen mining review collected after cooling does not extend observer lifetime", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  const finalState = structuredClone(f.records[5].state);
  f.records[5].state.running = true; f.records[5].state.deviceLeaseInactive = false;
  f.records.push({ sequence: 7, state: finalState });
  Object.assign(f.phases[2], { finished_sequence: 7, measurement_end_sequence: 6, collected_at_unix_ms: 346000 });
  await f.write("cadence-mining.json", f.phases[2]);
  // Act / Assert
  const evidence = await requireCadenceEvidence(f.root, f.context, f.records);
  assert.equal(evidence[2].measurement_end_sequence, 6);
  assert.equal(evidence[4].result.closedAtUnixMs, 210000);
});

test("observer must remain through the five-second fault tail, while ordinary phases collect synchronously", async t => {
  // Arrange
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  const f = await sealedFixture(t);
  // Act / Assert
  await f.write("cadence-observer-result.json", { ...f.result, closedAtUnixMs: 205999 });
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_observer_phase_coverage" });
  await f.write("cadence-observer-result.json", f.result);
  await f.write("cadence-idle.json", { ...f.phases[0], collected_at_unix_ms: 61001 });
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_nonmining_collection" });
});

test("mining measurement ends at the witnessed fault and before device gate closure", async () => {
  // Arrange
  const { judgeCadence } = await import("./cadence-judge.mjs"); const f = liveFixture();
  // Act / Assert
  const wrongEnd = structuredClone(f.evidence); wrongEnd[2].measurement_end_sequence = 6;
  assert.throws(() => judgeCadence(f.context, f.records, f.fault, wrongEnd), /cadence_measurement_fault_binding/u);
  const closedEarly = structuredClone(f.records); Object.assign(closedEarly.at(-1).state.qualification, { gate_closed_ms: 200999, last_valid_heartbeat_ms: 198199, shutdown_started_ms: 201025 });
  assert.throws(() => judgeCadence(f.context, closedEarly, f.fault, f.evidence), /cadence_capture_after_gate/u);
});

test("same-boot u32 millisecond rollover cannot reverse capture and gate ordering", async () => {
  // Arrange
  const { judgeCadence } = await import("./cadence-judge.mjs"); const f = liveFixture();
  f.evidence[2].summary.endedAtUs = (0x100000000 - 2) * 1000;
  Object.assign(f.records.at(-1).state.qualification, { last_valid_heartbeat_ms: 0xffffffff - 100, gate_closed_ms: 100, shutdown_started_ms: 126 });
  // Act / Assert
  assert.equal(judgeCadence(f.context, f.records, f.fault, f.evidence).gate_close_delay_ms, 201);
  f.evidence[2].summary.endedAtUs = (0x100000000 + 101) * 1000;
  assert.throws(() => judgeCadence(f.context, f.records, f.fault, f.evidence), /cadence_capture_after_gate/u);
});

test("a delayed process reap cannot hide observer closure before the fault tail ends", async t => {
  // Arrange
  const f = await sealedFixture(t); const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  const { digest } = await import("./contract.mjs");
  f.rows[2].observedAtUnixMs = 205999; f.rows[2].event.elapsedMs = 204999;
  await f.writeLines("cadence-observer.jsonl", f.rows);
  await f.write("cadence-observer-result.json", { ...f.result, journalSha256: digest(f.rows.map(row => JSON.stringify(row) + "\n").join("")) });
  // Act / Assert
  await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records), { code: "cadence_observer_journal" });
});

test("mining measurement witness binds exact context, fault sequence and host observation", async t => {
  // Arrange / Act / Assert
  const { requireCadenceEvidence } = await import("./cadence-judge.mjs");
  const { fileDigest } = await import("./contract.mjs");
  const valid = await sealedFixture(t);
  const evidence = await requireCadenceEvidence(valid.root, valid.context, valid.records);
  assert.equal(evidence[2].measurement_end_sha256, await fileDigest(valid.resolve(valid.root, "cadence-mining-measurement-end.json")));
  for (const change of [{ context_sha256: "0".repeat(64) }, { sequence: 5 }, { observedAtUnixMs: 201001 }, { private: "fixture" }]) {
    const f = await sealedFixture(t); await f.write("cadence-mining-measurement-end.json", { ...f.measurement, ...change });
    await assert.rejects(requireCadenceEvidence(f.root, f.context, f.records));
  }
});
