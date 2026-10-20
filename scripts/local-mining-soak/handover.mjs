// The handover loop (lease interruption check attempt 002's panic): local mining, repeatedly
// interrupted by a Gate Connect that is refused, an admitted Connect that idles with status polls,
// and a closed page, after which local mining comes back. No lease is signed. Each cycle is one break
// in mining that ends with mining again; the loop passes after its cycle count with no reset.
import { finalFailures, hashing, increases, limitMetrics, liveStop, mining, START_ALLOWANCE_MS, statusOnlySamples, wifiMetrics } from "./judge.mjs";

/** The breaks in mining, each from the last hashing sample before it to the first one after. */
export function handoverBreaks(samples) {
  const breaks = [];
  let maybeLastMining = null, broken = false;
  for (const sample of samples) {
    if (!hashing(sample)) { if (maybeLastMining !== null) broken = true; continue; }
    if (maybeLastMining !== null && broken) breaks.push(sample.atMs - maybeLastMining.atMs);
    maybeLastMining = sample; broken = false;
  }
  return breaks;
}

/** The loop's phases for `sampleLoop`: mine, count recovered breaks, and stop on an unrecovered one. */
export function handoverPhases({ soak, samples, resumedAt, note }) {
  let counted = 0;
  return {
    // A handover idles for minutes by design; only a break past its bound stops the loop.
    outageSamples: () => [],
    async onSample(sample) {
      if (!samples.some(hashing) && sample.atMs - resumedAt > START_ALLOWANCE_MS) return "soak_mining_not_started";
      const breaks = handoverBreaks(samples);
      if (breaks.length > counted) {
        counted = breaks.length;
        await note({ event: "handover_cycle", cycle: counted, breakMs: breaks.at(-1), atMs: sample.atMs });
      }
      const maybeLastMining = samples.findLast(hashing);
      if (maybeLastMining !== undefined && sample.atMs - maybeLastMining.atMs > soak.breakBoundMs) return "handover_not_restored";
      return counted >= soak.cycles ? null : undefined;
    },
  };
}

/** The loop's verdict: its cycles, every live limit, a nonzero hashrate and shares, and the stop. */
export function judgeHandover({ baseline, samples, maybeFinal, maybeStop, soak }) {
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  samples.forEach((sample, index) => { const stop = liveStop(sample, baseline, samples[index - 1] ?? null); if (stop) fail(stop); });
  const breaks = handoverBreaks(samples);
  if (breaks.length < soak.cycles) fail("handover_cycles_short");
  const active = samples.filter(hashing);
  if (soak.requireHashrate && !active.some((sample) => sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  const accepted = increases(samples, "sharesAccepted");
  if (accepted < 1) fail("soak_no_accepted_share");
  finalFailures(baseline, maybeFinal).forEach(fail);
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      cycles: breaks.length,
      minimum_break_ms: breaks.length === 0 ? null : Math.min(...breaks),
      maximum_break_ms: breaks.length === 0 ? null : Math.max(...breaks),
      ...wifiMetrics(samples),
      active_samples: active.length,
      status_only_samples: statusOnlySamples(samples),
      samples: samples.length,
      shares_accepted: accepted,
      shares_rejected: increases(samples, "sharesRejected"),
      // The maxima cover every sample the live limits judged.
      ...limitMetrics(samples.filter(mining)),
    },
  };
}
