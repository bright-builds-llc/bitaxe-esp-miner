import assert from "node:assert/strict";
import test from "node:test";
import { SOAKS } from "./judge.mjs";
import { runSoak } from "./run.mjs";

const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };
const LEASE = SOAKS["lease-001"];

/**
 * A device mining locally from 4 s after the resume, except while `idle(t)` holds (a refused Hello's
 * safe-stop, the lease's preparation and safe-stop, the 15 s quiet window). The lease's own mining
 * reads the same as local mining. `leaseEndsAtMs` is when the Gate soak's result appears.
 */
function fakeLease({ idle = (t) => (t >= 400_000 && t < 450_000) || (t >= 1_050_000 && t < 1_100_000), leaseEndsAtMs = 1_070_000,
  lease = { result: "passed", failures: [] }, stall = () => false } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, resumedAt: null, calls: [] };
  const info = () => {
    const active = !state.paused && state.resumedAt !== null && clock.now - state.resumedAt >= 4000 && !idle(clock.now);
    return {
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
      bootSession: "s".repeat(32), miningPaused: state.paused, miningActivity: active ? "active" : state.paused ? "paused" : "safe_blocked",
      startMiningOnBoot: false, hashRate: active ? 500 : 0, sharesAccepted: Math.floor(clock.now / 60_000), sharesRejected: 0,
      temp: 55, vrTemp: 0, voltage: 5100, power: active && !stall(clock.now) ? 12 : 1, fanrpm: 4000, overheat_mode: 0,
      stratumURL: SETTINGS.stratumURL, stratumPort: SETTINGS.stratumPort, stratumUser: SETTINGS.stratumUser,
      wifiStatus: "connected", wifiRSSI: -40,
      powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
    };
  };
  const client = {
    async info() { return info(); },
    async patch() { state.calls.push("patch"); },
    async command(name) { state.calls.push(name); state.paused = name === "pause"; if (name === "resume") state.resumedAt = clock.now; },
  };
  const leaseResult = async () => (clock.now >= leaseEndsAtMs ? lease : null);
  return { client, clock, state, leaseResult };
}

async function run(device) {
  const events = [];
  const verdict = await runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); }, soak: LEASE,
    leaseResult: device.leaseResult });
  return { verdict, events };
}

test("a lease that interrupts local mining and is followed by its restoration passes", async () => {
  // Arrange
  const device = fakeLease();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.lease_result, "passed");
  assert.ok(verdict.metrics.after_active_ms >= LEASE.afterActiveMs);
  assert.equal(device.state.paused, true);
});

test("the phases are journaled in order", async () => {
  // Arrange
  const device = fakeLease();
  // Act
  const { events } = await run(device);
  // Assert
  const order = ["lease_ready", "local_stopped", "lease_ended", "local_restored", "verdict"]
    .map((name) => events.findIndex((event) => event.event === name));
  assert.ok(order.every((index, position) => index >= 0 && (position === 0 || index > order[position - 1])), JSON.stringify(order));
});

test("local mining that never comes back after the lease stops the check", async () => {
  // Arrange
  const device = fakeLease({ idle: (t) => t >= 400_000 && (t < 450_000 || t >= 1_050_000) });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("lease_check_not_restored"));
  assert.ok(device.clock.now < 1_070_000 + LEASE.restoreBoundMs + 200_000);
  assert.equal(device.state.paused, true);
});

test("a lease that failed fails the check", async () => {
  // Arrange
  const device = fakeLease({ lease: { result: "unverified", failures: ["window_http_window_uncredited"] } });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("lease_check_lease_failed"));
  assert.deepEqual(verdict.metrics.lease_failures, ["window_http_window_uncredited"]);
});

