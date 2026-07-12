import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  ATTEMPT_STATES,
  ATTEMPT_STATE_FIELDS,
  ATTEMPT_TRANSITIONS,
  BLOCKER_REASONS,
  CHECKPOINT_DEFINITIONS,
  EFFECT_AUTHORIZATION_STATES,
  EFFECT_IDS,
  EFFECT_RESULT_SCHEMA_VERSION,
  PERMITTED_LIFECYCLE_KEYS,
  PROHIBITED_SENTINEL_KEYS,
  PUBLIC_CHECKPOINT_FIELDS,
  AttemptStateError,
  activeAttemptExpiryBlocker,
  applyEffectCompletion,
  applyLifecycleOwnerEvent,
  ambiguousEffectOnRestart,
  assertFreshBootSession,
  attachLifecycleOwner,
  authorizeEffect,
  classifyLostResumeHandleOrphanState,
  consumeCheckpoint,
  createConnectedEntryState,
  createAttemptState,
  finalizeClassifiedAttempt,
  deriveLinuxBootSessionDigest,
  deriveMacBootSessionDigest,
  finishEffect,
  markEffectInvoked,
  markClassifierInvoked,
  normalizeFinishedEffect,
  observeBootSessionDigest,
  publicCheckpoint,
  publicAction,
  persistStrictClassification,
  routeBlocker,
  terminalizeAttempt,
  transitionAttemptState,
  validateAttemptState,
} from "./phase28.1.1-hardware-attempt-state.mjs";
import { classifyStrictPostCaptureState } from "./phase28.1.1-strict-production-evidence.mjs";

const HEAD = "1".repeat(40);
const RESUME_DIGEST = "2".repeat(64);
const BOOT_DIGEST = "3".repeat(64);
const DIGEST = "4".repeat(64);

function baseState() {
  return createAttemptState({
    exactHead: HEAD,
    resumeHandleSha256: RESUME_DIGEST,
    createdMonotonicMs: 1_000,
    observeBootSession: () => BOOT_DIGEST,
    randomHex: () => "5".repeat(32),
  });
}

