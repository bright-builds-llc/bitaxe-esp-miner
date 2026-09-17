import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { judgeObserver, judgeShareSafety } from "./safety-judge.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";

// This exercises only safety joins; it makes no protocol, nonce or hardware-pass claim.
function safetyFixture() {
  const context = { scope: "share", gate_commit: "b".repeat(40), firmware_commit: "a".repeat(40), app_elf_sha256: "c".repeat(64) };
  const q = { generation: 2, attempt: { ordinal: 18, purpose: "normal", maximum_active_ms: 180000, reserved_ms: 180000, complete: true },
    budget_reserved_ms: 240000, budget_complete: true, active_limit_ms: 180000, shutdown_budget_ms: 15550,
    safe_stop_complete: true, safe_stop_stage: "fan_paused", revocation_reason: "heartbeat_timeout", active_ms: 9000,
    generation_elapsed_ms: 10000, work_dispatched: 2, nonce_work_correlations: 1, submitted: 1, accepted: 1, rejected: 0,
    last_valid_heartbeat_ms: 7000, gate_closed_ms: 9800, shutdown_started_ms: 9801,
    watchdog_alive: true, mine_on_boot: false, voltage_fresh: true, voltage_volts: 5, power_fresh: true, power_watts: 5,
    temperature_fresh: true, chip_temp_celsius: 35, fan_fresh: true, fan_rpm: 3000,
    owner_resources: { generation: 2, phase: "shutdown_complete", stack_free_bytes: 4096 } };
  const active = { ...q, safe_stop_complete: false, safe_stop_stage: "not_started", active_ms: 8000, generation_elapsed_ms: 9000,
    owner_resources: { generation: 2, phase: "active", stack_free_bytes: 4096 } };
  const base = state(context);
  const restored = { ...base, qualification: q, authorizationRecovery: { generation: 2, matched: true, checkpointId: "synthetic" } };
  const rows = [{ sequence: 1, atHostMs: 10, state: { ...base, running: true, qualification: active } },
    { sequence: 2, atHostMs: 101, state: { ...base, running: true, heartbeatSuppressed: true, qualification: active } },
    { sequence: 3, atHostMs: 145100, state: restored }];
  const record = { workerGeneration: 2, admittedAtDeviceUs: 1000000, observedAtUs: 15000000,
    events: [{ kind: "cooled", atDeviceUs: 12000000 }] };
  const execution = { restoration: { observedSequence: 3 }, faultConfirmed: { confirmedAtHostMs: 100 },
    fault: { suppressionRequestedAtHostMs: 99, lastValidHeartbeatAtDeviceUs: 7000000,
      gateClosedAtDeviceUs: 9800000, shutdownStartedAtDeviceUs: 9801000 } };
  return { context, rows, devices: [{ record }], execution, accounting: { after: { state: restored } } };
}
const inspect = f => judgeShareSafety(f.context, f.rows, f.devices, f.execution, f.accounting);

test("safety joins actual native atoms and existing4096 owner limit", () => {
  const f = safetyFixture();
  assert.equal(inspect(f).heartbeatToRevocationMs, 2800);
  f.rows[0].state.qualification.owner_resources.stack_free_bytes = 4095;
  assert.throws(() => inspect(f), { code: "v2_active_owner_resources" });
});

test("safety rejects counter reset, event-time substitution, early re-admission and missing checkpoint", () => {
  const cases = [
    [f => { f.rows[0].state.qualification.work_dispatched = 3; }, "v2_counter_reset"],
    [f => { f.execution.fault.gateClosedAtDeviceUs++; }, "v2_safety_atom_join"],
    [f => { f.rows[2].atHostMs--; }, "v2_restoration_wait"],
    [f => { f.accounting.after.state.authorizationRecovery.matched = false; }, "v2_authorization_checkpoint"],
    [f => { f.rows[2].state.renewalsConfirmed = 1; }, "v2_renewal_after_fault"],
  ];
  for (const [mutate, code] of cases) { const f = safetyFixture(); mutate(f); assert.throws(() => inspect(f), { code }); }
});

test("postwork checkpoint does not rewrite the original high-water comparison", () => {
  const f = safetyFixture(); f.accounting.after.state.preservation.authorization_high_water_match = false;
  inspect(f);
  assert.equal(f.accounting.after.state.preservation.authorization_high_water_match, false);
});

test("passive observer binds stream bytes and tail to confirmed cut without crossing clock domains", async t => {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), "v2-observer-judge-")));
  t.after(() => rm(root, { recursive: true, force: true }));
  const context = { cadence_observer: { sha256: "a".repeat(64) } }, contextSha256 = sha256(JSON.stringify(context));
  const start = { schema: "str005-v2-observer-claim-v1", contextSha256, atHostMs: 3, sourceObservedAtDeviceUs: 1,
    bootOrdinal: 1, workerGeneration: 2, stateSequence: 1, binarySha256: context.cadence_observer.sha256, lifetimeMs: 360000 };
  const stop = { schema: "str005-v2-observer-stop-v1", contextSha256, requestedAtHostMs: 6000, completedAtHostMs: 6001,
    tailMs: 5000, requestedCleanupMs: 1 };
  const entries = ["connected", "arrival", "closed"].map((event, index) => ({ sequence: index + 1, observedAtUnixMs: 100001 + index,
    event: { schema: "cpu0-cadence-observer-v1", event, elapsedMs: index, messageCount: index > 0 ? 1 : 0,
      totalBytes: index > 0 ? 3 : 0, byteCount: index === 1 ? 3 : 0, reason: index === 2 ? "requested" : null } }));
  const text = entries.map(row => JSON.stringify(row) + "\n").join("");
  await writeNew(resolve(root, "observer-start.claim.json"), start); await writeNew(resolve(root, "observer-stop.json"), stop);
  await writeFile(resolve(root, "cadence-observer.jsonl"), text, { mode: 0o600 });
  await writeNew(resolve(root, "cadence-observer-result.json"), { schema: "worker-cadence-observer-result-v1", connected: true, closed: true,
    exitCode: 0, reason: "requested", cleanupComplete: true, startedAtUnixMs: 100000, connectedAtUnixMs: 100001, closedAtUnixMs: 100004,
    messageCount: 1, totalBytes: 3, eventCount: 3, journalSha256: sha256(text) });
  const inspect = () => judgeObserver(root, context, [{ atHostMs: 2 }], [{ atHostMs: 4, record: { bootOrdinal: 1, workerGeneration: 2 } }], { confirmedAtHostMs: 1000 });
  assert.equal((await inspect()).tailMs, 5000);
  stop.tailMs = 5001; await writeFile(resolve(root, "observer-stop.json"), JSON.stringify(stop));
  await assert.rejects(inspect(), { code: "v2_observer_stop_join" });
  stop.tailMs = 5000; await writeFile(resolve(root, "observer-stop.json"), JSON.stringify(stop));
  await writeFile(resolve(root, "cadence-observer.jsonl"), text + "\n");
  await assert.rejects(inspect(), { code: "v2_observer_result" });
});
