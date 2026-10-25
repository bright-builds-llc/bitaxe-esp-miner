// The local mining soak's pure judge (ADR-0038 task 6): samples of /api/system/info, the
// live stop rule applied to each, and the verdict over the whole run. Samples carry only
// metadata; no endpoint, pool or credential value ever enters them.

/** Input power at or below this, freshly measured, shows the Ultra 205's ASIC powered down. */
export const IDLE_POWER_W = 5;
/**
 * The Ultra 205's live stop limits, as for the lease soaks: input 4.5–5.5 V, at most 15 W, ASIC below
 * 75 C, and its idle power. It reports no regulator temperature to judge. A soak's `limits` replace them.
 * `restartCooling` is what counts as cool for the overheat recovery's restart (`restartCool`): on the
 * 205, a fresh chip reading at or below 45 C.
 */
export const LIMITS = Object.freeze({ minimumInputMv: 4500, maximumInputMv: 5500, maximumPowerW: 15, maximumChipTempC: 75,
  idlePowerW: IDLE_POWER_W, maybeMaximumVrTempC: null, restartCooling: Object.freeze({ kind: "chip", maximumC: 45 }) });
/**
 * The Gamma 601's limits (ADR-0041):
 * - power up to 22 W, the firmware's own stop (19 W target plus the 3 W margin,
 *   `crates/bitaxe-api/src/observation/limits.rs`);
 * - idle up to 6.5 W: its power reads VOUT x IOUT plus a fixed 5 W offset
 *   (`crates/bitaxe-safety/src/gamma601_sensors.rs`), so a powered-down ASIC reads about 5.0 W, and
 *   the 205's 5 W would refuse it on a rounding. 1.5 W above the offset absorbs the regulator's
 *   IOUT offset on a disabled output, and stays far below hashing power, which already adds the
 *   offset to the ASIC's own several watts;
 * - the regulator temperature stops above 90 C, under the firmware's 105 C throttle;
 * - cool for the overheat restart: the chip's diode has no reading once its ASIC is unpowered, so
 *   the chip reads unavailable with reason `asic_unpowered` and the regulator reads fresh at or
 *   below 95 C;
 * - the chip and input limits as on the 205.
 */
export const GAMMA601_LIMITS = Object.freeze({ ...LIMITS, maximumPowerW: 22, idlePowerW: 6.5, maybeMaximumVrTempC: 90,
  restartCooling: Object.freeze({ kind: "vrUnpowered", maximumVrC: 95 }) });
/** The limits a soak runs under: its own, or the Ultra 205's. */
export const limitsOf = (soak) => soak?.limits ?? LIMITS;

/** Whether a sample counts as cool for the overheat recovery's restart, under `limits.restartCooling`. */
export function restartCool(limits) {
  const rule = limits.restartCooling;
  if (rule.kind === "chip") {
    return (sample) => sample.fresh.chipTemp && sample.chipTempC !== null && sample.chipTempC <= rule.maximumC;
  }
  if (rule.kind === "vrUnpowered") {
    return (sample) => sample.chipTempUnavailableReason === "asic_unpowered" && sample.vrTempFresh && sample.vrTempC !== null &&
      sample.vrTempC <= rule.maximumVrC;
  }
  throw new Error(`unknown restart cooling rule: ${rule.kind}`);
}

