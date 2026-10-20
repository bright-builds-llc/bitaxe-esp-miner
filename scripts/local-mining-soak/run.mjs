// One local mining soak against an admitted, paused Ultra 205 (ADR-0038 task 6): confirm the
// installed identity and the paused baseline, save the owner's pool, resume, observe until the
// soak's mining time (or, for a lease check, its phases), a live stop or the run deadline, then
// pause and confirm the ASIC powered down. From the resume on, every path pauses the device. The
// device, clock, journal and lease result are injected, so the sequence is tested without hardware.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { activeSpanMs, hardwareStopped, judge, liveStop, mining, miningLog, runDeadlineMs, sampleFromInfo,
  SAMPLE_INTERVAL_MS, SOAKS, START_ALLOWANCE_MS, STOP_ALLOWANCE_MS } from "./judge.mjs";
import { judgeLease, leasePhases } from "./lease.mjs";
import { handoverPhases, judgeHandover } from "./handover.mjs";
import { breakWatcher } from "./break-log.mjs";
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
  const baseline = await admitBaseline({ client, expected, settings, now, note });

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
  const maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
  const judged = { baseline, samples, maybeFinal, soak, maybeStop: maybeStop ?? (journalFailed ? "soak_journal_failed" : null) };
  const verdict = soak.kind === "lease" ? judgeLease({ ...judged, marks }) : soak.kind === "handover" ? judgeHandover(judged) : judge(judged);
  await note({ event: "verdict", result: verdict.result, failures: verdict.failures });
  return verdict;
}

/**
 * Admits the run's paused baseline and saves the owner's pool; refuses before any write unless the
 * device runs the expected firmware on the handed-off boot, paused, with the ASIC shown powered down.
 */
export async function admitBaseline({ client, expected, settings, now, note }) {
  const baselineInfo = await client.info();
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
  requireCondition(hardwareStopped(baseline), "soak_baseline_not_stopped");
  await note({ event: "baseline", sample: baseline });
  await client.patch(settings);
  requireCondition(poolSaved(await client.info(), settings), "soak_pool_not_saved");
  await note({ event: "pool_saved" });
  return baseline;
}

/** A plain soak's phases: mine until its mining time, continuous or, if tolerant, accumulated. */
export function localPhases({ soak, samples, resumedAt }) {
  return {
    outageSamples: () => samples,
    onSample(sample) {
      if (!samples.some(mining) && sample.atMs - resumedAt > START_ALLOWANCE_MS) return "soak_mining_not_started";
      if (soak.maybeMaximumOutageMs === null) return activeSpanMs(samples) >= soak.activeMs ? null : undefined;
      const log = miningLog(samples);
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
 * while the device cannot be reached.
 */
export async function sampleLoop({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted, soak, phases, watcher }) {
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
      const maybeLastMiningAtMs = miningLog(phases.outageSamples()).maybeLastMiningAtMs;
      if (soak.maybeMaximumOutageMs !== null && maybeLastMiningAtMs !== null &&
        now() - maybeLastMiningAtMs > soak.maybeMaximumOutageMs) return "soak_outage_unrecovered";
      continue;
    }
    maybeLastReachedAt = now();
    const sample = sampleFromInfo(info, maybeLastReachedAt);
    const maybePrevious = samples.at(-1) ?? null;
    samples.push(sample);
    await note({ event: "sample", sample });
    const maybeStop = liveStop(sample, baseline, maybePrevious);
    if (maybeStop !== null) return maybeStop;
    await watcher?.observe(sample, mining(sample));
    // A log read can take seconds; it never counts against the unreachable bound.
    maybeLastReachedAt = Math.max(maybeLastReachedAt, now());
    const outcome = await phases.onSample(sample);
    if (outcome !== undefined) return outcome;
  }
}

/**
 * Pauses, then waits for the ordered safe-stop: two samples in a row with the ASIC shown powered
 * down. Never throws; the pause is retried while the allowance lasts.
 */
export async function pauseAndConfirm({ client, now, sleep, note }) {
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
