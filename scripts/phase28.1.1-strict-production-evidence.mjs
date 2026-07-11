import {
  PHASE28_CLASSIFIER_VERSION,
  TERMINAL_IDS,
  canonicalDigest,
  routeBlocker,
} from "./phase28.1.1-hardware-attempt-state.mjs";

export const CLASSIFIER_PROJECTION_FIELDS = Object.freeze([
  "exact_head",
  "boot_session_sha256",
  "reference_commit",
  "manifest_sha256",
  "reinit_raw_log_sha256",
  "lifecycle_raw_log_sha256",
  "same_chain_raw_log_set_sha256",
  "capture_started_ms",
  "capture_ended_ms",
  "capture_duration_ms",
  "armed_prohibited_sentinel_sha256",
  "capture_complete_prohibited_sentinel_sha256",
  "armed_permitted_lifecycle_sha256",
  "capture_complete_permitted_lifecycle_sha256",
  "lifecycle_authorized",
  "lifecycle_status",
  "process_running",
  "cleanup_state",
  "result_correlated",
  "power_delta_class",
  "share_submission_status",
  "blocker_reason",
  "classifier_version",
  "classifier_input_sha256",
]);

const DIGEST_FIELDS = Object.freeze([
  "boot_session_sha256",
  "manifest_sha256",
  "reinit_raw_log_sha256",
  "lifecycle_raw_log_sha256",
  "same_chain_raw_log_set_sha256",
  "armed_prohibited_sentinel_sha256",
  "capture_complete_prohibited_sentinel_sha256",
  "armed_permitted_lifecycle_sha256",
  "capture_complete_permitted_lifecycle_sha256",
]);

function assertExactKeys(value, keys) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("classifier_input_invalid: projection must be an object");
  }
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length ||
    actual.some((key, index) => key !== expected[index])
  ) {
    throw new Error("classifier_input_invalid: projection key set is not closed");
  }
}

function projectionCommitment(projection) {
  const committed = { ...projection };
  delete committed.classifier_input_sha256;
  return canonicalDigest(committed);
}

export function buildClassifierProjection(state) {
  const projection = {
    exact_head: state.exact_head,
    boot_session_sha256: state.boot_session_sha256,
    reference_commit: state.reference_commit,
    manifest_sha256: state.manifest_sha256,
    reinit_raw_log_sha256: state.reinit_raw_log_sha256,
    lifecycle_raw_log_sha256: state.lifecycle_raw_log_sha256,
    same_chain_raw_log_set_sha256: state.same_chain_raw_log_set_sha256,
    capture_started_ms: state.capture_started_ms,
    capture_ended_ms: state.capture_ended_ms,
    capture_duration_ms: state.capture_duration_ms,
    armed_prohibited_sentinel_sha256: state.armed_prohibited_sentinel_sha256,
    capture_complete_prohibited_sentinel_sha256:
      state.capture_complete_prohibited_sentinel_sha256,
    armed_permitted_lifecycle_sha256:
      state.armed_permitted_lifecycle_sha256,
    capture_complete_permitted_lifecycle_sha256:
      state.capture_complete_permitted_lifecycle_sha256,
    lifecycle_authorized: state.attempt_state !== "new" && state.lifecycle_substate !== "not_started",
    lifecycle_status: state.lifecycle_status,
    process_running: state.process_running,
    cleanup_state: state.cleanup_state,
    result_correlated: state.result_correlated,
    power_delta_class: state.power_delta_class,
    share_submission_status: state.share_submission_status,
    blocker_reason: state.blocker_reason,
    classifier_version: PHASE28_CLASSIFIER_VERSION,
    classifier_input_sha256: null,
  };
  projection.classifier_input_sha256 = projectionCommitment(projection);
  return projection;
}

