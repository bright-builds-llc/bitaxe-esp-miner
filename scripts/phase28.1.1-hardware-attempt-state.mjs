import { createHash, randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

export const PHASE28_ATTEMPT_SCHEMA_VERSION = "exact-head-attempt-v2";
export const PHASE28_RESUME_TOMBSTONE_SCHEMA_VERSION =
  "exact-head-resume-tombstone-v1";
export const PHASE28_CLASSIFIER_VERSION = "strict-production-v2";
export const PHASE28_LIFECYCLE_ID = "28.1.1-2026-07-09T19-24-27";

export const EFFECT_IDS = Object.freeze([
  "detector_board_info",
  "credential_presence_bind",
  "reference_guard",
  "package",
  "flash_reinit_runtime",
  "lifecycle_start",
  "post_capture_detector_board_info",
]);

export const EFFECT_PHASES = Object.freeze([
  "none",
  "authorized",
  "invoked",
  "completed",
  "failed",
]);

export const EFFECT_AUTHORIZATION_STATES = Object.freeze({
  detector_board_info: Object.freeze([
    "connected_entry_waiting",
    "recovery_waiting",
  ]),
  credential_presence_bind: Object.freeze(["detector_passed"]),
  reference_guard: Object.freeze(["credentials_bound"]),
  package: Object.freeze(["reference_checked"]),
  flash_reinit_runtime: Object.freeze(["packaged"]),
  lifecycle_start: Object.freeze(["reinit_validated"]),
  post_capture_detector_board_info: Object.freeze(["capture_complete"]),
});

export const ATTEMPT_STATES = Object.freeze([
  "new",
  "connected_entry_waiting",
  "recovery_waiting",
  "detector_passed",
  "credentials_bound",
  "reference_checked",
  "packaged",
  "reinit_captured",
  "reinit_validated",
  "lifecycle_authorized",
  "armed",
  "removal_attested",
  "absence_observing",
  "restore_waiting",
  "restore_attested",
  "reappearance_observing",
  "capture_running",
  "capture_complete",
  "post_capture_validated",
  "classified",
  "terminal",
]);

export const ATTEMPT_TRANSITIONS = Object.freeze({
  new: ["connected_entry_waiting", "terminal"],
  connected_entry_waiting: ["recovery_waiting", "detector_passed", "terminal"],
  recovery_waiting: ["recovery_waiting", "detector_passed", "terminal"],
  detector_passed: ["credentials_bound", "terminal"],
  credentials_bound: ["reference_checked", "terminal"],
  reference_checked: ["packaged", "terminal"],
  packaged: ["reinit_captured", "terminal"],
  reinit_captured: ["reinit_validated", "terminal"],
  reinit_validated: ["lifecycle_authorized", "terminal"],
  lifecycle_authorized: ["armed", "terminal"],
  armed: ["removal_attested", "terminal"],
  removal_attested: ["absence_observing", "terminal"],
  absence_observing: ["restore_waiting", "terminal"],
  restore_waiting: ["restore_attested", "terminal"],
  restore_attested: ["reappearance_observing", "terminal"],
  reappearance_observing: ["capture_running", "terminal"],
  capture_running: ["capture_complete", "terminal"],
  capture_complete: ["post_capture_validated", "terminal"],
  post_capture_validated: ["classified", "terminal"],
  classified: ["terminal"],
  terminal: [],
});

export const CHECKPOINT_IDS = Object.freeze([
  "plan13-connected-entry",
  "plan13-recovery-usb-replug",
  "plan13-recovery-both-power",
  "plan13-recovery-reset",
  "plan13-recovery-boot-reset",
  "plan13-lifecycle-removal",
  "plan13-lifecycle-restore",
]);

export const CHECKPOINT_TOKENS = Object.freeze([
  "plan13-connected-entry-v1",
  "plan13-usb-replug-recovery-v1",
  "plan13-both-power-recovery-v1",
  "plan13-reset-recovery-v1",
  "plan13-boot-reset-recovery-v1",
  "plan13-armed-removal-v1",
  "plan13-barrel-usb-restore-v1",
]);

export const EXPECTED_RESPONSE_TOKENS = Object.freeze([
  "ultra205-remains-connected",
  "plan13-usb-replugged-v1",
  "plan13-both-power-restored-v1",
  "plan13-reset-pressed-v1",
  "plan13-boot-reset-assisted-v1",
  "plan13-both-power-paths-removed",
  "plan13-barrel-then-usb-restored",
]);

export const EXPECTED_USER_ACTIONS = Object.freeze([
  "keep-connected",
  "replug-usb-with-barrel-retained",
  "remove-both-wait-five-seconds-restore-barrel-then-usb",
  "press-reset-once",
  "perform-boot-reset-board-info-assist-once",
  "remove-both-power-paths-and-attest",
  "restore-barrel-then-usb",
]);

export const PUBLIC_CHECKPOINT_FIELDS = Object.freeze([
  "resume_handle",
  "checkpoint_id",
  "checkpoint_token",
  "checkpoint_generation",
  "exact_head",
  "attempt_state",
  "lifecycle_substate",
  "created_monotonic_ms",
  "monotonic_deadline_ms",
  "expected_response_token",
  "expected_user_action",
  "detector_attempt_count",
  "usb_replug_count",
  "both_power_count",
  "reset_count",
  "boot_reset_count",
  "reinit_five_stage_result",
  "capture_category",
  "process_running",
]);

export const CHECKPOINT_DEFINITIONS = Object.freeze({
  "plan13-connected-entry": Object.freeze({
    checkpointToken: "plan13-connected-entry-v1",
    expectedResponseToken: "ultra205-remains-connected",
    expectedUserAction: "keep-connected",
    timeoutMs: 600_000,
    attemptState: "connected_entry_waiting",
    lifecycleSubstate: "not_started",
  }),
  "plan13-recovery-usb-replug": Object.freeze({
    checkpointToken: "plan13-usb-replug-recovery-v1",
    expectedResponseToken: "plan13-usb-replugged-v1",
    expectedUserAction: "replug-usb-with-barrel-retained",
    timeoutMs: 300_000,
    attemptState: "recovery_waiting",
    lifecycleSubstate: "not_started",
  }),
  "plan13-recovery-both-power": Object.freeze({
    checkpointToken: "plan13-both-power-recovery-v1",
    expectedResponseToken: "plan13-both-power-restored-v1",
    expectedUserAction:
      "remove-both-wait-five-seconds-restore-barrel-then-usb",
    timeoutMs: 300_000,
    attemptState: "recovery_waiting",
    lifecycleSubstate: "not_started",
  }),
  "plan13-recovery-reset": Object.freeze({
    checkpointToken: "plan13-reset-recovery-v1",
    expectedResponseToken: "plan13-reset-pressed-v1",
    expectedUserAction: "press-reset-once",
    timeoutMs: 300_000,
    attemptState: "recovery_waiting",
    lifecycleSubstate: "not_started",
  }),
  "plan13-recovery-boot-reset": Object.freeze({
    checkpointToken: "plan13-boot-reset-recovery-v1",
    expectedResponseToken: "plan13-boot-reset-assisted-v1",
    expectedUserAction: "perform-boot-reset-board-info-assist-once",
    timeoutMs: 300_000,
    attemptState: "recovery_waiting",
    lifecycleSubstate: "not_started",
  }),
  "plan13-lifecycle-removal": Object.freeze({
    checkpointToken: "plan13-armed-removal-v1",
    expectedResponseToken: "plan13-both-power-paths-removed",
    expectedUserAction: "remove-both-power-paths-and-attest",
    timeoutMs: 300_000,
    attemptState: "armed",
    lifecycleSubstate: "removal_waiting",
  }),
  "plan13-lifecycle-restore": Object.freeze({
    checkpointToken: "plan13-barrel-usb-restore-v1",
    expectedResponseToken: "plan13-barrel-then-usb-restored",
    expectedUserAction: "restore-barrel-then-usb",
    timeoutMs: 60_000,
    attemptState: "restore_waiting",
    lifecycleSubstate: "restore_waiting",
  }),
});

export const EFFECT_RESULT_SCHEMA_VERSION = "exact-head-effect-result-v1";

export const EFFECT_RESULT_FIELDS = Object.freeze({
  detector_board_info: Object.freeze(["selected_port_fingerprint_sha256"]),
  credential_presence_bind: Object.freeze([
    "wifi_credential_state",
    "pool_credential_state",
    "wifi_credential_binding_id",
    "pool_credential_binding_id",
    "credential_capability_status",
    "credential_capability_sha256",
  ]),
  reference_guard: Object.freeze([
    "reference_commit",
    "reference_guard_output_sha256",
  ]),
  package: Object.freeze([
    "manifest_source_commit",
    "manifest_sha256",
    "factory_image_sha256",
  ]),
  flash_reinit_runtime: Object.freeze([
    "runtime_credential_consumption",
    "credential_capability_status",
    "credential_capability_sha256",
    "reinit_capture_started_ms",
    "reinit_capture_ended_ms",
    "reinit_capture_duration_ms",
    "reinit_capture_category",
    "reinit_raw_log_sha256",
    "reinit_classifier_input_sha256",
    "reinit_five_stage_result",
  ]),
  lifecycle_start: Object.freeze([]),
  post_capture_detector_board_info: Object.freeze([
    "result_correlated",
    "power_delta_class",
    "share_submission_status",
    "lifecycle_status",
    "classifier_input_sha256",
    "classifier_output_sha256",
    "classifier_version",
  ]),
});

export const LIFECYCLE_SUBSTATES = Object.freeze([
  "not_started",
  "lease_issued",
  "owner_attached",
  "removal_waiting",
  "removal_attested",
  "absence_observing",
  "restore_waiting",
  "restore_attested",
  "reappearance_observing",
  "monitoring",
  "complete",
  "failed",
]);

export const PROHIBITED_SENTINEL_KEYS = Object.freeze([
  "detector_board_info_effect_count",
  "credential_presence_bind_effect_count",
  "reference_guard_effect_count",
  "package_effect_count",
  "flash_reinit_runtime_effect_count",
  "lifecycle_start_effect_count",
  "post_capture_detector_board_info_effect_count",
  "reset_action_count",
  "discovery_action_count",
  "unrelated_process_action_count",
]);

export const PERMITTED_LIFECYCLE_KEYS = Object.freeze([
  "removal_pty_write_count",
  "restore_pty_write_count",
  "usb_absence_observation_session_count",
  "usb_reappearance_observation_session_count",
  "retained_monitor_start_count",
]);

export const TERMINAL_IDS = Object.freeze([
  "blocked_safe_attempt_prerequisite",
  "blocked_safe_unresolved_process",
  "blocked_safe_evidence_invalid",
  "gaps_found_same_chain_production_markers_absent",
  "passed_same_chain_hardware",
]);

const CONTEXTUAL_BLOCKERS = Object.freeze([
  "resume_handle_missing",
  "resume_handle_malformed",
  "resume_handle_wrong",
  "resume_handle_stale",
  "resume_handle_ambiguous",
  "checkpoint_token_mismatch",
  "checkpoint_generation_mismatch",
  "checkpoint_state_mismatch",
  "checkpoint_expired",
  "boot_session_observation_unavailable",
  "boot_session_mismatch",
  "exact_head_mismatch",
  "dirty_head",
  "manifest_mismatch",
  "reference_mismatch",
  "state_malformed",
  "state_missing_field",
  "state_unknown_field",
  "validator_error",
  "lock_failure",
  "persistence_failure",
  "lease_conflict",
  "lease_owner_mismatch",
  "lease_dead_or_reused_process",
  "effect_in_flight_ambiguous",
  "detector_failed",
  "detector_recovery_exhausted",
  "credential_binding_failed",
  "private_capability_invalid",
  "credential_consumption_failed",
  "reference_guard_failed",
  "package_failed",
  "reinit_capture_failed",
  "reinit_capture_short",
  "reinit_stage_mismatch",
  "lifecycle_attestation_rejected",
  "usb_absence_too_short_or_interrupted",
  "usb_reappearance_late",
  "monitor_failed",
  "lifecycle_capture_failed",
  "lifecycle_capture_short",
  "post_capture_detector_failed",
  "cancelled_or_abandoned",
]);

export const BLOCKER_REASONS = Object.freeze([
  "none",
  ...CONTEXTUAL_BLOCKERS,
  "sentinel_mismatch",
  "classifier_input_invalid",
  "classification_inconsistent",
  "process_cleanup_unresolved",
  "production_markers_absent",
]);

export class AttemptStateError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "AttemptStateError";
    this.code = code;
  }
}

