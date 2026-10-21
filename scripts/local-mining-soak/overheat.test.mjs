import assert from "node:assert/strict";
import test from "node:test";
import { sampleFromInfo, SOAKS } from "./judge.mjs";
import { overheatLiveStop, recoveryFinished, runOverheat, stopLogged, tripCause } from "./overheat.mjs";

const SOAK = SOAKS["overheat-001"];
const EXPECTED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64), bootOrdinal: 7 };
const SETTINGS = { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" };
const TRIP_DETAIL = "worker_revocation_detail schema=v1 generation=3 reason=unsafe_observation trigger=unsafe_sample fact=none state=none value_milli=unavailable age_ms=unavailable since_safe_ms=1 closed_ms=2 redacted=true";
const OTHER_DETAIL = "worker_revocation_detail schema=v1 generation=3 reason=unsafe_observation trigger=unsafe_sample fact=input_voltage state=out_of_range value_milli=5600000 age_ms=10 since_safe_ms=1 closed_ms=2 redacted=true";

/**
 * A fake device running the lowered trip, as the firmware behaves:
 * - hashing heats the chip toward `hotC` (`loweredHotC` at the lowered pair); a reading at `tripC`
 *   stops the ASIC at once, but its safe-stop takes `safeStopMs` before `overheat_mode` and the
 *   100% fan are saved, logging `detail` then the overheat `run_ended` line;
 * - recovery runs whether or not the device is paused: 30 s at 45 C or less saves the pair 100 MHz
 *   and 100 mV lower, and restarts only when not paused (`restartAfterMs` overrides the 30 s);
 * - a trip whose next step leaves the catalog pauses the device; only a resume clears that pause.
 */
function fakeOverheat({ hotC = 60, loweredHotC = 50, coolC = 35, tripC = 55, restartAfterMs = null, lowerBy = 100, safeStopMs = 90_000,
  detail = TRIP_DETAIL, overwriteAfterRestoreMs = null, floorMhz = 400 } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, frequency: 485, coreVoltage: 1200, autofan: 1, manualFan: 70, overheat: 0, pauseReason: null,
    chip: 40, runningSince: null, trippedAt: null, entered: false, coolSince: null, pool: {}, log: [], calls: [], overwriteAt: null };
  const hashingNow = () => !state.paused && state.runningSince !== null && clock.now - state.runningSince >= 4000 && state.trippedAt === null;
  const step = () => {
    if (state.overwriteAt !== null && clock.now >= state.overwriteAt) { state.frequency = 400; state.overwriteAt = null; }
    const target = hashingNow() ? (state.frequency < 485 ? loweredHotC : hotC) : coolC;
    state.chip += Math.sign(target - state.chip) * Math.min(Math.abs(target - state.chip), hashingNow() ? 0.4 : 0.6);
    if (hashingNow() && state.chip >= tripC) { state.trippedAt = clock.now; state.entered = false; return; }
    if (state.trippedAt === null) return;
    if (!state.entered) {
      if (clock.now - state.trippedAt < safeStopMs) return;
      state.entered = true; state.overheat = 1; state.autofan = 0; state.manualFan = 100; state.coolSince = null;
      state.staysStopped = state.frequency - 100 < floorMhz;
      state.log.push(detail, "local_mining=run_ended generation=3 ending=Some(Overheat)", `local_mining=overheat stays_stopped=${state.staysStopped}`);
      return;
    }
    if (state.chip <= 45) state.coolSince ??= clock.now; else state.coolSince = null;
    const cooled = restartAfterMs === null ? state.coolSince !== null && clock.now - state.coolSince >= 30_000
      : clock.now - state.trippedAt >= restartAfterMs;
    if (!cooled) return;
    // As in the firmware, a stop that stays stopped pauses the device only once it has cooled.
    if (state.staysStopped) {
      state.paused = true; state.pauseReason = "overheat"; state.trippedAt = null;
      state.log.push("local_mining=overheat_stopped paused_by_device=true");
      return;
    }
    state.frequency -= lowerBy; state.coreVoltage -= lowerBy; state.trippedAt = null;
    state.log.push("local_mining=overheat_recovered");
    if (!state.paused) { state.runningSince = clock.now - 4000; state.overheat = 0; }
  };
  const info = () => {
    step();
    const active = hashingNow();
    return {
      sourceCommit: EXPECTED.firmwareCommit, appElfSha256: EXPECTED.appElfSha256, bootOrdinal: EXPECTED.bootOrdinal,
      bootSession: "s".repeat(32), resetReasonCategory: "power_on", miningPaused: state.paused,
      miningActivity: active ? "active" : state.paused ? "paused" : "safe_blocked", startMiningOnBoot: false,
      hashRate: active ? 400 : 0, sharesAccepted: Math.floor(clock.now / 60_000), sharesRejected: 0,
      temp: Math.round(state.chip * 10) / 10, vrTemp: 50, voltage: 5100, power: active ? 11 : 1, fanrpm: 4000,
      overheat_mode: state.overheat, autofanspeed: state.autofan, manualFanSpeed: state.manualFan,
      frequency: state.frequency, coreVoltage: state.coreVoltage, pauseReason: state.pauseReason, ...state.pool,
      powerStatus: { state: "fresh" }, voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" },
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
      if ("overheat_mode" in settings) {
        state.overheat = settings.overheat_mode;
        if (overwriteAfterRestoreMs !== null) state.overwriteAt = clock.now + overwriteAfterRestoreMs;
      }
    },
    async command(name) {
      state.calls.push(name);
      // A pause stops the ASIC but, as in the firmware, never cancels a recovery in progress.
      if (name === "pause") { state.paused = true; state.runningSince = null; return; }
      state.paused = false; state.pauseReason = null; state.runningSince = clock.now;
    },
  };
  return { client, clock, state };
}

