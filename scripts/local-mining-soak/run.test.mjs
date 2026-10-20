import assert from "node:assert/strict";
import test from "node:test";
import { runDeadlineMs, SOAKS } from "./judge.mjs";

const SOAK_ACTIVE_MS = SOAKS["001"].activeMs;
const RUN_DEADLINE_MS = runDeadlineMs(SOAKS["001"]);
import { runSoak, UNREACHABLE_STOP_MS } from "./run.mjs";

const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };

/** A fake device that mines once resumed, with `overrides(t)` applied to each mining reading. */
function fakeDevice({ overrides = () => ({}), failInfoAfterMs = Infinity, startAfterMs = 4000 } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, resumedAt: null, pool: {}, calls: [] };
  const info = () => {
    if (state.resumedAt !== null && clock.now - state.resumedAt > failInfoAfterMs && !state.paused) throw new Error("unreachable");
    const active = !state.paused && state.resumedAt !== null && clock.now - state.resumedAt >= startAfterMs;
    const minedMs = active ? clock.now - state.resumedAt - startAfterMs : 0;
    return {
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
      bootSession: "s".repeat(32), miningPaused: state.paused, miningActivity: active ? "active" : "paused",
      startMiningOnBoot: false, hashRate: active ? 500 : 0, sharesAccepted: Math.floor(minedMs / 60_000), sharesRejected: 0,
      temp: 55, vrTemp: 50, voltage: 5100, power: active ? 12 : 1, fanrpm: 4000, overheat_mode: 0, ...state.pool,
      powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
      ...(active ? overrides(clock.now) : {}),
    };
  };
  const client = {
    async info() { return info(); },
    async patch(settings) { state.calls.push("patch"); state.pool = { stratumURL: settings.stratumURL, stratumPort: settings.stratumPort, stratumUser: settings.stratumUser }; },
    async command(name) { state.calls.push(name); state.paused = name === "pause"; if (name === "resume") state.resumedAt = clock.now; },
  };
  return { client, clock, state };
}

async function run(device, record = null) {
  const events = [];
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS,
    now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; },
    record: record ?? (async (event) => { events.push(event); }) });
  return { verdict, events };
}

test("a healthy device passes and ends paused", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict, events } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.deepEqual(device.state.calls, ["patch", "resume", "pause"]);
  assert.ok(verdict.metrics.active_span_ms >= SOAK_ACTIVE_MS);
  assert.equal(events.at(-1).event, "verdict");
});

test("no journal event carries the pool or the endpoint", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { events } = await run(device);
  // Assert
  assert.equal(/pool\.invalid|owner\.worker/u.test(JSON.stringify(events)), false);
});

test("an over-limit reading stops the soak and pauses at once", async () => {
  // Arrange
  const device = fakeDevice({ overrides: (now) => (now > 120_000 ? { temp: 80 } : {}) });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_chip_temperature"));
  assert.equal(device.state.paused, true);
  assert.ok(device.clock.now < SOAK_ACTIVE_MS);
});

test("mining that never starts stops after the start allowance", async () => {
  // Arrange
  const device = fakeDevice({ startAfterMs: Infinity });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_mining_not_started"));
  assert.equal(device.state.paused, true);
});

test("an unreachable device stops the soak", async () => {
  // Arrange
  const device = fakeDevice({ failInfoAfterMs: 60_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_device_unreachable"));
  assert.ok(UNREACHABLE_STOP_MS === 30_000);
});

test("a device that is not paused at the start refuses before any write", async () => {
  // Arrange
  const device = fakeDevice();
  device.state.paused = false;
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_baseline_not_paused" });
  assert.deepEqual(device.state.calls, []);
});

test("other firmware refuses before any write", async () => {
  // Arrange
  const device = fakeDevice();
  const info = device.client.info;
  device.client.info = async () => ({ ...(await info()), sourceCommit: "d".repeat(40) });
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_firmware_identity" });
  assert.deepEqual(device.state.calls, []);
});

test("a resume that throws after taking effect still pauses", async () => {
  // Arrange: the device resumes, but the reply times out.
  const device = fakeDevice();
  const command = device.client.command;
  device.client.command = async (name) => { await command(name); if (name === "resume") throw new Error("timeout"); };
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_run_error"));
  assert.equal(device.state.paused, true);
});

test("a failing journal never stops the pause", async () => {
  // Arrange
  const device = fakeDevice();
  let writes = 0;
  const record = async () => { writes += 1; if (writes > 3) throw new Error("disk full"); };
  // Act
  const { verdict } = await run(device, record);
  // Assert
  assert.ok(verdict.failures.includes("soak_journal_failed"));
  assert.equal(device.state.paused, true);
});

test("mining that keeps breaking off stops at the run deadline", async () => {
  // Arrange: the pool drops for 30 s every 10 minutes, so the span never reaches 30 minutes.
  const device = fakeDevice({ overrides: (now) => (now % 600_000 < 30_000 ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_deadline"));
  assert.equal(device.state.paused, true);
  assert.ok(device.clock.now <= RUN_DEADLINE_MS + 200_000);
});

test("a baseline whose idle power is not measured refuses before any write", async () => {
  // Arrange
  const device = fakeDevice();
  const info = device.client.info;
  device.client.info = async () => ({ ...(await info()), powerStatus: { state: "unavailable" } });
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_baseline_not_stopped" });
  assert.deepEqual(device.state.calls, []);
});

test("an interruption before the resume never starts mining", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, isInterrupted: () => true });
  // Assert
  assert.ok(verdict.failures.includes("soak_run_error"));
  assert.ok(!device.state.calls.includes("resume"));
});

test("both powered-down samples are recorded", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { events } = await run(device);
  // Assert
  const kinds = events.map((event) => event.event);
  assert.ok(kinds.indexOf("stopped_sample") >= 0 && kinds.indexOf("stopped_sample") < kinds.indexOf("final"));
});

test("soak 002 runs for two hours", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, soak: SOAKS["002"] });
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.ok(verdict.metrics.active_span_ms >= SOAKS["002"].activeMs);
});