export class BootSessionObservationError extends Error {
  constructor(message) {
    super(message);
    this.name = "BootSessionObservationError";
    this.code = "boot_session_observation_unavailable";
  }
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function trimAsciiWhitespace(value) {
  return value.replace(/^[\t\n\v\f\r ]+|[\t\n\v\f\r ]+$/g, "");
}

export function deriveLinuxBootSessionDigest(rawBootId) {
  const bootId = trimAsciiWhitespace(String(rawBootId));
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(bootId)) {
    throw new BootSessionObservationError("Linux boot ID is not a canonical UUID");
  }
  return sha256(Buffer.concat([Buffer.from("linux-boot-id-v1\0"), Buffer.from(bootId)]));
}

export function deriveMacBootSessionDigest(rawBootTime) {
  const text = trimAsciiWhitespace(String(rawBootTime));
  const match = /(?:^|[{,\s])sec\s*=\s*([0-9]+)\s*,?\s*usec\s*=\s*([0-9]+)(?:\s*}|\s*$)/.exec(text);
  if (!match) {
    throw new BootSessionObservationError("macOS boot time is malformed");
  }
  const sec = Number(match[1]);
  const usec = Number(match[2]);
  if (!Number.isSafeInteger(sec) || sec < 0 || !Number.isInteger(usec) || usec < 0 || usec > 999_999) {
    throw new BootSessionObservationError("macOS boot time is outside its canonical domain");
  }
  const normalized = `sec=${sec};usec=${usec}`;
  return sha256(Buffer.concat([Buffer.from("macos-kern-boottime-v1\0"), Buffer.from(normalized)]));
}

