// The boot-resume check (ADR-0038): with mining on boot turned on, a software restart must bring
// local mining back with no resume command; with it turned off again, a restart must leave the
// device paused. From the moment mining on boot is requested, every path turns it off again and
// pauses, because while it is on the Gate refuses Connect and only this endpoint can reach the
// device. Each boot's limits count from the first sample of that boot the check sees. Each restart is requested once; an ambiguous response is resolved by observing the new
// boot, never by a second request.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { finalFailures, hardwareStopped, increases, limitMetrics, liveStop, mining, miningLog, sampleFromInfo,
  SAMPLE_INTERVAL_MS, STOP_ALLOWANCE_MS, wifiMetrics } from "./judge.mjs";
import { admitBaseline, localPhases, pauseAndConfirm, sampleLoop, UNREACHABLE_STOP_MS } from "./run.mjs";
import { breakWatcher } from "./break-log.mjs";

/** A requested restart must show its new boot within this. */
export const RESTART_BOUND_MS = 180_000;
/** After the second restart, the device must stay paused this long. */
export const HOLD_MS = 120_000;
/** The reset category a requested restart reports. */
const RESTART_CATEGORY = "software_cpu";

/**
 * Runs the check; `expected` names the installed firmware and the handed-off boot. It returns the
 * verdict; the device ends paused with mining on boot off on every path it can still reach.
 */
export async function runBootResume({ client, expected, settings, now, sleep, record, isInterrupted = () => false, soak,
  keepLogs = async () => {} }) {
  let journalFailed = false;
  const note = async (event) => { try { await record(event); } catch { journalFailed = true; } };
  const baseline = await admitBaseline({ client, expected, settings, now, note });
  const context = { client, expected, now, sleep, note, isInterrupted };
  const marks = {}, miningSamples = [], holdSamples = [];
  let maybeStop = null, maybeFinal;
  const watcher = typeof client.logs === "function" ? breakWatcher({ readLogs: () => client.logs(), keepLogs, note }) : null;
  try {
    requireCondition(!isInterrupted(), "soak_interrupted_before_resume");
    // Marked before the request, so even an ambiguous response is undone.
    marks.enableRequested = true;
    await setMineOnBoot(client, true);
    await note({ event: "mine_on_boot_enabled", atMs: now() });
    marks.miningBoot = await restartOnce({ ...context, previous: baseline, bootOrdinal: expected.bootOrdinal + 1 });
    const boot = marks.miningBoot.sample;
    maybeStop = await sampleLoop({ client, baseline: boot, samples: miningSamples, resumedAt: boot.atMs, now, sleep, note,
      isInterrupted, soak, phases: localPhases({ soak, samples: miningSamples, resumedAt: boot.atMs }), watcher });
    if (maybeStop === null) {
      await setMineOnBoot(client, false);
      marks.disabled = true;
      await note({ event: "mine_on_boot_disabled", atMs: now() });
      marks.pausedFinal = await pauseAndConfirm({ client, now, sleep, note });
      requireCondition(marks.pausedFinal !== undefined && marks.pausedFinal.bootSession === boot.bootSession, "boot_resume_not_stopped");
      marks.heldBoot = await restartOnce({ ...context, previous: boot, bootOrdinal: expected.bootOrdinal + 2 });
      maybeFinal = await holdPaused({ ...context, boot: marks.heldBoot.sample, samples: holdSamples });
    }
  } catch (error) {
    maybeStop ??= typeof error?.code === "string" ? error.code : "soak_run_error";
  }
  if (maybeStop !== null) {
    await watcher?.stopped(now());
    await note({ event: "live_stop", code: maybeStop });
    if (marks.enableRequested) {
      const recovered = await pauseThenRestore({ client, now, sleep, note });
      marks.restored = recovered.mineOnBootOff;
      maybeFinal = recovered.maybeFinal;
    } else {
      maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
    }
  }
  const verdict = judgeBootResume({ baseline, marks, miningSamples, holdSamples, maybeFinal, soak,
    maybeStop: maybeStop ?? (journalFailed ? "soak_journal_failed" : null) });
  await note({ event: "verdict", result: verdict.result, failures: verdict.failures });
  return verdict;
}

