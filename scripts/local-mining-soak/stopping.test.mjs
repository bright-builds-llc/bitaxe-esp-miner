import assert from "node:assert/strict";
import test from "node:test";
import { SOAKS } from "./judge.mjs";
import { runSoak } from "./run.mjs";
import { judgeStopPhase } from "./stopping.mjs";

const SOAK = SOAKS["stopping-001"];
const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };
const FRESH = { power: true, input: true, chipTemp: true, fan: true };

const sample = (atMs, miningActivity, powerW, fresh = FRESH) => ({ atMs, miningActivity, powerW, fresh });

/**
 * A fake device whose pause idles the ASIC at once, then confirms the stop after `stopMs`;
 * meanwhile it reads `during` (`stopping` as fixed, `active` as before the fix).
 */
function fakeDevice({ during = "stopping", stopMs = 14_000, loseFirstPauseResponse = false, startAfterMs = 4000,
  frozenHashrate = false } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, resumedAt: null, pausedAt: null, pool: {}, calls: [] };
  const activity = () => {
    if (!state.paused) return state.resumedAt !== null && clock.now - state.resumedAt >= startAfterMs ? "active" : "paused";
    return state.pausedAt !== null && clock.now - state.pausedAt < stopMs ? during : "paused";
  };
  const info = () => {
    const miningActivity = activity();
    const hashing = miningActivity === "active" && !state.paused;
    return {
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
      bootSession: "s".repeat(32), miningPaused: state.paused, miningActivity, startMiningOnBoot: false,
      hashRate: hashing || (frozenHashrate && miningActivity === "stopping") ? 500 : 0, sharesAccepted: hashing ? Math.floor((clock.now - state.resumedAt) / 30_000) : 0, sharesRejected: 0,
      temp: 55, vrTemp: 50, voltage: 5100, power: hashing ? 12 : 1, fanrpm: 4000, overheat_mode: 0, ...state.pool,
      powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
    };
  };
  const client = {
    async info() { return info(); },
    async patch(settings) { state.pool = { stratumURL: settings.stratumURL, stratumPort: settings.stratumPort, stratumUser: settings.stratumUser }; },
    async command(name) {
      state.calls.push(name);
      if (name === "resume") { state.paused = false; state.resumedAt = clock.now; }
      if (name === "pause" && !state.paused) {
        state.paused = true; state.pausedAt = clock.now;
        // The device paused, but the response is lost; the client sees a timeout.
        if (loseFirstPauseResponse) { clock.now += 10_000; throw new Error("timeout"); }
      }
    },
  };
  return { client, clock, state };
}

async function run(device, soak = SOAK) {
  const events = [];
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); }, soak });
  return { verdict, events };
}

test("a pause that reads stopping until the stop confirms passes", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.result, "passed");
  assert.ok(verdict.metrics.stop_stopping_samples >= 6);
  assert.equal(verdict.metrics.stop_active_idle_samples, 0);
});

test("the pre-fix device, reading active at idle power, fails", async () => {
  // Arrange
  const device = fakeDevice({ during: "active" });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(verdict.result, "failed");
  assert.ok(verdict.failures.includes("stop_active_while_idle"));
  assert.ok(verdict.failures.includes("stop_stopping_not_seen"));
});

test("every sample of the pause is journaled", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { events, verdict } = await run(device);
  // Assert
  assert.equal(events.filter((event) => event.event === "stop_sample").length, verdict.metrics.stop_samples);
});

