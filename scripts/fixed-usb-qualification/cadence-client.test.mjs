import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";
import test from "node:test";
import { createCadenceSupervisor } from "./cadence-client.mjs";

function fixture(options = {}) {
  let elapsed = 0, observerStartedAt, observerStoppedAt, fault, miningBegan, loaded = false, trusted = false;
  const events = [], publicStates = [], arms = new Map();
  const local = { connected: true, running: false, deviceLeaseInactive: true, serialOwnershipReleased: false,
    heartbeatSuppressed: false, cadence: { enabled: true, suppressionRequested: false } };
  const now = () => 1000000 + elapsed;
  const record = event => { events.push({ event, at: elapsed }); };
  function update() {
    if (miningBegan === undefined || options.noCut) return;
    if (elapsed >= miningBegan + 62000 && fault === undefined) {
      fault = now(); local.heartbeatSuppressed = true; local.cadence.suppressionRequested = true; record("fault");
    }
    if (fault !== undefined && now() >= fault + 6000) {
      local.running = false; local.connected = false; local.serialOwnershipReleased = true;
    }
  }
  const worker = {
    state: () => structuredClone(local),
    async submitCoolingReview() { assert.equal(observerStartedAt, undefined); record("cooling"); elapsed += 3000; },
    async submitBudgetReview() { record("budget"); },
    async cadenceEndpoint() { record("endpoint"); return { ipv4: "192.168.222.123", controlSessionBindingSha256: "private" }; },
    async cadenceArm(phase) { record(`arm-${phase}`); elapsed += 2000; arms.set(phase, elapsed); return { phase, generation: 7, armedAtUs: elapsed * 1000 }; },
    async cadenceUsbPhase() { record("probes"); elapsed += 55000; },
    async cadenceReview() {
      const phase = arms.has("mining") ? "mining" : arms.has("usb") ? "usb" : "idle";
      if (phase !== "mining") assert.ok(elapsed - arms.get(phase) >= 62500, "arm setup must not shorten capture waiting");
      record(`review-${phase}`); return { phases: [] };
    },
    async prepareStartAuthorization() { record("authorize"); if (options.primary) throw options.primary; },
    async loadSignedWindow() { record("load"); loaded = true; },
    async startWindow() { assert.ok(loaded); record("start"); elapsed += 11000; miningBegan = elapsed; local.running = true; local.deviceLeaseInactive = false; },
    async stop() { record("stop"); if (options.stopFailure) throw options.stopFailure; local.running = false; local.deviceLeaseInactive = true; },
    async close() { record("close"); loaded = false; local.connected = false; local.running = false; local.serialOwnershipReleased = true; },
    connect() {
      assert.ok(trusted, "fresh connect must be invoked synchronously in the trusted resume task");
      assert.equal(loaded, false); assert.ok(observerStoppedAt !== undefined); record("connect");
      local.connected = true; local.deviceLeaseInactive = true; local.serialOwnershipReleased = false; return Promise.resolve();
    },
    async submitAttemptCompletion() {
      record("completion"); assert.ok(now() >= fault + 145000); assert.ok(observerStoppedAt !== undefined);
      local.connected = false; local.serialOwnershipReleased = true;
      return { result: "passed", cleanup_confirmed: true, ordinal: 16, purpose: "normal", cumulative_charged_ms: 1380000 };
    },
  };
  const supervisor = createCadenceSupervisor({ worker: () => worker, now, monotonic: () => elapsed,
    flush: async () => { update(); }, wait: async ms => { elapsed += ms; update(); }, every: () => () => {},
    maybeOnState: value => publicStates.push(value),
    fetch: async (route, init) => {
      update(); const input = init.body ? JSON.parse(init.body) : undefined;
      let value;
      if (route === "/cadence/status") value = { observer: { connected: observerStoppedAt === undefined, alive: observerStoppedAt === undefined,
        failed: options.observerFailure && elapsed > 10000 ? "runtime" : null }, faultObservedAtUnixMs: fault ?? null };
      else if (route === "/cadence/observer-context") { record("endpoint-context"); value = { nonce: "one-use" }; }
      else if (route === "/cadence/observer/start") {
        assert.equal(input.endpoint.ipv4, "192.168.222.123"); observerStartedAt = elapsed; record("observer-start"); value = { observer_connected: true };
      } else if (route === "/cadence/observer/stop") {
        if (observerStoppedAt === undefined) { observerStoppedAt = elapsed; record("observer-stop"); }
        value = { reason: "requested", closed: true, cleanupComplete: true, exitCode: 0 };
      } else if (route === "/cadence/phase/start") value = { cadence_phase_started: input.arm.phase };
      else if (route === "/cadence/phase/finish") { record(`finish-${input.phase}`); value = { cadence_phase_saved: input.phase }; }
      else throw new Error(`Unexpected synthetic route ${route}`);
      return { ok: true, json: async () => value };
    },
  });
  return { supervisor, events, publicStates, local,
    observer: () => ({ started: observerStartedAt, stopped: observerStoppedAt, fault: fault === undefined ? undefined : fault - 1000000 }),
    elapse: ms => { elapsed += ms; },
    resume: () => { trusted = true; const result = supervisor.resumeQualification(); trusted = false; return result; } };
}

