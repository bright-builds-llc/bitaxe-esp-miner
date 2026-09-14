import assert from "node:assert/strict";
import { chmod, mkdtemp, readdir, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { createCadenceRoutes } from "./cadence-server.mjs";
import { CADENCE_SCHEMA, CADENCE_TASK } from "./cadence-contract.mjs";
import { digest, writeNew } from "./contract.mjs";

const ID = Buffer.alloc(16, 1).toString("base64url"), BINDING = Buffer.alloc(32, 2).toString("base64url");
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
async function fixture(t, active = true) {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-routes-"))); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(resolve(root, "TASKS.md"), `${active ? "## Active" : "## Future"}\n### ${CADENCE_TASK} | fixture\n`);
  const context = { schema: CADENCE_SCHEMA, firmware_root: root, firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64) };
  const current = { sequence: 1, reviewedBinding: { binding: BINDING, expires: 20000 }, lastState: {
    schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "ready", connected: true, running: false,
    heartbeatSuppressed: false, renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceBaselineConfirmed: true,
    deviceLeaseInactive: true, serialOwnershipReleased: false, preservation: { schema: "worker-preservation-continuity-v1", baseline_id: ID,
      device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false } } };
  for (let cycle = 1; cycle <= 4; cycle++) await writeNew(resolve(root, `cycle-${cycle}.json`), {
    schema: "fixed-usb-cycle-report-v1", cycle, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, baseline_id: ID,
    browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true, device_identity_match: true,
    settings_match: true, authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false });
  let clock = 10000, starts = 0, created = 0;
  const status = { alive: false, connected: false, failed: false, startedAtUnixMs: null };
  const finishResult = { connected: false, closed: false, exitCode: null, reason: "observer_incomplete", cleanupComplete: true, startedAtUnixMs: null };
  const operations = { now: () => clock, createCadenceObserver: () => { created++; return {
    start: async endpoint => {
      assert.deepEqual(endpoint, { ipv4: "192.168.1.2", httpPort: 80 }); starts++;
      Object.assign(status, { alive: true, connected: true, startedAtUnixMs: clock });
      Object.assign(finishResult, { connected: true, closed: true, exitCode: 0, reason: "requested", startedAtUnixMs: clock });
      return { observer_connected: true };
    },
    status: () => status, finish: async () => { status.alive = !finishResult.cleanupComplete; return finishResult; } }; } };
  const create = () => createCadenceRoutes(root, context, () => current, operations);
  const endpoint = { schema: "worker-telemetry-endpoint-v1", ipv4: "192.168.1.2", httpPort: 80, observedAtUs: 1000, bootOrdinal: 1,
    generation: 2, controlSessionBindingSha256: BINDING };
  const f = { root, context, current, status, finishResult, create, created: () => created, starts: () => starts, tick: amount => { clock += amount; } };
  f.endpointInput = async routes => ({ ...(await routes.handle("/cadence/observer-context", {})), endpoint,
    requestedAtUnixMs: clock, receivedAtUnixMs: clock, state: structuredClone(current.lastState) });
  f.connect = async routes => routes.handle("/cadence/observer/start", await f.endpointInput(routes));
  return f;
}
function arm(value, name) {
  const phase = value.phases.find(item => item.phase === name);
  return { schema: "worker-telemetry-cadence-arm-v1", phase: name, armedAtUs: phase.armedAtUs, generation: phase.generation };
}
async function finishPhase(f, routes, value, phase) {
  f.current.sequence++;
  f.current.lastState.cadence = { schema: "worker-cadence-browser-v1", enabled: true, suppressionRequested: false, review: structuredClone(value) };
  return routes.handle("/cadence/phase/finish", { phase, review: value });
}

test("inactive task rejects before even constructing the observer bridge", async t => {
  // Arrange
  const f = await fixture(t, false);
  // Act / Assert
  await assert.rejects(f.create(), { code: "cadence_active_task_required" }); assert.equal(f.created(), 0);
});

test("fresh endpoint challenge requires authenticated binding and never persists private endpoint", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create();
  // Act
  await f.connect(routes);
  // Assert
  assert.equal(f.starts(), 1);
  for (const name of await readdir(f.root)) assert(!(await readFile(resolve(f.root, name), "utf8")).includes("192.168.1.2"));
  await routes.finish();
});

test("resource closure rejects a used observer that has not been reaped", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create(); await f.connect(routes);
  Object.assign(f.finishResult, { cleanupComplete: false, closed: false, exitCode: null, reason: "observer_cleanup" });
  // Act / Assert
  await assert.rejects(routes.finish(), { code: "cadence_observer_cleanup_failed" });
  assert.equal(f.status.alive, true);
});

test("resource closure accepts an unused observer without creating evidence or starting a child", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create(), before = await readdir(f.root);
  // Act
  const result = await routes.finish();
  // Assert
  assert.equal(result.cleanupComplete, true); assert.equal(result.startedAtUnixMs, null);
  assert.equal(f.starts(), 0); assert.deepEqual(await readdir(f.root), before);
});

