import assert from "node:assert/strict";
import test from "node:test";
import { finalFailures, GAMMA601_LIMITS, LIMITS, restartCool, sampleFromInfo, SOAKS } from "./judge.mjs";
import { restartActionAtMs, runOverheat } from "./overheat.mjs";

const SOAK = SOAKS["gamma601-overheat-001"];
const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };
const TRIP_DETAIL = "worker_revocation_detail schema=v1 generation=3 reason=unsafe_observation trigger=unsafe_sample fact=none state=none value_milli=unavailable age_ms=unavailable since_safe_ms=1 closed_ms=2 redacted=true";
const unavailable = (code) => ({ state: "unavailable", reason: { kind: "unavailable", code } });

/** A paused 601's `/api/system/info` fields that the cool rule reads; `overrides` replace any. */
const stoppedInfo = (overrides = {}) => ({
  bootSession: "s".repeat(32), miningPaused: true, miningActivity: "paused", startMiningOnBoot: false, pauseReason: null,
  temp: 0, vrTemp: 60, voltage: 5100, power: 5.8, fanrpm: 4000,
  powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
  chipTempStatus: unavailable("asic_unpowered"), vrTempStatus: { state: "fresh" }, ...overrides,
});
const cool601 = (overrides) => restartCool(GAMMA601_LIMITS)(sampleFromInfo(stoppedInfo(overrides), 0));

/**
 * A fake Gamma 601 running the lowered 55 C trip, as the firmware behaves:
 * - hashing heats the chip toward 60 C (`loweredHotC` at the lowered pair) and draws 15 W; an
 *   unpowered ASIC draws the 601's offset reading, 5.8 W, and its chip reads unavailable with reason
 *   `asic_unpowered` (`asic_not_initialized` while a run prepares);
 * - every run, the first and the restart, prepares for `prepareMs` before it hashes;
 * - a trip stops the ASIC at once; 90 s later `overheat_mode` and the 100% fan are saved;
 * - recovery: `coolingMs` (30 s) with the ASIC unpowered and the regulator fresh at 95 C or less
 *   saves the pair 100 MHz and 100 mV lower, restarting only when not paused; below the 400 MHz
 *   floor the device pauses itself instead.
 */
