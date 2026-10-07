// Pure per-scenario judges (firmware ADR-0035). Inputs are the page's closed journal and recorded results,
// the completion reviews, the host checkpoint trace and a small carry between scenarios. Every failure is a
// closed category; every fact is a boolean that the public projection may copy.
import { unsafeSample } from "../fixed-usb-soak/judge.mjs";
import { EXPIRY_EARLY_TOLERANCE_MS, MAXIMUM_REARMS, PAGE_FAILURE_EVENTS, PHYSICAL_PLANS, SCENARIO_PLANS, STIMULUS_OBSERVATION_MS,
  WINDOWS } from "./contract.mjs";

const SESSION_LOSS_SCENARIOS = new Set(["disconnect", "reboot", "authorization_negatives"]);

/** Live limits on any sample a future page exposes: 4.5–5.5 V, ≤15 W, <75 C, fresh nonzero fan, watchdog. */
export function liveSafetyFailures(samples) {
  return samples.some(unsafeSample) ? ["unsafe_sample"] : [];
}

/** The scenario's journal entries after its segment start, and whether the bounded journal lost any. */
export function segment(finalState, startOrdinal) {
  const entries = finalState.journal.entries;
  const truncated = finalState.journal.dropped > 0 && (entries[0]?.ordinal ?? Infinity) > startOrdinal + 1;
  return { entries: entries.filter((entry) => entry.ordinal > startOrdinal), truncated };
}

const indexOf = (entries, event, category, from = 0) =>
  entries.findIndex((entry, index) => index >= from && entry.event === event && (category === undefined || entry.category === category));
const count = (entries, event) => entries.filter((entry) => entry.event === event).length;
const ok = (records, operation) => records.filter((record) => record.operation === operation && record.outcome === "ok");

function commonFailures(input) {
  const { scenario, finalState } = input, plan = SCENARIO_PLANS[scenario];
  const failures = [];
  const check = (failed, category) => { if (failed) failures.push(category); };
  const { entries, truncated } = segment(finalState, input.segmentStartOrdinal);
  check(truncated, "journal_truncated");
  check(finalState.connected !== false || finalState.status !== "closed" || finalState.failure !== undefined, "page_not_closed_cleanly");
  const device = finalState.device;
  check(device?.state !== "baseline" || device.restoration !== "confirmed", "baseline_unconfirmed");
  check(device?.reason !== plan.terminal, "terminal_reason_mismatch");
  const forbidden = PAGE_FAILURE_EVENTS.filter((event) => !(event === "serial_failure" && SESSION_LOSS_SCENARIOS.has(scenario)));
  check(entries.some((entry) => forbidden.includes(entry.event)), "page_operation_failed");
  check(count(entries, "lease_started") !== 1, "lease_start_count");
  failures.push(...liveSafetyFailures(input.safetySamples ?? []));
  return { failures, entries };
}

/** completion, pause and cancel: the operator's own stop ends the lease before the device ends it. */
function operatorStop(input, entries) {
  const plan = SCENARIO_PLANS[input.scenario];
  const started = indexOf(entries, "lease_started"), stopped = indexOf(entries, plan.endedBy, undefined, started);
  const deviceFirst = indexOf(entries, "device_baseline_observed");
  const facts = { operatorEndedLease: started >= 0 && stopped > started && (deviceFirst < 0 || deviceFirst > stopped) };
  if (input.scenario === "completion") facts.renewalAccepted = indexOf(entries, "renewed", undefined, started) > started && stopped > indexOf(entries, "renewed");
  return facts;
}

function expiry(input, entries) {
  const records = input.records;
  const started = ok(records, "startScenarioLease").at(-1);
  const observed = records.find((record) => record.state.device?.state === "baseline" && record.state.device.reason === "lease_expired" &&
    started && record.receivedAtUnixMs >= started.receivedAtUnixMs);
  const window = WINDOWS.expiry.durationMilliseconds;
  return {
    noRenewal: indexOf(entries, "lease_loaded", "no_renewal") >= 0 && count(entries, "renewed") === 0,
    deviceEndedLease: indexOf(entries, "device_baseline_observed", "lease_expired") > indexOf(entries, "lease_started"),
    expiredAfterWindow: Boolean(observed) && observed.receivedAtUnixMs - started.receivedAtUnixMs >= window - EXPIRY_EARLY_TOLERANCE_MS,
    noOperatorStop: ["paused", "cancelled", "restored"].every((event) => count(entries, event) === 0),
  };
}