/** Saves mining on boot and reads it back. */
async function setMineOnBoot(client, value) {
  await client.patch({ startMiningOnBoot: value });
  requireCondition((await client.info()).startMiningOnBoot === value, "boot_resume_preference_unsaved");
}

/**
 * Recovery: pause at once, then turn mining on boot off if it does not already read off, then
 * confirm the ASIC powered down. The pause comes first because the setting only matters at the next
 * boot, and a refused write must not keep the device mining. Never throws.
 */
export async function pauseThenRestore({ client, now, sleep, note }) {
  let paused = false;
  const pause = async () => {
    try { await client.command("pause"); paused = true; await note({ event: "paused", atMs: now() }); }
    catch { await note({ event: "pause_failed", atMs: now() }); }
  };
  await pause();
  let mineOnBootOff = false;
  try { mineOnBootOff = (await client.info()).startMiningOnBoot === false; } catch { /* restore below */ }
  // A device still restarting may come back mining; until a pause lands, each write attempt re-sends it.
  if (!mineOnBootOff) mineOnBootOff = await restoreMineOnBoot({ client, now, sleep, note, beforeWrite: async () => { if (!paused) await pause(); } });
  const maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
  return { mineOnBootOff, maybeFinal };
}

/** Turns mining on boot off, retrying while the stop allowance lasts; whether it read back off. Never throws. */
export async function restoreMineOnBoot({ client, now, sleep, note, beforeWrite = async () => {} }) {
  const startedAt = now();
  while (now() - startedAt <= STOP_ALLOWANCE_MS) {
    await beforeWrite();
    try {
      await setMineOnBoot(client, false);
      await note({ event: "mine_on_boot_restored", atMs: now() });
      return true;
    } catch { /* the device may be restarting; try again */ }
    await sleep(SAMPLE_INTERVAL_MS);
  }
  await note({ event: "mine_on_boot_restore_failed", atMs: now() });
  return false;
}

/**
 * Requests one restart and waits for the next boot: a new boot session, reset by software, on the
 * same firmware, with the next boot ordinal. Returns that boot's first sample and the time taken.
 */
async function restartOnce({ client, expected, now, sleep, note, isInterrupted, previous, bootOrdinal }) {
  const requestedAt = now();
  try {
    await client.command("restart");
    await note({ event: "restart_requested", atMs: requestedAt });
  } catch {
    await note({ event: "restart_ambiguous", atMs: requestedAt });
  }
  for (;;) {
    await sleep(SAMPLE_INTERVAL_MS);
    requireCondition(!isInterrupted(), "soak_interrupted");
    requireCondition(now() - requestedAt <= RESTART_BOUND_MS, "boot_resume_restart_unobserved");
    let info;
    try { info = await client.info(); } catch { continue; }
    if (info.bootSession === previous.bootSession) continue;
    requireCondition(info.resetReasonCategory === RESTART_CATEGORY, "boot_resume_reset_category");
    requireCondition(info.sourceCommit === expected.firmwareCommit && info.appElfSha256 === expected.appElfSha256,
      "soak_firmware_identity");
    requireCondition(info.bootOrdinal === bootOrdinal, "boot_resume_boot_ordinal");
    const sample = sampleFromInfo(info, now());
    const restartMs = sample.atMs - requestedAt;
    await note({ event: "rebooted", sample, restartMs });
    return { sample, restartMs };
  }
}

/**
 * Watches the boot after mining on boot was turned off for the hold time, from its first sample: every sample paused, not
 * mining, with mining on boot off. Returns the final sample once the last two show the ASIC powered
 * down, else undefined.
 */
async function holdPaused({ client, now, sleep, note, isInterrupted, boot, samples }) {
  let lastReachedAt = now();
  // The hold is judged from the boot's first sample, so it ends on a sample at least that late.
  while (samples.length === 0 || samples.at(-1).atMs - boot.atMs < HOLD_MS) {
    await sleep(SAMPLE_INTERVAL_MS);
    requireCondition(!isInterrupted(), "soak_interrupted");
    let info;
    try { info = await client.info(); } catch {
      requireCondition(now() - lastReachedAt <= UNREACHABLE_STOP_MS, "soak_device_unreachable");
      continue;
    }
    lastReachedAt = now();
    const sample = sampleFromInfo(info, lastReachedAt);
    samples.push(sample);
    await note({ event: "hold_sample", sample });
    const maybeStop = liveStop(sample, boot, samples.at(-2) ?? null);
    requireCondition(maybeStop === null, maybeStop);
    requireCondition(heldPaused(sample), "boot_resume_mined_after_disable");
  }
  const [maybePrevious, maybeLast] = samples.slice(-2);
  const stopped = maybePrevious !== undefined && hardwareStopped(maybePrevious) && hardwareStopped(maybeLast);
  await note(stopped ? { event: "final", sample: maybeLast } : { event: "final_missing" });
  return stopped ? maybeLast : undefined;
}