test("soak 002 stops at its own deadline", async () => {
  // Arrange: recovered outages take half the time, so two hours never accumulate by the deadline.
  const device = fakeDevice({ overrides: (now) => (now % 80_000 < 40_000 ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, soak: SOAKS["002"] });
  // Assert
  assert.ok(verdict.failures.includes("soak_deadline"));
  assert.ok(device.clock.now <= runDeadlineMs(SOAKS["002"]) + 200_000);
  assert.equal(device.state.paused, true);
});

async function runSoak002(device) {
  const events = [];
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); }, soak: SOAKS["002"] });
  return { verdict, events };
}

test("soak 002 passes through recovered outages like attempt 001's", async () => {
  // Arrange: two outages of about 45 s, 72 minutes apart; the device recovers each time.
  const outage = (now) => (now > 2_000_000 && now < 2_045_000) || (now > 6_300_000 && now < 6_345_000);
  const device = fakeDevice({ overrides: (now) => (outage(now) ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const { verdict } = await runSoak002(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outage_count, 2);
  assert.ok(verdict.metrics.accumulated_active_ms >= SOAKS["002"].activeMs);
});

test("soak 002 stops at once on an outage past its bound", async () => {
  // Arrange: a 90 s outage after 10 minutes.
  const device = fakeDevice({ overrides: (now) => (now > 600_000 && now < 690_000 ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const { verdict } = await runSoak002(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_outage_unrecovered"));
  assert.ok(device.clock.now < 1_000_000);
  assert.equal(device.state.paused, true);
});

test("soak 002 stops on an outage that becomes unreachable", async () => {
  // Arrange: idle for 40 s, then unreachable; the outage passes 60 s while unreachable.
  const device = fakeDevice({ overrides: (now) => (now > 600_000 && now < 640_000 ? { miningActivity: "safe_blocked" } : {}),
    failInfoAfterMs: 630_000 });
  // Act
  const { verdict } = await runSoak002(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_outage_unrecovered"), JSON.stringify(verdict.failures));
  assert.ok(device.clock.now < 600_000 + 4_000 + 120_000 + 200_000);
});

test("soak 003 accumulates four hours through recovered outages", async () => {
  // Arrange: a 45 s outage every 72 minutes, as soak 002 attempt 001 met.
  const outage = (now) => now % 4_320_000 > 2_000_000 && now % 4_320_000 < 2_045_000;
  const device = fakeDevice({ overrides: (now) => (outage(now) ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, soak: SOAKS["003"] });
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.ok(verdict.metrics.accumulated_active_ms >= SOAKS["003"].activeMs);
  assert.ok(device.clock.now <= runDeadlineMs(SOAKS["003"]));
});

test("soak 005 accumulates eight hours within its nine-hour deadline", async () => {
  // Arrange: a 45 s outage every 90 minutes.
  const outage = (now) => now % 5_400_000 > 3_000_000 && now % 5_400_000 < 3_045_000;
  const device = fakeDevice({ overrides: (now) => (outage(now) ? { miningActivity: "safe_blocked" } : {}) });
  // Act
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, soak: SOAKS["005"] });
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.ok(verdict.metrics.accumulated_active_ms >= SOAKS["005"].activeMs);
  assert.ok(device.clock.now <= runDeadlineMs(SOAKS["005"]));
});

test("a device whose last reset was a panic refuses before any write", async () => {
  // Arrange
  const device = fakeDevice();
  const info = device.client.info;
  device.client.info = async () => ({ ...(await info()), resetReasonCategory: "panic" });
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_previous_panic" });
  assert.deepEqual(device.state.calls, []);
});
