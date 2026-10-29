// One local mining soak against an admitted, paused Ultra 205 (ADR-0038 task 6): confirm the
// installed identity and the paused baseline, save the owner's pool, resume, observe until the
// soak's mining time (or, for a lease check, its phases), a live stop or the run deadline, then
// pause and confirm the ASIC powered down. From the resume on, every path pauses the device. The
// device, clock, journal and lease result are injected, so the sequence is tested without hardware.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { activeSpanMs, hardwareStoppedUnder, hashingUnder, judge, LIMITS, limitsOf, liveStop, mining, miningLog, runDeadlineMs,
  sampleFromInfo, SAMPLE_INTERVAL_MS, SOAKS, START_ALLOWANCE_MS, STOP_ALLOWANCE_MS } from "./judge.mjs";
import { judgeLease, leasePhases } from "./lease.mjs";
import { handoverPhases, judgeHandover } from "./handover.mjs";
import { breakWatcher } from "./break-log.mjs";
import { judgeStopPhase, watchdogNotFresh } from "./stopping.mjs";
import { poolSaved } from "./pool.mjs";

/** The device stays unreachable this long before the soak stops. */
export const UNREACHABLE_STOP_MS = 30_000;
const PAUSE_RESEND_MS = 30_000;

/**
 * Runs the soak; `expected` names the installed firmware and the handed-off boot. A lease check
 * also takes `leaseResult`, which returns the Gate soak's result once it exists, else null.
 */
export async function runSoak({ client, expected, settings, now, sleep, record, isInterrupted = () => false, soak = SOAKS["001"],
  leaseResult = async () => null, keepLogs = async () => {} }) {
  // A journal failure must never stop the pause; it is noted in the verdict instead.
  let journalFailed = false;
  const note = async (event) => { try { await record(event); } catch { journalFailed = true; } };
  const baseline = await admitBaseline({ client, expected, settings, now, note, soak });

  const samples = [];
  const marks = {};
  let maybeStop = null;
  // Clients without a log route (older test fakes) skip the break diagnostics.
  const watcher = typeof client.logs === "function" ? breakWatcher({ readLogs: () => client.logs(), keepLogs, note }) : null;
  try {
    requireCondition(!isInterrupted(), "soak_interrupted_before_resume");
    const resumedAt = now();
    await client.command("resume");
    await note({ event: "resumed", atMs: resumedAt });
    const phases = soak.kind === "lease" ? leasePhases({ soak, samples, resumedAt, marks, note, leaseResult })
      : soak.kind === "handover" ? handoverPhases({ soak, samples, resumedAt, note }) : localPhases({ soak, samples, resumedAt });
    maybeStop = await sampleLoop({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted, soak, phases, watcher });
  } catch {
    maybeStop ??= "soak_run_error";
  }
  // An outage that never recovered still gets its log, before the pause; this never throws.
  if (maybeStop !== null) await watcher?.stopped(now());
  if (maybeStop !== null) await note({ event: "live_stop", code: maybeStop });
  // The stopping check journals and judges every sample of the pause.
  const maybeStopSamples = soak.kind === "stopping" ? [] : null;
  const maybeFinal = await pauseAndConfirm({ client, now, sleep, note, maybeStopSamples, limits: limitsOf(soak) });
  const judged = { baseline, samples, maybeFinal, soak, maybeStop: maybeStop ?? (journalFailed ? "soak_journal_failed" : null) };
  const verdict = soak.kind === "lease" ? judgeLease({ ...judged, marks }) : soak.kind === "handover" ? judgeHandover(judged) : judge(judged);
  if (maybeStopSamples !== null) {
    const stopPhase = judgeStopPhase(maybeStopSamples, { requireHashrateZeroed: soak.requireHashrateZeroed === true });
    // After a live stop the pause starts from another state, so only its metrics are kept.
    if (judged.maybeStop === null) {
      stopPhase.failures.filter((code) => !verdict.failures.includes(code)).forEach((code) => verdict.failures.push(code));
    }
    Object.assign(verdict.metrics, stopPhase.metrics);
    if (soak.requireWatchdogFresh === true) {
      // Preparation, mining and the stop: every sample from the resume to the final one.
      const notFresh = watchdogNotFresh([...samples, ...maybeStopSamples]);
      Object.assign(verdict.metrics, { watchdog_not_fresh_samples: notFresh.count, watchdog_not_fresh_reasons: notFresh.reasons,
        watchdog_not_fresh_subphases: notFresh.subphases });
      if (notFresh.count > 0 && !verdict.failures.includes("watchdog_not_fresh")) verdict.failures.push("watchdog_not_fresh");
    }
    verdict.result = verdict.failures.length === 0 ? "passed" : "failed";
  }
  await note({ event: "verdict", result: verdict.result, failures: verdict.failures });
  return verdict;
}

