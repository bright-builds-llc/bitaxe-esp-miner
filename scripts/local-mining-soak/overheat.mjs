// The overheat check (ADR-0038 decision 2, ADR-0040): on the test-only firmware whose local
// overheat trip is 55 C, local mining at a raised but catalog pair trips at normal mining
// temperatures. The device must stop, cool to 45 C or less for 30 s with the fan at 100%, and
// restart 100 MHz and 100 mV lower. Then it either mines at the lowered pair, or trips again and
// pauses itself, because the next step would leave the catalog. Every path pauses, waits until
// the firmware's recovery has settled (it continues while paused), then restores the original
// pair, the fan and `overheat_mode`. A device that paused itself is resumed once to clear its
// stop, which starts a short run, then paused again.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { finalFailures, hashing, increases, limitMetrics, liveStop, mining, miningLog, SAMPLE_INTERVAL_MS, sampleFromInfo,
  START_ALLOWANCE_MS, statusOnlySamples, wifiMetrics } from "./judge.mjs";
import { admitBaseline, pauseAndConfirm, sampleLoop } from "./run.mjs";

/** A hashing chip at or above this means the lowered trip did not fire: stop at once. */
export const TRIP_MISSED_C = 60;
/** The recovery restarts only after this long at or below the restart temperature. */
const COOLING_MS = 30_000;
const RESTART_TEMP_C = 45;
/**
 * The firmware's recovery has settled once the settings it writes have not changed for this long
 * (longer than the safe-stop's 120 s cooling proof) and the chip has read 45 C or less for the
 * last `SETTLED_COOL_MS`, so a pending restart would already have saved its pair.
 */
export const SETTLE_QUIET_MS = 150_000;
const SETTLED_COOL_MS = 40_000;
/** The longest wait for the recovery to settle before the check gives up and stays paused. */
export const SETTLE_BOUND_MS = 15 * 60_000;
/** How long a self-paused device may take to clear its pause after the resume. */
const UNPAUSE_BOUND_MS = 60_000;
/** After restoring, the settings must stay restored this long, seen by at least this many samples. */
export const RESTORE_WATCH_MS = 60_000;
const RESTORE_WATCH_SAMPLES = 20;

/**
 * The live stops of this check: soak 002's, except that an overheat and the device's own
 * overheat pause are expected. A hashing chip at `TRIP_MISSED_C` stops it.
 */
export function overheatLiveStop(sample, baseline, maybePrevious) {
  if (hashing(sample) && sample.chipTempC !== null && sample.fresh.chipTemp && sample.chipTempC >= TRIP_MISSED_C) {
    return "overheat_trip_missed";
  }
  const expected = { ...sample, overheatMode: false, pauseReason: sample.pauseReason === "overheat" ? null : sample.pauseReason };
  return liveStop(expected, baseline, maybePrevious);
}

/**
 * Why the latest overheat ended its run, from the retained log: the revocation detail just before
 * the latest overheat `run_ended` line. The lowered trip revokes with no failing fact; any other
 * unsafe stop names one. Null when the log holds no such pair.
 */
export function tripCause(log) {
  const lines = log.split("\n");
  const ended = lines.findLastIndex((line) => line.includes("local_mining=run_ended") && line.includes("ending=Some(Overheat)"));
  if (ended < 0) return null;
  const detail = lines.slice(0, ended).findLast((line) => line.includes("worker_revocation_detail"));
  if (detail === undefined) return null;
  return detail.includes(" trigger=unsafe_sample fact=none ") ? "lowered_trip" : "other_unsafe_stop";
}

/**
 * Whether the firmware's own record shows its overheat recovery finished, from the retained log:
 * the latest overheat entry followed by a restart (`overheat_recovered`) or a device pause
 * (`overheat_stopped`), with no failed save after that, which the firmware would retry. A log
 * with no overheat entry has nothing pending.
 */
export function recoveryFinished(log) {
  const lines = log.split("\n");
  // A recovery resumed after a reboot is an entry too.
  const entered = lines.findLastIndex((line) => line.includes("local_mining=overheat stays_stopped=") ||
    line.includes("local_mining=overheat_resumed_after_boot"));
  if (entered < 0) return true;
  const later = lines.slice(entered + 1);
  const finished = later.findLastIndex((line) =>
    line.includes("local_mining=overheat_recovered") || line.includes("local_mining=overheat_stopped paused_by_device=true"));
  return finished >= 0 && !later.slice(finished + 1).some((line) => line.includes("local_mining=save_failed"));
}