/** The seven monotonic facts: a reason alone (or a relabelling restore) never proves the real path fired. */
function monotonic(input, entries) {
  const records = input.records;
  const start = ok(records, "startScenarioLease").at(-1);
  const pre = ok(records, "clockDiscontinuityStimulusReview").filter((record) => start && record.receivedAtUnixMs <= start.receivedAtUnixMs).at(-1);
  const ack = ok(records, "triggerClockDiscontinuity").at(-1);
  const observed = ack && records.find((record) => record.receivedAtUnixMs >= ack.receivedAtUnixMs && record.state.device?.state === "baseline" &&
    record.state.device.reason === "monotonic_reset");
  const leaseIndex = indexOf(entries, "lease_started"), ackIndex = indexOf(entries, "stimulus_acknowledged", undefined, leaseIndex);
  const baselineIndex = indexOf(entries, "device_baseline_observed", "monotonic_reset", ackIndex);
  const between = baselineIndex > leaseIndex ? entries.slice(leaseIndex, baselineIndex) : [];
  const post = input.reviews.stimulus;
  return {
    stimulusIdleBefore: pre?.result.state === "idle",
    stimulusAcknowledged: Boolean(ack) && ackIndex > leaseIndex,
    deviceEndedLease: ackIndex >= 0 && baselineIndex > ackIndex,
    noRelabel: ["restored", "paused", "cancelled", "renewed"].every((event) => count(entries, event) === 0),
    endedWithinBound: Boolean(observed) && observed.receivedAtUnixMs - ack.receivedAtUnixMs <= STIMULUS_OBSERVATION_MS,
    sameSession: between.length > 0 && !between.some((entry) => entry.event === "disconnected" || entry.event === "connected"),
    monotonicDetectionCounted: Boolean(pre) && post.state === "consumed" && post.discontinuitiesDetected === pre.result.discontinuitiesDetected + 1,
  };
}

function physical(input, entries) {
  const plan = PHYSICAL_PLANS[input.scenario], checkpoint = input.checkpoint;
  const started = indexOf(entries, "lease_started"), begun = indexOf(entries, "physical_window_begun", plan.removeCheckpoint, started);
  const lost = indexOf(entries, "disconnected", undefined, begun), armed = indexOf(entries, "physical_window_armed", plan.restoreCheckpoint, lost);
  const reconnected = indexOf(entries, "connected", undefined, armed);
  const reviewed = indexOf(entries, "status_reviewed", SCENARIO_PLANS[input.scenario].terminal, reconnected);
  return {
    watcherBeforeRemovalInstruction: checkpoint?.watcherBeforeRemovalInstruction === true,
    removalObservedByWatcher: checkpoint?.removalObservedByWatcher === true,
    absenceBoundMet: checkpoint?.absenceBoundMet === true,
    restoreTokenBeforeRestoreInstruction: checkpoint?.restoreTokenBeforeRestoreInstruction === true,
    enumerationChanged: checkpoint?.enumerationChanged === true,
    samePhysicalIdentity: checkpoint?.samePhysicalIdentity === true,
    stableBeforeReconnect: checkpoint?.stableBeforeReconnect === true,
    rearmsWithinCap: (checkpoint?.rearms ?? Infinity) <= MAXIMUM_REARMS && checkpoint?.failure === null,
    pageOrderConfirmed: started >= 0 && begun > started && lost > begun && armed > lost && reconnected > armed && reviewed > reconnected,
    deviceEndedLease: ["restored", "paused", "cancelled"].every((event) => indexOf(entries, event, undefined, started) < 0),
  };
}

/** The high-water epoch the page held after the reboot Start was accepted and before power was removed. */
export function preRebootEpoch(records) {
  const begin = ok(records, "beginPhysicalWindow").at(-1);
  const status = begin && ok(records, "statusReview").filter((record) => record.receivedAtUnixMs <= begin.receivedAtUnixMs &&
    record.result?.state === "mining").at(-1);
  return status ? status.state.highWaterEpoch : null;
}

function reboot(input) {
  const { stimulus, rejection } = input.reviews;
  return {
    rebootClearedStimulus: stimulus.state === "idle" && stimulus.discontinuitiesDetected === 0,
    highWaterNotAdvancedAfterReboot: rejection.bootRejections === 0 && rejection.highWater.advancedThisBoot === false,
    preRebootStatusObserved: preRebootEpoch(input.records) !== null,
  };
}

