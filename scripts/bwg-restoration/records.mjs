// Closed parsers for what the restoration page and its supervisor client send. Shapes mirror the Gate page
// (web/worker-restoration-operations.ts, worker-restoration-qualification.ts); unknown fields fail closed.
import { exactObject, QualificationError, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { ADMISSION_FAILURES, ADMISSION_READINESS_MAXIMUM, ADMISSION_STAGES, PAGE_JOURNAL_EVENTS, PAGE_OPERATIONS, RESET_CAUSES, TOKEN,
  UNCATEGORIZED_JOURNAL_EVENTS } from "./contract.mjs";

const RESTORATION_REASONS = ["paused", "cancelled", "lease_expired", "lost_continuity", "monotonic_reset", "reboot", "challenge_satisfied",
  "challenge_expired", "tab_closed", "connectivity_lost", "control_failed"];
const u32 = (value, minimum = 0) => Number.isSafeInteger(value) && value >= minimum && value <= 0xffffffff;

function device(value) {
  exactObject(value, ["state", "restoration"], ["reason"]);
  requireCondition(["mining", "baseline"].includes(value.state) && ["pending", "not_required", "confirmed"].includes(value.restoration) &&
    (value.reason === undefined || RESTORATION_REASONS.includes(value.reason)), "page_device_shape");
  return value;
}

/** The page's last admission observation: `null` before any, else exactly `{stage, firstFailure, readiness}`. */
function admission(value) {
  if (value === null) return value;
  exactObject(value, ["stage", "firstFailure", "readiness"]);
  requireCondition(ADMISSION_STAGES.includes(value.stage) && ADMISSION_FAILURES.includes(value.firstFailure) &&
    Number.isSafeInteger(value.readiness) && value.readiness >= 0 && value.readiness <= ADMISSION_READINESS_MAXIMUM, "page_admission_shape");
  return value;
}

/**
 * The page-local device-identity tracker: `null` before the first preservation status, else exactly
 * `{epoch, observations}` (distinct identity digests seen in this page lifetime, and preservation observations).
 */
function deviceIdentity(value) {
  if (value === null) return value;
  exactObject(value, ["epoch", "observations"]);
  requireCondition(u32(value.epoch, 1) && u32(value.observations, 1) && value.epoch <= value.observations, "page_device_identity_shape");
  return value;
}

/** The page-local pool tracker: `null` before the first preservation v2 status, else exactly `{observations, changed}`. */
function poolConfiguration(value) {
  if (value === null) return value;
  exactObject(value, ["observations", "changed"]);
  requireCondition(u32(value.observations, 1) && typeof value.changed === "boolean", "page_pool_configuration_shape");
  return value;
}

/**
 * `admission_observed` carries exactly the observed first-failure boundary and `boot_reviewed` exactly the reset cause;
 * the identity and pool change events carry no category; other categories are closed tokens.
 */
function entryCategory(entry) {
  if (entry.event === "admission_observed") return ADMISSION_FAILURES.includes(entry.category);
  if (entry.event === "boot_reviewed") return RESET_CAUSES.includes(entry.category);
  if (UNCATEGORIZED_JOURNAL_EVENTS.includes(entry.event)) return entry.category === undefined;
  return entry.category === undefined || TOKEN.test(entry.category);
}

function journal(value) {
  exactObject(value, ["entries", "dropped"]);
  requireCondition(Array.isArray(value.entries) && value.entries.length <= 256 && u32(value.dropped), "page_journal_shape");
  let previous = 0;
  for (const entry of value.entries) {
    exactObject(entry, ["ordinal", "event"], ["category"]);
    requireCondition(u32(entry.ordinal, 1) && entry.ordinal > previous && PAGE_JOURNAL_EVENTS.includes(entry.event) && entryCategory(entry),
      "page_journal_entry");
    previous = entry.ordinal;
  }
  return value;
}

/** The published page state (`worker-restoration-page-v1`); identity values are checked against the context. */
export function parsePageState(value, context) {
  exactObject(value, ["schema", "gateCommit", "status", "connected", "leaseActive", "leaseLoaded", "renewalsRemaining", "stimulusUsed",
    "highWaterEpoch", "journal"], ["configurationFailure", "expectedFirmwareSourceCommit", "expectedAppElfSha256", "device", "failure", "admission",
    "deviceIdentity", "poolConfiguration"]);
  requireCondition(value.schema === "worker-restoration-page-v1" && value.gateCommit === context.gate_commit &&
    value.expectedFirmwareSourceCommit === context.firmware_commit && value.expectedAppElfSha256 === context.app_elf_sha256 &&
    value.configurationFailure === undefined, "page_identity");
  requireCondition(typeof value.status === "string" && TOKEN.test(value.status) && typeof value.connected === "boolean" &&
    typeof value.leaseActive === "boolean" && typeof value.leaseLoaded === "boolean" && [0, 1].includes(value.renewalsRemaining) &&
    typeof value.stimulusUsed === "boolean" && u32(value.highWaterEpoch) &&
    (value.failure === undefined || PAGE_JOURNAL_EVENTS.includes(value.failure)), "page_state_shape");
  if (value.device !== undefined) device(value.device);
  if (value.admission !== undefined) admission(value.admission);
  if (value.deviceIdentity !== undefined) deviceIdentity(value.deviceIdentity);
  if (value.poolConfiguration !== undefined) poolConfiguration(value.poolConfiguration);
  journal(value.journal);
  return value;
}

export function parseStimulusAck(value) {
  exactObject(value, ["schema", "offsetMilliseconds", "armedForMilliseconds"]);
  requireCondition(value.schema === "worker-clock-discontinuity-stimulus-v1" && value.offsetMilliseconds === 1000 &&
    value.armedForMilliseconds === 2000, "stimulus_ack_shape");
  return value;
}

export function parseStimulusReview(value) {
  exactObject(value, ["schema", "state", "offsetMilliseconds", "discontinuitiesDetected"]);
  requireCondition(value.schema === "worker-clock-discontinuity-stimulus-review-v1" && ["idle", "armed", "consumed", "expired"].includes(value.state) &&
    value.offsetMilliseconds === 1000 && u32(value.discontinuitiesDetected), "stimulus_review_shape");
  return value;
}

const REJECTION_REVIEW_SCHEMAS = { "worker-authorization-rejection-review-v1": [], "worker-authorization-rejection-review-v2": ["safeStop"] };

/**
 * The page's projection of the rejection review: no digest leaves the page, only epoch comparisons. Version 2 adds
 * `last.safeStop`, the safe stop the rejection itself triggered (`none` or a restoration reason).
 */
export function parseRejectionReview(value) {
  exactObject(value, ["schema", "bootRejections", "last", "highWater"]);
  const extra = REJECTION_REVIEW_SCHEMAS[value.schema];
  requireCondition(extra !== undefined && u32(value.bootRejections) && (value.bootRejections === 0) === (value.last === null), "rejection_review_shape");
  if (value.last !== null) {
    exactObject(value.last, ["ordinal", "operation", "signature", "context", "replayGuard", ...extra]);
    requireCondition(extra.length === 0 || value.last.safeStop === "none" || RESTORATION_REASONS.includes(value.last.safeStop), "rejection_safe_stop_shape");
    requireCondition(u32(value.last.ordinal, 1) && value.last.ordinal <= value.bootRejections && ["start", "renew"].includes(value.last.operation) &&
      ["valid", "invalid", "not_evaluated"].includes(value.last.signature) && ["current", "mismatch", "expired", "absent"].includes(value.last.context) &&
      ["fresh", "at_or_below_durable_high_water", "unavailable", "not_evaluated"].includes(value.last.replayGuard), "rejection_record_shape");
  }
  exactObject(value.highWater, ["advancedThisBoot", "fingerprintMatchesLatestObservation", "fingerprintFirstObservedEpoch"]);
  requireCondition(typeof value.highWater.advancedThisBoot === "boolean" && typeof value.highWater.fingerprintMatchesLatestObservation === "boolean" &&
    (value.highWater.fingerprintFirstObservedEpoch === null || u32(value.highWater.fingerprintFirstObservedEpoch, 1)), "rejection_high_water_shape");
  return value;
}

/** The device's read-only boot review: exactly this boot's reset cause. */
export function parseBootReview(value) {
  exactObject(value, ["schema", "resetCause"]);
  requireCondition(value.schema === "worker-boot-review-v1" && RESET_CAUSES.includes(value.resetCause), "boot_review_shape");
  return value;
}

function parseReplayOutcome(value) {
  exactObject(value, ["operation", "outcome"], ["category"]);
  requireCondition(["start", "renew"].includes(value.operation) && ["accepted", "rejected", "failed"].includes(value.outcome) &&
    (value.outcome === "accepted") === (value.category === undefined) && (value.category === undefined || TOKEN.test(value.category)), "replay_outcome_shape");
  return value;
}

function parseAdmissionDiagnostic(value) {
  exactObject(value, ["admission"]);
  admission(value.admission);
  return value;
}

function parseCheckpointAnswer(value) {
  exactObject(value, ["checkpoint"]);
  requireCondition(TOKEN.test(value.checkpoint), "checkpoint_answer_shape");
  return value;
}

/** Operations whose result is a closed value; every other operation reports only the page state. */
const RESULT_PARSERS = {
  triggerClockDiscontinuity: parseStimulusAck,
  clockDiscontinuityStimulusReview: parseStimulusReview,
  authorizationRejectionReview: parseRejectionReview,
  statusReview: (value) => value === null ? null : device(value),
  replayArtifact: parseReplayOutcome,
  beginPhysicalWindow: parseCheckpointAnswer,
  armPhysicalWindow: parseCheckpointAnswer,
  physicalWindowState: parseCheckpointAnswer,
  admissionDiagnostic: parseAdmissionDiagnostic,
  bootReview: parseBootReview,
};

/** One `POST /record` body from the supervisor client. */
export function parseRecord(input, context) {
  exactObject(input, ["operation", "outcome", "result", "state"], ["error"]);
  requireCondition(PAGE_OPERATIONS.includes(input.operation) && ["ok", "error"].includes(input.outcome), "record_operation");
  if (input.outcome === "error") requireCondition(typeof input.error === "string" && TOKEN.test(input.error) && input.result === null, "record_error_shape");
  else requireCondition(input.error === undefined, "record_error_shape");
  const parser = RESULT_PARSERS[input.operation];
  if (input.outcome === "ok" && parser) parser(input.result);
  else requireCondition(input.result === null, "record_result_shape");
  parsePageState(input.state, context);
  return input;
}

/** `POST /completion-review`: both reviews and the final state, as the Gate page submits them. */
export function parseCompletionReview(input, context) {
  exactObject(input, ["nonce", "reviews", "final_state"]);
  requireCondition(typeof input.nonce === "string", "completion_nonce");
  exactObject(input.reviews, ["stimulus", "rejection"]);
  parseStimulusReview(input.reviews.stimulus);
  parseRejectionReview(input.reviews.rejection);
  parsePageState(input.final_state, context);
  return input;
}

/** The secrets this server holds must never appear in anything the page sends back. */
export function requireSecretsAbsent(value, secrets) {
  const text = JSON.stringify(value);
  for (const secret of secrets) {
    if (typeof secret === "string" && secret.length >= 8 && text.includes(secret)) throw new QualificationError("credential_in_record");
  }
}