test("resource closure rejects failed or abnormal used observers even after reaping", async t => {
  // Arrange / Act / Assert
  for (const mode of ["failed", "disconnected", "not_closed", "exit", "reason"]) {
    const f = await fixture(t), routes = await f.create(); await f.connect(routes);
    if (mode === "failed") f.status.failed = true;
    if (mode === "disconnected") f.finishResult.connected = false;
    if (mode === "not_closed") f.finishResult.closed = false;
    if (mode === "exit") f.finishResult.exitCode = 1;
    if (mode === "reason") f.finishResult.reason = "observer_incomplete";
    await assert.rejects(routes.finish(), { code: "cadence_observer_cleanup_failed" });
  }
});

test("expired challenge, changed binding and changed journal state cannot launch observer", async t => {
  // Arrange / Act / Assert
  for (const mode of ["expired", "binding", "state"]) {
    const f = await fixture(t), routes = await f.create(), input = await f.endpointInput(routes);
    if (mode === "expired") f.tick(5001);
    if (mode === "binding") input.endpoint = { ...input.endpoint, controlSessionBindingSha256: Buffer.alloc(32, 3).toString("base64url") };
    if (mode === "state") input.state.renewalsConfirmed = 1;
    await assert.rejects(routes.handle("/cadence/observer/start", input)); assert.equal(f.starts(), 0);
    await assert.rejects(routes.handle("/cadence/observer/start", input), { code: "cadence_endpoint_freshness" });
  }
});

test("observer failure and phase reordering block arming and signing", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create(), value = review(); await f.connect(routes);
  // Act / Assert
  await assert.rejects(routes.readyToSign(), { code: "cadence_before_work_required" });
  await assert.rejects(routes.handle("/cadence/phase/start", { arm: arm(value, "usb") }), { code: "cadence_arm_shape" });
  f.status.failed = true;
  await assert.rejects(routes.handle("/cadence/phase/start", { arm: arm(value, "idle") }), { code: "cadence_phase_order" });
});

test("v2 context rejects legacy phase metadata before advancement or signing", async t => {
  // Arrange
  const f = await fixture(t); f.context.cadence_diagnostics_version = 2;
  const routes = await f.create(), value = review(); await f.connect(routes);
  await routes.handle("/cadence/phase/start", { arm: arm(value, "idle") });
  // Act / Assert
  await assert.rejects(finishPhase(f, routes, value, "idle"), { code: "cadence_diagnostics_identity" });
  assert(!(await readdir(f.root)).includes("cadence-idle.json"));
  await assert.rejects(routes.readyToSign(), { code: "cadence_before_work_required" });
});

test("phase completion binds arm, advancing journal and immutable earlier summaries", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create(), value = review(); await f.connect(routes);
  await routes.handle("/cadence/phase/start", { arm: arm(value, "idle") });
  // Act / Assert
  await assert.rejects(routes.handle("/cadence/phase/finish", { phase: "idle", review: value }), { code: "cadence_phase_finish_order" });
  await finishPhase(f, routes, value, "idle");
  const saved = JSON.parse(await readFile(resolve(f.root, "cadence-idle.json"), "utf8"));
  assert.equal(saved.context_sha256, digest(JSON.stringify(f.context))); assert.equal(saved.finished_sequence, 2);
  await routes.handle("/cadence/phase/start", { arm: arm(value, "usb") });
  const changed = structuredClone(value); changed.phases[0].maximumExecutionUs++;
  await assert.rejects(finishPhase(f, routes, changed, "usb"), { code: "cadence_frozen_summary_changed" });
});

test("twelve serialized exact probes precede matching-generation mining admission", async t => {
  // Arrange
  const f = await fixture(t), routes = await f.create(), value = review(); await f.connect(routes);
  await routes.handle("/cadence/phase/start", { arm: arm(value, "idle") }); await finishPhase(f, routes, value, "idle");
  await routes.handle("/cadence/phase/start", { arm: arm(value, "usb") });
  // Act / Assert
  await assert.rejects(finishPhase(f, routes, value, "usb"), { code: "cadence_probe_count" });
  for (let ordinal = 1; ordinal <= 12; ordinal++) {
    const scheduledAtMs = ordinal * 5000;
    const probe = { ordinal, scheduledAtMs, startedAtMs: scheduledAtMs, completedAtMs: scheduledAtMs + 500, requestPayloadBytes: 65536, responsePayloadBytes: 65536 };
    await routes.handle("/cadence/probe", probe);
    await assert.rejects(routes.handle("/cadence/probe", probe), { code: "cadence_usb_probe" });
  }
  await finishPhase(f, routes, value, "usb");
  await assert.rejects(routes.handle("/cadence/phase/start", { arm: { ...arm(value, "mining"), generation: 3 } }), { code: "cadence_endpoint_generation_changed" });
  await routes.handle("/cadence/phase/start", { arm: arm(value, "mining") }); await routes.readyToSign();
  f.status.alive = false;
  await assert.rejects(routes.readyToSign(), { code: "cadence_before_work_required" });
});

test("endpoint challenge cannot survive a replaced or expired authenticated review", async t => {
  // Arrange / Act / Assert
  for (const binding of [{ binding: Buffer.alloc(32, 4).toString("base64url"), expires: 20000 }, { binding: BINDING, expires: 9999 }]) {
    const f = await fixture(t), routes = await f.create(), input = await f.endpointInput(routes);
    f.current.reviewedBinding = binding;
    await assert.rejects(routes.handle("/cadence/observer/start", input)); assert.equal(f.starts(), 0);
  }
});