/**
 * Whether the retained log shows the overheat stop that stays stopped: the latest overheat entry
 * says `stays_stopped=true` and is followed by the device's own pause.
 */
export function stopLogged(log) {
  const lines = log.split("\n");
  const entered = lines.findLastIndex((line) => line.includes("local_mining=overheat stays_stopped="));
  return entered >= 0 && lines[entered].includes("stays_stopped=true") &&
    lines.slice(entered + 1).some((line) => line.includes("local_mining=overheat_stopped paused_by_device=true"));
}

/** The check's phases for `sampleLoop`: heat until the trip, cool until the restart, then mine at the lowered pair. */
export function overheatPhases({ soak, samples, resumedAt, marks, note, readCause, readStop }) {
  let phase = "heat";
  const begin = async (next, event, sample, detail = {}) => {
    phase = next;
    await note({ event, atMs: sample.atMs, ...detail });
  };
  // The trip's log is read while cooling; a failed read is retried on the next sample.
  const confirmCause = async () => {
    if (marks.tripCause !== undefined) return;
    const maybeCause = await readCause();
    if (maybeCause === null) return;
    marks.tripCause = maybeCause;
    await note({ event: "overheat_cause", cause: maybeCause });
  };
  const restartAccepted = soak.outcomes.some((outcome) => outcome.startsWith("restarted"));
  // The device paused itself: confirm from the retained log, retried on later samples for up to a
  // minute, that it was the overheat stop that stays stopped.
  const stopped = async (sample) => {
    if (marks.staysStoppedAtMs === undefined) {
      marks.staysStoppedAtMs = sample.atMs;
      await begin("stopped", "overheat_stays_stopped", sample);
    }
    // The firmware sets the pause reason just before it logs the stop, so a read without the line
    // is retried until the deadline.
    if (marks.stopLogged !== true && await readStop() === true) {
      marks.stopLogged = true;
      await note({ event: "overheat_stop_logged", logged: true });
    }
    if (marks.stopLogged === true) return null;
    if (sample.atMs - marks.staysStoppedAtMs <= 60_000) return undefined;
    marks.stopLogged = false;
    await note({ event: "overheat_stop_logged", logged: false });
    return null;
  };
  const steps = {
    async heat(sample) {
      const maybeFirst = samples.find(hashing);
      if (maybeFirst === undefined) return sample.atMs - resumedAt > START_ALLOWANCE_MS ? "soak_mining_not_started" : undefined;
      if (sample.overheatMode) {
        marks.trippedAtMs = sample.atMs;
        marks.tripChipC = samples.findLast(hashing)?.chipTempC ?? null;
        await begin("cool", "overheat_tripped", sample, { lastHashingChipC: marks.tripChipC });
        await confirmCause();
        return undefined;
      }
      // The trip's ordered safe-stop proves the chip cool before `overheat_mode` is saved, so a
      // break counts as an outage only past the soak's allowance, which covers that stop.
      if (!hashing(sample) && sample.atMs - samples.findLast(hashing).atMs > soak.maybeMaximumOutageMs) return "soak_outage_unrecovered";
      return sample.atMs - maybeFirst.atMs > soak.tripBoundMs ? "overheat_not_tripped" : undefined;
    },
    async cool(sample) {
      await confirmCause();
      if (sample.pauseReason === "overheat") return stopped(sample);
      // A check that expects the device to stay stopped stops at the first restart.
      if (hashing(sample) && !restartAccepted) return "overheat_unexpected_restart";
      if (hashing(sample)) {
        marks.restartedAtMs = sample.atMs;
        marks.afterIndex = samples.length - 1;
        await begin("after", "overheat_restarted", sample, { frequencyMhz: sample.frequencyMhz, coreVoltageMv: sample.coreVoltageMv });
        return undefined;
      }
      if (sample.atMs - marks.trippedAtMs <= soak.restartBoundMs) return undefined;
      return restartAccepted ? "overheat_not_restarted" : "overheat_not_stopped";
    },
    async after(sample) {
      // `overheat_mode` clears only once the restarted run has prepared, so a second trip is a
      // set flag after it has read clear.
      if (!sample.overheatMode) marks.overheatClearedAtMs ??= sample.atMs;
      if (sample.overheatMode && marks.overheatClearedAtMs !== undefined && marks.secondTripAtMs === undefined) {
        marks.secondTripAtMs = sample.atMs;
        await note({ event: "overheat_tripped_again", atMs: sample.atMs });
      }
      if (sample.pauseReason === "overheat") return stopped(sample);
      if (marks.secondTripAtMs !== undefined) {
        return sample.atMs - marks.secondTripAtMs > soak.restartBoundMs ? "overheat_not_stopped" : undefined;
      }
      const log = miningLog(samples.slice(marks.afterIndex));
      if (log.maybeLastMiningAtMs !== null && sample.atMs - log.maybeLastMiningAtMs > soak.maybeMaximumOutageMs) {
        return "soak_outage_unrecovered";
      }
      return log.activeMs >= soak.afterActiveMs ? null : undefined;
    },
  };
  return {
    // Outages count only while mining at the lowered pair; the trip and cooling idle by design.
    outageSamples: () => (phase === "after" && marks.secondTripAtMs === undefined ? samples.slice(marks.afterIndex) : []),
    onSample: (sample) => (phase === "stopped" ? stopped(sample) : steps[phase](sample)),
  };
}