test("a lease that ends while local mining never stopped fails at once", async () => {
  // Arrange: no idle period before the lease result appears.
  const device = fakeLease({ idle: () => false, leaseEndsAtMs: 500_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("lease_check_never_interrupted"));
  assert.ok(device.clock.now < 700_000);
});

test("an outage past its bound before the lease stops the check", async () => {
  // Arrange: local mining breaks off for 90 s after 2 minutes.
  const device = fakeLease({ idle: (t) => t >= 120_000 && t < 210_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_outage_unrecovered"));
  assert.ok(device.clock.now < 400_000);
});

test("the lease's own idle time is never an outage", async () => {
  // Arrange: the device stays idle for 200 s after the refused Hello, far past the 60 s outage bound.
  const device = fakeLease({ idle: (t) => (t >= 400_000 && t < 600_000) || (t >= 1_050_000 && t < 1_100_000) });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(verdict.metrics.outage_count, 0);
  assert.deepEqual(verdict.failures, []);
});

test("a hashrate seen only under the lease does not prove local mining", async () => {
  // Arrange: hashrate reads 0 outside the lease's mining, 450–1,050 s.
  const device = fakeLease();
  const info = device.client.info;
  device.client.info = async () => {
    const value = await info();
    return device.clock.now >= 450_000 && device.clock.now < 1_050_000 ? value : { ...value, hashRate: 0 };
  };
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("soak_no_hashrate"));
});

test("a status-only stall before the interrupt marks local mining stopped at its start", async () => {
  // Arrange
  const device = fakeLease({ stall: (t) => t >= 384_000 && t < 400_000 });
  // Act
  const { verdict, events } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  const stopped = events.find((event) => event.event === "local_stopped");
  assert.ok(stopped.atMs >= 384_000 && stopped.atMs < 400_000);
});

const GAMMA601_LEASE = SOAKS["gamma601-lease-001"];

/**
 * `fakeLease` as a Gamma 601: it reports its stored board and the soak's saved pair, hashes at
 * 16 W, and reads 5.8 W stopped, its regulator's fixed offset (ADR-0041).
 */
function fake601Lease() {
  const device = fakeLease();
  const info = device.client.info;
  device.client.info = async () => {
    const value = await info();
    return { ...value, storedBoard: "601", boardIdentity: "match", frequency: 400, coreVoltage: 1150, vrTemp: 60,
      power: value.power === 12 ? 16 : 5.8 };
  };
  return device;
}

async function runAs(device, soak) {
  return runSoak({ client: device.client, expected: EXPECTED, settings: SETTINGS, now: () => device.clock.now,
    sleep: async (ms) => { device.clock.now += ms; }, record: async () => {}, soak, leaseResult: device.leaseResult });
}

test("a 601 lease judged at 16 W passes under the 601's limits", async () => {
  // Arrange
  const device = fake601Lease();
  // Act
  const verdict = await runAs(device, GAMMA601_LEASE);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.maximum_power_w, 16);
  assert.equal(verdict.metrics.status_only_samples, 0);
  assert.equal(device.state.paused, true);
});

test("a lease hashing at 16 W fails the 205's lease check", async () => {
  // Arrange: the 205's own idle reading, so only the hashing power differs.
  const device = fakeLease();
  const info = device.client.info;
  device.client.info = async () => { const value = await info(); return { ...value, power: value.power === 12 ? 16 : value.power }; };
  // Act
  const verdict = await runAs(device, LEASE);
  // Assert
  assert.ok(verdict.failures.includes("soak_power"), JSON.stringify(verdict.failures));
});

test("a stopped 601 at 5.8 W counts as stopped only under the 601's limits", async () => {
  // Arrange
  const { judgeLease } = await import("./lease.mjs");
  const { sampleFromInfo } = await import("./judge.mjs");
  const stopped = sampleFromInfo({ bootSession: "s", miningPaused: true, miningActivity: "paused", power: 5.8, powerStatus: { state: "fresh" } }, 0);
  const judged = (soak) => judgeLease({ baseline: stopped, samples: [], maybeFinal: stopped, maybeStop: null, soak, marks: {} }).failures;
  // Act / Assert
  assert.ok(!judged(GAMMA601_LEASE).includes("soak_not_stopped_after"));
  assert.ok(judged(LEASE).includes("soak_not_stopped_after"));
});

test("the 601 lease check saves its 400 MHz / 1150 mV profile before the resume", async () => {
  // Arrange
  const device = fake601Lease();
  const patches = [];
  const patch = device.client.patch;
  device.client.patch = async (body) => { patches.push(body); return patch(body); };
  // Act
  await runAs(device, GAMMA601_LEASE);
  // Assert
  assert.deepEqual(patches.at(-1), { frequency: 400, coreVoltage: 1150 });
});