export function observeBootSessionDigest(adapters = {}) {
  const platform = adapters.platform ?? process.platform;
  try {
    if (platform === "linux") {
      const readLinuxBootId = adapters.readLinuxBootId ?? (() => readFileSync("/proc/sys/kernel/random/boot_id", "utf8"));
      return deriveLinuxBootSessionDigest(readLinuxBootId());
    }
    if (platform === "darwin") {
      const readMacBootTime = adapters.readMacBootTime ?? (() => execFileSync("/usr/sbin/sysctl", ["-n", "kern.boottime"], { encoding: "utf8" }));
      return deriveMacBootSessionDigest(readMacBootTime());
    }
  } catch (error) {
    if (error instanceof BootSessionObservationError) throw error;
    throw new BootSessionObservationError("boot session observation failed");
  }
  throw new BootSessionObservationError(`unsupported boot-session platform: ${platform}`);
}

function zeroCounts(keys) {
  return Object.fromEntries(keys.map((key) => [key, 0]));
}

function stateDefaults() {
  return {
    schema_version: PHASE28_ATTEMPT_SCHEMA_VERSION,
    phase: "28.1.1",
    tooling_plan: 12,
    execution_plan: 13,
    phase_lifecycle_id: PHASE28_LIFECYCLE_ID,
    attempt_id: "0".repeat(32),
    exact_head: "0".repeat(40),
    boot_session_sha256: "0".repeat(64),
    attempt_state: "new",
    effect_id: null,
    effect_phase: "none",
    effect_authorization_nonce: null,
    effect_sequence: 0,
    resume_handle_sha256: "0".repeat(64),
    checkpoint_id: null,
    checkpoint_token: null,
    expected_response_token: null,
    expected_user_action: null,
    checkpoint_generation: 0,
    created_monotonic_ms: 0,
    monotonic_deadline_ms: null,
    detector_attempt_count: 0,
    usb_replug_count: 0,
    both_power_count: 0,
    reset_count: 0,
    boot_reset_count: 0,
    wifi_credential_state: "not_run",
    pool_credential_state: "not_run",
    wifi_credential_binding_id: null,
    pool_credential_binding_id: null,
    credential_capability_status: "not_created",
    credential_capability_sha256: null,
    runtime_credential_consumption: "not_run",
    reference_guard_state: "not_run",
    reference_commit: null,
    reference_guard_output_sha256: null,
    selected_port_state: "unknown",
    selected_port_fingerprint_sha256: null,
    manifest_state: "not_run",
    manifest_source_commit: null,
    manifest_sha256: null,
    factory_image_sha256: null,
    reinit_capture_started_ms: null,
    reinit_capture_ended_ms: null,
    reinit_capture_duration_ms: null,
    reinit_capture_category: "not_run",
    reinit_raw_log_sha256: null,
    reinit_classifier_input_sha256: null,
    reinit_five_stage_result: "not_run",
    lifecycle_substate: "not_started",
    lifecycle_lease_id: null,
    lifecycle_capability_sha256: null,
    lifecycle_owner_pid: null,
    lifecycle_owner_start_fingerprint_sha256: null,
    lifecycle_deadline_ms: null,
    attestation_status: "not_requested",
    attestation_accepted_ms: null,
    usb_absence_started_ms: null,
    usb_absence_ended_ms: null,
    usb_absence_ms: null,
    usb_absence_category: "not_observed",
    restore_token_status: "not_requested",
    restore_accepted_ms: null,
    usb_reappearance_ms: null,
    reappearance_elapsed_ms: null,
    reappearance_category: "not_observed",
    armed_prohibited_sentinel_counts: zeroCounts(PROHIBITED_SENTINEL_KEYS),
    capture_complete_prohibited_sentinel_counts: zeroCounts(PROHIBITED_SENTINEL_KEYS),
    armed_permitted_lifecycle_counts: zeroCounts(PERMITTED_LIFECYCLE_KEYS),
    capture_complete_permitted_lifecycle_counts: zeroCounts(PERMITTED_LIFECYCLE_KEYS),
    armed_prohibited_sentinel_sha256: null,
    capture_complete_prohibited_sentinel_sha256: null,
    armed_permitted_lifecycle_sha256: null,
    capture_complete_permitted_lifecycle_sha256: null,
    capture_started_ms: null,
    capture_ended_ms: null,
    capture_duration_ms: null,
    capture_category: "not_started",
    lifecycle_raw_log_sha256: null,
    same_chain_raw_log_set_sha256: null,
    classifier_input_sha256: null,
    classifier_output_sha256: null,
    classifier_version: "not_run",
    lifecycle_status: "absent",
    process_running: false,
    cleanup_state: "not_needed",
    result_correlated: null,
    power_delta_class: null,
    share_submission_status: null,
    terminal_outcome: null,
    blocker_reason: "none",
  };
}

export const ATTEMPT_STATE_FIELDS = Object.freeze(Object.keys(stateDefaults()));

export function createAttemptState({ exactHead, resumeHandleSha256, createdMonotonicMs, observeBootSession = observeBootSessionDigest, randomHex = (bytes) => randomBytes(bytes).toString("hex") }) {
  const state = stateDefaults();
  state.attempt_id = randomHex(16);
  state.exact_head = exactHead;
  state.resume_handle_sha256 = resumeHandleSha256;
  state.created_monotonic_ms = createdMonotonicMs;
  state.boot_session_sha256 = observeBootSession();
  validateAttemptState(state);
  return state;
}

function fail(code, message) {
  throw new AttemptStateError(code, message);
}

function isUnsigned(value, max = Number.MAX_SAFE_INTEGER) {
  return Number.isSafeInteger(value) && value >= 0 && value <= max;
}

function requireEnum(value, values, field, nullable = false) {
  if (nullable && value === null) return;
  if (!values.includes(value)) fail("state_malformed", `${field} is outside its closed domain`);
}

function requirePattern(value, pattern, field, nullable = false) {
  if (nullable && value === null) return;
  if (typeof value !== "string" || !pattern.test(value)) fail("state_malformed", `${field} is malformed`);
}

function requireNullableUnsigned(value, field, positive = false) {
  if (value === null) return;
  if (!isUnsigned(value) || (positive && value === 0)) fail("state_malformed", `${field} is not a valid unsigned integer`);
}

function requireExactKeys(value, keys, field) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail("state_malformed", `${field} must be an object`);
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    fail("state_malformed", `${field} does not have the exact closed key set`);
  }
}

