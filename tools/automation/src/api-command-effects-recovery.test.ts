import assert from "node:assert/strict";
import test from "node:test";

import { campaignRecoveryFactsFromDocuments } from "./api-command-effects-recovery.js";

test("recovery preserves only a fully joined post-failure safe stop", () => {
  // Arrange
  const result = {
    terminal_reason: "operator_paused",
    safe_stop: "pending",
    usb_cleanup: "ready",
  };
  const network = {
    recovery_pause_request_count: 1,
    command_effects: {
      recovery_pause_api_confirmed: true,
      recovery_pause_serial_confirmed: true,
      recovery_safe_stop_confirmed: true,
      recovery_terminal_outcome: "confirmed",
    },
  };

  // Act
  const complete = campaignRecoveryFactsFromDocuments(result, network);
  const withoutRecovery = campaignRecoveryFactsFromDocuments(result, {
    ...network,
    recovery_pause_request_count: 0,
  });
  const withoutSerialJoin = campaignRecoveryFactsFromDocuments(result, {
    ...network,
    command_effects: { ...network.command_effects, recovery_pause_serial_confirmed: false },
  });
  const timedOut = campaignRecoveryFactsFromDocuments(result, {
    ...network,
    command_effects: {
      ...network.command_effects,
      recovery_safe_stop_confirmed: false,
      recovery_terminal_outcome: "timed_out",
    },
  });

  // Assert
  assert.deepEqual(complete, {
    safeStopConfirmed: true,
    cleanupComplete: true,
    recoveryAttempted: true,
    secondaryRecoveryFailure: false,
  });
  assert.equal(withoutRecovery.safeStopConfirmed, false);
  assert.equal(withoutSerialJoin.safeStopConfirmed, false);
  assert.equal(timedOut.safeStopConfirmed, false);
  assert.equal(timedOut.secondaryRecoveryFailure, true);
});

test("already proved paused safe stop needs no redundant recovery request", () => {
  // Arrange
  const result = { safe_stop: "pending", usb_cleanup: "ready" };
  const network = {
    recovery_pause_request_count: 0,
    command_effects: {
      recovery_pause_api_confirmed: true,
      recovery_pause_serial_confirmed: true,
      recovery_safe_stop_confirmed: true,
      recovery_terminal_outcome: "already_confirmed",
    },
  };

  // Act
  const facts = campaignRecoveryFactsFromDocuments(result, network);

  // Assert
  assert.deepEqual(facts, {
    safeStopConfirmed: true,
    cleanupComplete: true,
    recoveryAttempted: false,
    secondaryRecoveryFailure: false,
  });
});
