import assert from "node:assert/strict";
import {
  buildClassifierProjection,
  classifyStrictProductionEvidence,
  validateClassifierProjection,
} from "./phase28.1.1-strict-production-evidence.mjs";

const DIGEST = "a".repeat(64);

function classifierState() {
  return {
    exact_head: "1".repeat(40),
    boot_session_sha256: DIGEST,
    reference_commit: "2".repeat(40),
    manifest_sha256: DIGEST,
    reinit_raw_log_sha256: DIGEST,
    lifecycle_raw_log_sha256: DIGEST,
    same_chain_raw_log_set_sha256: DIGEST,
    capture_started_ms: 1_000,
    capture_ended_ms: 361_000,
    capture_duration_ms: 360_000,
    armed_prohibited_sentinel_sha256: DIGEST,
    capture_complete_prohibited_sentinel_sha256: DIGEST,
    armed_permitted_lifecycle_sha256: "b".repeat(64),
    capture_complete_permitted_lifecycle_sha256: "c".repeat(64),
    attempt_state: "capture_complete",
    lifecycle_substate: "complete",
    lifecycle_status: "match",
    process_running: false,
    cleanup_state: "complete",
    result_correlated: false,
    power_delta_class: "flat",
    share_submission_status: "not_observed",
    blocker_reason: "none",
  };
}

function projection(overrides = {}) {
  return buildClassifierProjection({ ...classifierState(), ...overrides });
}

function assertTerminal(input, terminal, verification, promotion) {
  // Act
  const result = classifyStrictProductionEvidence(input);

  // Assert
  assert.equal(result.terminal_outcome, terminal);
  assert.equal(result.verification_result, verification);
  assert.equal(result.phase30_promotion_input, promotion);
  assert.match(result.classifier_output_sha256, /^[0-9a-f]{64}$/);
}

function testAllFiveTerminals() {
  // Arrange / Act / Assert
  assertTerminal(
    projection({
      attempt_state: "new",
      lifecycle_substate: "not_started",
      blocker_reason: "detector_failed",
    }),
    "blocked_safe_attempt_prerequisite",
    "gaps_found",
    "pending",
  );
  assertTerminal(
    projection({
      cleanup_state: "unresolved",
      process_running: true,
      blocker_reason: "process_cleanup_unresolved",
    }),
    "blocked_safe_unresolved_process",
    "gaps_found",
    "pending",
  );
  assertTerminal(
    projection({ lifecycle_status: "mismatch" }),
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
  assertTerminal(
    projection(),
    "gaps_found_same_chain_production_markers_absent",
    "gaps_found",
    "pending",
  );
  assertTerminal(
    projection({
      result_correlated: true,
      power_delta_class: "rising_hashing",
      share_submission_status: "accepted",
    }),
    "passed_same_chain_hardware",
    "passed",
    "eligible",
  );
}

function testInvalidLifecycleAlwaysUsesEvidenceInvalid() {
  for (const lifecycleStatus of ["absent", "incomplete", "mismatch"]) {
    // Arrange
    const input = projection({
      lifecycle_status: lifecycleStatus,
      result_correlated: true,
      power_delta_class: "rising_hashing",
      share_submission_status: "rejected",
    });

    // Act / Assert
    assertTerminal(
      input,
      "blocked_safe_evidence_invalid",
      "gaps_found",
      "pending",
    );
  }
}

function testSuccessRequiresExactConjunction() {
  const cases = [
    { result_correlated: false },
    { power_delta_class: "flat" },
    { share_submission_status: "not_observed" },
    { share_submission_status: null },
  ];
  for (const missing of cases) {
    // Arrange
    const input = projection({
      result_correlated: true,
      power_delta_class: "rising_hashing",
      share_submission_status: "accepted",
      ...missing,
    });

    // Act / Assert
    assertTerminal(
      input,
      "gaps_found_same_chain_production_markers_absent",
      "gaps_found",
      "pending",
    );
  }
}

function testProjectionIsClosedAndDigestCommitted() {
  // Arrange
  const valid = projection();

  // Act / Assert
  assert.equal(validateClassifierProjection(valid), valid);
  const missing = { ...valid };
  delete missing.manifest_sha256;
  assertTerminal(
    missing,
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
  const unknown = { ...valid, unknown: true };
  assertTerminal(
    unknown,
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
  const tampered = { ...valid, exact_head: "f".repeat(40) };
  assertTerminal(
    tampered,
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
  const changedSentinel = {
    ...valid,
    capture_complete_prohibited_sentinel_sha256: "d".repeat(64),
  };
  assertTerminal(
    changedSentinel,
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
}

function testPostLifecycleBlockerCannotBecomeProductionNegative() {
  // Arrange
  const input = projection({ blocker_reason: "monitor_failed" });

  // Act / Assert
  assertTerminal(
    input,
    "blocked_safe_evidence_invalid",
    "gaps_found",
    "pending",
  );
}

testAllFiveTerminals();
testInvalidLifecycleAlwaysUsesEvidenceInvalid();
testSuccessRequiresExactConjunction();
testProjectionIsClosedAndDigestCommitted();
testPostLifecycleBlockerCannotBecomeProductionNegative();

console.log("phase28.1.1 strict production evidence tests: passed");
