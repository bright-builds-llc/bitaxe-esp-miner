import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  ATTEMPT_STATES,
  ATTEMPT_STATE_FIELDS,
  ATTEMPT_TRANSITIONS,
  BLOCKER_REASONS,
  EFFECT_AUTHORIZATION_STATES,
  EFFECT_IDS,
  PERMITTED_LIFECYCLE_KEYS,
  PROHIBITED_SENTINEL_KEYS,
  AttemptStateError,
  ambiguousEffectOnRestart,
  assertFreshBootSession,
  authorizeEffect,
  createAttemptState,
  deriveLinuxBootSessionDigest,
  deriveMacBootSessionDigest,
  finishEffect,
  markEffectInvoked,
  normalizeFinishedEffect,
  observeBootSessionDigest,
  routeBlocker,
  transitionAttemptState,
  validateAttemptState,
} from "./phase28.1.1-hardware-attempt-state.mjs";

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
  state.lifecycle_deadline_ms = 900_000;
  state.attestation_status = "accepted";
  state.attestation_accepted_ms = 500;
  state.usb_absence_started_ms = 500;
  state.usb_absence_ended_ms = 5_500;
  state.usb_absence_ms = 5_000;
  state.usb_absence_category = "continuous_at_least_5000";
  state.restore_token_status = "accepted";
  state.restore_accepted_ms = 5_600;
  state.usb_reappearance_ms = 6_000;
  state.reappearance_elapsed_ms = 5_500;
  state.reappearance_category = "within_60000";
  for (const key of PERMITTED_LIFECYCLE_KEYS) {
    state.capture_complete_permitted_lifecycle_counts[key] = 1;
  }
  state.armed_prohibited_sentinel_sha256 = DIGEST;
  state.capture_complete_prohibited_sentinel_sha256 = DIGEST;
  state.armed_permitted_lifecycle_sha256 = DIGEST;
  state.capture_complete_permitted_lifecycle_sha256 = DIGEST;
  state.capture_started_ms = 10_000;
  state.capture_ended_ms = 370_000;
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
  lateReappearance.reappearance_elapsed_ms = 60_001;
  expectStateError("state_malformed", () =>
    validateAttemptState(lateReappearance),
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

testBootSessionFixtures();
testClosedSchema();
testBootSessionReobservation();
testAttemptTransitionsAreClosed();
testEffectLifecycleForEveryEffect();
testBlockerRoutesAreExhaustive();
testTimingAndIdentityInvariants();
testSentinelInvariants();

console.log("phase28.1.1 hardware attempt state tests: passed");