test("automatic qualification finishes measurements and closes observer before an unbounded reconnect wait", async () => {
  // Arrange
  const f = fixture(); const completion = f.supervisor.runQualification();
  // Act
  await setImmediate();
  // Assert
  assert.equal(f.supervisor.state().stage, "awaiting_reconnect");
  const observed = f.observer();
  assert.ok(observed.stopped >= observed.fault + 5000);
  assert.ok(observed.stopped - observed.started < 360000);
  assert.deepEqual(f.events.filter(value => ["cooling", "observer-start", "arm-idle", "review-idle", "arm-usb", "probes", "review-usb", "arm-mining", "authorize", "load", "start", "observer-stop"].includes(value.event)).map(value => value.event),
    ["cooling", "observer-start", "arm-idle", "review-idle", "arm-usb", "probes", "review-usb", "arm-mining", "authorize", "load", "start", "observer-stop"]);
  assert.ok(f.events.find(value => value.event === "close").at >= observed.fault + 145000);
  f.elapse(1000000);
  assert.equal((await f.resume()).result, "passed"); assert.equal((await completion).result, "passed");
  assert.equal(f.supervisor.state().stage, "complete");
  assert.ok(f.events.findIndex(value => value.event === "connect") < f.events.findIndex(value => value.event === "review-mining"));
  assert.ok(!JSON.stringify(f.publicStates).includes("192.168.222.123")); assert.ok(!JSON.stringify(f.publicStates).includes("private"));
});

test("a primary signing failure is retained and both owned resources are cleaned without retry", async () => {
  const primary = new Error("synthetic signing failure"); const f = fixture({ primary });
  await assert.rejects(f.supervisor.runQualification(), error => error === primary);
  assert.ok(f.events.some(value => value.event === "stop")); assert.ok(f.events.some(value => value.event === "close"));
  assert.ok(f.events.some(value => value.event === "observer-stop")); assert.ok(!f.events.some(value => value.event === "start"));
  await assert.rejects(f.supervisor.runQualification(), /cadence_run_consumed/u);
});

test("cleanup failure is reported alongside the original cause and still closes the Worker", async () => {
  const primary = new Error("synthetic primary"), stopFailure = new Error("synthetic stop failure");
  const f = fixture({ primary, stopFailure });
  await assert.rejects(f.supervisor.runQualification(), error => error instanceof AggregateError && error.errors[0] === primary && error.errors.includes(stopFailure));
  assert.ok(f.events.some(value => value.event === "close")); assert.ok(f.events.some(value => value.event === "observer-stop"));
});

test("observer loss aborts an idle phase before any mining authorization", async () => {
  const f = fixture({ observerFailure: true });
  await assert.rejects(f.supervisor.runQualification(), /cadence_observer_failed/u);
  assert.ok(!f.events.some(value => value.event === "authorize")); assert.ok(f.events.some(value => value.event === "close"));
});

test("missing actual heartbeat-cut evidence cannot advance to cooling or reconnect", async () => {
  const f = fixture({ noCut: true });
  await assert.rejects(f.supervisor.runQualification(), /cadence_fault_timeout/u);
  assert.equal(f.supervisor.state().stage, "failed"); assert.ok(!f.events.some(value => value.event === "connect"));
  assert.ok(f.events.some(value => value.event === "stop"));
});

test("fresh reconnect is unavailable before the observer closes and recovery wait completes", async () => {
  const f = fixture(); await assert.rejects(f.resume(), /cadence_reconnect_not_armed/u);
  assert.equal(f.events.length, 0);
});
