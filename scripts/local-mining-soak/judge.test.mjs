import assert from "node:assert/strict";
import test from "node:test";
import { activeSpanMs, hardwareStopped, increases, judge, LIMITS, liveStop, MAXIMUM_SAMPLE_GAP_MS, sampleFromInfo, SOAKS } from "./judge.mjs";

const SOAK_ACTIVE_MS = SOAKS["001"].activeMs;

const fresh = { state: "fresh" };

const info = (overrides = {}) => ({
  bootSession: "a".repeat(32), uptimeSeconds: 100, resetReasonCategory: "power_on", miningPaused: false,
  miningActivity: "active", startMiningOnBoot: false, hashRate: 500, sharesAccepted: 0, sharesRejected: 0,
  temp: 55, vrTemp: 50, voltage: 5100, power: 12, fanrpm: 4000, frequency: 485, coreVoltage: 1200, overheat_mode: 0,
  powerStatus: fresh, voltageStatus: fresh, chipTempStatus: fresh, fanRpmStatus: fresh,
  ...overrides,
});

/** A run of active samples every 2 s for `durationMs`, accepting one share near the end. */
function activeRun(durationMs, overrides = {}) {
  const samples = [];
  for (let at = 0; at <= durationMs; at += 2000) {
    samples.push(sampleFromInfo(info({ sharesAccepted: at >= durationMs - 2000 ? 3 : 0, ...overrides }), at));
  }
  return samples;
}

const paused = (overrides = {}) => sampleFromInfo(info({ miningPaused: true, miningActivity: "paused", power: 1.5, ...overrides }), 0);

test("a sample keeps no endpoint, pool or credential value", () => {
  // Arrange
  const raw = info({ stratumURL: "pool.invalid", stratumUser: "owner.worker", ipv4: "192.168.1.2" });
  // Act
  const text = JSON.stringify(sampleFromInfo(raw, 0));
  // Assert
  assert.equal(/pool\.invalid|owner\.worker|192\.168/u.test(text), false);
});

test("thirty minutes of continuous mining with a share passes", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS);
  // Act
  const verdict = judge({ baseline: samples[0], samples, maybeFinal: paused() });
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.result, "passed");
  assert.equal(verdict.metrics.shares_accepted, 3);
});

test("a short run fails", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS - 60_000);
  // Act
  const verdict = judge({ baseline: samples[0], samples, maybeFinal: paused() });
  // Assert
  assert.deepEqual(verdict.failures, ["soak_active_time_short"]);
});

test("a gap breaks the active span and fails continuity", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS);
  samples.splice(10, 10);
  // Act
  const verdict = judge({ baseline: samples[0], samples, maybeFinal: paused() });
  // Assert
  assert.ok(verdict.failures.includes("soak_sample_gap"));
  assert.ok(activeSpanMs(samples) < SOAK_ACTIVE_MS);
});

test("no accepted share fails", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS, { sharesAccepted: 0 });
  // Act
  const verdict = judge({ baseline: samples[0], samples, maybeFinal: paused() });
  // Assert
  assert.deepEqual(verdict.failures, ["soak_no_accepted_share"]);
});

test("a reset is a live stop", () => {
  // Arrange
  const baseline = sampleFromInfo(info(), 0);
  // Act / Assert
  assert.equal(liveStop(sampleFromInfo(info({ bootSession: "b".repeat(32) }), 2000), baseline), "soak_device_reset");
});

test("limits stop a mining sample and spare an idle one", () => {
  // Arrange
  const baseline = sampleFromInfo(info(), 0);
  const hot = { temp: LIMITS.maximumChipTempC };
  // Act / Assert
  assert.equal(liveStop(sampleFromInfo(info(hot), 2000), baseline), "soak_chip_temperature");
  assert.equal(liveStop(sampleFromInfo(info({ power: 15.5 }), 2000), baseline), "soak_power");
  assert.equal(liveStop(sampleFromInfo(info({ voltage: 4400 }), 2000), baseline), "soak_input_voltage");
  assert.equal(liveStop(sampleFromInfo(info({ fanrpm: 0 }), 2000), baseline), "soak_fan_stopped");
  assert.equal(liveStop(sampleFromInfo(info({ ...hot, miningPaused: true, miningActivity: "paused" }), 2000), baseline), null);
});

