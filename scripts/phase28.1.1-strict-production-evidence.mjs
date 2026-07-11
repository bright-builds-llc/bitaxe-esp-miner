import {
  PHASE28_ATTEMPT_SCHEMA_VERSION,
  PHASE28_CLASSIFIER_VERSION,
  PHASE28_LIFECYCLE_ID,
  CLASSIFIER_PROJECTION_FIELDS,
  TERMINAL_IDS,
  buildClassifierProjection as buildAuthorityClassifierProjection,
  canonicalDigest,
  routeBlocker,
  validateAttemptState,
} from "./phase28.1.1-hardware-attempt-state.mjs";

export { CLASSIFIER_PROJECTION_FIELDS };

const DIGEST_FIELDS = Object.freeze([
  "boot_session_sha256",
  "selected_port_fingerprint_sha256",
  "reference_guard_output_sha256",
  "manifest_sha256",
  "factory_image_sha256",
  "reinit_raw_log_sha256",
  "lifecycle_capability_sha256",
  "lifecycle_owner_start_fingerprint_sha256",
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
  return buildAuthorityClassifierProjection(state);
}

export function validateClassifierProjection(projection) {
  assertExactKeys(projection, CLASSIFIER_PROJECTION_FIELDS);
  if (
    projection.schema_version !== PHASE28_ATTEMPT_SCHEMA_VERSION ||
    projection.phase_lifecycle_id !== PHASE28_LIFECYCLE_ID ||
    !/^[0-9a-f]{32}$/.test(projection.attempt_id ?? "")
  ) {
    throw new Error("classifier_input_invalid: attempt identity is malformed");
  }
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
    projection.manifest_source_commit !== projection.exact_head ||
    projection.classifier_version !== PHASE28_CLASSIFIER_VERSION
  ) {
    throw new Error("classifier_input_invalid: identity or classifier version mismatch");
  }
  const closedPostCaptureState =
    projection.attempt_state === "post_capture_validated" &&
    projection.classification_phase === "invoked" &&
    projection.effect_id === null &&
    projection.effect_phase === "none" &&
    Number.isSafeInteger(projection.effect_sequence) &&
    projection.effect_sequence >= 1 &&
    projection.lifecycle_substate === "complete" &&
    /^[0-9a-f]{32}$/.test(projection.lifecycle_lease_id ?? "") &&
    projection.capture_category === "complete_360s";
  const closedPrerequisiteState =
    projection.attempt_state === "new" &&
    projection.classification_phase === "not_run" &&
    projection.effect_id === null &&
    projection.effect_phase === "none" &&
    projection.effect_sequence === 0 &&
    projection.lifecycle_substate === "not_started";
  if (!closedPostCaptureState && !closedPrerequisiteState) {
    throw new Error("classifier_input_invalid: state or lease identity mismatch");
  }
  for (const field of [
    "reinit_capture_started_ms",
    "reinit_capture_ended_ms",
    "reinit_capture_duration_ms",
    "lifecycle_deadline_ms",
    "attestation_accepted_ms",
    "usb_absence_ms",
    "restore_accepted_ms",
    "reappearance_elapsed_ms",
  ]) {
    if (!Number.isSafeInteger(projection[field]) || projection[field] < 0) {
      throw new Error(`classifier_input_invalid: ${field} is malformed`);
    }
  }
  if (
    projection.reinit_capture_ended_ms < projection.reinit_capture_started_ms ||
    projection.reinit_capture_duration_ms !==
      projection.reinit_capture_ended_ms - projection.reinit_capture_started_ms ||
    projection.reinit_capture_duration_ms < 360_000 ||
    projection.usb_absence_ms < 5_000 ||
    projection.reappearance_elapsed_ms > 60_000
  ) {
    throw new Error("classifier_input_invalid: lifecycle measurements are inconsistent");
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

function output(terminalOutcome, blockerReason) {
  if (!TERMINAL_IDS.includes(terminalOutcome)) {
    throw new Error("classification_inconsistent: unknown terminal outcome");
  }
  const passed = terminalOutcome === "passed_same_chain_hardware";
  const result = {
    terminal_outcome: terminalOutcome,
    verification_result: passed ? "passed" : "gaps_found",
    phase30_promotion_input: passed ? "eligible" : "pending",
    blocker_reason: blockerReason,
  };
  return { ...result, classifier_output_sha256: canonicalDigest(result) };
}

export function classifyStrictProductionEvidence(projection) {
  try {
    validateClassifierProjection(projection);
  } catch {
    return output("blocked_safe_evidence_invalid", "classifier_input_invalid");
  }

  if (
    projection.process_running ||
    projection.cleanup_state === "unresolved" ||
    projection.blocker_reason === "process_cleanup_unresolved"
  ) {
    return output("blocked_safe_unresolved_process", "process_cleanup_unresolved");
  }

  const routed = routeBlocker(projection.blocker_reason, {
    lifecycleAuthorized: projection.lifecycle_authorized,
  });
  if (routed && routed !== "gaps_found_same_chain_production_markers_absent") {
    return output(routed, projection.blocker_reason);
  }

  if (projection.lifecycle_status !== "match") {
    return output("blocked_safe_evidence_invalid", "classification_inconsistent");
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
      return output("blocked_safe_evidence_invalid", "classification_inconsistent");
    }
    return output("passed_same_chain_hardware", "none");
  }

  if (
    !["none", "production_markers_absent"].includes(
      projection.blocker_reason,
    )
  ) {
    return output("blocked_safe_evidence_invalid", "classification_inconsistent");
  }
  return output(
    "gaps_found_same_chain_production_markers_absent",
    "production_markers_absent",
  );
}

export function classifyStrictPostCaptureState(state) {
  validateAttemptState(state);
  const projection = buildClassifierProjection(state);
  const result = classifyStrictProductionEvidence(projection);
  return {
    classifier_version: PHASE28_CLASSIFIER_VERSION,
    classifier_input_sha256: projection.classifier_input_sha256,
    ...result,
  };
}
