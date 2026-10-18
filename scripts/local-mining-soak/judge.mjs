// The local mining soak's pure judge (ADR-0038 task 6): samples of /api/system/info, the
// live stop rule applied to each, and the verdict over the whole run. Samples carry only
// metadata; no endpoint, pool or credential value ever enters them.

/**
 * The soaks a contract may run, each with its own enable line (owner decisions 2026-10-17):
 * soak 001 mines 30 minutes; soak 002 mines 2 hours and must show a nonzero hashrate, which
 * verifies the fix soak 001 led to.
 */
export const SOAKS = Object.freeze({
  "001": Object.freeze({ id: "001", activeMs: 30 * 60_000, requireHashrate: false }),
  "002": Object.freeze({ id: "002", activeMs: 120 * 60_000, requireHashrate: true }),
});
export const SAMPLE_INTERVAL_MS = 2000;
/** A longer gap between successive samples breaks continuity. */
export const MAXIMUM_SAMPLE_GAP_MS = 10_000;
/** Probe, preparation and the cooling proof may take this long before mining is seen. */
export const START_ALLOWANCE_MS = 5 * 60_000;
/** The ordered safe-stop, including its up-to-120-s cooling proof, must finish within this. */
export const STOP_ALLOWANCE_MS = 180_000;
/** The run never mines past this, measured from the resume: start, the soak's time and slack. */
export const runDeadlineMs = (soak) => START_ALLOWANCE_MS + soak.activeMs + 5 * 60_000;
/** Input power at or below this, freshly measured, shows the ASIC powered down. */
export const IDLE_POWER_W = 5;
/** Live stop limits, as for the lease soaks: input 4.5–5.5 V, at most 15 W, ASIC below 75 C. */
export const LIMITS = Object.freeze({ minimumInputMv: 4500, maximumInputMv: 5500, maximumPowerW: 15, maximumChipTempC: 75 });

const number = (value) => (typeof value === "number" && Number.isFinite(value) ? value : null);

/** The metadata one sample keeps from `/api/system/info` at host time `atMs`. */
export function sampleFromInfo(info, atMs) {
  return {
    atMs,
    bootSession: typeof info?.bootSession === "string" ? info.bootSession : null,
    uptimeSeconds: number(info?.uptimeSeconds),
    resetReasonCategory: typeof info?.resetReasonCategory === "string" ? info.resetReasonCategory : null,
    miningPaused: info?.miningPaused === true,
    miningActivity: typeof info?.miningActivity === "string" ? info.miningActivity : null,
    startMiningOnBoot: info?.startMiningOnBoot === true,
    hashRate: number(info?.hashRate),
    sharesAccepted: number(info?.sharesAccepted),
    sharesRejected: number(info?.sharesRejected),
    chipTempC: number(info?.temp),
    vrTempC: number(info?.vrTemp),
    inputMv: number(info?.voltage),
    powerW: number(info?.power),
    fanRpm: number(info?.fanrpm),
    frequencyMhz: number(info?.frequency),
    coreVoltageMv: number(info?.coreVoltage),
    overheatMode: number(info?.overheat_mode) === 1,
    pauseReason: typeof info?.pauseReason === "string" ? info.pauseReason : null,
    hardwareFault: typeof info?.hardware_fault === "string",
    powerFault: typeof info?.power_fault === "string",
    fresh: {
      power: info?.powerStatus?.state === "fresh",
      input: info?.voltageStatus?.state === "fresh",
      chipTemp: info?.chipTempStatus?.state === "fresh",
      fan: info?.fanRpmStatus?.state === "fresh",
    },
  };
}

/** Whether every reading the limits use is fresh. */
export const readingsFresh = (sample) => Object.values(sample.fresh).every(Boolean);

/** Whether the ASIC is shown powered down: paused, not mining, and freshly measured idle power. */
export const hardwareStopped = (sample) =>
  sample.miningPaused && !mining(sample) && sample.fresh.power && sample.powerW !== null && sample.powerW <= IDLE_POWER_W;

/** Whether the device reports active mining in this sample. */
export const mining = (sample) => sample.miningActivity === "active" && !sample.miningPaused;

/**
 * The first live stop this sample trips against the run's baseline, or null. A reset, a device
 * pause, a fault or an overheat always stops. While the device mines, the electrical and thermal
 * limits apply, as the firmware's own protection does, and readings stale on two samples in a row
 * stop the run (`maybePrevious` is the sample before).
 */
