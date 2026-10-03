// Independent validators for the safety diagnostics a Start attempt must retain. They mirror
// the Gate's closed grammars (worker-serial-diagnostics.ts, worker-preparation-diagnostics.ts);
// unknown keys or values are rejected, never persisted.
import { check, object } from "./values.mjs";

const CONTROL_ERRORS = ["invalid_frame", "invalid_request", "admission_required", "invalid_proof", "authentication_failed",
  "invalid_transition", "persistence_failed", "monotonic_reset", "session_failed", "restoration_pending", "stale_response",
  "encoding_failed"];
const ADMISSION_STAGES = ["idle", "admission", "readiness", "preparation", "pool_activation", "active", "cleanup", "complete"];
const ADMISSION_FAILURES = ["none", "admission", "readiness", "preparation", "pool_activation", "cleanup"];
const PREPARATION_INVALID = ["incomplete", "corrupt", "unavailable", "wrong_firmware"];
const PREPARATION_FAILURES = ["none", "cancelled", "safety_unavailable", "owner_unavailable", "queue_full", "reply_timeout",
  "hardware_write_failed", "fan_timeout", "unsupported_profile", "asic_failed", "asic_plan_invalid", "cooling_timeout",
  "cooling_proof_required"];
const TRIGGERS = ["unsafe_sample", "zero_fan", "no_safe_sample"];
const FACTS = ["none", "power", "bus_voltage", "current", "chip_temperature", "fan_rpm"];
const STATES = ["none", "expired", "stale", "unavailable", "fault", "out_of_range"];
export const SAFETY_CATEGORIES = Object.freeze(["control_failure", "worker_admission", "worker_preparation_receipt", "worker_revocation_detail"]);

const u32 = value => Number.isSafeInteger(value) && value >= 0 && value <= 0xffffffff;
const u32OrUnavailable = value => value === "unavailable" || u32(value);
const decimal = value => typeof value === "string" && /^\d{1,20}$/u.test(value) && BigInt(value) <= 18446744073709551615n;

function preparation(row) {
  if (row.status !== "valid") {
    object(row, ["category", "authoritative", "origin", "status"]);
    check(PREPARATION_INVALID.includes(row.status), "safety_diagnostic_shape");
  } else {
    object(row, ["category", "authoritative", "status", "origin", "interrupted", "source_hash", "boot_ordinal", "generation",
      "sequence", "uptime_ms", "last_completed_step", "current_step", "outcome", "failure", "heap_free", "heap_largest", "stack_free"]);
    const completed = row.last_completed_step, current = row.current_step;
    check(["true", "false"].includes(row.interrupted) && /^[0-9a-f]{16}$/u.test(row.source_hash) && decimal(row.boot_ordinal) &&
      decimal(row.uptime_ms) && u32(row.generation) && u32(row.sequence) && Number.isInteger(completed) && completed >= 0 &&
      completed <= 9 && Number.isInteger(current) && current >= 1 && current <= 9 &&
      ["started", "completed", "failed"].includes(row.outcome) && PREPARATION_FAILURES.includes(row.failure) &&
      ["heap_free", "heap_largest", "stack_free"].every(key => u32OrUnavailable(row[key])), "safety_diagnostic_shape");
    check(!(row.outcome === "started" && completed >= current) && !(row.outcome === "completed" && completed !== current) &&
      !(row.outcome === "failed" && completed > current) && (row.outcome === "failed") === (row.failure !== "none"),
    "safety_diagnostic_shape");
  }
  check(["current_boot", "previous_boot"].includes(row.origin), "safety_diagnostic_shape");
}

function revocation(row) {
  object(row, ["category", "authoritative", "generation", "reason", "trigger", "fact", "state", "value_milli", "age_ms",
    "since_safe_ms", "closed_ms"]);
  const value = row.value_milli;
  check(u32(row.generation) && row.reason === "unsafe_observation" && TRIGGERS.includes(row.trigger) && FACTS.includes(row.fact) &&
    STATES.includes(row.state) && (value === "unavailable" || Number.isSafeInteger(value) && value >= -2147483648 && value <= 2147483647) &&
    u32OrUnavailable(row.age_ms) && u32(row.since_safe_ms) && u32(row.closed_ms), "safety_diagnostic_shape");
  check((row.fact === "none") === (row.state === "none") && (row.trigger === "unsafe_sample" || row.fact === "none") &&
    (row.fact !== "none" || value === "unavailable" && row.age_ms === "unavailable") &&
    (row.state !== "unavailable" || value === "unavailable" && row.age_ms === "unavailable"), "safety_diagnostic_shape");
}

/** Validates one retained safety row in place; throws on any shape outside its producer grammar. */
export function validateSafetyDiagnostic(row) {
  check(SAFETY_CATEGORIES.includes(row?.category) && row.authoritative === false, "safety_diagnostic_shape");
  if (row.category === "control_failure") {
    object(row, ["category", "authoritative", "error"]);
    check(CONTROL_ERRORS.includes(row.error), "safety_diagnostic_shape");
  } else if (row.category === "worker_admission") {
    object(row, ["category", "authoritative", "stage", "first_failure", "readiness", "budget_reserved_ms", "budget_complete"]);
    check(ADMISSION_STAGES.includes(row.stage) && ADMISSION_FAILURES.includes(row.first_failure) && Number.isInteger(row.readiness) &&
      row.readiness >= 0 && row.readiness <= 63 && Number.isInteger(row.budget_reserved_ms) && row.budget_reserved_ms >= 0 &&
      row.budget_reserved_ms <= 240000 && ["true", "false"].includes(row.budget_complete), "safety_diagnostic_shape");
  } else if (row.category === "worker_preparation_receipt") preparation(row);
  else revocation(row);
  return structuredClone(row);
}

export const CONTROL_REJECTIONS = Object.freeze([...CONTROL_ERRORS]);