/**
 * The soaks a contract may run, each with its own enable line (owner decisions 2026-10-17/18).
 * - Soak 001 mines 30 continuous minutes.
 * - Soak 002 accumulates 2 hours of mining within 2 h 30 min, and must show a nonzero hashrate.
 *   It tolerates outages the device recovers from on its own (`maybeMaximumOutageMs`), because
 *   real networks drop: attempt 001 met two ~19 s network outages, and mining resumed each time.
 * - Soak 003 is soak 002 at several hours: 4 hours of mining accumulated within 5 hours.
 * - Soak 004 repeats soak 002 to catch outages, now that each break's retained log is journaled.
 * - Soak 005 runs overnight on the Wi-Fi power-save fix: 8 hours of mining accumulated within 9.
 * - Soak 006 measures the PSRAM atomics fix (ADR-0039): soak 003's rule, 4 hours within 5.
 * - Overheat check 001 (`overheat.mjs`) needs the test-only firmware whose local trip is 55 C
 *   (ADR-0040). At 500 MHz / 1200 mV it must trip, cool, restart at 400 MHz / 1100 mV and mine
 *   5 minutes there, or trip again and pause itself; then the original settings come back.
 * - Overheat check 002 starts at the default 485 MHz, so the lowered step leaves the catalog: the
 *   trip must leave the device paused by itself, with no restart.
 * - Handover loop 001 (`handover.mjs`) counts 20 lease-free handovers, to reproduce a panic.
 * - Lease check 001 (`lease.mjs`) mines locally for 5 minutes, then a Gate lease interrupts local
 *   mining. Local mining must come back within 3 minutes of the lease's end and mine 5 more
 *   minutes. Outages are tolerated as in soak 002, in the local phases only.
 * - Stopping check 001 (`stopping.mjs`) mines 3 minutes, then pauses: every sample until the stop
 *   confirms must read `active`, then `stopping`, then `paused`, never `active` at idle power.
 * - Stopping check 002 repeats it on the polled cooling proof (ADR-0016): the hashrate must also
 *   read zero while stopping, not the value frozen at the stop.
 * - Watchdog check 001 repeats stopping check 002 and also requires the owner's task watchdog
 *   fresh in every sample, through preparation, mining and the stop.
 * - Boot resume check 001 (`boot-resume.mjs`) turns mining on boot on and restarts: mining must
 *   come back with no resume command and accumulate 5 minutes within 15 of first seeing the boot,
 *   outages tolerated as in soak 002. It then turns mining on boot off, pauses and restarts: the
 *   device must stay paused for 2 minutes.
 * - Gamma 601 first mining 001 is the 601's first local mining (ADR-0041 phase F3), on the test-only
 *   `gamma601-mining` firmware: 10 continuous minutes at 400 MHz / 1150 mV, saved and read back with
 *   the pool before the resume, under the 601's limits (`GAMMA601_LIMITS`).
 * - Gamma 601 overheat check 001 is overheat check 001 on the 601 (ADR-0041 phase F3.4), on the
 *   test-only `gamma601-overheat-trip-test` firmware (55 C trip), under the 601's limits. The
 *   baseline must read 400 MHz / 1150 mV (`baselineProfile`, F3.3's measured pair), because the
 *   cleanup's clearing resume runs at the restored baseline. At 500 MHz / 1150 mV it must trip, cool
 *   by the 601's rule and restart at 400 MHz / 1050 mV (`restartProfile`), which must show a nonzero
 *   hashrate (`requireRestartHashrate`); then it mines 5 minutes there, or trips again and pauses
 *   itself, because the next step would leave the catalog.
 */
