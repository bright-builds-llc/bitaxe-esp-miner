import test from "node:test";
import assert from "node:assert/strict";
import { validateSafetyDiagnostic } from "./safety-diagnostics.mjs";
import { projectRecoveryPart } from "./recovery-evidence.mjs";
import { validateClientFailure } from "../str005-startup-probe/recovery-errors.mjs";

// Parsed form of the firmware's exact marker (bitaxe-runtime `the_detail_marker_is_a_closed_single_line`).
const detail = { category: "worker_revocation_detail", authoritative: false, generation: 1, reason: "unsafe_observation",
  trigger: "unsafe_sample", fact: "bus_voltage", state: "out_of_range", value_milli: 5512, age_ms: 40, since_safe_ms: 100, closed_ms: 500 };
const receipt = { category: "worker_preparation_receipt", authoritative: false, status: "valid", origin: "current_boot",
  interrupted: "false", source_hash: "0123456789abcdef", boot_ordinal: "23", generation: 3, sequence: 9, uptime_ms: "68990",
  last_completed_step: 4, current_step: 5, outcome: "failed", failure: "cancelled", heap_free: 3451, heap_largest: 2176, stack_free: 9516 };
const boot = { category: "boot", authoritative: false, boot_ordinal: 23, reset_reason: "other", uptime_ms: 1000 };
const exported = observations => ({ schema: "worker-diagnostic-export-v1", observations });

test("recovery diagnostics retain the revocation detail, receipt and rejection beside boot rows", () => {
  // Arrange
  const rejection = { category: "control_failure", authoritative: false, error: "session_failed" };
  const other = { category: "memory", authoritative: false, stage: "usb_install", free_bytes: 1, largest_block_bytes: 1, reserve_bytes: 1 };
  // Act
  const projected = projectRecoveryPart("diagnostics", exported([boot, detail, receipt, rejection, other]), { scope: "share" });
  // Assert
  assert.deepEqual(projected.observations, [boot, detail, receipt, rejection]);
  assert.equal(projected.omitted_count, 1);
});

test("a revocation detail outside its producer grammar is rejected rather than dropped", () => {
  // Arrange
  const cases = [{ ...detail, fact: "pool_url" }, { ...detail, trigger: "zero_fan" }, { ...detail, value_milli: 2 ** 31 },
    { ...detail, extra: true }, { ...detail, fact: "none", state: "none" }];
  // Act / Assert
  for (const row of cases) assert.throws(() => validateSafetyDiagnostic(row), /safety_diagnostic_shape|v2_object_shape/u);
});

test("a step-5 preparation receipt must agree with its outcome and failure", () => {
  // Arrange / Act / Assert
  assert.deepEqual(validateSafetyDiagnostic(receipt), receipt);
  assert.throws(() => validateSafetyDiagnostic({ ...receipt, failure: "none" }), /safety_diagnostic_shape/u);
  assert.throws(() => validateSafetyDiagnostic({ ...receipt, last_completed_step: 6 }), /safety_diagnostic_shape/u);
});

test("historical boot-only diagnostics still project unchanged", () => {
  // Arrange / Act
  const projected = projectRecoveryPart("diagnostics", exported([boot]), { scope: "share" });
  // Assert
  assert.deepEqual(projected, { schema: "str005-recovery-diagnostics-v1", observations: [boot], omitted_count: 0, authoritative: false });
});

test("a v2 client failure keeps only a closed Worker rejection", () => {
  // Arrange
  const failure = { schema: "str005-client-failure-v2", phase: "start", category: "command_rejected", observedAtMs: 12, rejection: "session_failed" };
  // Act / Assert
  assert.deepEqual(validateClientFailure(failure), failure);
  assert.deepEqual(validateClientFailure({ ...failure, rejection: null }).rejection, null);
  assert.throws(() => validateClientFailure({ ...failure, rejection: "pool password wrong" }), /startup_client_failure/u);
  assert.throws(() => validateClientFailure({ ...failure, schema: "str005-client-failure-v1" }), /v2_object_shape/u);
});
