import test from "node:test";
import assert from "node:assert/strict";
import { admitReconnect, armRestore, beginWindow, blocksActivation, checkpointFacts, createCheckpoint, humanWait, operatorReady, view, watcherEvent } from "./checkpoint.mjs";
import { REMOVAL_LEASE_HEADROOM_MS, REMOVAL_WINDOW_MS, RESTORE_WATCHER_TOKEN } from "./contract.mjs";

const A = "a".repeat(64), B = "b".repeat(64);
const event = (name, elapsed, extra = {}) => ({ schema: "bwg-usb-presence-watch-v1", sequence: 1, elapsed_ms: elapsed, event: name, ...extra });
const throwsWith = (operation, code) => assert.throws(operation, (error) => error.code === code);

/** Drive a checkpoint to its removal instruction, as the server does after the page begins the window. */
function removing(scenario = "disconnect", now = 1000) {
  const state = createCheckpoint(scenario);
  operatorReady(state, "awaiting_operator_ready", now);
  beginWindow(state, now, now);
  watcherEvent(state, event("present", 0, { enumeration_sha256: A, ready: false }), now);
  return state;
}

function restoring(scenario = "disconnect") {
  const state = removing(scenario);
  watcherEvent(state, event("absent", 1000), 2000);
  armRestore(state, 2000 + state.plan.minimumAbsenceMs);
  return state;
}

test("the removal instruction appears only after the watcher proves the device present", () => {
  // Arrange
  const state = createCheckpoint("disconnect");
  operatorReady(state, "awaiting_operator_ready", 1000);
  beginWindow(state, 1000, 1000);
  // Act
  const before = state.checkpoint;
  watcherEvent(state, event("present", 0, { enumeration_sha256: A }), 1100);
  // Assert
  assert.equal(before, "watcher_starting");
  assert.equal(state.checkpoint, "remove_usb");
  assert.equal(checkpointFacts(state).watcherBeforeRemovalInstruction, true);
});

test("removal is accepted only when the watcher observes disappearance", () => {
  // Arrange
  const state = removing();
  // Act
  const beforeAbsence = view(state, 1500).checkpoint;
  watcherEvent(state, event("absent", 500), 1500);
  // Assert
  assert.equal(beforeAbsence, "remove_usb");
  assert.equal(state.checkpoint, "absence_bounding");
});

test("a removal after the removal window is expired authority, not device evidence", () => {
  // Arrange
  const state = removing();
  // Act
  watcherEvent(state, event("absent", 40000), 1000 + REMOVAL_WINDOW_MS + 1);
  // Assert
  assert.deepEqual([state.checkpoint, state.maybeRearmReason], ["rearm_required", "removal_late"]);
});

test("a removal 40 s after the instruction is inside the widened 45 s window", () => {
  // Arrange
  const state = removing();
  // Act
  watcherEvent(state, event("absent", 40000), 1000 + 40000);
  // Assert
  assert.equal(REMOVAL_WINDOW_MS, 45000);
  assert.equal(state.checkpoint, "absence_bounding");
});

test("a removal 46 s after the instruction is late", () => {
  // Arrange
  const state = removing();
  // Act
  watcherEvent(state, event("absent", 46000), 1000 + 46000);
  // Assert
  assert.deepEqual([state.checkpoint, state.maybeRearmReason], ["rearm_required", "removal_late"]);
});

test("a removal that would race the delivered lease's end is late even inside the removal window", () => {
  // Arrange: the lease was delivered 15 s before the instruction, so the device may end it 45 s after the instruction.
  const state = createCheckpoint("reboot");
  operatorReady(state, "awaiting_operator_ready", 0);
  beginWindow(state, 15000, 0);
  watcherEvent(state, event("present", 0, { enumeration_sha256: A }), 15000);
  // Act
  const deadline = state.maybeRemovalDeadline;
  watcherEvent(state, event("absent", 41000), 15000 + 41000);
  // Assert
  assert.equal(deadline, 60000 - REMOVAL_LEASE_HEADROOM_MS);
  assert.deepEqual([state.checkpoint, state.maybeRearmReason], ["rearm_required", "removal_late"]);
});

test("a window cannot begin without a delivered lease", () => {
  // Arrange
  const state = createCheckpoint("disconnect");
  operatorReady(state, "awaiting_operator_ready", 0);
  // Act / Assert
  throwsWith(() => beginWindow(state, 1000, null), "physical_window_lease_missing");
});