/** Lets the device run on unobserved for `ms`, so a recovery still pending would finish. */
async function idle(device, ms) {
  for (let elapsed = 0; elapsed < ms; elapsed += 2000) {
    device.clock.now += 2000;
    await device.client.info();
  }
}

async function run(device, isInterrupted = () => false, soak = SOAK) {
  const events = [];
  const verdict = await runOverheat({ client: device.client, expected: EXPECTED, settings: SETTINGS, soak, isInterrupted,
    now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; }, record: async (event) => { events.push(event); } });
  return { verdict, events };
}

test("a device that trips, cools and restarts at the lowered pair passes", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outcome, "restarted");
});

test("the trip's cause is confirmed from the retained revocation detail", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.equal(verdict.metrics.trip_cause, "lowered_trip");
});

test("the original pair, fan and overheat mode come back, paused", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  await run(device);
  // Assert
  const { state } = device;
  assert.deepEqual([state.frequency, state.coreVoltage, state.autofan, state.manualFan, state.overheat, state.paused],
    [485, 1200, 1, 70, 0, true]);
});

test("a second trip that pauses the device passes", async () => {
  // Arrange
  const device = fakeOverheat({ loweredHotC: 58 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outcome, "restarted_then_stays_stopped");
});

test("one resume and a second pause clear the device's own overheat pause", async () => {
  // Arrange
  const device = fakeOverheat({ loweredHotC: 58 });
  // Act
  await run(device);
  // Assert
  assert.deepEqual([device.state.pauseReason, device.state.paused], [null, true]);
});

test("a stop during cooling waits for the recovery to settle before restoring", async () => {
  // Arrange: interrupted once the trip shows, while the recovery still has to save its pair.
  const device = fakeOverheat();
  // Act
  const { verdict } = await run(device, () => device.state.entered && device.state.trippedAt !== null);
  await idle(device, 10 * 60_000);
  // Assert
  assert.ok(verdict.failures.includes("soak_interrupted"));
  assert.deepEqual([device.state.frequency, device.state.coreVoltage, device.state.overheat], [485, 1200, 0]);
});

test("a recovery that never settles fails and is left paused", async () => {
  // Arrange: the room stays above the restart temperature, so the recovery never finishes.
  const device = fakeOverheat({ coolC: 50 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_recovery_unsettled"));
  assert.equal(device.state.paused, true);
});

test("a trip by another unsafe fact fails as the wrong cause", async () => {
  // Arrange
  const device = fakeOverheat({ detail: OTHER_DETAIL });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_wrong_cause"));
});

test("a device whose trip never fires fails as not tripped", async () => {
  // Arrange
  const device = fakeOverheat({ hotC: 54 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_not_tripped"));
});

test("a hashing chip at the missed-trip limit stops the check at once", async () => {
  // Arrange
  const device = fakeOverheat({ tripC: 99 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_trip_missed"));
});

test("a restart before 30 s of cooling fails", async () => {
  // Arrange
  const device = fakeOverheat({ restartAfterMs: 4000, safeStopMs: 0 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_restarted_before_cooling"));
});

test("a restart at a pair not lowered by 100 fails", async () => {
  // Arrange
  const device = fakeOverheat({ lowerBy: 50 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_pair_not_lowered"));
});

test("the trip cause is the detail just before the latest overheat run end", () => {
  // Arrange
  const log = [OTHER_DETAIL, "local_mining=run_ended generation=2 ending=Some(Failed)", TRIP_DETAIL,
    "local_mining=run_ended generation=3 ending=Some(Overheat)"].join("\n");
  // Act / Assert
  assert.equal(tripCause(log), "lowered_trip");
});

test("a log without an overheat run end gives no trip cause", () => {
  // Act / Assert
  assert.equal(tripCause(`${TRIP_DETAIL}\nlocal_mining=run_began generation=4`), null);
});

test("an overheat and the device's own overheat pause are no live stop here", () => {
  // Arrange
  const base = { bootSession: "s".repeat(32), pauseReason: null, hardwareFault: false, powerFault: false, overheatMode: false,
    miningActivity: "paused", miningPaused: true, fresh: { power: true, input: true, chipTemp: true, fan: true } };
  const sample = { ...base, overheatMode: true, pauseReason: "overheat" };
  // Act / Assert
  assert.equal(overheatLiveStop(sample, base, null), null);
});

test("any other device pause is still a live stop", () => {
  // Arrange
  const base = { bootSession: "s".repeat(32), pauseReason: null, hardwareFault: false, powerFault: false, overheatMode: false,
    miningActivity: "paused", miningPaused: true, fresh: { power: true, input: true, chipTemp: true, fan: true } };
  // Act / Assert
  assert.equal(overheatLiveStop({ ...base, pauseReason: "crash_loop" }, base, null), "soak_device_paused_itself");
});

test("a sample keeps the fan mode and manual speed", () => {
  // Act
  const sample = sampleFromInfo({ autofanspeed: 0, manualFanSpeed: 100 }, 0);
  // Assert
  assert.deepEqual([sample.autoFanSpeed, sample.manualFanSpeed], [false, 100]);
});

test("a restore the device overwrites afterwards fails", async () => {
  // Arrange
  const device = fakeOverheat({ overwriteAfterRestoreMs: 30_000 });
  // Act
  const { verdict } = await run(device);
  // Assert
  assert.ok(verdict.failures.includes("overheat_settings_not_restored"));
});

test("a log with no overheat entry has no recovery pending", () => {
  // Act / Assert
  assert.equal(recoveryFinished("local_mining=run_began generation=1\n"), true);
});

test("an overheat entry not yet followed by a restart or a pause is pending", () => {
  // Act / Assert
  assert.equal(recoveryFinished("local_mining=overheat stays_stopped=false\n"), false);
});

test("an overheat entry followed by a restart has finished", () => {
  // Act / Assert
  assert.equal(recoveryFinished("local_mining=overheat stays_stopped=false\nlocal_mining=overheat_recovered\n"), true);
});

test("an overheat entry followed by the device's pause has finished", () => {
  // Act / Assert
  assert.equal(recoveryFinished("local_mining=overheat stays_stopped=true\nlocal_mining=overheat_stopped paused_by_device=true\n"), true);
});

test("a failed save after the restart leaves the recovery pending", () => {
  // Act / Assert
  assert.equal(recoveryFinished(["local_mining=overheat stays_stopped=false", "local_mining=overheat_recovered",
    "local_mining=save_failed what=overheat_pair error=nvs"].join("\n")), false);
});

test("a later overheat entry reopens a finished recovery", () => {
  // Act / Assert
  assert.equal(recoveryFinished(["local_mining=overheat stays_stopped=false", "local_mining=overheat_recovered",
    "local_mining=overheat stays_stopped=true"].join("\n")), false);
});

test("a recovery resumed after a reboot is pending until it finishes", () => {
  // Act / Assert
  assert.equal(recoveryFinished("local_mining=overheat_resumed_after_boot stopped=false\n"), false);
});

test("a restore watched while the device is unreachable is not confirmed", async () => {
  // Arrange: once restored, the device stops answering.
  const device = fakeOverheat();
  const { info, patch } = device.client;
  let restoredAt = null;
  device.client.patch = async (settings) => { await patch(settings); if ("overheat_mode" in settings) restoredAt = device.clock.now; };
  device.client.info = async () => {
    if (restoredAt !== null && device.clock.now > restoredAt + 1000) throw new Error("unreachable");
    return info();
  };
  // Act
  const { events } = await run(device);
  // Assert
  assert.ok(events.some((event) => event.event === "settings_restore_unconfirmed"));
});

test("from the default pair, a trip whose device pauses itself after cooling passes check 002", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.deepEqual(verdict.failures, []);
  assert.equal(verdict.metrics.outcome, "stays_stopped");
});

test("check 002 clears the device's own pause and restores the settings", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  const { state } = device;
  assert.deepEqual([state.pauseReason, state.paused, state.frequency, state.overheat, state.autofan], [null, true, 485, 0, 1]);
});

test("a restart where check 002 expects the device to stay stopped fails", async () => {
  // Arrange: a device that wrongly takes 385 MHz as inside the catalog.
  const device = fakeOverheat({ floorMhz: 300 });
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.ok(verdict.failures.includes("overheat_unexpected_outcome"));
});

test("check 002 stops at the first restart", async () => {
  // Arrange: a device that wrongly takes 385 MHz as inside the catalog.
  const device = fakeOverheat({ floorMhz: 300 });
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.ok(verdict.failures.includes("overheat_unexpected_restart"));
  assert.equal(verdict.metrics.after_active_ms, 0);
});

test("check 002 confirms the stop from the retained log", async () => {
  // Arrange
  const device = fakeOverheat();
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.equal(verdict.metrics.stop_logged, true);
});

test("a failed stop-marker clear fails the check", async () => {
  // Arrange
  const device = fakeOverheat();
  const { command } = device.client;
  device.client.command = async (name) => {
    await command(name);
    if (name === "resume" && device.state.log.some((line) => line.includes("overheat_stopped"))) {
      device.state.log.push("local_mining=save_failed what=overheat_marker_clear error=nvs");
    }
  };
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.ok(verdict.failures.includes("overheat_stop_marker_not_cleared"));
});

test("a device lost after the clearing resume is paused again", async () => {
  // Arrange: the device stops answering right after the resume that clears its stop.
  const device = fakeOverheat();
  const { command, info } = device.client;
  let resumedToClear = false;
  device.client.command = async (name) => {
    await command(name);
    if (name === "resume" && device.state.log.some((line) => line.includes("overheat_stopped"))) resumedToClear = true;
  };
  let failures = 0;
  device.client.info = async () => {
    if (resumedToClear && failures < 3) { failures += 1; throw new Error("unreachable"); }
    return info();
  };
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.equal(device.state.paused, true);
  assert.ok(verdict.failures.includes("overheat_settings_restore_failed"));
});

test("a log whose latest entry stays stopped and is followed by the pause shows the stop", () => {
  // Act / Assert
  assert.equal(stopLogged("local_mining=overheat stays_stopped=true\nlocal_mining=overheat_stopped paused_by_device=true\n"), true);
});

test("a log whose entry would restart does not show the stop", () => {
  // Act / Assert
  assert.equal(stopLogged("local_mining=overheat stays_stopped=false\nlocal_mining=overheat_stopped paused_by_device=true\n"), false);
});

test("a stop line logged just after the pause reason is still confirmed", async () => {
  // Arrange: the first logs read after the pause misses the stop line, as the firmware sets the
  // pause reason just before logging it.
  const device = fakeOverheat();
  const { logs } = device.client;
  let hidden = false;
  device.client.logs = async () => {
    const log = await logs();
    if (!hidden && log.includes("overheat_stopped")) { hidden = true; return log.replace("local_mining=overheat_stopped paused_by_device=true", ""); }
    return log;
  };
  // Act
  const { verdict } = await run(device, () => false, SOAKS["overheat-002"]);
  // Assert
  assert.equal(verdict.metrics.stop_logged, true);
});