export const SOAKS = Object.freeze({
  "001": Object.freeze({ id: "001", activeMs: 30 * 60_000, requireHashrate: false, maybeMaximumOutageMs: null,
    maybeDeadlineMs: null }),
  "002": Object.freeze({ id: "002", activeMs: 120 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000,
    maybeDeadlineMs: 150 * 60_000 }),
  "003": Object.freeze({ id: "003", activeMs: 240 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000,
    maybeDeadlineMs: 300 * 60_000 }),
  "004": Object.freeze({ id: "004", activeMs: 120 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000,
    maybeDeadlineMs: 150 * 60_000 }),
  "005": Object.freeze({ id: "005", activeMs: 480 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000,
    maybeDeadlineMs: 540 * 60_000 }),
  "006": Object.freeze({ id: "006", task: "task-local-mining-atomics-soak", activeMs: 240 * 60_000, requireHashrate: true,
    maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 300 * 60_000 }),
  "overheat-001": Object.freeze({ id: "overheat-001", kind: "overheat", task: "task-local-mining-overheat-check",
    firmwareVariant: "overheat-trip-test", frequencyMhz: 500, coreVoltageMv: 1200, tripBoundMs: 10 * 60_000,
    // A trip's ordered safe-stop can take up to 120 s before `overheat_mode` shows, so breaks get the
    // 180 s stop allowance; an unreachable device still stops it after 60 s.
    restartBoundMs: 10 * 60_000, afterActiveMs: 5 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 180_000,
    unreachableStopMs: 60_000, maybeDeadlineMs: 45 * 60_000, enableLine: "Ultra 205 local mining overheat check hardware: enabled.",
    outcomes: ["restarted", "restarted_then_stays_stopped"] }),
  // From the default 485 MHz the lowered step would leave the catalog (400 MHz floor), so the trip
  // must leave the device paused by itself, with no restart.
  "overheat-002": Object.freeze({ id: "overheat-002", kind: "overheat", task: "task-local-mining-overheat-stays-stopped",
    firmwareVariant: "overheat-trip-test", frequencyMhz: 485, coreVoltageMv: 1200, tripBoundMs: 10 * 60_000,
    restartBoundMs: 10 * 60_000, afterActiveMs: 5 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 180_000,
    unreachableStopMs: 60_000, maybeDeadlineMs: 45 * 60_000,
    enableLine: "Ultra 205 local mining overheat stays-stopped check hardware: enabled.", outcomes: ["stays_stopped"] }),
  "handover-001": Object.freeze({ id: "handover-001", kind: "handover", task: "task-handover-panic-diagnosis", cycles: 20, breakBoundMs: 10 * 60_000,
    unreachableStopMs: 60_000, requireHashrate: true, maybeMaximumOutageMs: null, maybeDeadlineMs: 150 * 60_000,
    enableLine: "Ultra 205 local mining handover loop hardware: enabled." }),
  "lease-001": Object.freeze({ id: "lease-001", kind: "lease", task: "task-handover-panic-diagnosis", activeMs: 5 * 60_000, afterActiveMs: 5 * 60_000, requireHashrate: true,
    maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 90 * 60_000, restoreBoundMs: 180_000,
    enableLine: "Ultra 205 lease interruption check hardware: enabled." }),
  "stopping-001": Object.freeze({ id: "stopping-001", kind: "stopping", task: "task-mining-activity-stopping",
    activeMs: 3 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 15 * 60_000,
    enableLine: "Ultra 205 mining activity stopping check hardware: enabled." }),
  "stopping-002": Object.freeze({ id: "stopping-002", kind: "stopping", task: "task-safe-stop-polled-cooling",
    activeMs: 3 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 15 * 60_000,
    requireHashrateZeroed: true, enableLine: "Ultra 205 polled cooling stopping check hardware: enabled." }),
  "watchdog-001": Object.freeze({ id: "watchdog-001", kind: "stopping", task: "task-preparation-rollback-cooling-watchdog",
    activeMs: 3 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 15 * 60_000,
    requireHashrateZeroed: true, requireWatchdogFresh: true,
    enableLine: "Ultra 205 preparation watchdog check hardware: enabled." }),
  // Phase C's regression check on the Ultra 205 (ADR-0041): watchdog-001's checks, and the baseline
  // must report the stored board as the 205 with a matching identity.
  "regression-001": Object.freeze({ id: "regression-001", kind: "stopping", task: "task-gamma-601-bring-up",
    activeMs: 3 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 15 * 60_000,
    requireHashrateZeroed: true, requireWatchdogFresh: true, requireStoredBoard: "205",
    enableLine: "Ultra 205 board seam regression check hardware: enabled." }),
  "boot-resume-001": Object.freeze({ id: "boot-resume-001", kind: "boot-resume", task: "task-local-mining-boot-resume",
    activeMs: 5 * 60_000, requireHashrate: true, maybeMaximumOutageMs: 60_000, maybeDeadlineMs: 15 * 60_000,
    enableLine: "Ultra 205 local mining boot resume check hardware: enabled." }),
  "gamma601-first-001": Object.freeze({ id: "gamma601-first-001", task: "task-gamma-601-bring-up",
    firmwareVariant: "gamma601-mining", activeMs: 10 * 60_000, requireHashrate: true, maybeMaximumOutageMs: null,
    maybeDeadlineMs: null, requireStoredBoard: "601", limits: GAMMA601_LIMITS,
    profile: Object.freeze({ frequencyMhz: 400, coreVoltageMv: 1150 }),
    enableLine: "Gamma 601 first local mining hardware: enabled." }),
  "gamma601-overheat-001": Object.freeze({ id: "gamma601-overheat-001", kind: "overheat", task: "task-gamma-601-bring-up",
    firmwareVariant: "gamma601-overheat-trip-test", requireStoredBoard: "601", limits: GAMMA601_LIMITS,
    frequencyMhz: 500, coreVoltageMv: 1150,
    baselineProfile: Object.freeze({ frequencyMhz: 400, coreVoltageMv: 1150 }),
    restartProfile: Object.freeze({ frequencyMhz: 400, coreVoltageMv: 1050 }), requireRestartHashrate: true,
    tripBoundMs: 10 * 60_000, restartBoundMs: 10 * 60_000, afterActiveMs: 5 * 60_000, requireHashrate: true,
    maybeMaximumOutageMs: 180_000, unreachableStopMs: 60_000, maybeDeadlineMs: 45 * 60_000,
    enableLine: "Gamma 601 local mining overheat check hardware: enabled.",
    outcomes: ["restarted", "restarted_then_stays_stopped"] }),
});
export const SAMPLE_INTERVAL_MS = 2000;
/** A longer gap between successive samples breaks continuity. */
export const MAXIMUM_SAMPLE_GAP_MS = 10_000;
/** Probe, preparation and the cooling proof may take this long before mining is seen. */
export const START_ALLOWANCE_MS = 5 * 60_000;
/** The ordered safe-stop, including its up-to-120-s cooling proof, must finish within this. */
export const STOP_ALLOWANCE_MS = 180_000;
/** The run never mines past this, measured from the resume: the soak's own, or start, its time and slack. */
export const runDeadlineMs = (soak) => soak.maybeDeadlineMs ?? START_ALLOWANCE_MS + soak.activeMs + 5 * 60_000;

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
    // Kept out of `fresh`, so the readings every board's live stops need are unchanged.
    vrTempFresh: info?.vrTempStatus?.state === "fresh",
    chipTempUnavailableReason: info?.chipTempStatus?.reason?.kind === "unavailable" && typeof info.chipTempStatus.reason.code === "string"
      ? info.chipTempStatus.reason.code : null,
    inputMv: number(info?.voltage),
    powerW: number(info?.power),
    fanRpm: number(info?.fanrpm),
    frequencyMhz: number(info?.frequency),
    coreVoltageMv: number(info?.coreVoltage),
    overheatMode: number(info?.overheat_mode) === 1,
    autoFanSpeed: number(info?.autofanspeed) === 1,
    manualFanSpeed: number(info?.manualFanSpeed),
    pauseReason: typeof info?.pauseReason === "string" ? info.pauseReason : null,
    hardwareFault: typeof info?.hardware_fault === "string",
    powerFault: typeof info?.power_fault === "string",
    // Wi-Fi state, to diagnose outages; the status label is reduced to a boolean.
    wifiConnected: info?.wifiStatus === "connected",
    wifiRssi: number(info?.wifiRSSI),
    // The owner's task watchdog, as runtime health reports it: its category labels only.
    watchdogReason: typeof info?.runtimeHealth?.taskWatchdogReason === "string" ? info.runtimeHealth.taskWatchdogReason : null,
    watchdogSubphase: typeof info?.runtimeHealth?.taskWatchdogOwnerSubphase === "string"
      ? info.runtimeHealth.taskWatchdogOwnerSubphase : null,
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