/** The device's settings this check changes, read from a sample. */
export const settingsOf = (sample) => ({ frequency: sample.frequencyMhz, coreVoltage: sample.coreVoltageMv,
  autofanspeed: sample.autoFanSpeed ? 1 : 0, manualFanSpeed: sample.manualFanSpeed });

/** Whether `sample` shows `settings` saved and `overheat_mode` clear. */
const restored = (sample, settings) => sample.frequencyMhz === settings.frequency && sample.coreVoltageMv === settings.coreVoltage &&
  sample.autoFanSpeed === (settings.autofanspeed === 1) && sample.manualFanSpeed === settings.manualFanSpeed && !sample.overheatMode;

/** What the firmware's recovery writes, as one comparable key. */
const recoveryKey = (sample) => JSON.stringify([sample.overheatMode, sample.frequencyMhz, sample.coreVoltageMv, sample.autoFanSpeed,
  sample.manualFanSpeed, sample.pauseReason]);

/**
 * Runs the check; `expected` names the installed test firmware and the handed-off boot. It returns
 * the verdict; the device ends paused with its original pair, fan and `overheat_mode` 0.
 */
export async function runOverheat({ client, expected, settings, now, sleep, record, isInterrupted = () => false, soak,
  keepLogs = async () => {} }) {
  let journalFailed = false;
  const note = async (event) => { try { await record(event); } catch { journalFailed = true; } };
  const baseline = await admitBaseline({ client, expected, settings, now, note });
  requireCondition(!baseline.overheatMode, "overheat_baseline_in_overheat");
  const original = settingsOf(baseline);
  const marks = {}, samples = [];
  let maybeStop = null, captures = 0;
  // The raw retained log can hold the station address: it is kept privately, never journaled.
  const readCause = async () => {
    try {
      const log = await client.logs();
      captures += 1;
      await keepLogs(log, captures);
      return tripCause(log);
    } catch { return null; }
  };
  const readStop = async () => {
    try {
      const log = await client.logs();
      captures += 1;
      await keepLogs(log, captures);
      return stopLogged(log);
    } catch { return null; }
  };
  try {
    requireCondition(!isInterrupted(), "soak_interrupted_before_resume");
    await client.patch({ frequency: soak.frequencyMhz, coreVoltage: soak.coreVoltageMv });
    const saved = sampleFromInfo(await client.info(), now());
    requireCondition(saved.frequencyMhz === soak.frequencyMhz && saved.coreVoltageMv === soak.coreVoltageMv, "overheat_profile_unsaved");
    await note({ event: "profile_saved", frequencyMhz: soak.frequencyMhz, coreVoltageMv: soak.coreVoltageMv });
    const resumedAt = now();
    await client.command("resume");
    await note({ event: "resumed", atMs: resumedAt });
    maybeStop = await sampleLoop({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted, soak,
      phases: overheatPhases({ soak, samples, resumedAt, marks, note, readCause, readStop }), watcher: null, stopRule: overheatLiveStop });
  } catch (error) {
    maybeStop ??= typeof error?.code === "string" ? error.code : "soak_run_error";
  }
  if (maybeStop !== null) await note({ event: "live_stop", code: maybeStop });
  const cleanup = await restoreAndStop({ client, original, now, sleep, note });
  const verdict = judgeOverheat({ baseline, samples, marks, original, cleanup, soak,
    maybeStop: maybeStop ?? (journalFailed ? "soak_journal_failed" : null) });
  await note({ event: "verdict", result: verdict.result, failures: verdict.failures });
  return verdict;
}

