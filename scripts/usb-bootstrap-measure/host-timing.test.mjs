import test from "node:test";
import assert from "node:assert/strict";
import { HOST_TIMING_STAGES, validateHostTiming } from "./host-timing.mjs";
function fixture() { return { schema: "bootstrap-host-timing-v1", clock: "host_monotonic", origin: "usb_session_acquired", physicalIdentityDigest: "a".repeat(64), sessionNonceSha256: "b".repeat(64), events: HOST_TIMING_STAGES.map((stage, i) => ({ stage, elapsedUs: i * 100000 })), resetChildSequence: 3, readerOpenCount: 1, readerReopenCount: 0, firstReadBytes: 92, missingStages: [], overflow: false, clockDiscontinuity: false, captureComplete: true, cleanupComplete: true, earliestFailure: null }; }
test("complete host timing proves only its own observation chain", () => { const value = fixture(); assert.equal(validateHostTiming(value), value); });
test("incomplete duration remains readable despite every stage being present", () => { const value = fixture(); value.captureComplete = false; assert.equal(validateHostTiming(value).captureComplete, false); });
test("pre-session failure carries no invented identity", () => { const value = { ...fixture(), origin: null, physicalIdentityDigest: null, sessionNonceSha256: null, events: [], resetChildSequence: null, readerOpenCount: 0, firstReadBytes: null, missingStages: [...HOST_TIMING_STAGES], captureComplete: false, earliestFailure: { stage: "preparation", category: "preparation_failed" } }; assert.equal(validateHostTiming(value), value); value.sessionNonceSha256 = "b".repeat(64); assert.throws(() => validateHostTiming(value), /unbound/u); });
for (const [name, mutate] of Object.entries({
  extra_private_field: v => { v.port = "private"; },
  event_private_field: v => { v.events[0].payload = "secret"; },
  missing_stage: v => { v.events.splice(3, 1); },
  clock_regression: v => { v.events[2].elapsedUs = 0; },
  duplicate_stage: v => { v.events[2].stage = v.events[1].stage; },
  reorder: v => { [v.events[1], v.events[2]] = [v.events[2], v.events[1]]; },
  unsafe_integer: v => { v.events[0].elapsedUs = Number.MAX_SAFE_INTEGER + 1; },
  first_byte_missing: v => { v.firstReadBytes = null; },
  reopen_false_completion: v => { v.readerOpenCount = 2; v.readerReopenCount = 1; },
  overflow_false_completion: v => { v.overflow = true; },
  unknown_failure: v => { v.earliestFailure = { stage: "read", category: "raw-error" }; },
})) test(`rejects ${name}`, () => { const value = fixture(); mutate(value); assert.throws(() => validateHostTiming(value), /host_timing_/u); });
test("observed clock failure survives as incomplete evidence", () => { const value = fixture(); value.events[2].elapsedUs = 0; value.clockDiscontinuity = true; value.captureComplete = false; value.earliestFailure = { stage: "clock", category: "clock_discontinuity" }; assert.equal(validateHostTiming(value).clockDiscontinuity, true); });