/** Whether a fresh input power reading shows the ASIC idle, under `limits`. */
export const idlePowerUnder = (limits) => (sample) => sample.fresh.power && sample.powerW !== null && sample.powerW <= limits.idlePowerW;
/** `idlePowerUnder` the Ultra 205's limits. */
export const idlePower = idlePowerUnder(LIMITS);

/**
 * Whether the ASIC is shown powered down under `limits`: paused, not mining, the stop confirmed, and
 * freshly measured idle power. `stopping` means the ordered safe-stop has not yet confirmed, though
 * the ASIC already idles.
 */
export const hardwareStoppedUnder = (limits) => {
  const idle = idlePowerUnder(limits);
  return (sample) => sample.miningPaused && !mining(sample) && sample.miningActivity !== "stopping" && idle(sample);
};
/** `hardwareStoppedUnder` the Ultra 205's limits. */
export const hardwareStopped = hardwareStoppedUnder(LIMITS);

/** Whether the device reports active mining in this sample. The live limits key on this. */
export const mining = (sample) => sample.miningActivity === "active" && !sample.miningPaused;

/**
 * Whether the ASIC is actually hashing: the status reports mining and no fresh input power reading
 * shows it idle. Status alone read `active` at 0.8-4.8 W while the ASIC idled: for 13-18 s once in
 * soak 005 and around every lease check's interrupt, and for single samples in the handover loop.
 * Ramps read from 5.03 W, so 5 W has little margin there; that moves a break's edge by a sample at
 * most. Plain soaks hash above 10 W. A stale reading counts as hashing: the stale-readings live stop
 * bounds staleness, as for the limits. Mining time, outages and breaks count this.
 */