test("a device pause, a fault or an overheat always stops", () => {
  // Arrange
  const baseline = sampleFromInfo(info(), 0);
  // Act / Assert
  assert.equal(liveStop(sampleFromInfo(info({ pauseReason: "restart_guard" }), 2000), baseline), "soak_device_paused_itself");
  assert.equal(liveStop(sampleFromInfo(info({ hardware_fault: "Fan Control Failed (Auto)" }), 2000), baseline), "soak_hardware_fault");
  assert.equal(liveStop(sampleFromInfo(info({ overheat_mode: 1 }), 2000), baseline), "soak_overheat");
});

test("the run must end paused with mining on boot off", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS);
  // Act
  const running = judge({ baseline: samples[0], samples, maybeFinal: samples.at(-1) });
  const bootMining = judge({ baseline: samples[0], samples, maybeFinal: paused({ startMiningOnBoot: true }) });
  const missing = judge({ baseline: samples[0], samples, maybeFinal: undefined });
  // Assert
  assert.ok(running.failures.includes("soak_not_stopped_after"));
  assert.ok(bootMining.failures.includes("soak_mine_on_boot_enabled"));
  assert.ok(missing.failures.includes("soak_final_state_missing"));
});

test("a recorded live stop fails the run", () => {
  // Arrange
  const samples = activeRun(SOAK_ACTIVE_MS);
  // Act
  const verdict = judge({ baseline: samples[0], samples, maybeFinal: paused(), maybeStop: "soak_device_unreachable" });
  // Assert
  assert.ok(verdict.failures.includes("soak_device_unreachable"));
});

test("the gap bound is ten seconds", () => {
  // Arrange / Act / Assert
  assert.equal(MAXIMUM_SAMPLE_GAP_MS, 10_000);
});

test("a pool reconnect resetting the counters loses no shares", () => {
  // Arrange: 12 accepted, the counters reset on a new pool session, then 2 more.
  const samples = [0, 12, 0, 2].map((accepted, index) => sampleFromInfo(info({ sharesAccepted: accepted }), index * 2000));
  // Act / Assert
  assert.equal(increases(samples, "sharesAccepted"), 14);
});

test("readings stale on two mining samples in a row stop the run", () => {
  // Arrange
  const baseline = sampleFromInfo(info(), 0);
  const stale = sampleFromInfo(info({ powerStatus: { state: "stale" } }), 2000);
  // Act / Assert
  assert.equal(liveStop(stale, baseline, sampleFromInfo(info(), 0)), null);
  assert.equal(liveStop(stale, baseline, stale), "soak_readings_stale");
});

test("a power fault always stops", () => {
  // Arrange / Act / Assert
  const baseline = sampleFromInfo(info(), 0);
  assert.equal(liveStop(sampleFromInfo(info({ power_fault: "Input voltage" }), 2000), baseline), "soak_power_fault");
});

test("only fresh idle power shows the ASIC powered down", () => {
  // Arrange / Act / Assert
  assert.equal(hardwareStopped(paused()), true);
  assert.equal(hardwareStopped(paused({ power: 11 })), false);
  assert.equal(hardwareStopped(paused({ powerStatus: { state: "stale" } })), false);
  assert.equal(hardwareStopped(paused({ chipTempStatus: { state: "unavailable" } })), true);
  assert.equal(hardwareStopped(sampleFromInfo(info(), 0)), false);
});

test("a stale reading's zero never trips its limit", () => {
  // Arrange: the device reports a stale voltage as 0.
  const baseline = sampleFromInfo(info(), 0);
  const stale = sampleFromInfo(info({ voltage: 0, voltageStatus: { state: "stale" } }), 2000);
  // Act / Assert
  assert.equal(liveStop(stale, baseline, sampleFromInfo(info(), 0)), null);
});

test("soak 002 needs two hours and a nonzero hashrate", () => {
  // Arrange
  const soak = SOAKS["002"];
  const noHashrate = activeRun(soak.activeMs, { hashRate: 0 });
  const thirtyMinutes = activeRun(SOAK_ACTIVE_MS);
  // Act
  const blind = judge({ baseline: noHashrate[0], samples: noHashrate, maybeFinal: paused(), soak });
  const short = judge({ baseline: thirtyMinutes[0], samples: thirtyMinutes, maybeFinal: paused(), soak });
  const healthy = judge({ baseline: noHashrate[0], samples: activeRun(soak.activeMs), maybeFinal: paused(), soak });
  // Assert
  assert.deepEqual(blind.failures, ["soak_no_hashrate"]);
  assert.ok(short.failures.includes("soak_active_time_short"));
  assert.deepEqual(healthy.failures, []);
  assert.equal(healthy.soak, "002");
});