/**
 * Admits the run's paused baseline and saves the owner's pool, then the soak's `profile` if it has
 * one; refuses before any write unless the device runs the expected firmware on the handed-off boot,
 * paused, with the ASIC shown powered down and no `overheat_mode` left from an earlier trip
 * (`soak_baseline_overheat`), and before the resume unless each setting reads back.
 * With `requireProfile`, the overheat check's pinned baseline pair, it also refuses before any write
 * unless the baseline reads that frequency and core voltage (`overheat_baseline_profile`).
 */
export async function admitBaseline({ client, expected, settings, now, note, soak = null, requireProfile = null }) {
  const baselineInfo = await client.info();
  // A soak may require the firmware's board report (ADR-0041) to name its board.
  if (soak?.requireStoredBoard) {
    requireCondition(baselineInfo.storedBoard === soak.requireStoredBoard && baselineInfo.boardIdentity === "match",
      "soak_board_identity");
  }
  // Crash tripwire: a boot after a panic means a dump waits in the core-dump partition. Refuse before
  // any write, so the operator reads it first (task-handover-panic-diagnosis) instead of losing the
  // boot's evidence to later runs.
  requireCondition(baselineInfo.resetReasonCategory !== "panic", "soak_previous_panic");
  requireCondition(baselineInfo.sourceCommit === expected.firmwareCommit && baselineInfo.appElfSha256 === expected.appElfSha256,
    "soak_firmware_identity");
  requireCondition(baselineInfo.bootOrdinal === expected.bootOrdinal, "soak_boot_changed");
  const baseline = sampleFromInfo(baselineInfo, now());
  requireCondition(baseline.miningPaused && !mining(baseline) && !baseline.startMiningOnBoot && baseline.pauseReason === null,
    "soak_baseline_not_paused");
  // The stopped proof must be reachable before anything starts, or no run could end judged.
  requireCondition(hardwareStoppedUnder(limitsOf(soak))(baseline), "soak_baseline_not_stopped");
  // An earlier trip's `overheat_mode` stays saved until the firmware's next run has prepared or a
  // settings write clears it (the recovery `pause` does, from the tripped run's baseline). Over it,
  // the firmware's recovery may still be cooling and would save its lowered pair over a soak's
  // profile, and the run would stop on its first sample as a live overheat.
  requireCondition(!baseline.overheatMode, "soak_baseline_overheat");
  requireCondition(requireProfile === null || (baseline.frequencyMhz === requireProfile.frequencyMhz &&
    baseline.coreVoltageMv === requireProfile.coreVoltageMv), "overheat_baseline_profile");
  await note({ event: "baseline", sample: baseline });
  await client.patch(settings);
  requireCondition(poolSaved(await client.info(), settings), "soak_pool_not_saved");
  await note({ event: "pool_saved" });
  if (soak?.profile) await saveProfile({ client, profile: soak.profile, now, note });
  return baseline;
}

/** Saves the soak's frequency and core voltage and confirms the device reports both. */
async function saveProfile({ client, profile, now, note }) {
  await client.patch({ frequency: profile.frequencyMhz, coreVoltage: profile.coreVoltageMv });
  const saved = sampleFromInfo(await client.info(), now());
  requireCondition(saved.frequencyMhz === profile.frequencyMhz && saved.coreVoltageMv === profile.coreVoltageMv,
    "soak_profile_not_saved");
  await note({ event: "profile_saved", frequencyMhz: profile.frequencyMhz, coreVoltageMv: profile.coreVoltageMv });
}

/** A plain soak's phases: mine until its mining time, continuous or, if tolerant, accumulated. */
export function localPhases({ soak, samples, resumedAt }) {
  const limits = limitsOf(soak);
  const hashing = hashingUnder(limits);
  return {
    outageSamples: () => samples,
    onSample(sample) {
      if (!samples.some(hashing) && sample.atMs - resumedAt > START_ALLOWANCE_MS) return "soak_mining_not_started";
      if (soak.maybeMaximumOutageMs === null) return activeSpanMs(samples, limits) >= soak.activeMs ? null : undefined;
      const log = miningLog(samples, limits);
      if (log.maybeLastMiningAtMs !== null && sample.atMs - log.maybeLastMiningAtMs > soak.maybeMaximumOutageMs) {
        return "soak_outage_unrecovered";
      }
      return log.activeMs >= soak.activeMs ? null : undefined;
    },
  };
}

