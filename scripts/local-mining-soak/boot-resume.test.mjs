import assert from "node:assert/strict";
import test from "node:test";
import { IDLE_POWER_W, SOAKS } from "./judge.mjs";
import { HOLD_MS, judgeBootResume, pauseThenRestore, runBootResume } from "./boot-resume.mjs";

const SOAK = SOAKS["boot-resume-001"];
const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };

/**
 * A fake device whose restart takes `downMs`, after which it boots with its saved mining-on-boot
 * preference (unless `honorPreference(boot)` says otherwise) and mines `startAfterMs` later.
 */
function fakeDevice({ downMs = 20_000, startAfterMs = 30_000, restartCategory = "software_cpu", rebootOnRestart = true,
  restartThrows = false, honorPreference = () => true, ordinalStep = 1, stall = () => false, idlePowerW = 1, miningPowerW = 12,
  board = {} } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, mineOnBoot: false, boot: 0, bootOrdinal: EXPECTED.bootOrdinal, reset: "power_on",
    runningSince: null, downUntil: null, pool: {}, profile: {}, calls: [] };
  const boot = () => {
    state.boot += 1; state.bootOrdinal += ordinalStep; state.reset = restartCategory; state.downUntil = null;
    const mines = state.mineOnBoot && honorPreference(state.boot);
    state.paused = !mines; state.runningSince = mines ? clock.now : null;
  };
  const info = () => {
    if (state.downUntil !== null) {
      if (clock.now < state.downUntil) throw new Error("restarting");
      boot();
    }
    const active = !state.paused && state.runningSince !== null && clock.now - state.runningSince >= startAfterMs;
    const minedMs = active ? clock.now - state.runningSince - startAfterMs : 0;
    return {
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: state.bootOrdinal,
      bootSession: `session-${state.boot}`, resetReasonCategory: state.reset, uptimeSeconds: 40,
      miningPaused: state.paused, miningActivity: active ? "active" : "paused", startMiningOnBoot: state.mineOnBoot,
      hashRate: active ? 500 : 0, sharesAccepted: Math.floor(minedMs / 60_000), sharesRejected: 0,
      temp: 55, vrTemp: 50, voltage: 5100, power: active && !stall(clock.now) ? miningPowerW : idlePowerW, fanrpm: 4000,
      overheat_mode: 0, ...state.pool, ...state.profile, ...board,
      powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
    };
  };
  const client = {
    async info() { return info(); },
    async patch(settings) {
      if (state.downUntil !== null && clock.now < state.downUntil) throw new Error("restarting");
      if ("startMiningOnBoot" in settings) { state.calls.push(`mine_on_boot_${settings.startMiningOnBoot}`); state.mineOnBoot = settings.startMiningOnBoot; return; }
      if ("frequency" in settings) {
        state.calls.push("patch_profile");
        state.profile = { frequency: settings.frequency, coreVoltage: settings.coreVoltage };
        return;
      }
      state.calls.push("patch_pool");
      state.pool = { stratumURL: settings.stratumURL, stratumPort: settings.stratumPort, stratumUser: settings.stratumUser };
    },
    async command(name) {
      if (state.downUntil !== null && clock.now < state.downUntil) throw new Error("restarting");
      state.calls.push(name);
      if (name === "restart") {
        if (rebootOnRestart) state.downUntil = clock.now + downMs;
        if (restartThrows) throw new Error("connection reset");
        return;
      }
      state.paused = name === "pause";
      if (name === "resume") state.runningSince = clock.now;
    },
  };
  return { client, clock, state };
}

async function run(device, isInterrupted = () => false, soak = SOAK) {
  const events = [];
  const verdict = await runBootResume({ client: device.client, expected: EXPECTED, settings: SETTINGS, soak, isInterrupted,
    now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); } });
  return { verdict, events };
}

test("a device that honors mining on boot passes and ends paused with it off", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.deepEqual(device.state.calls, ["patch_pool", "mine_on_boot_true", "restart", "mine_on_boot_false", "pause", "restart"]);
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
  assert.ok(verdict.metrics.accumulated_active_ms >= SOAK.activeMs);
  assert.ok(verdict.metrics.hold_ms >= HOLD_MS);
});

test("mining resumes with no resume command", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(device.state.calls.includes("resume"), false);
  assert.ok(verdict.metrics.active_samples > 0);
});