export const hashingUnder = (limits) => (sample) =>
  mining(sample) && !(sample.fresh.power && (sample.powerW === null || sample.powerW <= limits.idlePowerW));
/** `hashingUnder` the Ultra 205's limits. */
export const hashing = hashingUnder(LIMITS);

/** Samples whose status reports mining while the ASIC is not hashing. */
export const statusOnlySamples = (samples, limits = LIMITS) => {
  const hashes = hashingUnder(limits);
  return samples.filter((sample) => mining(sample) && !hashes(sample)).length;
};

/**
 * The first live stop this sample trips against the run's baseline, or null. A reset, a device
 * pause, a fault or an overheat always stops. While the device mines, the electrical and thermal
 * limits apply, as the firmware's own protection does, and readings stale on two samples in a row
 * stop the run (`maybePrevious` is the sample before). `limits` are the soak's.
 */
export function liveStop(sample, baseline, maybePrevious = null, limits = LIMITS) {
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
  if (fresh.input && (sample.inputMv === null || sample.inputMv < limits.minimumInputMv || sample.inputMv > limits.maximumInputMv)) {
    return "soak_input_voltage";
  }
  if (fresh.power && (sample.powerW === null || sample.powerW > limits.maximumPowerW)) return "soak_power";
  if (fresh.chipTemp && (sample.chipTempC === null || sample.chipTempC >= limits.maximumChipTempC)) return "soak_chip_temperature";
  // Judged only when the device reports it. A stale regulator reading reads as 0, so it never trips;
  // the firmware itself refuses to mine on a TPS546 board without a fresh one.
  if (limits.maybeMaximumVrTempC !== null && sample.vrTempC !== null && sample.vrTempC > limits.maybeMaximumVrTempC) {
    return "soak_vr_temperature";
  }
  if (fresh.fan && (sample.fanRpm === null || sample.fanRpm <= 0)) return "soak_fan_stopped";
  return null;
}

/** Continuous hashing measured from the first hashing sample, broken by any gap or other sample. */
export function activeSpanMs(samples, limits = LIMITS) {
  const hashing = hashingUnder(limits);
  let longest = 0, maybeStart = null, maybeLast = null;
  for (const sample of samples) {
    const continuous = maybeLast !== null && sample.atMs - maybeLast.atMs <= MAXIMUM_SAMPLE_GAP_MS;
    if (hashing(sample)) {
      if (maybeStart === null || !continuous || !hashing(maybeLast)) maybeStart = sample.atMs;
      longest = Math.max(longest, sample.atMs - maybeStart);
    } else {
      maybeStart = null;
    }
    maybeLast = sample;
  }
  return longest;
}

/**
 * Mining accumulated across samples: each step between two hashing samples at most the gap bound
 * apart counts. Outages are the intervals from one hashing sample to the next hashing sample when
 * another sample or a longer gap lies between; they start only once hashing has begun.
 */
