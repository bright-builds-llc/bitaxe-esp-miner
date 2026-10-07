// Pure physical checkpoint state machine for the disconnect and reboot scenarios (AGENTS.md: Asynchronous
// Human Checkpoints; Ultra 205 Serial Session Reuse, Plan-13 restore-watcher ordering). Human readiness and
// restoration waits have no deadline; the removal window and the absence bound are finite effect bounds.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { MAXIMUM_REARMS, PHYSICAL_PLANS, REMOVAL_WINDOW_MS, RESTORE_WATCHER_TOKEN } from "./contract.mjs";

/**
 * Checkpoints, in order: awaiting_operator_ready → ready_for_lease → watcher_starting → remove_* →
 * absence_bounding → (arm) restore_* → stabilizing → reconnect_ready → reconnected. A late or early human
 * action is expired authority (`rearm_required`), never device evidence; a broken identity rule is `failed`.
 */
export function createCheckpoint(scenario) {
  const plan = PHYSICAL_PLANS[scenario];
  requireCondition(plan !== undefined, "checkpoint_scenario");
  return { scenario, plan, checkpoint: "awaiting_operator_ready", rearms: 0, trace: [],
    maybeRemovalDeadline: null, maybeAbsentAt: null, maybePreRemovalEnumeration: null, maybeFailure: null, maybeRearmReason: null };
}

function move(state, checkpoint, now, detail = {}) {
  state.checkpoint = checkpoint;
  state.trace.push({ checkpoint, atUnixMs: now, ...detail });
}

function expire(state, reason, now) {
  state.maybeRearmReason = reason;
  move(state, "rearm_required", now, { reason });
}

/** Typed checkpoint failure (watcher timeout, identity rule); the attempt stops. */
export function failCheckpoint(state, reason, now) { fail(state, reason, now); }

function fail(state, reason, now) {
  state.maybeFailure = reason;
  move(state, "failed", now, { reason });
}

/** The operator's local readiness trigger; after an expired attempt it is a counted re-arm. */
export function operatorReady(state, expectedCheckpoint, now) {
  requireCondition(state.checkpoint === expectedCheckpoint, "checkpoint_stale");
  requireCondition(["awaiting_operator_ready", "rearm_required"].includes(state.checkpoint), "checkpoint_not_waiting");
  if (state.checkpoint === "rearm_required") {
    requireCondition(state.rearms < MAXIMUM_REARMS, "rearm_exhausted");
    state.rearms += 1;
  }
  state.maybeRearmReason = null; state.maybeAbsentAt = null; state.maybePreRemovalEnumeration = null; state.maybeRemovalDeadline = null;
  move(state, "ready_for_lease", now, { rearm: state.rearms });
}

/** The page began the window with an active lease; the watcher must prove presence before any instruction. */
export function beginWindow(state, now) {
  requireCondition(state.checkpoint === "ready_for_lease", "physical_window_not_ready");
  move(state, "watcher_starting", now);
}

/** Watcher change event (tools/flash usb-presence-watch). `now` is host time of receipt. */
export function watcherEvent(state, event, now) {
  const at = { atUnixMs: now, watcherElapsedMs: event.elapsed_ms };
  if (event.event === "failed") return fail(state, "watcher_failed", now);
  if (state.checkpoint === "watcher_starting") return starting(state, event, now, at);
  if (state.checkpoint === state.plan.removeCheckpoint) return removing(state, event, now, at);
  if (state.checkpoint === "absence_bounding") return bounding(state, event, now);
  if (state.checkpoint === state.plan.restoreCheckpoint) return restoring(state, event, now, at);
  if (state.checkpoint === "stabilizing") return stabilizing(state, event, now);
  if (state.checkpoint === "reconnect_ready" && ["absent", "reappeared", "enumeration_changed"].includes(event.event)) {
    return fail(state, "presence_changed_before_reconnect", now);
  }
  return undefined;
}

function starting(state, event, now, at) {
  if (event.event === "absent") return fail(state, "device_absent_at_begin", now);
  if (event.event !== "present") return undefined;
  state.maybePreRemovalEnumeration = event.enumeration_sha256;
  state.maybeRemovalDeadline = now + REMOVAL_WINDOW_MS;
  move(state, state.plan.removeCheckpoint, now, { ...at, watcherPresent: true, removalWindowMs: REMOVAL_WINDOW_MS });
  return undefined;
}

function removing(state, event, now, at) {
  if (event.event === "enumeration_changed") return fail(state, "enumeration_changed_without_absence", now);
  if (event.event !== "absent") return undefined;
  if (now > state.maybeRemovalDeadline) return expire(state, "removal_late", now);
  state.maybeAbsentAt = at;
  move(state, "absence_bounding", now, { ...at, removalObservedByWatcher: true });
  return undefined;
}

