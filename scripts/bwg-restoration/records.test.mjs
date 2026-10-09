import test from "node:test";
import assert from "node:assert/strict";
import { context, journal, pageState } from "./fixtures.test-helper.mjs";
import { parsePageState, parseRecord } from "./records.mjs";

const throwsWith = (operation, ...codes) => assert.throws(operation, (error) => codes.includes(error.code));
const SHAPE_ERRORS = ["page_admission_shape", "object_fields", "object_shape"];
const SETTLED = Object.freeze({ stage: "complete", firstFailure: "none", readiness: 63 });
const record = (operation, result, state = pageState()) => ({ operation, outcome: "ok", result, state });

test("a page state carries the Gate's admission observation, null before any", () => {
  // Arrange
  const observed = pageState({ admission: SETTLED });
  const unobserved = pageState({ admission: null });
  // Act
  const parsed = [parsePageState(observed, context).admission, parsePageState(unobserved, context).admission];
  // Assert
  assert.deepEqual(parsed, [SETTLED, null]);
});

test("a page state from a page without the admission field still parses", () => {
  // Arrange
  const state = pageState();
  // Act
  const parsed = parsePageState(state, context);
  // Assert
  assert.equal(parsed.admission, undefined);
});

test("an admission observation outside the closed shape is refused", () => {
  // Arrange
  const malformed = [
    { ...SETTLED, stage: "settled" },
    { ...SETTLED, firstFailure: "active" },
    { ...SETTLED, readiness: 64 },
    { ...SETTLED, readiness: -1 },
    { ...SETTLED, readiness: 1.5 },
    { ...SETTLED, budget_reserved_ms: 0 },
    { stage: "idle", firstFailure: "none" },
    "idle",
  ];
  // Act / Assert
  for (const admission of malformed) throwsWith(() => parsePageState(pageState({ admission }), context), ...SHAPE_ERRORS);
});

test("the admissionDiagnostic operation records exactly its admission observation", () => {
  // Arrange
  const observed = record("admissionDiagnostic", { admission: SETTLED }, pageState({ admission: SETTLED }));
  const unobserved = record("admissionDiagnostic", { admission: null }, pageState({ admission: null }));
  // Act
  const parsed = [parseRecord(observed, context).result, parseRecord(unobserved, context).result];
  // Assert
  assert.deepEqual(parsed, [{ admission: SETTLED }, { admission: null }]);
});

test("a malformed admissionDiagnostic result is refused", () => {
  // Arrange
  const malformed = [null, {}, { admission: SETTLED, extra: true }, { admission: { ...SETTLED, stage: "unknown" } }];
  // Act / Assert
  for (const result of malformed) throwsWith(() => parseRecord(record("admissionDiagnostic", result), context), ...SHAPE_ERRORS);
});

test("the admission_observed journal event carries exactly a closed first-failure boundary", () => {
  // Arrange
  const state = pageState({ entries: journal(["connected", "admission_observed:none", "admission_observed:preparation"]) });
  // Act
  const parsed = parsePageState(state, context);
  // Assert
  assert.deepEqual(parsed.journal.entries.map((entry) => entry.category), [undefined, "none", "preparation"]);
});

test("an admission_observed event without a closed first-failure category is refused", () => {
  // Arrange
  const states = [pageState({ entries: journal(["admission_observed"]) }), pageState({ entries: journal(["admission_observed:active"]) })];
  // Act / Assert
  for (const state of states) throwsWith(() => parsePageState(state, context), "page_journal_entry");
});

test("a page state carries the identity and pool trackers, null before their first observation", () => {
  // Arrange
  const observed = pageState({ deviceIdentity: { epoch: 1, observations: 4 }, poolConfiguration: { observations: 3, changed: false } });
  const unobserved = pageState({ deviceIdentity: null, poolConfiguration: null });
  // Act
  const parsed = [parsePageState(observed, context), parsePageState(unobserved, context)];
  // Assert
  assert.deepEqual(parsed.map((state) => [state.deviceIdentity, state.poolConfiguration]),
    [[{ epoch: 1, observations: 4 }, { observations: 3, changed: false }], [null, null]]);
});