export function miningLog(samples, limits = LIMITS) {
  const hashing = hashingUnder(limits);
  let activeMs = 0, maybeLastMining = null, broken = false;
  const outagesMs = [];
  for (const sample of samples) {
    if (!hashing(sample)) { if (maybeLastMining !== null) broken = true; continue; }
    if (maybeLastMining !== null) {
      const stepMs = sample.atMs - maybeLastMining.atMs;
      if (broken || stepMs > MAXIMUM_SAMPLE_GAP_MS) outagesMs.push(stepMs); else activeMs += stepMs;
    }
    maybeLastMining = sample; broken = false;
  }
  // An outage still open at the last sample counts too, so a stop during one is reported.
  const maybeLastSample = samples.at(-1);
  const maybeOpenOutageMs = maybeLastMining !== null && maybeLastSample !== undefined && !hashing(maybeLastSample)
    ? maybeLastSample.atMs - maybeLastMining.atMs : null;
  return { activeMs, outagesMs, maybeOpenOutageMs, maybeLastMiningAtMs: maybeLastMining?.atMs ?? null };
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
  const limits = limitsOf(soak);
  const failures = [];
  const fail = (code) => { if (!failures.includes(code)) failures.push(code); };
  if (maybeStop) fail(maybeStop);
  samples.forEach((sample, index) => { const stop = liveStop(sample, baseline, samples[index - 1] ?? null, limits); if (stop) fail(stop); });
  const active = samples.filter(hashingUnder(limits));
  const span = activeSpanMs(samples, limits);
  const log = miningLog(samples, limits);
  const tolerant = soak.maybeMaximumOutageMs !== null;
  if ((tolerant ? log.activeMs : span) < soak.activeMs) fail("soak_active_time_short");
  if (tolerant && log.outagesMs.some((outageMs) => outageMs > soak.maybeMaximumOutageMs)) fail("soak_outage_unrecovered");
  if (soak.requireHashrate && !active.some((sample) => sample.hashRate !== null && sample.hashRate > 0)) fail("soak_no_hashrate");
  let maximumGapMs = 0;
  for (let index = 1; index < samples.length; index += 1) maximumGapMs = Math.max(maximumGapMs, samples[index].atMs - samples[index - 1].atMs);
  // A tolerant soak judges gaps as outages instead; anything longer than its bound fails above.
  if (!tolerant && maximumGapMs > MAXIMUM_SAMPLE_GAP_MS) fail("soak_sample_gap");
  const accepted = increases(samples, "sharesAccepted");
  const rejected = increases(samples, "sharesRejected");
  if (accepted < 1) fail("soak_no_accepted_share");
  finalFailures(baseline, maybeFinal, limits).forEach(fail);
  return {
    schema: "local-mining-soak-result-v1",
    soak: soak.id,
    result: failures.length === 0 ? "passed" : "failed",
    failures,
    metrics: {
      active_span_ms: span,
      accumulated_active_ms: log.activeMs,
      outage_count: log.outagesMs.length + (log.maybeOpenOutageMs === null ? 0 : 1),
      maximum_outage_ms: Math.max(0, ...log.outagesMs, log.maybeOpenOutageMs ?? 0),
      ...wifiMetrics(samples),
      active_samples: active.length,
      status_only_samples: statusOnlySamples(samples, limits),
      samples: samples.length,
      maximum_sample_gap_ms: maximumGapMs,
      shares_accepted: accepted,
      shares_rejected: rejected,
      // The maxima cover every sample the live limits judged.
      ...limitMetrics(samples.filter(mining)),
    },
  };
}

/** The final sample's failures: the same boot, the ASIC powered down, mining on boot off, no pause reason. */
export function finalFailures(baseline, maybeFinal, limits = LIMITS) {
  if (!maybeFinal) return ["soak_final_state_missing"];
  return [
    maybeFinal.bootSession !== baseline.bootSession && "soak_device_reset",
    !hardwareStoppedUnder(limits)(maybeFinal) && "soak_not_stopped_after",
    maybeFinal.startMiningOnBoot && "soak_mine_on_boot_enabled",
    maybeFinal.pauseReason !== null && "soak_device_paused_itself",
  ].filter(Boolean);
}

const finite = (value) => (Number.isFinite(value) ? value : null);

/** The extremes of each limited reading over the mining samples. */
export function limitMetrics(active) {
  const maximum = (key) => active.reduce((top, sample) => (sample[key] !== null && sample[key] > top ? sample[key] : top), -Infinity);
  const minimum = (key) => active.reduce((low, sample) => (sample[key] !== null && sample[key] < low ? sample[key] : low), Infinity);
  return {
    maximum_chip_temp_c: finite(maximum("chipTempC")),
    maximum_vr_temp_c: finite(maximum("vrTempC")),
    maximum_power_w: finite(maximum("powerW")),
    minimum_input_mv: finite(minimum("inputMv")),
    maximum_input_mv: finite(maximum("inputMv")),
    minimum_fan_rpm: finite(minimum("fanRpm")),
    minimum_hashrate_ghs: finite(minimum("hashRate")),
    maximum_hashrate_ghs: finite(maximum("hashRate")),
  };
}

/** Wi-Fi state over every sample, to diagnose outages. */
export function wifiMetrics(samples) {
  return {
    minimum_wifi_rssi_dbm: finite(samples.reduce((low, sample) => (sample.wifiRssi !== null && sample.wifiRssi < low ? sample.wifiRssi : low), Infinity)),
    wifi_disconnected_samples: samples.filter((sample) => !sample.wifiConnected).length,
  };
}
