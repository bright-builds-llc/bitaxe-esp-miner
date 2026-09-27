export const HOST_TIMING_STAGES = Object.freeze([
  "reset_command_call_start", "reset_command_call_end", "handoff_start", "handoff_admitted",
  "monitor_admission_start", "monitor_admitted", "reader_open_start", "reader_opened",
  "first_nonempty_read", "reader_closed",
]);
const fields = ["schema", "clock", "origin", "physicalIdentityDigest", "sessionNonceSha256", "events", "resetChildSequence", "readerOpenCount", "readerReopenCount", "firstReadBytes", "missingStages", "overflow", "clockDiscontinuity", "captureComplete", "cleanupComplete", "earliestFailure"];
const failureStages = new Set("preparation session_admission reset handoff monitor_admission open read close clock overflow cleanup evidence".split(" "));
const categories = new Set(("concurrent_repo_session foreign_holder transport_absent identity_drift runtime_profile_unknown handoff_unsupported handoff_rejected_unsafe_state handoff_ready_timeout handoff_commit_timeout bus_reset_timeout same_worker_after_commit handoff_transition_timeout bootloader_ambiguous physical_identity_drift bootloader_sync_failed rom_admission_failed application_reappearance_timeout application_identity_mismatch recovery_required bootloader_connect_failed usb_enumeration_lost flash_failed_before_transfer flash_failed_after_transfer monitor_failed cleanup_failed recovery_not_observed repeated_boundary preparation_failed clock_discontinuity observation_overflow evidence_write_failed").split(" "));
const check = (condition, code) => { if (!condition) throw Object.assign(Error("host_timing_" + code), { code: "host_timing_" + code }); };
const object = (value, keys) => value !== null && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
const uint = value => Number.isSafeInteger(value) && value >= 0;
const sha = value => typeof value === "string" && /^[a-f0-9]{64}$/u.test(value);
/** Validate claims independently; an incomplete but internally consistent capture remains readable. */
export function validateHostTiming(value) {
  check(object(value, fields), "fields");
  check(value.schema === "bootstrap-host-timing-v1" && value.clock === "host_monotonic", "schema");
  for (const key of ["overflow", "clockDiscontinuity", "captureComplete", "cleanupComplete"]) check(typeof value[key] === "boolean", "boolean");
  check([value.readerOpenCount, value.readerReopenCount].every(n => uint(n) && n <= 0xffffffff), "counts");
  check(value.readerReopenCount === Math.max(0, value.readerOpenCount - 1), "reopens");
  check(value.resetChildSequence === null || uint(value.resetChildSequence) && value.resetChildSequence > 0 && value.resetChildSequence <= 0xffffffff, "reset_sequence");
  check(value.firstReadBytes === null || uint(value.firstReadBytes) && value.firstReadBytes > 0, "first_read");
  check(Array.isArray(value.events) && value.events.length <= 16, "events");
  const seen = new Set(); let previous = -1, elapsed = -1;
  for (const event of value.events) {
    check(object(event, ["stage", "elapsedUs"]) && HOST_TIMING_STAGES.includes(event.stage) && uint(event.elapsedUs), "event");
    const index = HOST_TIMING_STAGES.indexOf(event.stage);
    check(!seen.has(event.stage), "duplicate_stage");
    // Retries may first yield bytes after the original reader closed; that chain is incomplete.
    check(index > previous || value.readerReopenCount > 0, "stage_order");
    check(event.elapsedUs >= elapsed || value.clockDiscontinuity, "clock");
    seen.add(event.stage); previous = index; elapsed = event.elapsedUs;
  }
  const missing = HOST_TIMING_STAGES.filter(stage => !seen.has(stage));
  check(JSON.stringify(value.missingStages) === JSON.stringify(missing), "missing_stages");
  check(seen.has("first_nonempty_read") === (value.firstReadBytes !== null), "first_read_join");
  check(seen.has("reader_open_start") === (value.readerOpenCount > 0), "open_join");
  check(!seen.has("reader_opened") || seen.has("reader_open_start"), "opened_without_start");
  check(!seen.has("reader_closed") || seen.has("reader_opened"), "closed_without_open");
  if (value.origin === null) {
    check(value.physicalIdentityDigest === null && value.sessionNonceSha256 === null && value.events.length === 0 && !value.captureComplete && value.resetChildSequence === null && value.readerOpenCount === 0, "unbound");
  } else check(value.origin === "usb_session_acquired" && sha(value.physicalIdentityDigest) && sha(value.sessionNonceSha256), "binding");
  if (value.earliestFailure !== null) check(object(value.earliestFailure, ["stage", "category"]) && failureStages.has(value.earliestFailure.stage) && categories.has(value.earliestFailure.category), "failure");
  // All stages alone do not prove that the configured capture duration elapsed.
  if (value.captureComplete) check(missing.length === 0 && value.readerOpenCount === 1 && value.readerReopenCount === 0 && !value.overflow && !value.clockDiscontinuity && value.resetChildSequence !== null, "false_completion");
  return value;
}