function expectStateError(code, action) {
  assert.throws(action, (error) => {
    assert.ok(error instanceof AttemptStateError);
    assert.equal(error.code, code);
    return true;
  });
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function completeLifecycleState() {
  const state = baseState();
  state.attempt_state = "capture_complete";
  state.reference_guard_state = "pass";
  state.reference_commit = "6".repeat(40);
  state.reference_guard_output_sha256 = DIGEST;
  state.selected_port_state = "one_board205";
  state.selected_port_fingerprint_sha256 = DIGEST;
  state.manifest_state = "pass";
  state.manifest_source_commit = HEAD;
  state.manifest_sha256 = DIGEST;
  state.factory_image_sha256 = DIGEST;
  state.reinit_capture_started_ms = 10;
  state.reinit_capture_ended_ms = 360_010;
  state.reinit_capture_duration_ms = 360_000;
  state.reinit_capture_category = "complete_360s";
  state.reinit_raw_log_sha256 = DIGEST;
  state.reinit_classifier_input_sha256 = DIGEST;
  state.reinit_five_stage_result = "pass";
  state.lifecycle_substate = "complete";
  state.lifecycle_lease_id = "7".repeat(32);
  state.lifecycle_capability_sha256 = DIGEST;
  state.lifecycle_owner_pid = 123;
  state.lifecycle_owner_start_fingerprint_sha256 = DIGEST;
  state.lifecycle_deadline_ms = 4_145_600;
  state.attestation_status = "accepted";
  state.attestation_accepted_ms = 500;
  state.usb_absence_started_ms = 500;
  state.usb_absence_ended_ms = 5_500;
  state.usb_absence_ms = 5_000;
  state.usb_absence_category = "continuous_at_least_5000";
  state.restore_watcher_status = "attached";
  state.restore_watcher_armed_ms = 5_600;
  state.restore_watcher_deadline_ms = 1_805_600;
  state.usb_reappearance_ms = 6_000;
  state.reappearance_elapsed_ms = 400;
  state.reappearance_category = "within_1800000";
  state.monitor_attachment_ms = 6_100;
  state.monitor_attachment_elapsed_ms = 100;
  state.monitor_attachment_category = "within_60000";
  for (const key of PERMITTED_LIFECYCLE_KEYS) {
    state.capture_complete_permitted_lifecycle_counts[key] = 1;
  }
  state.armed_prohibited_sentinel_sha256 = DIGEST;
  state.capture_complete_prohibited_sentinel_sha256 = DIGEST;
  state.armed_permitted_lifecycle_sha256 = DIGEST;
  state.capture_complete_permitted_lifecycle_sha256 = DIGEST;
  state.capture_started_ms = 6_100;
  state.capture_ended_ms = 366_100;
  state.capture_duration_ms = 360_000;
  state.capture_category = "complete_360s";
  state.lifecycle_raw_log_sha256 = DIGEST;
  state.same_chain_raw_log_set_sha256 = DIGEST;
  state.classifier_input_sha256 = DIGEST;
  state.lifecycle_status = "match";
  state.result_correlated = false;
  state.power_delta_class = "flat";
  state.share_submission_status = "not_observed";
  return state;
}

function testBootSessionFixtures() {
  // Arrange
  const uuid = "01234567-89ab-cdef-0123-456789abcdef";
  const expectedLinux = sha256(
    Buffer.concat([
      Buffer.from("linux-boot-id-v1\0"),
      Buffer.from(uuid),
    ]),
  );
  const expectedMac = sha256(
    Buffer.concat([
      Buffer.from("macos-kern-boottime-v1\0"),
      Buffer.from("sec=123;usec=456"),
    ]),
  );

  // Act
  const linux = deriveLinuxBootSessionDigest(` \t${uuid}\r\n`);
  const mac = deriveMacBootSessionDigest(
    "{ sec = 123, usec = 456 } Sat Jul 11 00:00:00 2026",
  );
  const observedLinux = observeBootSessionDigest({
    platform: "linux",
    readLinuxBootId: () => `${uuid}\n`,
  });
  const observedMac = observeBootSessionDigest({
    platform: "darwin",
    readMacBootTime: () => "{ sec = 123, usec = 456 }",
  });

  // Assert
  assert.equal(linux, expectedLinux);
  assert.equal(mac, expectedMac);
  assert.equal(observedLinux, expectedLinux);
  assert.equal(observedMac, expectedMac);
  assert.throws(() => deriveLinuxBootSessionDigest("NOT-A-UUID"));
  assert.throws(() => deriveMacBootSessionDigest("unavailable"));
  assert.throws(() =>
    observeBootSessionDigest({
      platform: "linux",
      readLinuxBootId: () => {
        throw new Error("missing");
      },
    }),
  );
}

function testClosedSchema() {
  // Arrange
  const state = baseState();

  // Act / Assert
  assert.equal(validateAttemptState(state), state);
  for (const field of ATTEMPT_STATE_FIELDS) {
    const missing = structuredClone(state);
    delete missing[field];
    expectStateError("state_missing_field", () => validateAttemptState(missing));
  }
  const unknown = { ...state, unexpected_field: true };
  expectStateError("state_unknown_field", () => validateAttemptState(unknown));
  const invalidNull = { ...state, exact_head: null };
  expectStateError("state_malformed", () => validateAttemptState(invalidNull));
}

function testBootSessionReobservation() {
  // Arrange
  const state = baseState();

  // Act / Assert
  assert.equal(assertFreshBootSession(state, () => BOOT_DIGEST), BOOT_DIGEST);
  expectStateError("boot_session_mismatch", () =>
    assertFreshBootSession(state, () => "9".repeat(64)),
  );
  expectStateError("boot_session_observation_unavailable", () =>
    assertFreshBootSession(state, () => {
      throw new Error("unavailable");
    }),
  );
}

function testAttemptTransitionsAreClosed() {
  // Arrange
  const state = baseState();

  // Act
  const waiting = transitionAttemptState(state, "connected_entry_waiting");

  // Assert
  assert.equal(waiting.attempt_state, "connected_entry_waiting");
  assert.deepEqual(Object.keys(ATTEMPT_TRANSITIONS).sort(), [...ATTEMPT_STATES].sort());
  for (const [from, targets] of Object.entries(ATTEMPT_TRANSITIONS)) {
    assert.ok(targets.every((target) => ATTEMPT_STATES.includes(target)), from);
  }
  expectStateError("state_malformed", () =>
    transitionAttemptState(state, "packaged"),
  );
}

function testEffectLifecycleForEveryEffect() {
  for (const effectId of EFFECT_IDS) {
    // Arrange
    const state = baseState();
    state.attempt_state = EFFECT_AUTHORIZATION_STATES[effectId][0];
    const nonce = "a".repeat(32);

    // Act
    const authorized = authorizeEffect(state, effectId, nonce);
    const invoked = markEffectInvoked(
      authorized,
      nonce,
      authorized.effect_sequence,
    );
    const completed = finishEffect(
      invoked,
      nonce,
      invoked.effect_sequence,
      true,
    );
    const idle = normalizeFinishedEffect(completed);

    // Assert
    assert.equal(authorized.effect_phase, "authorized");
    assert.equal(authorized.effect_sequence, 1);
    assert.equal(ambiguousEffectOnRestart(authorized), true);
    assert.equal(ambiguousEffectOnRestart(invoked), true);
    assert.equal(completed.effect_phase, "completed");
    assert.equal(idle.effect_phase, "none");
    assert.equal(idle.effect_id, null);
    assert.equal(idle.effect_authorization_nonce, null);
    expectStateError("effect_in_flight_ambiguous", () =>
      markEffectInvoked(authorized, "b".repeat(32), 1),
    );
  }
}

function testBlockerRoutesAreExhaustive() {
  for (const blocker of BLOCKER_REASONS) {
    // Arrange / Act
    const before = routeBlocker(blocker, { lifecycleAuthorized: false });
    const after = routeBlocker(blocker, { lifecycleAuthorized: true });

    // Assert
    if (blocker === "none") {
      assert.equal(before, null);
      assert.equal(after, null);
    } else if (blocker === "process_cleanup_unresolved") {
      assert.equal(before, "blocked_safe_unresolved_process");
      assert.equal(after, "blocked_safe_unresolved_process");
    } else if (
      [
        "sentinel_mismatch",
        "classifier_input_invalid",
        "classification_inconsistent",
      ].includes(blocker)
    ) {
      assert.equal(before, "blocked_safe_evidence_invalid");
      assert.equal(after, "blocked_safe_evidence_invalid");
    } else if (blocker === "production_markers_absent") {
      assert.equal(before, "blocked_safe_evidence_invalid");
      assert.equal(after, "gaps_found_same_chain_production_markers_absent");
    } else {
      assert.equal(before, "blocked_safe_attempt_prerequisite", blocker);
      assert.equal(after, "blocked_safe_evidence_invalid", blocker);
    }
  }
}

function testTimingAndIdentityInvariants() {
  // Arrange
  const valid = completeLifecycleState();

  // Act / Assert
  assert.equal(validateAttemptState(valid), valid);
  const manifestMismatch = structuredClone(valid);
  manifestMismatch.manifest_source_commit = "f".repeat(40);
  expectStateError("state_malformed", () =>
    validateAttemptState(manifestMismatch),
  );
  const shortCapture = structuredClone(valid);
  shortCapture.capture_ended_ms -= 1;
  shortCapture.capture_duration_ms -= 1;
  expectStateError("state_malformed", () => validateAttemptState(shortCapture));
  const inconsistentDuration = structuredClone(valid);
  inconsistentDuration.usb_absence_ms = 5_001;
  expectStateError("state_malformed", () =>
    validateAttemptState(inconsistentDuration),
  );
  const shortAbsence = structuredClone(valid);
  shortAbsence.usb_absence_ended_ms = 5_499;
  shortAbsence.usb_absence_ms = 4_999;
  expectStateError("state_malformed", () => validateAttemptState(shortAbsence));
  const lateReappearance = structuredClone(valid);
  lateReappearance.reappearance_elapsed_ms = 1_800_001;
  expectStateError("state_malformed", () =>
    validateAttemptState(lateReappearance),
  );
  const lateAttachment = structuredClone(valid);
  lateAttachment.monitor_attachment_elapsed_ms = 60_001;
  expectStateError("state_malformed", () =>
    validateAttemptState(lateAttachment),
  );
}

function testSentinelInvariants() {
  // Arrange
  const valid = completeLifecycleState();

  // Act / Assert
  for (const key of PROHIBITED_SENTINEL_KEYS) {
    const changed = structuredClone(valid);
    changed.capture_complete_prohibited_sentinel_counts[key] += 1;
    expectStateError("sentinel_mismatch", () => validateAttemptState(changed));
  }
  for (const key of PERMITTED_LIFECYCLE_KEYS) {
    const missing = structuredClone(valid);
    missing.capture_complete_permitted_lifecycle_counts[key] = 0;
    expectStateError("sentinel_mismatch", () => validateAttemptState(missing));
  }
  const armed = baseState();
  armed.attempt_state = "armed";
  armed.armed_permitted_lifecycle_counts.removal_pty_write_count = 1;
  expectStateError("sentinel_mismatch", () => validateAttemptState(armed));
}

function completedEffect(state, effectId, result) {
  const nonce = "a".repeat(32);
  const authorized = authorizeEffect(state, effectId, nonce);
  const invoked = markEffectInvoked(
    authorized,
    nonce,
    authorized.effect_sequence,
  );
  const finished = finishEffect(
    invoked,
    nonce,
    invoked.effect_sequence,
    result.status === "completed",
  );
  return applyEffectCompletion(finished, result, {
    completedMonotonicMs: 2_000,
  });
}

function effectResult(effectId, outputs, status = "completed", blocker = "none") {
  return {
    schema_version: EFFECT_RESULT_SCHEMA_VERSION,
    effect_id: effectId,
    status,
    blocker_reason: blocker,
    outputs,
  };
}

function testConnectedCheckpointContract() {
  // Arrange
  const handle = "6".repeat(64);
  const state = createConnectedEntryState({
    exactHead: HEAD,
    resumeHandleSha256: sha256(handle),
    createdMonotonicMs: 1_000,
    observeBootSession: () => BOOT_DIGEST,
    randomHex: () => "5".repeat(32),
  });

  // Act
  const handoff = publicCheckpoint(state, handle);
  const consumed = consumeCheckpoint(state, {
    checkpointToken: "plan13-connected-entry-v1",
    responseToken: "ultra205-remains-connected",
    nowMonotonicMs: 1_001,
  });

  // Assert
  assert.deepEqual(Object.keys(handoff), [...PUBLIC_CHECKPOINT_FIELDS]);
  assert.equal(handoff.checkpoint_id, "plan13-connected-entry");
  assert.equal(handoff.capture_category, "not_started");
  assert.equal(handoff.monotonic_deadline_ms, 601_000);
  assert.equal(consumed.checkpoint_id, null);
  assert.equal(consumed.attempt_state, "connected_entry_waiting");
  expectStateError("checkpoint_state_mismatch", () =>
    consumeCheckpoint(consumed, {
      checkpointToken: "plan13-connected-entry-v1",
      responseToken: "ultra205-remains-connected",
      nowMonotonicMs: 1_002,
    }),
  );
  expectStateError("checkpoint_expired", () =>
    consumeCheckpoint(state, {
      checkpointToken: "plan13-connected-entry-v1",
      responseToken: "ultra205-remains-connected",
      nowMonotonicMs: 601_000,
    }),
  );
}

function testLostResumeHandleOrphanStateContract() {
  // Arrange
  const state = createConnectedEntryState({
    exactHead: HEAD,
    resumeHandleSha256: RESUME_DIGEST,
    createdMonotonicMs: 1_000,
    observeBootSession: () => BOOT_DIGEST,
    randomHex: () => "5".repeat(32),
  });
  const request = {
    expectedHead: HEAD,
    expectedState: "connected_entry_waiting",
    reason: "lost_resume_handle",
    observedBootSessionSha256: BOOT_DIGEST,
  };

  // Act
  const initialCategory = classifyLostResumeHandleOrphanState(state, request);
  const terminalCategory = classifyLostResumeHandleOrphanState(
    terminalizeAttempt(state, "cancelled_or_abandoned"),
    request,
  );

  // Assert
  assert.equal(initialCategory, "connected_entry_waiting");
  assert.equal(terminalCategory, "terminal_recovery");
  assert.equal(state.capture_category, "not_started");
  for (const changedState of [
    { ...state, exact_head: "f".repeat(40) },
    { ...state, effect_sequence: 1 },
    { ...state, capture_category: "running", capture_started_ms: 1_001 },
  ]) {
    expectStateError("orphan_cleanup_ineligible", () =>
      classifyLostResumeHandleOrphanState(changedState, request),
    );
  }
  expectStateError("orphan_cleanup_ineligible", () =>
    classifyLostResumeHandleOrphanState(state, {
      ...request,
      observedBootSessionSha256: "f".repeat(64),
    }),
  );
}

function testPhysicalActionCheckpointTimeoutPolicy() {
  // Arrange
  const physicalCheckpointIds = Object.keys(CHECKPOINT_DEFINITIONS).filter(
    (checkpointId) => checkpointId !== "plan13-connected-entry",
  );

  // Act / Assert
  for (const checkpointId of physicalCheckpointIds) {
    assert.equal(
      CHECKPOINT_DEFINITIONS[checkpointId].timeoutMs,
      1_800_000,
      checkpointId,
    );
  }
}

function testActiveAttemptExpiryClassification() {
  // Arrange
  const checkpointState = createConnectedEntryState({
    exactHead: HEAD,
    resumeHandleSha256: RESUME_DIGEST,
    createdMonotonicMs: 1_000,
    observeBootSession: () => BOOT_DIGEST,
    randomHex: () => "5".repeat(32),
  });
  const lifecycleState = completeLifecycleState();
  lifecycleState.attempt_state = "restore_watcher_armed";
  lifecycleState.lifecycle_substate = "restore_watcher_armed";
  lifecycleState.checkpoint_id = null;
  lifecycleState.checkpoint_token = null;
  lifecycleState.expected_response_token = null;
  lifecycleState.expected_user_action = null;
  lifecycleState.monotonic_deadline_ms = null;
  lifecycleState.restore_watcher_status = "armed";
  lifecycleState.restore_watcher_armed_ms = 1_000;
  lifecycleState.restore_watcher_deadline_ms = 1_801_000;
  lifecycleState.usb_reappearance_ms = 1_400;
  lifecycleState.reappearance_elapsed_ms = 400;
  lifecycleState.monitor_attachment_ms = 1_500;
  lifecycleState.monitor_attachment_elapsed_ms = 100;
  lifecycleState.capture_started_ms = 1_500;
  lifecycleState.capture_ended_ms = 361_500;
  lifecycleState.lifecycle_deadline_ms = 4_145_000;
  lifecycleState.process_running = true;
  lifecycleState.effect_id = "lifecycle_start";
  lifecycleState.effect_phase = "invoked";
  lifecycleState.effect_authorization_nonce = "8".repeat(32);
  const leaseState = structuredClone(lifecycleState);
  leaseState.attempt_state = "reappearance_observing";
  leaseState.lifecycle_substate = "reappearance_observing";
  leaseState.restore_watcher_status = "appearance_observed";
  leaseState.reappearance_category = "within_1800000";

  // Act
  const beforeCheckpointDeadline = activeAttemptExpiryBlocker(
    checkpointState,
    600_999,
  );
  const atCheckpointDeadline = activeAttemptExpiryBlocker(
    checkpointState,
    601_000,
  );
  const atLeaseDeadline = activeAttemptExpiryBlocker(
    leaseState,
    4_145_000,
  );

  // Assert
  assert.equal(beforeCheckpointDeadline, null);
  assert.equal(atCheckpointDeadline, "checkpoint_expired");
  assert.equal(atLeaseDeadline, "checkpoint_expired");
  assert.equal(
    activeAttemptExpiryBlocker(lifecycleState, 1_801_000),
    "usb_appearance_timeout",
  );
}

function testDetectorEffectTransitionsAndRecoveryOrder() {
  // Arrange
  let state = baseState();
  state.attempt_state = "connected_entry_waiting";

  // Act
  state = completedEffect(
    state,
    "detector_board_info",
    effectResult(
      "detector_board_info",
      {},
      "failed",
      "detector_failed",
    ),
  );

  // Assert
  assert.equal(state.attempt_state, "recovery_waiting");
  assert.equal(state.checkpoint_id, "plan13-recovery-usb-replug");
  assert.equal(state.detector_attempt_count, 1);
  assert.deepEqual(
    Object.keys(CHECKPOINT_DEFINITIONS),
    [
      "plan13-connected-entry",
      "plan13-recovery-usb-replug",
      "plan13-recovery-both-power",
      "plan13-recovery-reset",
      "plan13-recovery-boot-reset",
      "plan13-lifecycle-removal",
    ],
  );
  state = consumeCheckpoint(state, {
    checkpointToken: "plan13-usb-replug-recovery-v1",
    responseToken: "plan13-usb-replugged-v1",
    nowMonotonicMs: 2_001,
  });
  assert.equal(state.usb_replug_count, 1);
  const passed = completedEffect(
    state,
    "detector_board_info",
    effectResult("detector_board_info", {
      selected_port_fingerprint_sha256: DIGEST,
    }),
  );
  assert.equal(passed.attempt_state, "detector_passed");
  assert.equal(passed.detector_attempt_count, 2);
  assert.equal(passed.selected_port_state, "one_board205");
}

function lifecycleReadyState() {
  const state = baseState();
  state.attempt_state = "reinit_validated";
  state.reference_guard_state = "pass";
  state.reference_commit = "6".repeat(40);
  state.reference_guard_output_sha256 = DIGEST;
  state.selected_port_state = "one_board205";
  state.selected_port_fingerprint_sha256 = DIGEST;
  state.manifest_state = "pass";
  state.manifest_source_commit = HEAD;
  state.manifest_sha256 = DIGEST;
  state.factory_image_sha256 = DIGEST;
  state.reinit_capture_started_ms = 10;
  state.reinit_capture_ended_ms = 360_010;
  state.reinit_capture_duration_ms = 360_000;
  state.reinit_capture_category = "complete_360s";
  state.reinit_raw_log_sha256 = DIGEST;
  state.reinit_classifier_input_sha256 = DIGEST;
  state.reinit_five_stage_result = "pass";
  return state;
}

function testLifecycleLeaseAndSubordinateTransitions() {
  // Arrange
  const state = lifecycleReadyState();
  const nonce = "a".repeat(32);
  const authorized = authorizeEffect(state, "lifecycle_start", nonce);
  const invoked = markEffectInvoked(
    authorized,
    nonce,
    authorized.effect_sequence,
  );

  // Act
  let next = attachLifecycleOwner(invoked, {
    leaseId: "7".repeat(32),
    capabilitySha256: DIGEST,
    ownerPid: 123,
    ownerStartFingerprintSha256: DIGEST,
    lifecycleDeadlineMs: 4_146_000,
    checkpointCreatedMonotonicMs: 1_000,
  });
  const removalCheckpoint = structuredClone(next);
  next = consumeCheckpoint(next, {
    checkpointToken: "plan13-armed-removal-v1",
    responseToken: "plan13-both-power-paths-removed",
    nowMonotonicMs: 1_001,
  });
  next = applyLifecycleOwnerEvent(next, "absence-observing", {
    usb_absence_started_ms: 1_001,
  });
  next = applyLifecycleOwnerEvent(next, "restore-watcher-armed", {
    usb_absence_ended_ms: 6_001,
    usb_absence_ms: 5_000,
    restore_watcher_armed_ms: 6_001,
    restore_watcher_deadline_ms: 1_806_001,
  });
  const restoreWatcher = structuredClone(next);
  next = applyLifecycleOwnerEvent(next, "usb-reappearance-observed", {
    usb_reappearance_ms: 7_000,
    reappearance_elapsed_ms: 999,
  });
  next = applyLifecycleOwnerEvent(next, "capture-running", {
    monitor_attachment_ms: 7_001,
    monitor_attachment_elapsed_ms: 1,
    capture_started_ms: 7_001,
  });
  next = applyLifecycleOwnerEvent(next, "capture-complete", {
    capture_ended_ms: 367_001,
    capture_duration_ms: 360_000,
    lifecycle_raw_log_sha256: DIGEST,
    same_chain_raw_log_set_sha256: DIGEST,
    classifier_input_sha256: DIGEST,
    lifecycle_status: "match",
    result_correlated: false,
    power_delta_class: "flat",
    share_submission_status: "not_observed",
  });
  const finished = finishEffect(
    next,
    nonce,
    next.effect_sequence,
    true,
  );
  const completed = applyEffectCompletion(
    finished,
    effectResult("lifecycle_start", {}),
    { completedMonotonicMs: 367_002 },
  );

  // Assert
  assert.equal(completed.attempt_state, "capture_complete");
  assert.equal(completed.lifecycle_substate, "complete");
  assert.equal(completed.process_running, false);
  assert.equal(completed.capture_category, "complete_360s");
  assert.equal(removalCheckpoint.created_monotonic_ms, 1_000);
  assert.equal(removalCheckpoint.monotonic_deadline_ms, 1_801_000);
  assert.equal(
    removalCheckpoint.lifecycle_deadline_ms -
      removalCheckpoint.created_monotonic_ms,
    4_145_000,
  );
  assert.doesNotThrow(() =>
    consumeCheckpoint(removalCheckpoint, {
      checkpointToken: "plan13-armed-removal-v1",
      responseToken: "plan13-both-power-paths-removed",
      nowMonotonicMs: 1_800_999,
    }),
  );
  expectStateError("checkpoint_expired", () =>
    consumeCheckpoint(removalCheckpoint, {
      checkpointToken: "plan13-armed-removal-v1",
      responseToken: "plan13-both-power-paths-removed",
      nowMonotonicMs: 1_801_000,
    }),
  );
  assert.equal(restoreWatcher.checkpoint_id, null);
  assert.equal(restoreWatcher.restore_watcher_deadline_ms, 1_806_001);
  assert.equal(
    activeAttemptExpiryBlocker(restoreWatcher, 1_806_001),
    "usb_appearance_timeout",
  );
  expectStateError("lease_conflict", () =>
    attachLifecycleOwner(invoked, {
      leaseId: "7".repeat(32),
      capabilitySha256: DIGEST,
      ownerPid: 123,
      ownerStartFingerprintSha256: DIGEST,
      lifecycleDeadlineMs: 4_145_999,
      checkpointCreatedMonotonicMs: 1_000,
    }),
  );
  for (const key of PERMITTED_LIFECYCLE_KEYS) {
    assert.equal(completed.capture_complete_permitted_lifecycle_counts[key], 1);
  }
}

function testRestoreWatcherActionHasNoResponseSurface() {
  // Arrange
  const state = lifecycleReadyState();
  const nonce = "a".repeat(32);
  const authorized = authorizeEffect(state, "lifecycle_start", nonce);
  const invoked = markEffectInvoked(
    authorized,
    nonce,
    authorized.effect_sequence,
  );
  let next = attachLifecycleOwner(invoked, {
    leaseId: "7".repeat(32),
    capabilitySha256: DIGEST,
    ownerPid: 123,
    ownerStartFingerprintSha256: DIGEST,
    lifecycleDeadlineMs: 4_146_000,
    checkpointCreatedMonotonicMs: 1_000,
  });
  next = consumeCheckpoint(next, {
    checkpointToken: "plan13-armed-removal-v1",
    responseToken: "plan13-both-power-paths-removed",
    nowMonotonicMs: 1_001,
  });
  next = applyLifecycleOwnerEvent(next, "absence-observing", {
    usb_absence_started_ms: 1_001,
  });
  next = applyLifecycleOwnerEvent(next, "restore-watcher-armed", {
    usb_absence_ended_ms: 6_001,
    usb_absence_ms: 5_000,
    restore_watcher_armed_ms: 6_001,
    restore_watcher_deadline_ms: 1_806_001,
  });

  // Act
  const action = publicAction(next);

  // Assert
  assert.deepEqual(action, {
    action_id: "plan13-lifecycle-restore",
    action_token: "plan13-restore-watcher-armed-v1",
    attempt_state: "restore_watcher_armed",
    expected_user_action: "restore-barrel-then-usb",
    response_required: false,
  });
  assert.equal(next.checkpoint_id, null);
  expectStateError("checkpoint_state_mismatch", () =>
    consumeCheckpoint(next, {
      checkpointToken: "plan13-barrel-usb-restore-v1",
      responseToken: "plan13-barrel-then-usb-restored",
      nowMonotonicMs: 6_002,
    }),
  );
}

function testStrictClassificationPersistenceAndReplayGuards() {
  // Arrange
  const state = completeLifecycleState();
  state.attempt_state = "post_capture_validated";
  state.effect_sequence = 7;
  state.classifier_input_sha256 = null;
  const invoked = markClassifierInvoked(state);
  const result = classifyStrictPostCaptureState(invoked);

  // Act
  const classified = persistStrictClassification(invoked, result);
  const terminal = finalizeClassifiedAttempt(classified);

  // Assert
  assert.equal(classified.attempt_state, "classified");
  assert.equal(
    classified.terminal_outcome,
    "gaps_found_same_chain_production_markers_absent",
  );
  assert.equal(classified.verification_result, "gaps_found");
  assert.equal(classified.phase30_promotion_input, "pending");
  assert.equal(terminal.attempt_state, "terminal");
  const incomplete = { ...result };
  delete incomplete.classifier_output_sha256;
  expectStateError("state_malformed", () =>
    persistStrictClassification(invoked, incomplete),
  );
  expectStateError("classifier_input_invalid", () =>
    persistStrictClassification(invoked, {
      ...result,
      classifier_input_sha256: "f".repeat(64),
    }),
  );
  expectStateError("classification_inconsistent", () =>
    persistStrictClassification(invoked, {
      ...result,
      classifier_output_sha256: "f".repeat(64),
    }),
  );
  expectStateError("classification_inconsistent", () =>
    persistStrictClassification(classified, result),
  );
}

testBootSessionFixtures();
testClosedSchema();
testBootSessionReobservation();
testAttemptTransitionsAreClosed();
testEffectLifecycleForEveryEffect();
testBlockerRoutesAreExhaustive();
testTimingAndIdentityInvariants();
testSentinelInvariants();
testConnectedCheckpointContract();
testPhysicalActionCheckpointTimeoutPolicy();
testLostResumeHandleOrphanStateContract();
testActiveAttemptExpiryClassification();
testDetectorEffectTransitionsAndRecoveryOrder();
testLifecycleLeaseAndSubordinateTransitions();
testRestoreWatcherActionHasNoResponseSurface();
testStrictClassificationPersistenceAndReplayGuards();

console.log("phase28.1.1 hardware attempt state tests: passed");