export function validateClassifierProjection(projection) {
  assertExactKeys(projection, CLASSIFIER_PROJECTION_FIELDS);
  if (!/^[0-9a-f]{40}$/.test(projection.exact_head)) {
    throw new Error("classifier_input_invalid: exact_head is malformed");
  }
  for (const field of DIGEST_FIELDS) {
    if (!/^[0-9a-f]{64}$/.test(projection[field] ?? "")) {
      throw new Error(`classifier_input_invalid: ${field} is malformed`);
    }
  }
  if (
    !/^[0-9a-f]{40}$/.test(projection.reference_commit ?? "") ||
    projection.classifier_version !== PHASE28_CLASSIFIER_VERSION
  ) {
    throw new Error("classifier_input_invalid: identity or classifier version mismatch");
  }
  if (
    !Number.isSafeInteger(projection.capture_started_ms) ||
    !Number.isSafeInteger(projection.capture_ended_ms) ||
    !Number.isSafeInteger(projection.capture_duration_ms) ||
    projection.capture_started_ms < 0 ||
    projection.capture_ended_ms < projection.capture_started_ms ||
    projection.capture_duration_ms !==
      projection.capture_ended_ms - projection.capture_started_ms ||
    projection.capture_duration_ms < 360_000
  ) {
    throw new Error("classifier_input_invalid: capture timing is inconsistent");
  }
  if (
    typeof projection.lifecycle_authorized !== "boolean" ||
    !["absent", "incomplete", "mismatch", "match"].includes(
      projection.lifecycle_status,
    ) ||
    typeof projection.process_running !== "boolean" ||
    !["not_needed", "complete", "unresolved"].includes(
      projection.cleanup_state,
    ) ||
    ![true, false, null].includes(projection.result_correlated) ||
    !["rising_hashing", "flat", "falling", "unavailable", null].includes(
      projection.power_delta_class,
    ) ||
    !["accepted", "rejected", "not_observed", null].includes(
      projection.share_submission_status,
    )
  ) {
    throw new Error("classifier_input_invalid: result domain is invalid");
  }
  if (
    projection.armed_prohibited_sentinel_sha256 !==
    projection.capture_complete_prohibited_sentinel_sha256
  ) {
    throw new Error("classifier_input_invalid: prohibited sentinel digest changed");
  }
  if (projection.classifier_input_sha256 !== projectionCommitment(projection)) {
    throw new Error("classifier_input_invalid: classifier commitment mismatch");
  }
  routeBlocker(projection.blocker_reason, {
    lifecycleAuthorized: projection.lifecycle_authorized,
  });
  return projection;
}

function output(terminalOutcome) {
  if (!TERMINAL_IDS.includes(terminalOutcome)) {
    throw new Error("classification_inconsistent: unknown terminal outcome");
  }
  const passed = terminalOutcome === "passed_same_chain_hardware";
  const result = {
    terminal_outcome: terminalOutcome,
    verification_result: passed ? "passed" : "gaps_found",
    phase30_promotion_input: passed ? "eligible" : "pending",
  };
  return { ...result, classifier_output_sha256: canonicalDigest(result) };
}

export function classifyStrictProductionEvidence(projection) {
  try {
    validateClassifierProjection(projection);
  } catch {
    return output("blocked_safe_evidence_invalid");
  }

  if (
    projection.process_running ||
    projection.cleanup_state === "unresolved" ||
    projection.blocker_reason === "process_cleanup_unresolved"
  ) {
    return output("blocked_safe_unresolved_process");
  }

  const routed = routeBlocker(projection.blocker_reason, {
    lifecycleAuthorized: projection.lifecycle_authorized,
  });
  if (routed && routed !== "gaps_found_same_chain_production_markers_absent") {
    return output(routed);
  }

  if (projection.lifecycle_status !== "match") {
    return output("blocked_safe_evidence_invalid");
  }

  const shareObserved = ["accepted", "rejected"].includes(
    projection.share_submission_status,
  );
  if (
    projection.result_correlated === true &&
    projection.power_delta_class === "rising_hashing" &&
    shareObserved
  ) {
    if (projection.blocker_reason !== "none") {
      return output("blocked_safe_evidence_invalid");
    }
    return output("passed_same_chain_hardware");
  }

  if (
    !["none", "production_markers_absent"].includes(
      projection.blocker_reason,
    )
  ) {
    return output("blocked_safe_evidence_invalid");
  }
  return output("gaps_found_same_chain_production_markers_absent");
}
