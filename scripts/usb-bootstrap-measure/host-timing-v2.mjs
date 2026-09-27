import { HOST_TIMING_STAGES } from "./host-timing.mjs";
const required = [...HOST_TIMING_STAGES, "candidate_observed", "reader_bound", "quarantine_released", "capture_deadline_reached", "reader_joined"];
const fields = ["schema", "clock", "origin", "physicalIdentityDigest", "sessionNonceSha256", "events", "resetChildSequence", "readerOpenCount", "readerReopenCount", "firstReadBytes", "missingStages", "overflow", "clockDiscontinuity", "captureComplete", "cleanupComplete", "earliestFailure", "captureMode", "captureDurationMs", "readerBindingSha256", "quarantinedBytes", "capturedBytes", "quarantineReleased", "readerJoined", "captureOverflow"];
const stages = new Set("preparation session_admission reset handoff monitor_admission open read close clock overflow cleanup evidence binding quarantine join".split(" "));
const categories = new Set("concurrent_repo_session foreign_holder transport_absent identity_drift runtime_profile_unknown handoff_unsupported handoff_rejected_unsafe_state handoff_ready_timeout handoff_commit_timeout bus_reset_timeout same_worker_after_commit handoff_transition_timeout bootloader_ambiguous physical_identity_drift bootloader_sync_failed rom_admission_failed application_reappearance_timeout application_identity_mismatch recovery_required bootloader_connect_failed usb_enumeration_lost flash_failed_before_transfer flash_failed_after_transfer monitor_failed cleanup_failed recovery_not_observed repeated_boundary preparation_failed clock_discontinuity observation_overflow evidence_write_failed reader_binding_changed capture_overflow reader_join_timeout admission_incomplete".split(" "));
const check = (condition, code) => { if (!condition) throw Object.assign(Error(`host_timing_v2_${code}`), { code: `host_timing_v2_${code}` }); };
const object = (v, keys) => v && typeof v === "object" && !Array.isArray(v) && Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v, k));
const uint = v => Number.isSafeInteger(v) && v >= 0;
const sha = v => typeof v === "string" && /^[a-f0-9]{64}$/u.test(v);
export function validateHostTimingV2(value) {
  check(object(value, fields), "fields");
  check(value.schema === "bootstrap-host-timing-v2" && value.clock === "host_monotonic" && value.captureMode === "early_quarantined", "schema");
  for (const key of ["overflow", "clockDiscontinuity", "captureComplete", "cleanupComplete", "quarantineReleased", "readerJoined", "captureOverflow"]) check(typeof value[key] === "boolean", "boolean");
  check(uint(value.captureDurationMs) && value.captureDurationMs > 0 && Number.isSafeInteger(value.captureDurationMs * 1000) && uint(value.readerOpenCount) && value.readerOpenCount <= 0xffffffff &&
    uint(value.readerReopenCount) && value.readerReopenCount === Math.max(0, value.readerOpenCount - 1), "counts");
  check(uint(value.quarantinedBytes) && uint(value.capturedBytes) && value.quarantinedBytes <= value.capturedBytes && value.capturedBytes <= 16777216, "bytes");
  check(value.resetChildSequence === null || uint(value.resetChildSequence) && value.resetChildSequence > 0, "reset_sequence");
  check(value.firstReadBytes === null || uint(value.firstReadBytes) && value.firstReadBytes > 0 && value.firstReadBytes <= Math.min(value.capturedBytes, 4096), "first_read");
  check(Array.isArray(value.events) && value.events.length <= 24, "events");
  const events = new Map(); let previous = -1;
  for (const e of value.events) {
    check(object(e, ["stage", "elapsedUs"]) && [...required, "cancellation_requested"].includes(e.stage) && uint(e.elapsedUs) && !events.has(e.stage), "event");
    check(e.elapsedUs >= previous || value.clockDiscontinuity, "clock"); events.set(e.stage, e.elapsedUs); previous = e.elapsedUs;
  }
  check(Array.isArray(value.missingStages) && new Set(value.missingStages).size === value.missingStages.length && value.missingStages.length === required.filter(stage => !events.has(stage)).length && value.missingStages.every(stage => required.includes(stage) && !events.has(stage)), "missing_stages");
  const before = (a, b) => { if (events.has(b)) check(events.has(a) && (events.get(a) <= events.get(b) || value.clockDiscontinuity), "partial_order"); };
  for (const [a, b] of [["reset_command_call_start", "reset_command_call_end"], ["reset_command_call_end", "candidate_observed"], ["candidate_observed", "reader_open_start"],
    ["reader_open_start", "reader_opened"], ["reader_opened", "reader_bound"], ["reader_bound", "handoff_admitted"], ["handoff_start", "handoff_admitted"],
    ["handoff_admitted", "monitor_admitted"], ["monitor_admission_start", "monitor_admitted"], ["monitor_admitted", "quarantine_released"],
    ["reader_opened", "first_nonempty_read"], ["reader_opened", "reader_closed"], ["reader_closed", "reader_joined"], ["reader_opened", "capture_deadline_reached"]]) before(a, b);
  if (events.has("first_nonempty_read") && events.has("reader_closed")) before("first_nonempty_read", "reader_closed");
  check(events.has("first_nonempty_read") === (value.firstReadBytes !== null) && events.has("reader_open_start") === (value.readerOpenCount > 0), "joins");
  check(events.has("reader_bound") === (value.readerBindingSha256 !== null) && (value.readerBindingSha256 === null || sha(value.readerBindingSha256)), "binding");
  check(events.has("quarantine_released") === value.quarantineReleased && events.has("reader_joined") === value.readerJoined, "joins");
  if (value.origin === null) check(value.physicalIdentityDigest === null && value.sessionNonceSha256 === null && value.events.length === 0 && !value.captureComplete && value.readerOpenCount === 0 && value.resetChildSequence === null, "unbound");
  else check(value.origin === "usb_session_acquired" && sha(value.physicalIdentityDigest) && sha(value.sessionNonceSha256), "binding");
  if (value.earliestFailure !== null) check(object(value.earliestFailure, ["stage", "category"]) && stages.has(value.earliestFailure.stage) && categories.has(value.earliestFailure.category), "failure");
  if (events.has("capture_deadline_reached")) check(events.get("capture_deadline_reached") - events.get("reader_opened") >= value.captureDurationMs * 1000 || value.clockDiscontinuity, "duration");
  if (value.captureComplete) {
    check(value.earliestFailure === null && value.missingStages.length === 0 && value.readerOpenCount === 1 && value.readerReopenCount === 0 && value.quarantineReleased && value.readerJoined &&
      !value.overflow && !value.clockDiscontinuity && !value.captureOverflow && !events.has("cancellation_requested") && value.resetChildSequence !== null, "false_completion");
    check(events.get("quarantine_released") - events.get("reader_opened") < value.captureDurationMs * 1000, "admission_expired");
    before("quarantine_released", "capture_deadline_reached"); before("capture_deadline_reached", "reader_closed");
  }
  return value;
}