function requireCountObject(value, keys, field) {
  requireExactKeys(value, keys, field);
  for (const key of keys) {
    if (!isUnsigned(value[key], 0xffff_ffff)) fail("state_malformed", `${field}.${key} is not u32`);
  }
}

function validateDuration(state, startField, endField, durationField, categoryField, completeCategory, minimumDuration) {
  const start = state[startField];
  const end = state[endField];
  const duration = state[durationField];
  for (const [field, value] of [[startField, start], [endField, end], [durationField, duration]]) {
    requireNullableUnsigned(value, field);
  }
  const values = [start, end, duration];
  const present = values.filter((value) => value !== null).length;
  const startOnly = start !== null && end === null && duration === null;
  if (present !== 0 && present !== 3 && !startOnly) fail("state_malformed", `${durationField} timing triple is incomplete`);
  if (present === 3 && (end < start || duration !== end - start)) fail("state_malformed", `${durationField} timing is inconsistent`);
  if (state[categoryField] === completeCategory && (duration === null || duration < minimumDuration)) {
    fail("state_malformed", `${categoryField} requires at least ${minimumDuration} ms`);
  }
}

function assertDigestForPass(state, statusField, passValue, digestFields) {
  if (state[statusField] !== passValue) return;
  for (const field of digestFields) requirePattern(state[field], /^[0-9a-f]{64}$/, field);
}