const ATTRIBUTION = {
  n1: { operation: "start", signature: "valid", context: "mismatch", replayGuard: "at_or_below_durable_high_water", advanced: false, wire: "authentication_failed" },
  n2: { operation: "start", signature: "not_evaluated", context: "expired", replayGuard: "not_evaluated", advanced: false, wire: "admission_required" },
  n3: { operation: "start", signature: "valid", context: "mismatch", replayGuard: "fresh", advanced: false, wire: "authentication_failed" },
  n4: { operation: "renew", signature: "valid", context: "current", replayGuard: "at_or_below_durable_high_water", advanced: true, wire: "authentication_failed" },
};

/** One negative leg: the device rejected the replay and its own review attributes it exactly. */
export function legAttributed(leg, name, ordinal) {
  const expected = ATTRIBUTION[name], last = leg?.review?.last;
  return leg?.replay?.operation === expected.operation && leg.replay.outcome === "rejected" && leg.replay.category === expected.wire &&
    last !== null && last !== undefined &&
    last.operation === expected.operation && last.signature === expected.signature && last.context === expected.context &&
    last.replayGuard === expected.replayGuard && last.ordinal === ordinal && leg.review.bootRejections === ordinal &&
    leg.review.highWater.advancedThisBoot === expected.advanced;
}

/** The device refuses N1 until a status in the same connection has reported the reboot (firmware P2). */
function rebootReportedBeforeN1(entries) {
  const replay = indexOf(entries, "replay_rejected");
  if (replay < 0) return false;
  const before = entries.slice(0, replay);
  const connected = before.findLastIndex((entry) => entry.event === "connected");
  return connected >= 0 && before.findLastIndex((entry) => entry.event === "status_reviewed" && entry.category === "reboot") > connected &&
    before.findLastIndex((entry) => entry.event === "disconnected") < connected;
}

function negatives(input, entries) {
  const legs = input.legs ?? {}, base = input.carry.rebootRejections ?? 0;
  const n1 = legs.n1?.review?.highWater;
  return {
    rebootReportedBeforeN1: rebootReportedBeforeN1(entries),
    durableReplayAttributed: legAttributed(legs.n1, "n1", base + 1),
    expiredContextAttributed: legAttributed(legs.n2, "n2", base + 2),
    crossContextAttributed: legAttributed(legs.n3, "n3", base + 3),
    renewalReplayAttributed: legAttributed(legs.n4, "n4", base + 4),
    highWaterUnchangedAcrossReboot: n1?.fingerprintMatchesLatestObservation === true && Number.isInteger(n1.fingerprintFirstObservedEpoch) &&
      Number.isInteger(input.carry.preRebootEpoch) && n1.fingerprintFirstObservedEpoch <= input.carry.preRebootEpoch,
    rejectedStartsNeverStarted: count(entries, "replay_rejected") === 4 && count(entries, "replay_accepted") === 0 && count(entries, "renewed") === 1,
  };
}

/** The stimulus counter must move exactly once, in monotonic_uncertainty, and reset with the reboot. */
function stimulusCarry(input) {
  const review = input.reviews.stimulus, base = input.carry.stimulusBaseline;
  const before = ["completion", "pause", "cancel", "expiry"], consumed = ["monotonic_uncertainty", "disconnect"];
  if (input.scenario === "completion") return review.state === "idle";
  if (before.includes(input.scenario)) return review.state === "idle" && review.discontinuitiesDetected === base;
  if (consumed.includes(input.scenario)) return review.state === "consumed" && review.discontinuitiesDetected === base + 1;
  return review.state === "idle" && review.discontinuitiesDetected === 0;
}

const SCENARIO_FACTS = {
  completion: operatorStop, pause: operatorStop, cancel: operatorStop, expiry, monotonic_uncertainty: monotonic,
  disconnect: physical, reboot: (input, entries) => ({ ...physical(input, entries), ...reboot(input) }), authorization_negatives: negatives,
};

/**
 * Judge one scenario. Returns the closed result plus the carry the next scenario needs; the result is
 * `passed` only when every common check and every scenario fact holds.
 */
export function judgeScenario(input) {
  const { failures, entries } = commonFailures(input);
  const facts = { ...SCENARIO_FACTS[input.scenario](input, entries), stimulusCounterConsistent: stimulusCarry(input) };
  for (const [name, value] of Object.entries(facts)) if (value !== true) failures.push(`fact_${name}`);
  const carry = { ...input.carry };
  if (input.scenario === "completion") carry.stimulusBaseline = input.reviews.stimulus.discontinuitiesDetected;
  if (input.scenario === "reboot") { carry.preRebootEpoch = preRebootEpoch(input.records); carry.rebootRejections = input.reviews.rejection.bootRejections; }
  return { scenario: input.scenario, result: failures.length === 0 ? "passed" : "unverified", failures, facts, carry,
    terminal_reason: input.finalState.device?.reason ?? null };
}
