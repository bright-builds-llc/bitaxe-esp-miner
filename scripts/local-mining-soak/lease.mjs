// The lease interruption check (ADR-0038 task 6): local mining, then a Gate lease that interrupts
// it, then local mining restored. Over HTTP a lease's mining looks like local mining, so the phases
// follow the order of events. The check mines locally first (`before`). It then waits while a Gate
// Connect is refused and stops local mining, and the lease runs (`interrupt`), until the Gate soak's
// result exists. It requires local mining back within a bound (`restore`), then mines locally
// again (`after`). Outages count in the local phases only. Every check runs under the soak's limits
// (`limitsOf`): the Ultra 205's, or the Gamma 601's for its lease check.
import { finalFailures, hashingUnder, increases, limitMetrics, limitsOf, liveStop, mining, miningLog, START_ALLOWANCE_MS,
  statusOnlySamples, wifiMetrics } from "./judge.mjs";

/** The lease check's phases for `sampleLoop`; `marks` records when each phase began. */
export function leasePhases({ soak, samples, resumedAt, marks, note, leaseResult }) {
  const limits = limitsOf(soak);
  const hashing = hashingUnder(limits);
  let phase = "before", phaseStart = 0;
  const outage = (current, sample) => {
    const { maybeLastMiningAtMs } = miningLog(current, limits);
    return maybeLastMiningAtMs !== null && sample.atMs - maybeLastMiningAtMs > soak.maybeMaximumOutageMs;
  };
  const begin = async (next, event, atMs, detail = {}) => {
    phase = next; phaseStart = samples.length;
    await note({ event, atMs, ...detail });
  };
  const steps = {
    async before(sample) {
      const current = samples.slice(phaseStart);
      if (!current.some(hashing) && sample.atMs - resumedAt > START_ALLOWANCE_MS) return "soak_mining_not_started";
      if (outage(current, sample)) return "soak_outage_unrecovered";
      if (miningLog(current, limits).activeMs < soak.activeMs) return undefined;
      marks.leaseReadyAtMs = sample.atMs;
      await begin("interrupt", "lease_ready", sample.atMs);
      return undefined;
    },
    async interrupt(sample) {
      if (marks.localStoppedAtMs === undefined && !hashing(sample)) {
        marks.localStoppedAtMs = sample.atMs;
        await note({ event: "local_stopped", atMs: sample.atMs });
      }
      const maybeLease = await leaseResult();
      if (maybeLease === null) return undefined;
      marks.leaseEndedAtMs = sample.atMs;
      marks.lease = maybeLease;
      await begin("restore", "lease_ended", sample.atMs, { result: maybeLease.result, failures: maybeLease.failures });
      return marks.localStoppedAtMs === undefined ? "lease_check_never_interrupted" : undefined;
    },
    async restore(sample) {
      if (!hashing(sample)) return sample.atMs - marks.leaseEndedAtMs > soak.restoreBoundMs ? "lease_check_not_restored" : undefined;
      marks.localRestoredAtMs = sample.atMs;
      await begin("after", "local_restored", sample.atMs);
      // The restored sample opens the after phase, so its mining counts.
      phaseStart = samples.length - 1;
      return undefined;
    },
    async after(sample) {
      const current = samples.slice(phaseStart);
      if (outage(current, sample)) return "soak_outage_unrecovered";
      return miningLog(current, limits).activeMs >= soak.afterActiveMs ? null : undefined;
    },
  };
  // Outages count only while local mining should run.
  return {
    outageSamples: () => (phase === "before" || phase === "after" ? samples.slice(phaseStart) : []),
    onSample: (sample) => steps[phase](sample),
    // From `lease_ready` until the Gate soak's result, the device's web server is the Gate soak's to
    // judge, so no retained-log read may stall it. Starting at `lease_ready`, not at the local stop,
    // also holds a break the device was unreachable across.
    holdsBreakLog: () => marks.leaseReadyAtMs !== undefined && marks.leaseEndedAtMs === undefined,
  };
}

/** The samples from `fromMs` up to and including `maybeThroughMs`, as each phase saw them. */
const between = (samples, fromMs, maybeThroughMs) =>
  samples.filter((sample) => sample.atMs >= fromMs && (maybeThroughMs === undefined || sample.atMs <= maybeThroughMs));

/** The lease check's verdict from its samples and phase marks. */
export function judgeLease({ baseline, samples, maybeFinal, maybeStop, soak, marks }) {
  const limits = limitsOf(soak);
  const hashing = hashingUnder(limits);
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  samples.forEach((sample, index) => { const stop = liveStop(sample, baseline, samples[index - 1] ?? null, limits); if (stop) fail(stop); });
  const beforeSamples = between(samples, -Infinity, marks.leaseReadyAtMs);
  const afterSamples = marks.localRestoredAtMs === undefined ? [] : between(samples, marks.localRestoredAtMs);
  const before = miningLog(beforeSamples, limits);
  const maybeAfter = marks.localRestoredAtMs === undefined ? null : miningLog(afterSamples, limits);
  if (before.activeMs < soak.activeMs) fail("lease_check_before_short");
  if (marks.localStoppedAtMs === undefined) fail("lease_check_never_interrupted");
  if (marks.lease === undefined) fail("lease_check_lease_unended");
  else if (marks.lease.result !== "passed") fail("lease_check_lease_failed");
  const maybeRestoreMs = marks.localRestoredAtMs === undefined ? null : marks.localRestoredAtMs - marks.leaseEndedAtMs;
  if (maybeRestoreMs === null || maybeRestoreMs > soak.restoreBoundMs) fail("lease_check_not_restored");
  if ((maybeAfter?.activeMs ?? 0) < soak.afterActiveMs) fail("lease_check_after_short");
  const outagesMs = [...before.outagesMs, ...(maybeAfter?.outagesMs ?? [])];
  if (outagesMs.some((outageMs) => outageMs > soak.maybeMaximumOutageMs)) fail("soak_outage_unrecovered");
  // Over HTTP the lease's mining reads like local mining, so the hashrate and shares that prove
  // local mining come from the local phases only; the Gate soak judges the lease's own.
  const local = [...beforeSamples, ...afterSamples];
  const active = samples.filter(hashing);
  if (soak.requireHashrate && !local.some((sample) => hashing(sample) && sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  const accepted = increases(beforeSamples, "sharesAccepted") + increases(afterSamples, "sharesAccepted");
  if (accepted < 1) fail("soak_no_accepted_share");
  finalFailures(baseline, maybeFinal, limits).forEach(fail);
  const since = (atMs) => (atMs === undefined || marks.leaseReadyAtMs === undefined ? null : atMs - marks.leaseReadyAtMs);
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      before_active_ms: before.activeMs,
      after_active_ms: maybeAfter?.activeMs ?? 0,
      local_stopped_after_ready_ms: since(marks.localStoppedAtMs),
      lease_ended_after_ready_ms: since(marks.leaseEndedAtMs),
      restored_after_lease_ms: maybeRestoreMs,
      lease_result: marks.lease?.result ?? null,
      lease_failures: marks.lease?.failures ?? null,
      outage_count: outagesMs.length,
      maximum_outage_ms: Math.max(0, ...outagesMs),
      ...wifiMetrics(samples),
      active_samples: active.length,
      status_only_samples: statusOnlySamples(samples, limits),
      samples: samples.length,
      local_shares_accepted: accepted,
      local_shares_rejected: increases(beforeSamples, "sharesRejected") + increases(afterSamples, "sharesRejected"),
      // The maxima cover every sample the live limits judged.
      ...limitMetrics(samples.filter(mining)),
    },
  };
}
