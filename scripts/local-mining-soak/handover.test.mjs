import assert from "node:assert/strict";
import test from "node:test";
import { SOAKS } from "./judge.mjs";
import { runSoak } from "./run.mjs";

const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };
const LOOP = SOAKS["handover-001"];

/** Local mining from 4 s after the resume, idle while `idle(t)`, and `reset(t)` changes the boot session. */
function fakeLoop({ idle = (t) => t % 180_000 >= 120_000, reset = () => false } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, resumedAt: null, logReads: 0 };
  const client = {
    async info() {
      const active = !state.paused && state.resumedAt !== null && clock.now - state.resumedAt >= 4000 && !idle(clock.now);
      return { sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
        bootSession: reset(clock.now) ? "b".repeat(32) : "a".repeat(32), uptimeSeconds: Math.floor(clock.now / 1000),
        miningPaused: state.paused, miningActivity: active ? "active" : state.paused ? "paused" : "safe_blocked", startMiningOnBoot: false,
        hashRate: active ? 500 : 0, sharesAccepted: Math.floor(clock.now / 60_000), sharesRejected: 0, temp: 55, vrTemp: 0,
        voltage: 5100, power: active ? 12 : 1, fanrpm: 4000, overheat_mode: 0, stratumURL: SETTINGS.stratumURL,
        stratumPort: SETTINGS.stratumPort, stratumUser: SETTINGS.stratumUser, wifiStatus: "connected", wifiRSSI: -40,
        powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" } };
    },
    async patch() {},
    async command(name) { state.paused = name === "pause"; if (name === "resume") state.resumedAt = clock.now; },
    async logs() { state.logReads += 1; return ""; },
  };
  return { client, clock, state };
}

async function run(device) {
  const events = [];
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); }, soak: LOOP });
  return { verdict, events };
}

test("twenty recovered handovers pass and end paused", async () => {
  // Arrange
  const device = fakeLoop();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.cycles, LOOP.cycles);
  assert.equal(device.state.paused, true);
});

test("each handover reads the retained log once", async () => {
  // Arrange
  const device = fakeLoop();
  // Act
  await run(device);
  // Assert
  assert.equal(device.state.logReads, LOOP.cycles);
});

test("a reset during the loop stops it at once", async () => {
  // Arrange: the device panics and reboots at 7 minutes.
  const device = fakeLoop({ reset: (t) => t >= 420_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_device_reset"));
  assert.ok(device.clock.now < 600_000);
});

test("a break that never recovers stops the loop", async () => {
  // Arrange: local mining never comes back after 4 minutes.
  const device = fakeLoop({ idle: (t) => t >= 240_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("handover_not_restored"));
  assert.ok(device.clock.now < 240_000 + LOOP.breakBoundMs + 200_000);
});
