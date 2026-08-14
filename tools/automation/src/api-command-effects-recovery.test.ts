import assert from "node:assert/strict";
import test from "node:test";

import { campaignRecoveryFactsFromDocuments } from "./api-command-effects-recovery.js";

test("declined checkpoint preserves only a fully joined paused safe stop", () => {
  // Arrange
  const result = {
    terminal_reason: "operator_paused",
    safe_stop: "pending",
    usb_cleanup: "ready",
  };
  const network = {
    recovery_pause_request_count: 1,
    command_effects: {
      pause_confirmed: true,
      resume_request_count: 0,
      identify_terminal_outcome: "declined",
    },
  };

  // Act
  const complete = campaignRecoveryFactsFromDocuments(result, network);
  const withoutRecovery = campaignRecoveryFactsFromDocuments(result, {
    ...network,
    recovery_pause_request_count: 0,
  });
  const withoutTypedTerminal = campaignRecoveryFactsFromDocuments(result, {
    ...network,
    command_effects: { ...network.command_effects, identify_terminal_outcome: "none" },
  });

  // Assert
  assert.deepEqual(complete, {
    safeStopConfirmed: true,
    cleanupComplete: true,
    recoveryAttempted: true,
    secondaryRecoveryFailure: false,
  });
  assert.equal(withoutRecovery.safeStopConfirmed, false);
  assert.equal(withoutTypedTerminal.safeStopConfirmed, false);
});