function fake601({ loweredHotC = 50, restartHashRate = 300, baselineFrequency = 400, prepareMs = 6000, coolingMs = 30_000 } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, frequency: baselineFrequency, coreVoltage: 1150, autofan: 1, manualFan: 70, overheat: 0,
    pauseReason: null, chip: 40, runningSince: null, trippedAt: null, entered: false, coolSince: null, pool: {}, log: [], calls: [] };
  const preparing = () => !state.paused && state.runningSince !== null && state.trippedAt === null;
  const hashingNow = () => preparing() && clock.now - state.runningSince >= prepareMs;
  const step = () => {
    const target = hashingNow() ? (state.frequency < 500 ? loweredHotC : 60) : 35;
    state.chip += Math.sign(target - state.chip) * Math.min(Math.abs(target - state.chip), hashingNow() ? 0.4 : 0.6);
    if (hashingNow() && state.chip >= 55) { state.trippedAt = clock.now; state.entered = false; return; }
    if (state.trippedAt === null) return;
    if (!state.entered) {
      if (clock.now - state.trippedAt < 90_000) return;
      state.entered = true; state.overheat = 1; state.autofan = 0; state.manualFan = 100; state.coolSince = null;
      state.staysStopped = state.frequency - 100 < 400;
      state.log.push(TRIP_DETAIL, "local_mining=run_ended generation=3 ending=Some(Overheat)", `local_mining=overheat stays_stopped=${state.staysStopped}`);
      return;
    }
    state.coolSince ??= clock.now;
    if (clock.now - state.coolSince < coolingMs) return;
    if (state.staysStopped) {
      state.paused = true; state.pauseReason = "overheat"; state.trippedAt = null;
      state.log.push("local_mining=overheat_stopped paused_by_device=true");
      return;
    }
    state.frequency -= 100; state.coreVoltage -= 100; state.trippedAt = null;
    state.log.push("local_mining=overheat_recovered");
    if (!state.paused) { state.runningSince = clock.now; state.overheat = 0; }
  };
  const info = () => {
    step();
    const active = hashingNow();
    return {
      ...stoppedInfo({ miningPaused: state.paused, pauseReason: state.pauseReason,
        miningActivity: active ? "active" : state.paused ? "paused" : "safe_blocked",
        chipTempStatus: active ? { state: "fresh" } : unavailable(preparing() ? "asic_not_initialized" : "asic_unpowered"),
        temp: active ? Math.round(state.chip * 10) / 10 : 0, vrTemp: active ? 70 : 60, power: active ? 15 : 5.8 }),
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
      storedBoard: "601", boardIdentity: "match", resetReasonCategory: "power_on",
      hashRate: active ? (state.frequency >= 500 ? 600 : restartHashRate) : 0, sharesAccepted: Math.floor(clock.now / 60_000),
      sharesRejected: 0, overheat_mode: state.overheat, autofanspeed: state.autofan, manualFanSpeed: state.manualFan,
      frequency: state.frequency, coreVoltage: state.coreVoltage, ...state.pool,
    };
  };
  const client = {
    async info() { return info(); },
    async logs() { return `boot\n${state.log.join("\n")}\n`; },
    async patch(settings) {
      state.calls.push(`patch:${Object.keys(settings).sort().join(",")}`);
      if ("stratumURL" in settings) { state.pool = { stratumURL: settings.stratumURL, stratumPort: settings.stratumPort, stratumUser: settings.stratumUser }; return; }
      if ("frequency" in settings) state.frequency = settings.frequency;
      if ("coreVoltage" in settings) state.coreVoltage = settings.coreVoltage;
      if ("autofanspeed" in settings) state.autofan = settings.autofanspeed;
      if ("manualFanSpeed" in settings) state.manualFan = settings.manualFanSpeed;
      if ("overheat_mode" in settings) state.overheat = settings.overheat_mode;
    },
    async command(name) {
      state.calls.push(name);
      if (name === "pause") { state.paused = true; state.runningSince = null; return; }
      state.paused = false; state.pauseReason = null; state.runningSince = clock.now;
    },
  };
  return { client, clock, state };
}

async function run(device, soak = SOAK) {
  const events = [];
  const verdict = await runOverheat({ client: device.client, expected: EXPECTED, settings: SETTINGS, soak,
    now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); } });
  return { verdict, events };
}

test("the 601 counts as cool with its ASIC unpowered and a fresh regulator", () => {
  // Act / Assert
  assert.equal(cool601(), true);
});

test("a chip powered but not initialized is not cool on the 601", () => {
  // Act / Assert
  assert.equal(cool601({ chipTempStatus: unavailable("asic_not_initialized") }), false);
});

test("a stale regulator reading is not cool on the 601", () => {
  // Act / Assert
  assert.equal(cool601({ vrTempStatus: { state: "stale", reason: { kind: "stale", code: "expired" } } }), false);
});

test("the 601's regulator is cool at 95 C and not at 95.1 C", () => {
  // Act / Assert
  assert.equal(cool601({ vrTemp: 95 }), true);
  assert.equal(cool601({ vrTemp: 95.1 }), false);
});

test("the 205's cool rule still needs a fresh chip at 45 C or less", () => {
  // Arrange
  const cool205 = restartCool(LIMITS);
  // Act / Assert
  assert.equal(cool205(sampleFromInfo(stoppedInfo({ temp: 45, chipTempStatus: { state: "fresh" } }), 0)), true);
  assert.equal(cool205(sampleFromInfo(stoppedInfo(), 0)), false);
});