test("identity and pool trackers outside their closed shapes are refused", () => {
  // Arrange
  const malformed = [
    { deviceIdentity: { epoch: 0, observations: 1 } },
    { deviceIdentity: { epoch: 2, observations: 1 } },
    { deviceIdentity: { epoch: 1, observations: 1, fingerprint: "a".repeat(64) } },
    { deviceIdentity: { epoch: 1 } },
    { deviceIdentity: 1 },
    { poolConfiguration: { observations: 0, changed: false } },
    { poolConfiguration: { observations: 1, changed: "no" } },
    { poolConfiguration: { observations: 1, changed: false, sha256: "a".repeat(64) } },
    { poolConfiguration: { changed: false } },
  ];
  // Act / Assert
  for (const fields of malformed) {
    throwsWith(() => parsePageState(pageState(fields), context), "page_device_identity_shape", "page_pool_configuration_shape", ...SHAPE_ERRORS);
  }
});

test("the identity and pool change events are accepted only without a category", () => {
  // Arrange
  const accepted = pageState({ entries: journal(["connected", "device_identity_changed", "pool_configuration_changed"]) });
  const categorized = [pageState({ entries: journal(["device_identity_changed:mismatch"]) }), pageState({ entries: journal(["pool_configuration_changed:true"]) })];
  // Act
  const events = parsePageState(accepted, context).journal.entries.map((entry) => entry.event);
  // Assert
  assert.deepEqual(events, ["connected", "device_identity_changed", "pool_configuration_changed"]);
  for (const state of categorized) throwsWith(() => parsePageState(state, context), "page_journal_entry");
});

test("the bootReview operation records exactly this boot's reset cause", () => {
  // Arrange
  const causes = ["power_on", "software_cpu", "watchdog", "panic", "brownout", "other"];
  // Act
  const parsed = causes.map((resetCause) => parseRecord(record("bootReview", { schema: "worker-boot-review-v1", resetCause }), context).result.resetCause);
  // Assert
  assert.deepEqual(parsed, causes);
});

test("a malformed boot review is refused", () => {
  // Arrange
  const malformed = [null, { schema: "worker-boot-review-v1", resetCause: "unplugged" }, { schema: "worker-boot-review-v2", resetCause: "power_on" },
    { schema: "worker-boot-review-v1", resetCause: "power_on", bootCount: 3 }, { schema: "worker-boot-review-v1" }];
  // Act / Assert
  for (const result of malformed) throwsWith(() => parseRecord(record("bootReview", result), context), "boot_review_shape", ...SHAPE_ERRORS);
});

const rejection = (schema, last) => ({ schema, bootRejections: 1, last,
  highWater: { advancedThisBoot: false, fingerprintMatchesLatestObservation: true, fingerprintFirstObservedEpoch: 1 } });
const LAST = Object.freeze({ ordinal: 1, operation: "renew", signature: "valid", context: "current", replayGuard: "at_or_below_durable_high_water" });

test("a version 2 rejection review carries the safe stop the rejection triggered", () => {
  // Arrange
  const reviews = [rejection("worker-authorization-rejection-review-v2", { ...LAST, safeStop: "control_failed" }),
    rejection("worker-authorization-rejection-review-v2", { ...LAST, safeStop: "none" }), rejection("worker-authorization-rejection-review-v1", LAST)];
  // Act
  const parsed = reviews.map((review) => parseRecord(record("authorizationRejectionReview", review), context).result.last.safeStop);
  // Assert
  assert.deepEqual(parsed, ["control_failed", "none", undefined]);
});

test("a rejection review whose safe stop does not match its version is refused", () => {
  // Arrange
  const malformed = [rejection("worker-authorization-rejection-review-v1", { ...LAST, safeStop: "none" }),
    rejection("worker-authorization-rejection-review-v2", LAST), rejection("worker-authorization-rejection-review-v2", { ...LAST, safeStop: "unplugged" }),
    rejection("worker-authorization-rejection-review-v3", { ...LAST, safeStop: "none" })];
  // Act / Assert
  for (const review of malformed) {
    throwsWith(() => parseRecord(record("authorizationRejectionReview", review), context), "rejection_review_shape", "rejection_safe_stop_shape",
      ...SHAPE_ERRORS);
  }
});

test("the boot_reviewed journal event carries exactly a closed reset cause", () => {
  // Arrange
  const accepted = pageState({ entries: journal(["connected", "status_reviewed:reboot", "boot_reviewed:power_on", "boot_reviewed:brownout"]) });
  const refused = [pageState({ entries: journal(["boot_reviewed"]) }), pageState({ entries: journal(["boot_reviewed:unplugged"]) })];
  // Act
  const categories = parsePageState(accepted, context).journal.entries.map((entry) => entry.category);
  // Assert
  assert.deepEqual(categories, [undefined, "reboot", "power_on", "brownout"]);
  for (const state of refused) throwsWith(() => parsePageState(state, context), "page_journal_entry");
});