test("a stop that confirms faster than one sample fails as unseen", () => {
  // Arrange
  const samples = [sample(0, "paused", 1), sample(2000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_stopping_not_seen"]);
});

test("active while still hashing, then stopping, then paused passes", () => {
  // Arrange
  const samples = [sample(0, "active", 12), sample(2000, "stopping", 1), sample(4000, "paused", 1), sample(6000, "paused", 1)];
  // Act
  const { failures, metrics } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, []);
  assert.equal(metrics.stop_first_stopping_ms, 2000);
  assert.equal(metrics.stop_first_paused_ms, 4000);
});

test("active after stopping fails as a regression", () => {
  // Arrange
  const samples = [sample(0, "stopping", 1), sample(2000, "active", 12), sample(4000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_status_regressed"]);
});

test("a failed stop's safe_blocked is unexpected", () => {
  // Arrange
  const samples = [sample(0, "stopping", 1), sample(2000, "safe_blocked", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_unexpected_status"]);
});

test("a gap longer than the bound fails", () => {
  // Arrange
  const samples = [sample(0, "stopping", 1), sample(12_000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_sample_gap"]);
});

test("no stop samples fail", () => {
  // Arrange
  const samples = [];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.ok(failures.includes("stop_samples_missing"));
});

test("a pause whose response is lost is still sampled from the attempt", async () => {
  // Arrange: the pause takes effect, but its response times out after 10 s.
  const device = fakeDevice({ loseFirstPauseResponse: true });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.ok(verdict.metrics.stop_stopping_samples >= 1);
});

test("after a live stop only the stop phase's metrics are kept", async () => {
  // Arrange: mining never starts, so the run stops before the pause.
  const device = fakeDevice({ startAfterMs: Infinity });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures.filter((code) => code.startsWith("stop_")), []);
  assert.ok(verdict.failures.includes("soak_mining_not_started"));
  assert.equal(typeof verdict.metrics.stop_samples, "number");
});

test("other soaks journal no stop samples", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { events } = await run(device, SOAKS["001"]);
  // Assert
  assert.equal(events.filter((event) => event.event === "stop_sample").length, 0);
});

test("active at fresh idle power fails", () => {
  // Arrange
  const samples = [sample(0, "active", 3), sample(2000, "stopping", 1), sample(4000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_active_while_idle"]);
});

test("active at a stale power reading is not judged idle", () => {
  // Arrange
  const stale = { ...FRESH, power: false };
  const samples = [sample(0, "active", 1, stale), sample(2000, "stopping", 1), sample(4000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, []);
});

test("stopping after paused fails as a regression", () => {
  // Arrange
  const samples = [sample(0, "paused", 1), sample(2000, "stopping", 1), sample(4000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_status_regressed"]);
});

test("a status named like an object property is unexpected", () => {
  // Arrange
  const samples = [sample(0, "stopping", 1), sample(2000, "toString", 1), sample(4000, "paused", 1)];
  // Act
  const { failures } = judgeStopPhase(samples);
  // Assert
  assert.deepEqual(failures, ["stop_unexpected_status"]);
});

test("stopping-002 passes when the hashrate reads zero while stopping", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict } = await run(device, SOAKS["stopping-002"]);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.stop_hashrate_frozen_samples, 0);
});

test("stopping-002 fails on a hashrate frozen while stopping", async () => {
  // Arrange: the owner blocked through the cooling proof, as before ADR-0016's polled proof.
  const device = fakeDevice({ frozenHashrate: true });
  // Act
  const { verdict } = await run(device, SOAKS["stopping-002"]);
  // Assert
  assert.deepEqual(verdict.failures, ["stop_hashrate_frozen"]);
});

test("stopping-001 only measures a frozen hashrate", async () => {
  // Arrange
  const device = fakeDevice({ frozenHashrate: true });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.ok(verdict.metrics.stop_hashrate_frozen_samples > 0);
});

test("the first stopping sample may still show the hashrate", () => {
  // Arrange
  const samples = [
    { ...sample(0, "stopping", 10), hashRate: 430 },
    { ...sample(2000, "stopping", 1), hashRate: 0 },
    { ...sample(4000, "paused", 1), hashRate: 0 },
  ];
  // Act
  const { failures, metrics } = judgeStopPhase(samples, { requireHashrateZeroed: true });
  // Assert
  assert.deepEqual(failures, []);
  assert.equal(metrics.stop_hashrate_frozen_samples, 0);
});