test("a stopped 601 at 5.8 W passes under the 601's limits and fails under the 205's", () => {
  // Arrange
  const sample = sampleFromInfo(stoppedInfo(), 0);
  // Act / Assert
  assert.deepEqual(finalFailures(sample, sample, GAMMA601_LIMITS), []);
  assert.deepEqual(finalFailures(sample, sample, LIMITS), ["soak_not_stopped_after"]);
});

test("a 601 that trips, cools by its rule and restarts at 400 MHz / 1050 mV passes", async () => {
  // Arrange
  const device = fake601();
  // Act
  const { verdict, events } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outcome, "restarted");
  assert.deepEqual([verdict.metrics.lowered_frequency_mhz, verdict.metrics.lowered_core_voltage_mv], [400, 1050]);
  assert.ok(verdict.metrics.cooled_ms >= 30_000);
  const restart = events.find((event) => event.event === "overheat_restarted");
  assert.deepEqual([restart.frequencyMhz, restart.coreVoltageMv], [400, 1050]);
  assert.ok(events.some((event) => event.event === "sample" && event.sample.chipTempUnavailableReason === "asic_unpowered" &&
    event.sample.vrTempFresh));
});

test("the 601 ends paused at its baseline pair, fan and overheat mode", async () => {
  // Arrange
  const device = fake601();
  // Act
  await run(device);
  // Assert
  const { state } = device;
  assert.deepEqual([state.frequency, state.coreVoltage, state.autofan, state.manualFan, state.overheat, state.paused],
    [400, 1150, 1, 70, 0, true]);
});

test("a 601 that trips again at the lowered pair and pauses itself passes", async () => {
  // Arrange
  const device = fake601({ loweredHotC: 58 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outcome, "restarted_then_stays_stopped");
});

test("a 601 restart that never shows a hashrate fails", async () => {
  // Arrange
  const device = fake601({ restartHashRate: 0 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_restart_no_hashrate"), verdict.failures.join(","));
});

test("a second trip after a restart with no hashrate fails too", async () => {
  // Arrange
  const device = fake601({ loweredHotC: 58, restartHashRate: 0 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_restart_no_hashrate"), verdict.failures.join(","));
});

test("a 601 baseline off its baseline pair is refused before the run", async () => {
  // Arrange
  const device = fake601({ baselineFrequency: 485 });
  // Act / Assert
  await assert.rejects(run(device), { code: "overheat_baseline_profile" });
  assert.deepEqual(device.state.calls, []);
  assert.equal(device.state.frequency, 485);
});

test("the 601's stopped baseline is refused under the 205's limits", async () => {
  // Arrange
  const device = fake601();
  // Act / Assert
  await assert.rejects(run(device, { ...SOAK, limits: LIMITS }), { code: "soak_baseline_not_stopped" });
});

test("a 601 restart that cooled 30 s before its preparation passes the cooling check", async () => {
  // Arrange
  const device = fake601({ prepareMs: 6000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(verdict.failures.includes("overheat_restarted_before_cooling"), false, verdict.failures.join(","));
  assert.ok(verdict.metrics.cooled_ms >= 30_000);
});

test("a 601 restart whose preparation starts under 30 s cool fails", async () => {
  // Arrange
  const device = fake601({ coolingMs: 20_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_restarted_before_cooling"), verdict.failures.join(","));
});

test("the restart action begins at the 601's preparation, and is unchanged on the 205", () => {
  // Arrange
  const at = (atMs, code) => sampleFromInfo(stoppedInfo({ chipTempStatus: code === null ? { state: "fresh" } : unavailable(code) }), atMs);
  const samples = [at(0, "asic_unpowered"), at(2000, "asic_unpowered"), at(4000, "asic_not_initialized"), at(6000, null)];
  // Act / Assert
  assert.equal(restartActionAtMs(samples, 0, 6000, GAMMA601_LIMITS), 4000);
  assert.equal(restartActionAtMs(samples, 0, 6000, LIMITS), 6000);
  assert.equal(restartActionAtMs(samples.slice(2), 4000, 6000, GAMMA601_LIMITS), 6000);
});
