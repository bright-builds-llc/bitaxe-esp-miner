import assert from "node:assert/strict";
import test from "node:test";

import { parsePanicDiagnostic } from "./hashrate-monitor-panic-diagnostic.js";

const emptyDiagnostic = {
  schema: "mining-campaign-serial-diagnostics-v4",
  runtime_attestation_mixed_reset_reason: "none",
  panic_signature: "none",
  panic_task_family: "none",
  panic_signature_count: 0,
};

test("recognized stack overflow returns only its closed tuple", () => {
  // Arrange
  const diagnostic = {
    ...emptyDiagnostic,
    runtime_attestation_mixed_reset_reason: "panic",
    panic_signature: "stack_overflow",
    panic_task_family: "production_mining_session",
    panic_signature_count: 2,
  };

  // Act
  const parsed = parsePanicDiagnostic(diagnostic);

  // Assert
  assert.equal(parsed.mixedResetReason, "panic");
  assert.deepEqual(parsed.maybeFailure, {
    panic_signature: "stack_overflow",
    panic_task_family: "production_mining_session",
    panic_signature_count: 2,
  });
});

test("panic reset without a captured signature returns unknown", () => {
  // Arrange
  const diagnostic = {
    ...emptyDiagnostic,
    runtime_attestation_mixed_reset_reason: "panic",
    panic_signature: "unknown",
  };

  // Act
  const parsed = parsePanicDiagnostic(diagnostic);

  // Assert
  assert.deepEqual(parsed.maybeFailure, {
    panic_signature: "unknown",
    panic_task_family: "none",
    panic_signature_count: 0,
  });
});

test("ordinary observed diagnostics contain no panic failure", () => {
  // Arrange / Act
  const parsed = parsePanicDiagnostic(emptyDiagnostic);

  // Assert
  assert.equal(parsed.mixedResetReason, "none");
  assert.equal(parsed.maybeFailure, undefined);
});

test("inconsistent or open panic values fail closed", () => {
  for (const diagnostic of [
    { ...emptyDiagnostic, schema: "private-v5" },
    { ...emptyDiagnostic, runtime_attestation_mixed_reset_reason: "private-reset" },
    { ...emptyDiagnostic, panic_signature: "private-signature" },
    { ...emptyDiagnostic, panic_task_family: "private-task" },
    { ...emptyDiagnostic, panic_signature_count: -1 },
    { ...emptyDiagnostic, runtime_attestation_mixed_reset_reason: "panic" },
    { ...emptyDiagnostic, panic_signature: "unknown" },
    {
      ...emptyDiagnostic,
      panic_signature: "stack_smashing",
      panic_task_family: "main",
      panic_signature_count: 1,
    },
  ]) {
    // Act / Assert
    assert.throws(() => parsePanicDiagnostic(diagnostic));
  }
});