/**
 * Waits until the firmware's recovery has settled: its settings unchanged for `SETTLE_QUIET_MS`,
 * the chip at 45 C or less for the last `SETTLED_COOL_MS`, and the retained log showing the
 * recovery finished (`recoveryFinished`). Sampling every 2 s can miss a warm reading that restarts
 * the firmware's 30 s, and a failed save changes nothing visible, so the log has the last word.
 * Whether it settled within `SETTLE_BOUND_MS`. Never throws.
 */
export async function awaitRecoverySettled({ client, now, sleep, note }) {
  const startedAt = now();
  let maybeKey = null, quietSince = startedAt, maybeCoolSince = null;
  while (now() - startedAt <= SETTLE_BOUND_MS) {
    let maybeSample = null;
    try { maybeSample = sampleFromInfo(await client.info(), now()); } catch { /* the next sample decides */ }
    if (maybeSample !== null) {
      const key = recoveryKey(maybeSample);
      if (key !== maybeKey) { maybeKey = key; quietSince = maybeSample.atMs; }
      const cool = maybeSample.fresh.chipTemp && maybeSample.chipTempC !== null && maybeSample.chipTempC <= RESTART_TEMP_C;
      maybeCoolSince = cool ? (maybeCoolSince ?? maybeSample.atMs) : null;
      const quietAndCool = maybeSample.atMs - quietSince >= SETTLE_QUIET_MS && maybeCoolSince !== null &&
        maybeSample.atMs - maybeCoolSince >= SETTLED_COOL_MS;
      // The raw log can hold the station address; it is only parsed here, never kept.
      if (quietAndCool && await logShowsFinished(client)) {
        await note({ event: "recovery_settled", atMs: maybeSample.atMs });
        return true;
      }
    }
    await sleep(SAMPLE_INTERVAL_MS);
  }
  await note({ event: "recovery_unsettled", atMs: now() });
  return false;
}

/** Whether the retained log shows the recovery finished; an unreadable log does not. Never throws. */
async function logShowsFinished(client) {
  try { return recoveryFinished(await client.logs()); } catch { return false; }
}

/**
 * Samples for `RESTORE_WATCH_MS` after a restore; whether every sample kept the original settings
 * and enough samples were read to show it. Never throws.
 */
async function restoreHeld({ client, original, now, sleep }) {
  const startedAt = now();
  let seen = 0;
  while (now() - startedAt < RESTORE_WATCH_MS) {
    await sleep(SAMPLE_INTERVAL_MS);
    try {
      if (!restored(sampleFromInfo(await client.info(), now()), original)) return false;
      seen += 1;
    } catch { /* a missed sample proves nothing either way */ }
  }
  return seen >= RESTORE_WATCH_SAMPLES;
}

/**
 * Pauses, waits for the recovery to settle, restores the original settings, clears a device pause
 * by one resume and a second pause, and reads the result back. Returns whether the recovery settled
 * and the final powered-down sample, or undefined. Never throws. An unsettled recovery is left
 * paused and unrestored, for the operator.
 */
