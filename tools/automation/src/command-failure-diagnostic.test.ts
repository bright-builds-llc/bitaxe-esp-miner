import assert from "node:assert/strict";
import test from "node:test";

import {
  campaignFailureFactsFromDocuments,
  parseCommandFailureDiagnostic,
} from "./operator-sensor-diagnostic.js";

const validDiagnostic = {
  schema: "mining-command-failure-diagnostic-v1",
  phase: "pause",
  cause: "websocket_witness",
} as const;

test("command failure diagnostic accepts only the closed public labels", () => {
  // Arrange
  const diagnostic = { ...validDiagnostic, trusted_origin: "sensitive-origin" };

  // Act
  const parsed = parseCommandFailureDiagnostic(diagnostic);

  // Assert
  assert.deepEqual(parsed, validDiagnostic);
  assert.doesNotMatch(JSON.stringify(parsed), /sensitive-origin/u);
});

test("command request is a closed redaction-safe failure cause", () => {
  // Arrange
  const diagnostic = { ...validDiagnostic, cause: "command_request" } as const;

  // Act
  const parsed = parseCommandFailureDiagnostic(diagnostic);

  // Assert
  assert.deepEqual(parsed, diagnostic);
});

test("malformed command failure diagnostic cannot erase recovery facts", () => {
  // Arrange
  const result = { safe_stop: "confirmed", usb_cleanup: "ready" };
  const network = {
    recovery_pause_request_count: 1,
    command_effects: {
      recovery_pause_api_confirmed: true,
      recovery_pause_serial_confirmed: true,
      recovery_safe_stop_confirmed: true,
      recovery_terminal_outcome: "confirmed",
    },
    command_failure: { ...validDiagnostic, phase: "private-origin" },
  };

  // Act
  const facts = campaignFailureFactsFromDocuments(result, network);

  // Assert
  assert.equal(facts.maybeCommandFailure, undefined);
  assert.deepEqual(facts.recovery, {
    safeStopConfirmed: true,
    cleanupComplete: true,
    recoveryAttempted: true,
    secondaryRecoveryFailure: false,
  });
});

test("campaign failure facts expose no private diagnostic values", () => {
  // Arrange
  const result = { safe_stop: "pending", usb_cleanup: "ready" };
  const network = {
    command_failure: {
      ...validDiagnostic,
      boot_session: "private-session",
      origin: "private-origin",
      port: "private-port",
    },
  };

  // Act
  const facts = campaignFailureFactsFromDocuments(result, network);
  const serialized = JSON.stringify(facts);

  // Assert
  assert.deepEqual(facts.maybeCommandFailure, validDiagnostic);
  assert.doesNotMatch(serialized, /private-session|private-origin|private-port/u);
});
