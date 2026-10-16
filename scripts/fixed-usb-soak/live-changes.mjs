// Live settings changes during a soak's lease (firmware ADR-0037 decision 4). The soak steps the
// device down to the conservative pair and back while it mines, then judges from the observer journal
// that the saved settings, the measured core voltage and the counter hashrate followed. Ending on the
// starting pair keeps the settings fingerprint equal to the page's baseline.

/** Opted into at preflight with `--live-changes`; both pairs already have hardware evidence. */
export const SOAK_LIVE_CHANGES = Object.freeze([
  Object.freeze({ atActiveMs: 200000, settings: Object.freeze({ frequency: 400, coreVoltage: 1100 }) }),
  Object.freeze({ atActiveMs: 400000, settings: Object.freeze({ frequency: 485, coreVoltage: 1200 }) }),
]);
/** Samples this long after a change's read-back count toward its segment; the owner steps once a second. */
export const LIVE_SETTLE_MS = 20000;
/**
 * Steps are judged relative to the first segment, so a fixed ADC offset cannot fail them: the median
 * measured core voltage must move by the settings' step, and the median counter hashrate (BM1366 hash
 * counters, proportional to frequency) by the frequency ratio.
 */
export const LIVE_VOLTAGE_STEP_TOLERANCE_MV = 30;
export const LIVE_HASH_RATIO_TOLERANCE = 0.12;
export const LIVE_MINIMUM_SAMPLES = 5;

/**
 * The index of the next planned change once the work gate has run long enough, or undefined. Changes
 * are only sent while the soak mines unrevoked with its work gate open, one at a time and in order.
 */
export function dueLiveChange(planned, sent, state) {
  const maybeNext = planned[sent];
  const q = state?.qualification;
  if (!maybeNext || !state.running || !q?.soak || q.revocation_reason !== "none") return undefined;
  if (!(Number.isSafeInteger(q.work_gate_remaining_ms) && q.work_gate_remaining_ms > 0)) return undefined;
  return Number.isFinite(q.active_ms) && q.active_ms >= maybeNext.atActiveMs ? sent : undefined;
}

function median(values) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function segmentSummary(target, samples) {
  const measured = samples.map((sample) => sample.maybeCoreVoltageActualMv).filter(Number.isFinite);
  const hashRates = samples.map((sample) => sample.hashRateGhs).filter((value) => Number.isFinite(value) && value > 0);
  return {
    target: { frequency: target.frequency, coreVoltage: target.coreVoltage },
    samples: samples.length,
    settings_match: samples.length > 0 && samples.every((sample) => sample.frequencyMhz === target.frequency && sample.coreVoltageMv === target.coreVoltage),
    measured_samples: measured.length,
    measured_core_mv: measured.length ? { min: Math.min(...measured), median: median(measured), max: Math.max(...measured) } : null,
    hash_rate_samples: hashRates.length,
    median_hash_rate_ghs: median(hashRates),
    median_power_watts: median(samples.map((sample) => sample.powerWatts).filter(Number.isFinite)),
  };
}

/** Observation only: from a change's read-back to the first sample measuring within tolerance of its expected voltage. */
function settleMs(mining, record, expectedMv) {
  const maybeSettled = mining.find((line) => line.hostUnixMs >= record.completed_at_unix_ms &&
    Math.abs(line.sample.maybeCoreVoltageActualMv - expectedMv) <= LIVE_VOLTAGE_STEP_TOLERANCE_MV);
  return maybeSettled ? maybeSettled.hostUnixMs - record.completed_at_unix_ms : null;
}

/** Each later segment against the first: the voltage step and the hashrate ratio its settings imply. */
function stepFailures(first, segment) {
  const failures = [];
  const expectedStep = segment.target.coreVoltage - first.target.coreVoltage;
  const step = segment.measured_core_mv.median - first.measured_core_mv.median;
  if (Math.abs(step - expectedStep) > LIVE_VOLTAGE_STEP_TOLERANCE_MV) failures.push("live_change_voltage_unconfirmed");
  const expectedRatio = segment.target.frequency / first.target.frequency;
  const ratio = segment.median_hash_rate_ghs / first.median_hash_rate_ghs;
  if (!(Math.abs(ratio - expectedRatio) <= LIVE_HASH_RATIO_TOLERANCE)) failures.push("live_change_frequency_unconfirmed");
  return failures;
}

/**
 * Judge planned changes against their records and the HTTP journal. Segments run from a settled change
 * to the next request; the first starts at settled mining, and the last ends at the last open-gate
 * record, before the ordered shutdown lowers anything. Every segment needs the target settings on every
 * mining sample, and enough fresh voltage and hashrate readings; later segments must step as judged above.
 */
export function judgeLiveChanges({ initial, planned, recorded, lines, maybeGateOpenUntilUnixMs }) {
  if (planned.length === 0) return { failures: [], segments: [] };
  const indices = planned.map((_, index) => index);
  if (recorded.length !== planned.length || indices.some((index) => !recorded[index])) return { failures: ["live_change_missing"], segments: [] };
  if (indices.some((index) => recorded[index].failure || !Number.isFinite(recorded[index].completed_at_unix_ms))) {
    return { failures: ["live_change_failed"], segments: [] };
  }
  const mining = lines.filter((line) => line.event === "sample" && line.transport === "http" && line.sample?.miningActive);
  if (mining.length === 0 || !Number.isFinite(maybeGateOpenUntilUnixMs)) return { failures: ["live_change_unobserved"], segments: [] };
  const bounds = [{ target: initial, from: mining[0].hostUnixMs + LIVE_SETTLE_MS, to: recorded[0].requested_at_unix_ms },
    ...indices.map((index) => ({ target: planned[index].settings, from: recorded[index].completed_at_unix_ms + LIVE_SETTLE_MS,
      to: recorded[index + 1]?.requested_at_unix_ms ?? maybeGateOpenUntilUnixMs }))];
  const segments = bounds.map(({ target, from, to }) =>
    segmentSummary(target, mining.filter((line) => line.hostUnixMs >= from && line.hostUnixMs <= to).map((line) => line.sample)));
  const failures = [];
  for (const segment of segments) {
    if (segment.samples < LIVE_MINIMUM_SAMPLES || !segment.settings_match) failures.push("live_change_unobserved");
    if (segment.measured_samples < LIVE_MINIMUM_SAMPLES || segment.hash_rate_samples < LIVE_MINIMUM_SAMPLES) failures.push("live_change_readings_missing");
  }
  if (failures.length > 0) return { failures: [...new Set(failures)], segments };
  const [first, ...later] = segments;
  for (const segment of later) failures.push(...stepFailures(first, segment));
  indices.forEach((index) => {
    segments[index + 1].settle_ms = settleMs(mining, recorded[index],
      first.measured_core_mv.median + segments[index + 1].target.coreVoltage - first.target.coreVoltage);
  });
  return { failures: [...new Set(failures)], segments };
}