export async function restoreAndStop({ client, original, now, sleep, note }) {
  let maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
  const settled = await awaitRecoverySettled({ client, now, sleep, note });
  if (!settled) return { settled, maybeFinal, markerCleared: true };
  let resumeSent = false, markerCleared = true, restoreFailed = false;
  try {
    await client.patch({ ...original, overheat_mode: 0 });
    if (sampleFromInfo(await client.info(), now()).pauseReason === "overheat") {
      // Only an operator resume ends an overheat stop and clears its marker; it starts a short run.
      resumeSent = true;
      await client.command("resume");
      await note({ event: "resumed_to_clear_stop", atMs: now() });
      const startedAt = now();
      while (now() - startedAt <= UNPAUSE_BOUND_MS && sampleFromInfo(await client.info(), now()).pauseReason === "overheat") {
        await sleep(SAMPLE_INTERVAL_MS);
      }
      maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
      // The resume saves the stop marker clear once and never retries it; a failure would leave
      // the next boot paused by itself.
      markerCleared = await markerClearSaved(client);
      await note({ event: markerCleared ? "stop_marker_cleared" : "stop_marker_clear_unconfirmed" });
    }
    // A restore the firmware overwrites afterwards is no restore.
    const held = restored(sampleFromInfo(await client.info(), now()), original) && await restoreHeld({ client, original, now, sleep });
    await note({ event: held ? "settings_restored" : "settings_restore_unconfirmed" });
    // The final sample is the end state, read after the watch.
    if (maybeFinal !== undefined) maybeFinal = sampleFromInfo(await client.info(), now());
  } catch {
    restoreFailed = true;
    await note({ event: "settings_restore_failed", atMs: now() });
    // A resume sent to clear a stop must never leave the device mining.
    if (resumeSent) maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
  }
  return { settled, maybeFinal, markerCleared, restoreFailed };
}

/** Whether the retained log shows no failed stop-marker clear; an unreadable log does not. Never throws. */
async function markerClearSaved(client) {
  try { return !(await client.logs()).includes("local_mining=save_failed what=overheat_marker_clear"); } catch { return false; }
}

/** The check's verdict over its samples, marks, the cleanup and the final sample. */
export function judgeOverheat({ baseline, samples, marks, original, cleanup, maybeStop, soak }) {
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  samples.forEach((sample, index) => { const stop = overheatLiveStop(sample, baseline, samples[index - 1] ?? null); if (stop) fail(stop); });
  if (marks.trippedAtMs === undefined) fail("overheat_not_tripped");
  if (marks.trippedAtMs !== undefined && marks.tripCause !== "lowered_trip") {
    fail(marks.tripCause === undefined ? "overheat_cause_unconfirmed" : "overheat_wrong_cause");
  }
  const cooling = samples.filter((sample) => marks.trippedAtMs !== undefined && sample.atMs >= marks.trippedAtMs &&
    (marks.restartedAtMs === undefined || sample.atMs < marks.restartedAtMs));
  // The saved fan setting is manual 100% from the trip on, as upstream leaves it.
  if (cooling.some((sample) => sample.autoFanSpeed || sample.manualFanSpeed !== 100)) fail("overheat_fan_not_full");
  const lowered = { frequencyMhz: soak.frequencyMhz - 100, coreVoltageMv: soak.coreVoltageMv - 100 };
  if (marks.restartedAtMs !== undefined) {
    const restart = samples.find((sample) => sample.atMs === marks.restartedAtMs);
    if (restart.frequencyMhz !== lowered.frequencyMhz || restart.coreVoltageMv !== lowered.coreVoltageMv) fail("overheat_pair_not_lowered");
    const cooled = cooledBeforeMs(samples, marks.trippedAtMs, marks.restartedAtMs);
    if (cooled === null) fail("overheat_cooling_inconclusive");
    else if (cooled < COOLING_MS) fail("overheat_restarted_before_cooling");
  } else if (marks.staysStoppedAtMs === undefined && soak.outcomes.some((outcome) => outcome.startsWith("restarted"))) {
    fail("overheat_not_restarted");
  } else if (marks.staysStoppedAtMs !== undefined) {
    const cooled = cooledBeforeMs(samples, marks.trippedAtMs, marks.staysStoppedAtMs);
    if (cooled === null) fail("overheat_cooling_inconclusive");
    else if (cooled < COOLING_MS) fail("overheat_stopped_before_cooling");
  }
  if (marks.staysStoppedAtMs !== undefined && marks.stopLogged !== true) fail("overheat_stop_unlogged");
  if (cleanup.markerCleared === false) fail("overheat_stop_marker_not_cleared");
  // A cleanup that broke off skipped the restore watch and the marker check.
  if (cleanup.restoreFailed) fail("overheat_settings_restore_failed");
  const after = marks.afterIndex === undefined ? [] : samples.slice(marks.afterIndex);
  const afterLog = miningLog(after);
  const staysStopped = marks.staysStoppedAtMs !== undefined;
  if (marks.restartedAtMs !== undefined && !staysStopped && afterLog.activeMs < soak.afterActiveMs) fail("soak_active_time_short");
  const active = samples.filter(hashing);
  if (!active.some((sample) => sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  if (!cleanup.settled) fail("overheat_recovery_unsettled");
  const outcome = staysStopped ? (marks.restartedAtMs === undefined ? "stays_stopped" : "restarted_then_stays_stopped")
    : marks.restartedAtMs !== undefined ? "restarted" : "none";
  if (marks.trippedAtMs !== undefined && !soak.outcomes.includes(outcome)) fail("overheat_unexpected_outcome");
  finalFailures(baseline, cleanup.maybeFinal).forEach(fail);
  if (cleanup.maybeFinal && !restored(cleanup.maybeFinal, original)) fail("overheat_settings_not_restored");
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      outcome,
      trip_cause: marks.tripCause ?? null,
      trip_chip_c: marks.tripChipC ?? null,
      first_hashing_to_trip_ms: marks.trippedAtMs === undefined || active.length === 0 ? null : marks.trippedAtMs - active[0].atMs,
      trip_to_restart_ms: marks.restartedAtMs === undefined ? null : marks.restartedAtMs - marks.trippedAtMs,
      cooled_ms: cooledBeforeMs(samples, marks.trippedAtMs, marks.restartedAtMs ?? marks.staysStoppedAtMs),
      minimum_cooling_chip_c: minimum(cooling.map((sample) => sample.chipTempC)),
      // A pair below the catalog is never applied, so it is reported only once a restart used it.
      lowered_frequency_mhz: marks.restartedAtMs === undefined ? null : lowered.frequencyMhz,
      lowered_core_voltage_mv: marks.restartedAtMs === undefined ? null : lowered.coreVoltageMv,
      stop_logged: marks.stopLogged ?? null,
      after_active_ms: afterLog.activeMs,
      maximum_after_chip_c: maximum(after.filter(hashing).map((sample) => sample.chipTempC)),
      second_trip: marks.secondTripAtMs !== undefined,
      recovery_settled: cleanup.settled,
      ...wifiMetrics(samples),
      active_samples: active.length,
      status_only_samples: statusOnlySamples(samples),
      samples: samples.length,
      shares_accepted: increases(samples, "sharesAccepted"),
      ...limitMetrics(samples.filter(mining)),
    },
  };
}