function bounding(state, event, now) {
  if (event.event !== "reappeared" && event.event !== "present") return undefined;
  const absentFor = event.elapsed_ms - state.maybeAbsentAt.watcherElapsedMs;
  return expire(state, absentFor < state.plan.minimumAbsenceMs ? "absence_too_short" : "restored_before_instruction", now);
}

function restoring(state, event, now, at) {
  if (event.event !== "reappeared" && event.event !== "present") return undefined;
  const absenceMs = event.elapsed_ms - state.maybeAbsentAt.watcherElapsedMs;
  if (absenceMs < state.plan.minimumAbsenceMs) return fail(state, "absence_bound_violated", now);
  if (event.event !== "reappeared" || event.enumeration_changed !== true || event.enumeration_sha256 === state.maybePreRemovalEnumeration) {
    return fail(state, "enumeration_unchanged", now);
  }
  move(state, "stabilizing", now, { ...at, absenceMs, samePhysicalIdentity: true, enumerationChanged: true });
  return undefined;
}

function stabilizing(state, event, now) {
  if (event.event === "absent") { move(state, state.plan.restoreCheckpoint, now, { reason: "absent_while_stabilizing" }); return undefined; }
  if (event.event === "stable" && event.ready === true) move(state, "reconnect_ready", now, { stable: true });
  return undefined;
}

/** Lazily expire a removal window nobody acted on, and report whether the absence bound has elapsed. */
export function view(state, now) {
  if (state.checkpoint === state.plan.removeCheckpoint && now > state.maybeRemovalDeadline) expire(state, "removal_late", now);
  const absenceSatisfied = state.checkpoint === "absence_bounding" && now - state.maybeAbsentAt.atUnixMs >= state.plan.minimumAbsenceMs;
  return { checkpoint: absenceSatisfied ? "absence_satisfied" : state.checkpoint, absenceSatisfied };
}

/**
 * The page asks to arm restoration. Only after a watcher-observed removal and the full absence bound does the
 * restore watcher arm; the token is emitted before the restore instruction becomes visible.
 */
export function armRestore(state, now) {
  const { absenceSatisfied } = view(state, now);
  if (!absenceSatisfied) return null;
  const token = { action_token: RESTORE_WATCHER_TOKEN, response_required: false, scenario: state.scenario };
  state.trace.push({ checkpoint: "restore_watcher_armed", atUnixMs: now, token: RESTORE_WATCHER_TOKEN });
  move(state, state.plan.restoreCheckpoint, now, { instructionAfterToken: true });
  return token;
}

/** The page reconnects only after reappearance passed the stability gate. */
export function admitReconnect(state, now) {
  requireCondition(state.checkpoint === "reconnect_ready", "physical_checkpoint_not_ready");
  move(state, "reconnected", now);
}

/** True while a reconnect (`/activate`) must be refused because the device is mid-checkpoint. */
export function blocksActivation(state) {
  return ["watcher_starting", state.plan.removeCheckpoint, "absence_bounding", state.plan.restoreCheckpoint, "stabilizing", "failed"]
    .includes(state.checkpoint);
}

/** States in which the next step is a human action with no deadline. */
export function humanWait(state) {
  return ["awaiting_operator_ready", "rearm_required", state.plan.restoreCheckpoint].includes(state.checkpoint);
}

/** Checkpoint facts the judge reads; derived only from the trace, never from page claims. */
export function checkpointFacts(state) {
  const index = (name) => state.trace.findIndex((row) => row.checkpoint === name);
  const last = (name) => state.trace.findLastIndex((row) => row.checkpoint === name);
  const removal = last(state.plan.removeCheckpoint), absent = last("absence_bounding"), token = last("restore_watcher_armed");
  const restore = last(state.plan.restoreCheckpoint), stable = last("reconnect_ready"), reconnected = last("reconnected");
  const stabilizing = state.trace[last("stabilizing")];
  return {
    watcherBeforeRemovalInstruction: removal >= 0 && state.trace[removal].watcherPresent === true && index("watcher_starting") < removal,
    removalObservedByWatcher: absent > removal && removal >= 0,
    restoreTokenBeforeRestoreInstruction: token >= 0 && token < restore && token > absent,
    absenceMs: stabilizing?.absenceMs ?? null,
    absenceBoundMet: (stabilizing?.absenceMs ?? -1) >= state.plan.minimumAbsenceMs,
    enumerationChanged: stabilizing?.enumerationChanged === true,
    samePhysicalIdentity: stabilizing?.samePhysicalIdentity === true,
    stableBeforeReconnect: stable >= 0 && reconnected > stable && stable > restore,
    rearms: state.rearms,
    failure: state.maybeFailure,
  };
}