const heldPaused = (sample) => sample.miningPaused && !mining(sample) && !sample.startMiningOnBoot;

/** The check's verdict over its marks, the mining boot's samples, the held boot's samples and the final sample. */
export function judgeBootResume({ baseline, marks, miningSamples, holdSamples, maybeFinal, maybeStop, soak }) {
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  const maybeMiningBoot = marks.miningBoot?.sample ?? null;
  const maybeHeldBoot = marks.heldBoot?.sample ?? null;
  if (maybeMiningBoot === null || maybeHeldBoot === null || !marks.disabled) fail("boot_resume_incomplete");
  // Mining on boot: the boot itself unpaused, then the soak's mining with no resume command.
  if (maybeMiningBoot !== null && maybeMiningBoot.miningPaused) fail("boot_resume_started_paused");
  miningSamples.forEach((sample, index) => {
    const stop = liveStop(sample, maybeMiningBoot, miningSamples[index - 1] ?? null); if (stop) fail(stop);
  });
  const log = miningLog(miningSamples);
  if (log.activeMs < soak.activeMs) fail("soak_active_time_short");
  if (log.outagesMs.some((outageMs) => outageMs > soak.maybeMaximumOutageMs)) fail("soak_outage_unrecovered");
  const active = miningSamples.filter(mining);
  if (!active.some((sample) => sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  const accepted = increases(miningSamples, "sharesAccepted");
  if (accepted < 1) fail("soak_no_accepted_share");
  // Off again: the next boot stays paused.
  if (maybeHeldBoot !== null && !heldPaused(maybeHeldBoot)) fail("boot_resume_mined_after_disable");
  holdSamples.forEach((sample, index) => {
    const stop = liveStop(sample, maybeHeldBoot, holdSamples[index - 1] ?? null); if (stop) fail(stop);
    if (!heldPaused(sample)) fail("boot_resume_mined_after_disable");
  });
  const holdMs = holdSamples.length < 1 || maybeHeldBoot === null ? 0 : holdSamples.at(-1).atMs - maybeHeldBoot.atMs;
  if (maybeHeldBoot !== null && holdMs < HOLD_MS) fail("boot_resume_hold_short");
  // The final state belongs to the last boot the run saw; any later boot is a reset.
  finalFailures(maybeHeldBoot ?? maybeMiningBoot ?? baseline, maybeFinal).forEach(fail);
  if (marks.enableRequested && marks.restored === false) fail("boot_resume_preference_not_restored");
  const firstMining = active[0] ?? null;
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      mining_restart_ms: marks.miningBoot?.restartMs ?? null,
      mining_boot_reset: maybeMiningBoot?.resetReasonCategory ?? null,
      first_mining_uptime_s: firstMining?.uptimeSeconds ?? null,
      first_mining_after_boot_ms: firstMining === null || maybeMiningBoot === null ? null : firstMining.atMs - maybeMiningBoot.atMs,
      accumulated_active_ms: log.activeMs,
      outage_count: log.outagesMs.length + (log.maybeOpenOutageMs === null ? 0 : 1),
      maximum_outage_ms: Math.max(0, ...log.outagesMs, log.maybeOpenOutageMs ?? 0),
      shares_accepted: accepted,
      shares_rejected: increases(miningSamples, "sharesRejected"),
      held_restart_ms: marks.heldBoot?.restartMs ?? null,
      held_boot_reset: maybeHeldBoot?.resetReasonCategory ?? null,
      hold_ms: holdMs,
      hold_samples: holdSamples.length,
      ...wifiMetrics([...miningSamples, ...holdSamples]),
      active_samples: active.length,
      samples: miningSamples.length,
      ...limitMetrics(active),
    },
  };
}