export function validateAttemptState(state) {
  if (!state || typeof state !== "object" || Array.isArray(state)) fail("state_malformed", "attempt state must be an object");
  const actual = Object.keys(state);
  for (const field of ATTEMPT_STATE_FIELDS) {
    if (!Object.hasOwn(state, field)) fail("state_missing_field", `missing field: ${field}`);
  }
  for (const field of actual) {
    if (!ATTEMPT_STATE_FIELDS.includes(field)) fail("state_unknown_field", `unknown field: ${field}`);
  }

  if (state.schema_version !== PHASE28_ATTEMPT_SCHEMA_VERSION || state.phase !== "28.1.1" || state.tooling_plan !== 12 || state.execution_plan !== 13 || state.phase_lifecycle_id !== PHASE28_LIFECYCLE_ID) {
    fail("state_malformed", "attempt identity constants do not match Plan 12");
  }
  requirePattern(state.attempt_id, /^[0-9a-f]{32}$/, "attempt_id");
  requirePattern(state.exact_head, /^[0-9a-f]{40}$/, "exact_head");
  requirePattern(state.boot_session_sha256, /^[0-9a-f]{64}$/, "boot_session_sha256");
  requirePattern(state.resume_handle_sha256, /^[0-9a-f]{64}$/, "resume_handle_sha256");
  requireEnum(state.attempt_state, ATTEMPT_STATES, "attempt_state");
  requireEnum(state.effect_id, EFFECT_IDS, "effect_id", true);
  requireEnum(state.effect_phase, EFFECT_PHASES, "effect_phase");
  requirePattern(state.effect_authorization_nonce, /^[0-9a-f]{32}$/, "effect_authorization_nonce", true);
  if (!isUnsigned(state.effect_sequence, 0xffff_ffff)) fail("state_malformed", "effect_sequence is not u32");
  const idle = state.effect_phase === "none";
  if (idle !== (state.effect_id === null && state.effect_authorization_nonce === null)) fail("state_malformed", "effect lifecycle triple is inconsistent");
  if (!idle && (state.effect_id === null || state.effect_authorization_nonce === null)) fail("state_malformed", "active effect requires identity and nonce");

  requireEnum(state.checkpoint_id, CHECKPOINT_IDS, "checkpoint_id", true);
  requireEnum(state.checkpoint_token, CHECKPOINT_TOKENS, "checkpoint_token", true);
  requireEnum(state.expected_response_token, EXPECTED_RESPONSE_TOKENS, "expected_response_token", true);
  requireEnum(state.expected_user_action, EXPECTED_USER_ACTIONS, "expected_user_action", true);
  for (const field of ["checkpoint_generation", "created_monotonic_ms", "detector_attempt_count", "usb_replug_count", "both_power_count", "reset_count", "boot_reset_count"]) {
    if (!isUnsigned(state[field], 0xffff_ffff)) fail("state_malformed", `${field} is not u32`);
  }
  requireNullableUnsigned(state.monotonic_deadline_ms, "monotonic_deadline_ms");
  const checkpointValues = [state.checkpoint_id, state.checkpoint_token, state.expected_response_token, state.expected_user_action, state.monotonic_deadline_ms];
  if (state.checkpoint_id === null && checkpointValues.some((value) => value !== null)) fail("state_malformed", "idle checkpoint fields are inconsistent");
  if (state.checkpoint_id !== null && checkpointValues.some((value) => value === null)) fail("state_malformed", "active checkpoint fields are incomplete");
  if (state.detector_attempt_count > 5 || ["usb_replug_count", "both_power_count", "reset_count", "boot_reset_count"].some((field) => state[field] > 1)) {
    fail("state_malformed", "finite recovery counts exceeded");
  }

  for (const field of ["wifi_credential_state", "pool_credential_state"]) requireEnum(state[field], ["not_run", "present", "absent"], field);
  for (const field of ["wifi_credential_binding_id", "pool_credential_binding_id"]) requirePattern(state[field], /^[0-9a-f]{32}$/, field, true);
  requireEnum(state.credential_capability_status, ["not_created", "sealed", "consumed", "destroyed"], "credential_capability_status");
  requirePattern(state.credential_capability_sha256, /^[0-9a-f]{64}$/, "credential_capability_sha256", true);
  requireEnum(state.runtime_credential_consumption, ["not_run", "pass", "fail"], "runtime_credential_consumption");
  if (state.credential_capability_status === "sealed" && (state.credential_capability_sha256 === null || state.wifi_credential_binding_id === null || state.pool_credential_binding_id === null)) fail("state_malformed", "sealed credential capability lacks opaque bindings");
  if (["not_created", "destroyed"].includes(state.credential_capability_status) && state.credential_capability_sha256 !== null) fail("state_malformed", "inactive credential capability retained a live digest");
  requireEnum(state.reference_guard_state, ["not_run", "pass", "fail"], "reference_guard_state");
  requirePattern(state.reference_commit, /^[0-9a-f]{40}$/, "reference_commit", true);
  requirePattern(state.reference_guard_output_sha256, /^[0-9a-f]{64}$/, "reference_guard_output_sha256", true);
  requireEnum(state.selected_port_state, ["unknown", "one_board205", "invalid"], "selected_port_state");
  requirePattern(state.selected_port_fingerprint_sha256, /^[0-9a-f]{64}$/, "selected_port_fingerprint_sha256", true);
  requireEnum(state.manifest_state, ["not_run", "pass", "fail"], "manifest_state");
  requirePattern(state.manifest_source_commit, /^[0-9a-f]{40}$/, "manifest_source_commit", true);
  for (const field of ["manifest_sha256", "factory_image_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field, true);
  if (state.manifest_state === "pass" && state.manifest_source_commit !== state.exact_head) fail("state_malformed", "manifest source is not exact HEAD");
  assertDigestForPass(state, "reference_guard_state", "pass", ["reference_guard_output_sha256"]);
  if (state.reference_guard_state === "pass") requirePattern(state.reference_commit, /^[0-9a-f]{40}$/, "reference_commit");
  assertDigestForPass(state, "manifest_state", "pass", ["manifest_sha256", "factory_image_sha256"]);

  requireEnum(state.reinit_capture_category, ["not_run", "complete_360s", "short", "failed"], "reinit_capture_category");
  validateDuration(state, "reinit_capture_started_ms", "reinit_capture_ended_ms", "reinit_capture_duration_ms", "reinit_capture_category", "complete_360s", 360_000);
  for (const field of ["reinit_raw_log_sha256", "reinit_classifier_input_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field, true);
  requireEnum(state.reinit_five_stage_result, ["not_run", "pass", "fail"], "reinit_five_stage_result");
  if (state.reinit_five_stage_result === "pass") assertDigestForPass(state, "reinit_five_stage_result", "pass", ["reinit_raw_log_sha256", "reinit_classifier_input_sha256"]);

  requireEnum(state.lifecycle_substate, LIFECYCLE_SUBSTATES, "lifecycle_substate");
  requirePattern(state.lifecycle_lease_id, /^[0-9a-f]{32}$/, "lifecycle_lease_id", true);
  requirePattern(state.lifecycle_capability_sha256, /^[0-9a-f]{64}$/, "lifecycle_capability_sha256", true);
  requireNullableUnsigned(state.lifecycle_owner_pid, "lifecycle_owner_pid", true);
  requirePattern(state.lifecycle_owner_start_fingerprint_sha256, /^[0-9a-f]{64}$/, "lifecycle_owner_start_fingerprint_sha256", true);
  requireNullableUnsigned(state.lifecycle_deadline_ms, "lifecycle_deadline_ms");
  if (state.lifecycle_substate !== "not_started" && [state.lifecycle_lease_id, state.lifecycle_capability_sha256, state.lifecycle_owner_pid, state.lifecycle_owner_start_fingerprint_sha256, state.lifecycle_deadline_ms].some((value) => value === null)) {
    fail("state_malformed", "active lifecycle is missing immutable lease identity");
  }

  requireEnum(state.attestation_status, ["not_requested", "waiting", "accepted", "rejected", "expired"], "attestation_status");
  requireNullableUnsigned(state.attestation_accepted_ms, "attestation_accepted_ms");
  requireEnum(state.usb_absence_category, ["not_observed", "continuous_at_least_5000", "too_short", "interrupted"], "usb_absence_category");
  requireEnum(state.restore_token_status, ["not_requested", "waiting", "accepted", "rejected", "expired"], "restore_token_status");
  requireNullableUnsigned(state.restore_accepted_ms, "restore_accepted_ms");
  requireNullableUnsigned(state.usb_reappearance_ms, "usb_reappearance_ms");
  requireNullableUnsigned(state.reappearance_elapsed_ms, "reappearance_elapsed_ms");
  requireEnum(state.reappearance_category, ["not_observed", "within_60000", "late"], "reappearance_category");
  validateDuration(state, "usb_absence_started_ms", "usb_absence_ended_ms", "usb_absence_ms", "usb_absence_category", "continuous_at_least_5000", 5_000);
  if (state.usb_absence_category === "continuous_at_least_5000" && state.usb_absence_ms < 5_000) fail("state_malformed", "continuous USB absence is shorter than 5000 ms");
  if (state.reappearance_category === "within_60000" && (state.reappearance_elapsed_ms === null || state.reappearance_elapsed_ms > 60_000)) fail("state_malformed", "USB reappearance exceeded 60000 ms");

  requireCountObject(state.armed_prohibited_sentinel_counts, PROHIBITED_SENTINEL_KEYS, "armed_prohibited_sentinel_counts");
  requireCountObject(state.capture_complete_prohibited_sentinel_counts, PROHIBITED_SENTINEL_KEYS, "capture_complete_prohibited_sentinel_counts");
  requireCountObject(state.armed_permitted_lifecycle_counts, PERMITTED_LIFECYCLE_KEYS, "armed_permitted_lifecycle_counts");
  requireCountObject(state.capture_complete_permitted_lifecycle_counts, PERMITTED_LIFECYCLE_KEYS, "capture_complete_permitted_lifecycle_counts");
  for (const field of ["armed_prohibited_sentinel_sha256", "capture_complete_prohibited_sentinel_sha256", "armed_permitted_lifecycle_sha256", "capture_complete_permitted_lifecycle_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field, true);
  if (["armed", "removal_attested", "absence_observing", "restore_waiting", "restore_attested", "reappearance_observing", "capture_running"].includes(state.attempt_state)) {
    if (PERMITTED_LIFECYCLE_KEYS.some((key) => state.armed_permitted_lifecycle_counts[key] !== 0)) fail("sentinel_mismatch", "permitted lifecycle count was nonzero at arm");
  }
  if (["capture_complete", "post_capture_validated", "classified", "terminal"].includes(state.attempt_state) && state.lifecycle_status !== "absent") {
    if (PERMITTED_LIFECYCLE_KEYS.some((key) => state.capture_complete_permitted_lifecycle_counts[key] !== 1)) fail("sentinel_mismatch", "capture-complete lifecycle counts are not exactly one");
    if (JSON.stringify(state.armed_prohibited_sentinel_counts) !== JSON.stringify(state.capture_complete_prohibited_sentinel_counts) || state.armed_prohibited_sentinel_sha256 !== state.capture_complete_prohibited_sentinel_sha256) fail("sentinel_mismatch", "prohibited sentinels changed after arm");
    for (const field of ["armed_prohibited_sentinel_sha256", "capture_complete_prohibited_sentinel_sha256", "armed_permitted_lifecycle_sha256", "capture_complete_permitted_lifecycle_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field);
  }

  requireEnum(state.capture_category, ["not_started", "running", "complete_360s", "short", "failed"], "capture_category");
  validateDuration(state, "capture_started_ms", "capture_ended_ms", "capture_duration_ms", "capture_category", "complete_360s", 360_000);
  for (const field of ["lifecycle_raw_log_sha256", "same_chain_raw_log_set_sha256", "classifier_input_sha256", "classifier_output_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field, true);
  requireEnum(state.classifier_version, ["not_run", PHASE28_CLASSIFIER_VERSION], "classifier_version");
  requireEnum(state.lifecycle_status, ["absent", "incomplete", "mismatch", "match"], "lifecycle_status");
  if (["capture_complete", "post_capture_validated", "classified", "terminal"].includes(state.attempt_state) && state.lifecycle_status !== "absent") {
    if (state.capture_category !== "complete_360s" || state.reinit_capture_category !== "complete_360s" || state.reinit_five_stage_result !== "pass") {
      fail("state_malformed", "completed lifecycle lacks complete capture prerequisites");
    }
    if (state.reference_guard_state !== "pass" || state.manifest_state !== "pass" || state.selected_port_state !== "one_board205") {
      fail("state_malformed", "completed lifecycle lacks identity prerequisites");
    }
    for (const field of ["lifecycle_raw_log_sha256", "same_chain_raw_log_set_sha256", "classifier_input_sha256"]) requirePattern(state[field], /^[0-9a-f]{64}$/, field);
  }
  if (typeof state.process_running !== "boolean") fail("state_malformed", "process_running is not boolean");
  requireEnum(state.cleanup_state, ["not_needed", "complete", "unresolved"], "cleanup_state");
  if (state.result_correlated !== null && typeof state.result_correlated !== "boolean") fail("state_malformed", "result_correlated is invalid");
  requireEnum(state.power_delta_class, ["rising_hashing", "flat", "falling", "unavailable"], "power_delta_class", true);
  requireEnum(state.share_submission_status, ["accepted", "rejected", "not_observed"], "share_submission_status", true);
  requireEnum(state.terminal_outcome, TERMINAL_IDS, "terminal_outcome", true);
  requireEnum(state.blocker_reason, BLOCKER_REASONS, "blocker_reason");
  if (state.attempt_state === "terminal") {
    if (state.terminal_outcome === null) fail("state_malformed", "terminal state lacks terminal outcome");
    const expected = routeBlocker(state.blocker_reason, { lifecycleAuthorized: state.lifecycle_substate !== "not_started", cleanupUnresolved: state.cleanup_state === "unresolved" });
    if (state.blocker_reason !== "none" && state.terminal_outcome !== expected) fail("classification_inconsistent", "blocker does not route to terminal outcome");
  } else if (state.terminal_outcome !== null) {
    fail("state_malformed", "non-terminal state has terminal outcome");
  }
  return state;
}

export function assertFreshBootSession(state, observeBootSession = observeBootSessionDigest) {
  let observed;
  try {
    observed = observeBootSession();
  } catch {
    fail("boot_session_observation_unavailable", "fresh boot session observation failed");
  }
  if (observed !== state.boot_session_sha256) fail("boot_session_mismatch", "boot session changed");
  return observed;
}

export function transitionAttemptState(state, nextState) {
  validateAttemptState(state);
  requireEnum(nextState, ATTEMPT_STATES, "next attempt state");
  if (!ATTEMPT_TRANSITIONS[state.attempt_state].includes(nextState)) fail("state_malformed", `invalid attempt transition ${state.attempt_state} -> ${nextState}`);
  const next = structuredClone(state);
  next.attempt_state = nextState;
  validateAttemptState(next);
  return next;
}

export function authorizeEffect(state, effectId, nonce = randomBytes(16).toString("hex")) {
  validateAttemptState(state);
  requireEnum(effectId, EFFECT_IDS, "effect_id");
  if (state.effect_phase !== "none" || state.lifecycle_substate !== "not_started" && effectId !== "lifecycle_start") fail("effect_in_flight_ambiguous", "effect dispatch is not idle");
  if (!EFFECT_AUTHORIZATION_STATES[effectId].includes(state.attempt_state)) fail("checkpoint_state_mismatch", "effect is not authorized from the current attempt state");
  const next = structuredClone(state);
  next.effect_id = effectId;
  next.effect_phase = "authorized";
  next.effect_authorization_nonce = nonce;
  next.effect_sequence += 1;
  validateAttemptState(next);
  return next;
}

export function markEffectInvoked(state, nonce, sequence) {
  validateAttemptState(state);
  if (state.effect_phase !== "authorized" || state.effect_authorization_nonce !== nonce || state.effect_sequence !== sequence) fail("effect_in_flight_ambiguous", "effect invocation does not match authorization");
  const next = structuredClone(state);
  next.effect_phase = "invoked";
  validateAttemptState(next);
  return next;
}

export function finishEffect(state, nonce, sequence, succeeded) {
  validateAttemptState(state);
  if (state.effect_phase !== "invoked" || state.effect_authorization_nonce !== nonce || state.effect_sequence !== sequence) fail("effect_in_flight_ambiguous", "effect completion does not match invocation");
  const next = structuredClone(state);
  next.effect_phase = succeeded ? "completed" : "failed";
  validateAttemptState(next);
  return next;
}

export function normalizeFinishedEffect(state) {
  validateAttemptState(state);
  if (!["completed", "failed"].includes(state.effect_phase)) fail("effect_in_flight_ambiguous", "only finished effects can return to idle");
  const next = structuredClone(state);
  next.effect_id = null;
  next.effect_phase = "none";
  next.effect_authorization_nonce = null;
  validateAttemptState(next);
  return next;
}

export function ambiguousEffectOnRestart(state) {
  validateAttemptState(state);
  return ["authorized", "invoked"].includes(state.effect_phase);
}

export function installCheckpoint(state, checkpointId, createdMonotonicMs) {
  validateAttemptState(state);
  const definition = CHECKPOINT_DEFINITIONS[checkpointId];
  if (!definition) fail("checkpoint_state_mismatch", "checkpoint ID is not closed");
  if (state.checkpoint_id !== null) {
    fail("checkpoint_state_mismatch", "another checkpoint is already active");
  }
  if (
    state.attempt_state !== definition.attemptState ||
    state.lifecycle_substate !== definition.lifecycleSubstate
  ) {
    fail("checkpoint_state_mismatch", "checkpoint does not match current state");
  }
  if (!isUnsigned(createdMonotonicMs)) {
    fail("state_malformed", "checkpoint creation time is invalid");
  }
  const next = structuredClone(state);
  next.checkpoint_id = checkpointId;
  next.checkpoint_token = definition.checkpointToken;
  next.expected_response_token = definition.expectedResponseToken;
  next.expected_user_action = definition.expectedUserAction;
  next.checkpoint_generation += 1;
  next.created_monotonic_ms = createdMonotonicMs;
  next.monotonic_deadline_ms =
    checkpointId === "plan13-lifecycle-restore"
      ? next.attestation_accepted_ms + definition.timeoutMs
      : createdMonotonicMs + definition.timeoutMs;
  validateAttemptState(next);
  return next;
}

export function createConnectedEntryState(options) {
  const state = createAttemptState(options);
  state.attempt_state = "connected_entry_waiting";
  return installCheckpoint(
    state,
    "plan13-connected-entry",
    options.createdMonotonicMs,
  );
}

export function publicCheckpoint(state, resumeHandle) {
  validateAttemptState(state);
  requirePattern(resumeHandle, /^[0-9a-f]{64}$/, "resume_handle");
  if (sha256(resumeHandle) !== state.resume_handle_sha256) {
    fail("resume_handle_wrong", "resume handle does not match state");
  }
  if (state.checkpoint_id === null) {
    fail("checkpoint_state_mismatch", "attempt has no active checkpoint");
  }
  const projection = {
    resume_handle: resumeHandle,
    checkpoint_id: state.checkpoint_id,
    checkpoint_token: state.checkpoint_token,
    checkpoint_generation: state.checkpoint_generation,
    exact_head: state.exact_head,
    attempt_state: state.attempt_state,
    lifecycle_substate: state.lifecycle_substate,
    created_monotonic_ms: state.created_monotonic_ms,
    monotonic_deadline_ms: state.monotonic_deadline_ms,
    expected_response_token: state.expected_response_token,
    expected_user_action: state.expected_user_action,
    detector_attempt_count: state.detector_attempt_count,
    usb_replug_count: state.usb_replug_count,
    both_power_count: state.both_power_count,
    reset_count: state.reset_count,
    boot_reset_count: state.boot_reset_count,
    reinit_five_stage_result: state.reinit_five_stage_result,
    capture_category: state.capture_category,
    process_running: state.process_running,
  };
  requireExactKeys(projection, PUBLIC_CHECKPOINT_FIELDS, "public checkpoint");
  return projection;
}

export function consumeCheckpoint(
  state,
  { checkpointToken, responseToken, nowMonotonicMs },
) {
  validateAttemptState(state);
  if (state.checkpoint_id === null) {
    fail("checkpoint_state_mismatch", "checkpoint is already consumed");
  }
  const definition = CHECKPOINT_DEFINITIONS[state.checkpoint_id];
  if (
    state.attempt_state !== definition.attemptState ||
    state.lifecycle_substate !== definition.lifecycleSubstate
  ) {
    fail("checkpoint_state_mismatch", "checkpoint state changed");
  }
  if (
    checkpointToken !== state.checkpoint_token ||
    responseToken !== state.expected_response_token
  ) {
    fail("checkpoint_token_mismatch", "checkpoint response does not match");
  }
  if (
    !isUnsigned(nowMonotonicMs) ||
    nowMonotonicMs >= state.monotonic_deadline_ms
  ) {
    fail("checkpoint_expired", "checkpoint deadline elapsed");
  }

  const next = structuredClone(state);
  const consumedId = next.checkpoint_id;
  next.checkpoint_id = null;
  next.checkpoint_token = null;
  next.expected_response_token = null;
  next.expected_user_action = null;
  next.monotonic_deadline_ms = null;

  switch (consumedId) {
    case "plan13-recovery-usb-replug":
      next.usb_replug_count += 1;
      break;
    case "plan13-recovery-both-power":
      next.both_power_count += 1;
      break;
    case "plan13-recovery-reset":
      next.reset_count += 1;
      break;
    case "plan13-recovery-boot-reset":
      next.boot_reset_count += 1;
      break;
    case "plan13-lifecycle-removal":
      next.attempt_state = "removal_attested";
      next.lifecycle_substate = "removal_attested";
      next.attestation_status = "accepted";
      next.attestation_accepted_ms = nowMonotonicMs;
      break;
    case "plan13-lifecycle-restore":
      next.attempt_state = "restore_attested";
      next.lifecycle_substate = "restore_attested";
      next.restore_token_status = "accepted";
      next.restore_accepted_ms = nowMonotonicMs;
      break;
  }
  validateAttemptState(next);
  return next;
}

function validateEffectResult(effectId, result, succeeded) {
  requireExactKeys(
    result,
    ["schema_version", "effect_id", "status", "blocker_reason", "outputs"],
    "effect result",
  );
  if (
    result.schema_version !== EFFECT_RESULT_SCHEMA_VERSION ||
    result.effect_id !== effectId ||
    result.status !== (succeeded ? "completed" : "failed")
  ) {
    fail("effect_in_flight_ambiguous", "effect result identity is inconsistent");
  }
  requireEnum(result.blocker_reason, BLOCKER_REASONS, "blocker_reason");
  requireExactKeys(
    result.outputs,
    succeeded ? EFFECT_RESULT_FIELDS[effectId] : [],
    "effect outputs",
  );
  if (succeeded && result.blocker_reason !== "none") {
    fail("effect_in_flight_ambiguous", "successful effect has a blocker");
  }
  if (!succeeded && result.blocker_reason === "none") {
    fail("effect_in_flight_ambiguous", "failed effect lacks a blocker");
  }
}

function returnEffectToIdle(state) {
  state.effect_id = null;
  state.effect_phase = "none";
  state.effect_authorization_nonce = null;
}

export function terminalizeAttempt(
  state,
  blockerReason,
  { cleanupUnresolved = false } = {},
) {
  const next = structuredClone(state);
  next.blocker_reason = cleanupUnresolved
    ? "process_cleanup_unresolved"
    : blockerReason;
  next.process_running = false;
  next.cleanup_state = cleanupUnresolved ? "unresolved" : "complete";
  next.terminal_outcome = routeBlocker(next.blocker_reason, {
    lifecycleAuthorized: next.lifecycle_substate !== "not_started",
    cleanupUnresolved,
  });
  next.attempt_state = "terminal";
  next.checkpoint_id = null;
  next.checkpoint_token = null;
  next.expected_response_token = null;
  next.expected_user_action = null;
  next.monotonic_deadline_ms = null;
  returnEffectToIdle(next);
  validateAttemptState(next);
  return next;
}

export function applyEffectCompletion(
  state,
  result,
  { completedMonotonicMs },
) {
  validateAttemptState(state);
  if (!isUnsigned(completedMonotonicMs)) {
    fail("state_malformed", "effect completion time is invalid");
  }
  if (!state.effect_id || !["completed", "failed"].includes(state.effect_phase)) {
    fail("effect_in_flight_ambiguous", "effect is not durably finished");
  }
  const effectId = state.effect_id;
  const succeeded = state.effect_phase === "completed";
  validateEffectResult(effectId, result, succeeded);

  if (!succeeded) {
    if (effectId !== "detector_board_info") {
      return terminalizeAttempt(state, result.blocker_reason);
    }
    const next = structuredClone(state);
    next.detector_attempt_count += 1;
    returnEffectToIdle(next);
    if (next.detector_attempt_count >= 5) {
      return terminalizeAttempt(next, "detector_recovery_exhausted");
    }
    next.attempt_state = "recovery_waiting";
    const recoveryIds = [
      "plan13-recovery-usb-replug",
      "plan13-recovery-both-power",
      "plan13-recovery-reset",
      "plan13-recovery-boot-reset",
    ];
    return installCheckpoint(
      next,
      recoveryIds[next.detector_attempt_count - 1],
      completedMonotonicMs,
    );
  }

  const next = structuredClone(state);
  Object.assign(next, result.outputs);
  returnEffectToIdle(next);
  switch (effectId) {
    case "detector_board_info":
      next.detector_attempt_count += 1;
      next.selected_port_state = "one_board205";
      next.attempt_state = "detector_passed";
      break;
    case "credential_presence_bind":
      next.attempt_state = "credentials_bound";
      break;
    case "reference_guard":
      next.reference_guard_state = "pass";
      next.attempt_state = "reference_checked";
      break;
    case "package":
      next.manifest_state = "pass";
      next.attempt_state = "packaged";
      break;
    case "flash_reinit_runtime":
      next.attempt_state = "reinit_validated";
      break;
    case "lifecycle_start":
      if (next.attempt_state !== "capture_complete") {
        fail("effect_in_flight_ambiguous", "lifecycle ended before capture complete");
      }
      next.process_running = false;
      break;
    case "post_capture_detector_board_info":
      next.attempt_state = "post_capture_validated";
      break;
  }
  validateAttemptState(next);
  return next;
}

export function attachLifecycleOwner(
  state,
  {
    leaseId,
    capabilitySha256,
    ownerPid,
    ownerStartFingerprintSha256,
    lifecycleDeadlineMs,
    checkpointCreatedMonotonicMs,
  },
) {
  validateAttemptState(state);
  if (
    state.effect_id !== "lifecycle_start" ||
    state.effect_phase !== "invoked" ||
    state.attempt_state !== "reinit_validated"
  ) {
    fail("lease_conflict", "lifecycle owner attachment is not authorized");
  }
  const next = structuredClone(state);
  next.attempt_state = "armed";
  next.lifecycle_substate = "removal_waiting";
  next.lifecycle_lease_id = leaseId;
  next.lifecycle_capability_sha256 = capabilitySha256;
  next.lifecycle_owner_pid = ownerPid;
  next.lifecycle_owner_start_fingerprint_sha256 = ownerStartFingerprintSha256;
  next.lifecycle_deadline_ms = lifecycleDeadlineMs;
  next.process_running = true;
  next.attestation_status = "waiting";
  next.armed_prohibited_sentinel_sha256 = canonicalDigest(
    next.armed_prohibited_sentinel_counts,
  );
  next.armed_permitted_lifecycle_sha256 = canonicalDigest(
    next.armed_permitted_lifecycle_counts,
  );
  return installCheckpoint(
    next,
    "plan13-lifecycle-removal",
    checkpointCreatedMonotonicMs,
  );
}

export function applyLifecycleOwnerEvent(state, event, values = {}) {
  validateAttemptState(state);
  if (
    state.effect_id !== "lifecycle_start" ||
    state.effect_phase !== "invoked" ||
    !state.process_running
  ) {
    fail("lease_owner_mismatch", "lifecycle owner is not active");
  }
  const next = structuredClone(state);
  switch (event) {
    case "absence-observing":
      if (next.attempt_state !== "removal_attested") {
        fail("checkpoint_state_mismatch", "removal was not attested");
      }
      next.attempt_state = "absence_observing";
      next.lifecycle_substate = "absence_observing";
      next.usb_absence_started_ms = values.usb_absence_started_ms;
      break;
    case "restore-waiting":
      if (next.attempt_state !== "absence_observing") {
        fail("checkpoint_state_mismatch", "absence observation is not active");
      }
      next.usb_absence_ended_ms = values.usb_absence_ended_ms;
      next.usb_absence_ms = values.usb_absence_ms;
      next.usb_absence_category = "continuous_at_least_5000";
      next.attempt_state = "restore_waiting";
      next.lifecycle_substate = "restore_waiting";
      next.restore_token_status = "waiting";
      next.capture_complete_permitted_lifecycle_counts.usb_absence_observation_session_count =
        1;
      return installCheckpoint(
        next,
        "plan13-lifecycle-restore",
        values.usb_absence_ended_ms,
      );
    case "reappearance-observing":
      if (next.attempt_state !== "restore_attested") {
        fail("checkpoint_state_mismatch", "restore was not attested");
      }
      next.attempt_state = "reappearance_observing";
      next.lifecycle_substate = "reappearance_observing";
      break;
    case "capture-running":
      if (next.attempt_state !== "reappearance_observing") {
        fail("checkpoint_state_mismatch", "reappearance observation is not active");
      }
      next.usb_reappearance_ms = values.usb_reappearance_ms;
      next.reappearance_elapsed_ms = values.reappearance_elapsed_ms;
      next.reappearance_category = "within_60000";
      next.capture_started_ms = values.capture_started_ms;
      next.capture_category = "running";
      next.attempt_state = "capture_running";
      next.lifecycle_substate = "monitoring";
      next.capture_complete_permitted_lifecycle_counts.usb_reappearance_observation_session_count =
        1;
      next.capture_complete_permitted_lifecycle_counts.retained_monitor_start_count =
        1;
      break;
    case "capture-complete":
      if (next.attempt_state !== "capture_running") {
        fail("checkpoint_state_mismatch", "capture is not running");
      }
      Object.assign(next, values);
      next.capture_category = "complete_360s";
      next.attempt_state = "capture_complete";
      next.lifecycle_substate = "complete";
      next.capture_complete_prohibited_sentinel_counts = structuredClone(
        next.armed_prohibited_sentinel_counts,
      );
      next.capture_complete_prohibited_sentinel_sha256 =
        next.armed_prohibited_sentinel_sha256;
      next.capture_complete_permitted_lifecycle_counts.removal_pty_write_count =
        1;
      next.capture_complete_permitted_lifecycle_counts.restore_pty_write_count =
        1;
      next.capture_complete_permitted_lifecycle_sha256 = canonicalDigest(
        next.capture_complete_permitted_lifecycle_counts,
      );
      break;
    default:
      fail("checkpoint_state_mismatch", "lifecycle event is not closed");
  }
  validateAttemptState(next);
  return next;
}

export function routeBlocker(reason, { lifecycleAuthorized = false, cleanupUnresolved = false } = {}) {
  requireEnum(reason, BLOCKER_REASONS, "blocker_reason");
  if (cleanupUnresolved || reason === "process_cleanup_unresolved") return "blocked_safe_unresolved_process";
  if (["sentinel_mismatch", "classifier_input_invalid", "classification_inconsistent"].includes(reason)) return "blocked_safe_evidence_invalid";
  if (reason === "production_markers_absent") return lifecycleAuthorized ? "gaps_found_same_chain_production_markers_absent" : "blocked_safe_evidence_invalid";
  if (reason === "none") return null;
  return lifecycleAuthorized ? "blocked_safe_evidence_invalid" : "blocked_safe_attempt_prerequisite";
}

export function canonicalDigest(value) {
  const stable = (input) => {
    if (Array.isArray(input)) return input.map(stable);
    if (input && typeof input === "object") {
      return Object.fromEntries(
        Object.keys(input)
          .sort()
          .map((key) => [key, stable(input[key])]),
      );
    }
    return input;
  };
  return sha256(JSON.stringify(stable(value)));
}