/**
 * Samples until the phases finish (null) or a stop code. `phases.onSample` returns null when done,
 * a stop code, or undefined to go on; `phases.outageSamples` names the samples whose outage counts
 * while the device cannot be reached. `stopRule` also receives the soak's limits.
 */
export async function sampleLoop({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted, soak, phases, watcher,
  stopRule = liveStop }) {
  const limits = limitsOf(soak);
  let maybeLastReachedAt = resumedAt;
  // A tolerant soak waits out a recovering outage up to its bound; a strict one stops sooner.
  const unreachableStopMs = soak.unreachableStopMs ?? soak.maybeMaximumOutageMs ?? UNREACHABLE_STOP_MS;
  for (;;) {
    await sleep(SAMPLE_INTERVAL_MS);
    if (isInterrupted()) return "soak_interrupted";
    if (now() - resumedAt > runDeadlineMs(soak)) return "soak_deadline";
    let info;
    try { info = await client.info(); } catch {
      watcher?.unreachable(samples.at(-1));
      if (now() - maybeLastReachedAt > unreachableStopMs) return "soak_device_unreachable";
      // An outage keeps counting while the device cannot be reached.
      const maybeLastMiningAtMs = miningLog(phases.outageSamples(), limits).maybeLastMiningAtMs;
      if (soak.maybeMaximumOutageMs !== null && maybeLastMiningAtMs !== null &&
        now() - maybeLastMiningAtMs > soak.maybeMaximumOutageMs) return "soak_outage_unrecovered";
      continue;
    }
    maybeLastReachedAt = now();
    const sample = sampleFromInfo(info, maybeLastReachedAt);
    const maybePrevious = samples.at(-1) ?? null;
    samples.push(sample);
    await note({ event: "sample", sample });
    const maybeStop = stopRule(sample, baseline, maybePrevious, limits);
    if (maybeStop !== null) return maybeStop;
    await watcher?.observe(sample, hashingUnder(limits)(sample), phases.holdsBreakLog?.() === true);
    // A log read can take seconds; it never counts against the unreachable bound.
    maybeLastReachedAt = Math.max(maybeLastReachedAt, now());
    const outcome = await phases.onSample(sample);
    if (outcome !== undefined) return outcome;
  }
}

/**
 * Pauses, then waits for the ordered safe-stop: two samples in a row with the ASIC shown powered
 * down. Never throws; the pause is retried while the allowance lasts. With `maybeStopSamples`,
 * every sample from the first pause attempt on is journaled and kept there: a request whose
 * response is lost may still have paused the device. `limits` set the idle power that shows it.
 */
export async function pauseAndConfirm({ client, now, sleep, note, maybeStopSamples = null, limits = LIMITS }) {
  const hardwareStopped = hardwareStoppedUnder(limits);
  const pausedAt = now();
  let pauseSent = false, maybeLastPauseAt = null, maybeStopped = null;
  while (now() - pausedAt <= STOP_ALLOWANCE_MS) {
    // Until the ASIC shows powered down, the pause is sent again every 30 s; it is idempotent.
    if (maybeStopped === null && (maybeLastPauseAt === null || now() - maybeLastPauseAt >= PAUSE_RESEND_MS)) {
      maybeLastPauseAt = now();
      try { await client.command("pause"); pauseSent = true; await note({ event: "paused", atMs: now() }); }
      catch { await note({ event: "pause_failed", atMs: now() }); }
    }
    try {
      const sample = sampleFromInfo(await client.info(), now());
      if (maybeLastPauseAt !== null && maybeStopSamples !== null) { maybeStopSamples.push(sample); await note({ event: "stop_sample", sample }); }
      if (pauseSent && hardwareStopped(sample)) {
        if (maybeStopped !== null) { await note({ event: "final", sample }); return sample; }
        maybeStopped = sample;
        await note({ event: "stopped_sample", sample });
      } else {
        maybeStopped = null;
      }
    } catch { maybeStopped = null; }
    await sleep(SAMPLE_INTERVAL_MS);
  }
  await note({ event: "final_missing" });
  return undefined;
}