test("no journal event carries the pool", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { events } = await run(device);
  // Assert
  assert.equal(/pool\.invalid|owner\.worker/u.test(JSON.stringify(events)), false);
});

test("a boot that ignores mining on boot fails, and the device ends paused with it off", async () => {
  // Arrange
  const device = fakeDevice({ honorPreference: () => false });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_started_paused"));
  assert.ok(verdict.failures.includes("soak_mining_not_started"));
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
});

test("a held boot that mines after mining on boot was turned off fails, then is paused", async () => {
  // Arrange
  const device = fakeDevice();
  const { info, patch } = device.client;
  // On the mining boot the off write is lost, though the device reads it back as off.
  device.client.patch = async (settings) => {
    if (settings.startMiningOnBoot === false && device.state.boot === 1) return;
    await patch(settings);
  };
  device.client.info = async () => {
    const value = await info();
    return device.state.boot === 1 ? { ...value, startMiningOnBoot: false } : value;
  };
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_mined_after_disable"));
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
});

test("a restart that never happens stops the check and turns mining on boot off", async () => {
  // Arrange
  const device = fakeDevice({ rebootOnRestart: false });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_restart_unobserved"));
  assert.equal(device.state.calls.filter((call) => call === "restart").length, 1);
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
});

test("an ambiguous restart response is resolved by the new boot, never by a second request", async () => {
  // Arrange
  const device = fakeDevice({ restartThrows: true });
  // Act
  const { verdict, events } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(device.state.calls.filter((call) => call === "restart").length, 2);
  assert.equal(events.filter((event) => event.event === "restart_ambiguous").length, 2);
});

test("a restart that reports another reset category fails", async () => {
  // Arrange
  const device = fakeDevice({ restartCategory: "watchdog" });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_reset_category"));
  assert.equal(device.state.mineOnBoot, false);
});

test("a device that last reset by panic is refused before any write", async () => {
  // Arrange
  const device = fakeDevice();
  device.state.reset = "panic";
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_previous_panic" });
  assert.deepEqual(device.state.calls, []);
});

test("a refused off write pauses first, and fails as not restored", async () => {
  // Arrange
  const device = fakeDevice();
  const { patch } = device.client;
  device.client.patch = async (settings) => {
    if (settings.startMiningOnBoot === false) throw new Error("persistence_failed");
    await patch(settings);
  };
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_preference_not_restored"));
  assert.equal(device.state.paused, true);
  assert.equal(device.state.calls.at(-1), "pause");
});

test("the pause precedes any restore write", async () => {
  // Arrange
  const device = fakeDevice({ rebootOnRestart: false });
  // Act
  await run(device);
  // Assert
  const calls = device.state.calls;
  assert.ok(calls.indexOf("pause") < calls.indexOf("mine_on_boot_false"));
});

test("an enable write whose response is lost is still undone, and no restart is sent", async () => {
  // Arrange
  const device = fakeDevice();
  const { patch } = device.client;
  device.client.patch = async (settings) => {
    await patch(settings);
    if (settings.startMiningOnBoot === true) throw new Error("response lost");
  };
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(verdict.result, "failed");
  assert.equal(device.state.calls.includes("restart"), false);
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
});