test("the restore watcher does not arm before the absence bound", () => {
  // Arrange
  const state = removing("reboot");
  watcherEvent(state, event("absent", 1000), 2000);
  // Act
  const early = armRestore(state, 2000 + 9999);
  const due = armRestore(state, 2000 + 10000);
  // Assert
  assert.equal(early, null);
  assert.deepEqual(due, { action_token: RESTORE_WATCHER_TOKEN, response_required: false, scenario: "reboot" });
  assert.equal(state.checkpoint, "restore_power");
});

test("the restore token is recorded before the restore instruction", () => {
  // Arrange
  const state = restoring("reboot");
  // Act
  const rows = state.trace.map((row) => row.checkpoint);
  // Assert
  assert.ok(rows.indexOf("restore_watcher_armed") < rows.indexOf("restore_power"));
});

test("reappearing before the restore instruction requires a re-arm", () => {
  // Arrange
  const state = removing();
  watcherEvent(state, event("absent", 1000), 2000);
  // Act
  watcherEvent(state, event("reappeared", 3000, { enumeration_sha256: B, enumeration_changed: true }), 4000);
  // Assert
  assert.deepEqual([state.checkpoint, state.maybeRearmReason], ["rearm_required", "absence_too_short"]);
});

test("reappearance with the same enumeration fails the checkpoint", () => {
  // Arrange
  const state = restoring();
  // Act
  watcherEvent(state, event("reappeared", 9000, { enumeration_sha256: A, enumeration_changed: false }), 9000);
  // Assert
  assert.deepEqual([state.checkpoint, state.maybeFailure], ["failed", "enumeration_unchanged"]);
});

test("reconnection waits for the stability gate", () => {
  // Arrange
  const state = restoring();
  watcherEvent(state, event("reappeared", 9000, { enumeration_sha256: B, enumeration_changed: true }), 9000);
  // Act
  const blockedBefore = blocksActivation(state);
  const refused = (() => { try { admitReconnect(state, 9100); return false; } catch (error) { return error.code; } })();
  watcherEvent(state, event("stable", 12000, { enumeration_sha256: B, ready: true }), 12000);
  admitReconnect(state, 12100);
  // Assert
  assert.deepEqual([blockedBefore, refused], [true, "physical_checkpoint_not_ready"]);
  const facts = checkpointFacts(state);
  assert.deepEqual([facts.absenceMs, facts.enumerationChanged, facts.stableBeforeReconnect, facts.restoreTokenBeforeRestoreInstruction], [8000, true, true, true]);
});

test("human waits have no deadline", () => {
  // Arrange
  const waiting = createCheckpoint("reboot"), restored = restoring("reboot");
  const muchLater = 1000 + 7 * 24 * 3600 * 1000;
  // Act
  const states = [view(waiting, muchLater).checkpoint, view(restored, muchLater).checkpoint];
  // Assert
  assert.deepEqual(states, ["awaiting_operator_ready", "restore_power"]);
  assert.deepEqual([humanWait(waiting), humanWait(restored)], [true, true]);
});

test("re-arms are capped at two per physical scenario", () => {
  // Arrange
  const state = createCheckpoint("disconnect");
  operatorReady(state, "awaiting_operator_ready", 0);
  for (let attempt = 0; attempt < 2; attempt += 1) {
    beginWindow(state, 0, 0);
    watcherEvent(state, event("present", 0, { enumeration_sha256: A }), 0);
    watcherEvent(state, event("absent", 60000), REMOVAL_WINDOW_MS + 1);
    operatorReady(state, "rearm_required", 0);
  }
  beginWindow(state, 0, 0);
  watcherEvent(state, event("present", 0, { enumeration_sha256: A }), 0);
  watcherEvent(state, event("absent", 60000), REMOVAL_WINDOW_MS + 1);
  // Act / Assert
  throwsWith(() => operatorReady(state, "rearm_required", 0), "rearm_exhausted");
  assert.equal(state.rearms, 2);
});

test("a stale operator reply names the wrong checkpoint and is refused", () => {
  // Arrange
  const state = removing();
  // Act / Assert
  throwsWith(() => operatorReady(state, "awaiting_operator_ready", 2000), "checkpoint_stale");
});