export function liveStop(sample, baseline, maybePrevious = null) {
  if (sample.bootSession !== baseline.bootSession) return "soak_device_reset";
  if (sample.pauseReason !== null) return "soak_device_paused_itself";
  if (sample.hardwareFault) return "soak_hardware_fault";
  if (sample.powerFault) return "soak_power_fault";
  if (sample.overheatMode) return "soak_overheat";
  if (!mining(sample)) return null;
  if (!readingsFresh(sample) && maybePrevious !== null && mining(maybePrevious) && !readingsFresh(maybePrevious)) {
    return "soak_readings_stale";
  }
  // A stale reading reads as 0, so each limit judges only a fresh value; the rule above bounds staleness.
  const { fresh } = sample;
  if (fresh.input && (sample.inputMv === null || sample.inputMv < LIMITS.minimumInputMv || sample.inputMv > LIMITS.maximumInputMv)) {
    return "soak_input_voltage";
  }
  if (fresh.power && (sample.powerW === null || sample.powerW > LIMITS.maximumPowerW)) return "soak_power";
  if (fresh.chipTemp && (sample.chipTempC === null || sample.chipTempC >= LIMITS.maximumChipTempC)) return "soak_chip_temperature";
  if (fresh.fan && (sample.fanRpm === null || sample.fanRpm <= 0)) return "soak_fan_stopped";
  return null;
}

/** Continuous active mining measured from the first active sample, broken by any gap or idle sample. */
export function activeSpanMs(samples) {
  let longest = 0, maybeStart = null, maybeLast = null;
  for (const sample of samples) {
    const continuous = maybeLast !== null && sample.atMs - maybeLast.atMs <= MAXIMUM_SAMPLE_GAP_MS;
    if (mining(sample)) {
      if (maybeStart === null || !continuous || !mining(maybeLast)) maybeStart = sample.atMs;
      longest = Math.max(longest, sample.atMs - maybeStart);
    } else {
      maybeStart = null;
    }
    maybeLast = sample;
  }
  return longest;
}

/** The counter's increases across samples; a pool reconnect resets it to zero, which is no loss. */
export function increases(samples, key) {
  let total = 0, maybeLast = null;
  for (const sample of samples) {
    const value = sample[key];
    if (value === null) continue;
    if (maybeLast !== null && value > maybeLast) total += value - maybeLast;
    maybeLast = value;
  }
  return total;
}

/** The verdict over a run: its baseline, the samples while mining, and the final paused sample. */
export function judge({ baseline, samples, maybeFinal, maybeStop, soak = SOAKS["001"] }) {
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  samples.forEach((sample, index) => { const stop = liveStop(sample, baseline, samples[index - 1] ?? null); if (stop) fail(stop); });
  const active = samples.filter(mining);
  const span = activeSpanMs(samples);
  if (span < soak.activeMs) fail("soak_active_time_short");
  if (soak.requireHashrate && !active.some((sample) => sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  let maximumGapMs = 0;
  for (let index = 1; index < samples.length; index += 1) maximumGapMs = Math.max(maximumGapMs, samples[index].atMs - samples[index - 1].atMs);
  if (maximumGapMs > MAXIMUM_SAMPLE_GAP_MS) fail("soak_sample_gap");
  const accepted = increases(samples, "sharesAccepted");
  const rejected = increases(samples, "sharesRejected");
  if (accepted < 1) fail("soak_no_accepted_share");
  if (!maybeFinal) fail("soak_final_state_missing");
  else {
    if (maybeFinal.bootSession !== baseline.bootSession) fail("soak_device_reset");
    if (!hardwareStopped(maybeFinal)) fail("soak_not_stopped_after");
    if (maybeFinal.startMiningOnBoot) fail("soak_mine_on_boot_enabled");
    if (maybeFinal.pauseReason !== null) fail("soak_device_paused_itself");
  }
  const maximum = (key) => active.reduce((top, sample) => (sample[key] !== null && sample[key] > top ? sample[key] : top), -Infinity);
  const minimum = (key) => active.reduce((low, sample) => (sample[key] !== null && sample[key] < low ? sample[key] : low), Infinity);
  const finite = (value) => (Number.isFinite(value) ? value : null);
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      active_span_ms: span,
      active_samples: active.length,
      samples: samples.length,
      maximum_sample_gap_ms: maximumGapMs,
      shares_accepted: accepted,
      shares_rejected: rejected,
      maximum_chip_temp_c: finite(maximum("chipTempC")),
      maximum_vr_temp_c: finite(maximum("vrTempC")),
      maximum_power_w: finite(maximum("powerW")),
      minimum_input_mv: finite(minimum("inputMv")),
      maximum_input_mv: finite(maximum("inputMv")),
      minimum_fan_rpm: finite(minimum("fanRpm")),
      minimum_hashrate_ghs: finite(minimum("hashRate")),
      maximum_hashrate_ghs: finite(maximum("hashRate")),
    },
  };
}