/** The gap before the action sample beyond which the action's time is too uncertain to judge. */
const ACTION_GAP_LIMIT_MS = 2 * SAMPLE_INTERVAL_MS + 1_000;

/**
 * The longest cooling consistent with the samples before the device acted (restarted or paused
 * itself), or null when the samples cannot bound it. The firmware's 30 s window starts no earlier
 * than its overheat entry, which comes after the last sample before the trip showed, and restarts
 * at any warm or stale reading. So the window began after the latest such sample, and the action
 * came no later than the sample that showed it. The action sample's reading follows the action, so
 * only its time counts. A correct device always measures at least 30 s. A gap of more than
 * `ACTION_GAP_LIMIT_MS` just before the action, or just after the start sample, could hide an early
 * action, so either gives null.
 */
export function cooledBeforeMs(samples, maybeTrippedAtMs, maybeActionAtMs) {
  if (maybeTrippedAtMs === undefined || maybeActionAtMs === undefined) return null;
  const before = samples.filter((sample) => sample.atMs < maybeActionAtMs);
  const maybeLastBefore = before.at(-1);
  if (maybeLastBefore === undefined || maybeActionAtMs - maybeLastBefore.atMs > ACTION_GAP_LIMIT_MS) return null;
  const warm = (sample) => !(sample.fresh.chipTemp && sample.chipTempC !== null && sample.chipTempC <= RESTART_TEMP_C);
  const startIndex = before.findLastIndex((sample) => sample.atMs < maybeTrippedAtMs || warm(sample));
  if (startIndex < 0) return null;
  const maybeNext = before[startIndex + 1] ?? null;
  const nextAtMs = maybeNext === null ? maybeActionAtMs : maybeNext.atMs;
  if (nextAtMs - before[startIndex].atMs > ACTION_GAP_LIMIT_MS) return null;
  return maybeActionAtMs - before[startIndex].atMs;
}

const minimum = (values) => { const kept = values.filter((value) => value !== null); return kept.length ? Math.min(...kept) : null; };
const maximum = (values) => { const kept = values.filter((value) => value !== null); return kept.length ? Math.max(...kept) : null; };
