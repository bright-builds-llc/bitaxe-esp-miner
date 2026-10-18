// One local mining soak against an admitted, paused Ultra 205 (ADR-0038 task 6): confirm the
// installed identity and the paused baseline, save the owner's pool, resume, observe until 30
// minutes of continuous mining, a live stop or the run deadline, then pause and confirm the ASIC
// powered down. From the resume on, every path pauses the device. The device, clock and journal
// are injected, so the sequence is tested without hardware.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { activeSpanMs, hardwareStopped, judge, liveStop, mining, RUN_DEADLINE_MS, sampleFromInfo, SAMPLE_INTERVAL_MS,
  SOAK_ACTIVE_MS, START_ALLOWANCE_MS, STOP_ALLOWANCE_MS } from "./judge.mjs";
import { poolSaved } from "./pool.mjs";

/** The device stays unreachable this long before the soak stops. */
export const UNREACHABLE_STOP_MS = 30_000;
const PAUSE_RESEND_MS = 30_000;

/** Runs the soak; `expected` names the installed firmware and the handed-off boot. */
export async function runSoak({ client, expected, settings, now, sleep, record, isInterrupted = () => false }) {
  // A journal failure must never stop the pause; it is noted in the verdict instead.
  let journalFailed = false;
  const note = async (event) => { try { await record(event); } catch { journalFailed = true; } };

  const baselineInfo = await client.info();
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

  const samples = [];
  let maybeStop = null;
  try {
    requireCondition(!isInterrupted(), "soak_interrupted_before_resume");
    const resumedAt = now();
    await client.command("resume");
    await note({ event: "resumed", atMs: resumedAt });
    maybeStop = await observe({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted });
  } catch {
    maybeStop ??= "soak_run_error";
  }
  if (maybeStop !== null) await note({ event: "live_stop", code: maybeStop });
  const maybeFinal = await pauseAndConfirm({ client, now, sleep, note });
  const verdict = judge({ baseline, samples, maybeFinal, maybeStop: maybeStop ?? (journalFailed ? "soak_journal_failed" : null) });
  await note({ event: "verdict", result: verdict.result, failures: verdict.failures });
  return verdict;
}

/** Samples until 30 minutes of continuous mining (null) or a stop code. */
async function observe({ client, baseline, samples, resumedAt, now, sleep, note, isInterrupted }) {
  let maybeLastReachedAt = resumedAt;
  for (;;) {
    await sleep(SAMPLE_INTERVAL_MS);
    if (isInterrupted()) return "soak_interrupted";
    if (now() - resumedAt > RUN_DEADLINE_MS) return "soak_deadline";
    let info;
    try { info = await client.info(); } catch {
      if (now() - maybeLastReachedAt > UNREACHABLE_STOP_MS) return "soak_device_unreachable";
      continue;
    }
    maybeLastReachedAt = now();
    const sample = sampleFromInfo(info, maybeLastReachedAt);
    const maybePrevious = samples.at(-1) ?? null;
    samples.push(sample);
    await note({ event: "sample", sample });
    const maybeStop = liveStop(sample, baseline, maybePrevious);
    if (maybeStop !== null) return maybeStop;
    if (!samples.some(mining) && sample.atMs - resumedAt > START_ALLOWANCE_MS) return "soak_mining_not_started";
    if (activeSpanMs(samples) >= SOAK_ACTIVE_MS) return null;
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
      } else {
        maybeStopped = null;
      }
    } catch { maybeStopped = null; }
    await sleep(SAMPLE_INTERVAL_MS);
  }
  await note({ event: "final_missing" });
  return undefined;
}