test("a skipped boot ordinal fails", async () => {
  // Arrange
  const device = fakeDevice({ ordinalStep: 2 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_boot_ordinal"));
  assert.equal(device.state.mineOnBoot, false);
});

test("an interruption while mining on boot stops, turns it off and pauses", async () => {
  // Arrange
  const device = fakeDevice();
  // Act
  const { verdict } = await run(device, () => device.state.boot === 1 && device.clock.now > 120_000);
  // Assert
  assert.ok(verdict.failures.includes("soak_interrupted"));
  assert.equal(device.state.mineOnBoot, false);
  assert.equal(device.state.paused, true);
});

test("recovery leaves an already-off setting unwritten", async () => {
  // Arrange
  const device = fakeDevice();
  const context = { client: device.client, now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, note: async () => {} };
  // Act
  const { mineOnBootOff, maybeFinal } = await pauseThenRestore(context);
  // Assert
  assert.equal(mineOnBootOff, true);
  assert.notEqual(maybeFinal, undefined);
  assert.equal(device.state.calls.some((call) => call.startsWith("mine_on_boot")), false);
});

test("recovery turns an on setting off", async () => {
  // Arrange
  const device = fakeDevice();
  device.state.mineOnBoot = true;
  const context = { client: device.client, now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, note: async () => {} };
  // Act
  const { mineOnBootOff } = await pauseThenRestore(context);
  // Assert
  assert.equal(mineOnBootOff, true);
  assert.equal(device.state.mineOnBoot, false);
});

test("a hold shorter than its bound fails", async () => {
  // Arrange
  const boot = (session, atMs, paused) => ({ atMs, bootSession: session, miningPaused: paused, miningActivity: paused ? "paused" : "active",
    startMiningOnBoot: false, pauseReason: null, hardwareFault: false, powerFault: false, overheatMode: false, powerW: 1,
    fresh: { power: true, input: true, chipTemp: true, fan: true } });
  const held = boot("b", 1000, true);
  const holdSamples = [boot("b", 3000, true), boot("b", 5000, true)];
  // Act
  const verdict = judgeBootResume({ baseline: boot("z", 0, true), marks: { miningBoot: { sample: boot("a", 0, false) },
    heldBoot: { sample: held }, disabled: true }, miningSamples: [], holdSamples, maybeFinal: holdSamples.at(-1), maybeStop: null, soak: SOAK });
  // Assert
  assert.ok(verdict.failures.includes("boot_resume_hold_short"));
});

test("recovery re-sends a failed pause before each restore write", async () => {
  // Arrange
  const device = fakeDevice();
  device.state.mineOnBoot = true;
  device.state.downUntil = 10_000;
  const context = { client: device.client, now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, note: async () => {} };
  // Act
  await pauseThenRestore(context);
  // Assert
  const calls = device.state.calls;
  assert.equal(calls[0], "pause");
  assert.ok(calls.indexOf("pause") < calls.indexOf("mine_on_boot_false"));
});

test("a status-only stall on the mining boot is an outage, not mining time", async () => {
  // Arrange
  const device = fakeDevice({ stall: (t) => t >= 150_000 && t < 170_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outage_count, 1);
  assert.ok(verdict.metrics.status_only_samples > 0);
});

const GAMMA601_SOAK = SOAKS["gamma601-boot-resume-001"];
/** The promoted 601: idle reads its 5 W offset plus a little, mining about 19 W, board 601 matched. */
const gamma601Device = (board = { storedBoard: "601", boardIdentity: "match" }) =>
  fakeDevice({ idlePowerW: IDLE_POWER_W + 0.4, miningPowerW: 19, board });

test("the 601's check passes at its own idle power", async () => {
  // Arrange
  const device = gamma601Device();
  // Act
  const { verdict } = await run(device, () => false, GAMMA601_SOAK);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(device.state.paused, true);
  assert.equal(device.state.mineOnBoot, false);
});

test("the 601's check saves its profile before mining on boot is turned on", async () => {
  // Arrange
  const device = gamma601Device();
  // Act
  await run(device, () => false, GAMMA601_SOAK);
  // Assert
  assert.deepEqual(device.state.calls.slice(0, 3), ["patch_pool", "patch_profile", "mine_on_boot_true"]);
  assert.deepEqual(device.state.profile, { frequency: 525, coreVoltage: 1150 });
});

test("the 601's check refuses another board before any write", async () => {
  // Arrange
  const device = gamma601Device({ storedBoard: "205", boardIdentity: "match" });
  // Act / Assert
  await assert.rejects(run(device, () => false, GAMMA601_SOAK), { code: "soak_board_identity" });
  assert.deepEqual(device.state.calls, []);
});

test("the 205's limits would refuse the 601's idle power as not stopped", async () => {
  // Arrange
  const device = gamma601Device();
  // Act / Assert
  await assert.rejects(run(device), { code: "soak_baseline_not_stopped" });
});

test("the 601's recovery confirms the stop at its own idle power", async () => {
  // Arrange
  const device = gamma601Device();
  device.state.mineOnBoot = true;
  const context = { client: device.client, now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, note: async () => {},
    limits: GAMMA601_SOAK.limits };
  // Act
  const { mineOnBootOff, maybeFinal } = await pauseThenRestore(context);
  // Assert
  assert.equal(mineOnBootOff, true);
  assert.notEqual(maybeFinal, undefined);
});
