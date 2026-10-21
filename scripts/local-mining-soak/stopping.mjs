// The stopping check (task-mining-activity-stopping): after a pause, the status must read
// `stopping` while the ordered safe-stop runs, never `active` while the ASIC idles. Before the
// fix it read `active` for 13-18 s at idle power, until the cooling proof confirmed the stop.
import { idlePower, MAXIMUM_SAMPLE_GAP_MS } from "./judge.mjs";

/** The only order a pause may show: still mining, then stopping, then paused. */
const RANK = Object.freeze({ active: 0, stopping: 1, paused: 2 });

/**
 * The stop phase's failures and metrics over every sample from the pause to the final one, in
 * order. The status must move forward only, show `stopping` at least once, and never read
 * `active` while fresh power shows the ASIC idle. With `requireHashrateZeroed` (stopping-002,
 * after the cooling proof stopped blocking the owner), every `stopping` sample after the first
 * must read a zero hashrate rather than the value frozen at the stop.
 */
export function judgeStopPhase(stopSamples, { requireHashrateZeroed = false } = {}) {
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (stopSamples.length === 0) fail("stop_samples_missing");
  let maybeRank = null, maximumGapMs = 0;
  stopSamples.forEach((sample, index) => {
    if (!Object.hasOwn(RANK, sample.miningActivity)) { fail("stop_unexpected_status"); return; }
    const rank = RANK[sample.miningActivity];
    if (maybeRank !== null && rank < maybeRank) fail("stop_status_regressed");
    maybeRank = Math.max(maybeRank ?? rank, rank);
    if (sample.miningActivity === "active" && idlePower(sample)) fail("stop_active_while_idle");
    if (index > 0) maximumGapMs = Math.max(maximumGapMs, sample.atMs - stopSamples[index - 1].atMs);
  });
  // A longer gap could hide an `active` reading at idle power.
  if (maximumGapMs > MAXIMUM_SAMPLE_GAP_MS) fail("stop_sample_gap");
  const stopping = stopSamples.filter((sample) => sample.miningActivity === "stopping");
  if (stopping.length === 0) fail("stop_stopping_not_seen");
  // The first `stopping` sample may precede the owner's next hashrate tick.
  const frozen = stopping.slice(1).filter((sample) => sample.hashRate !== 0).length;
  if (requireHashrateZeroed && frozen > 0) fail("stop_hashrate_frozen");
  const firstAt = stopSamples[0]?.atMs ?? null;
  const maybeFirstPaused = stopSamples.find((sample) => sample.miningActivity === "paused");
  return {
    failures,
    metrics: {
      stop_samples: stopSamples.length,
      stop_active_samples: stopSamples.filter((sample) => sample.miningActivity === "active").length,
      stop_active_idle_samples: stopSamples.filter((sample) => sample.miningActivity === "active" && idlePower(sample)).length,
      stop_stopping_samples: stopping.length,
      stop_stopping_idle_samples: stopping.filter(idlePower).length,
      stop_first_stopping_ms: stopping.length === 0 ? null : stopping[0].atMs - firstAt,
      stop_first_paused_ms: maybeFirstPaused === undefined ? null : maybeFirstPaused.atMs - firstAt,
      stop_maximum_sample_gap_ms: maximumGapMs,
      stop_hashrate_frozen_samples: frozen,
    },
  };
}
