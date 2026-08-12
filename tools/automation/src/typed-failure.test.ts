import assert from "node:assert/strict";
import test from "node:test";

import { ThemeDurabilityError } from "./theme-durability.js";
import { NetworkScanEvidenceError } from "./network-scan-evidence.js";
import { maybeTypedFailurePublicValue } from "./typed-failure.js";

test("theme durability failures retain their closed public projection", () => {
  // Arrange
  const error = new ThemeDurabilityError("process_failed", "safe failure", {
    stage: "initial_flash_monitor",
    flash_effect_status: "failed_no_device_effect",
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "initial_flash_monitor",
    flash_effect_status: "failed_no_device_effect",
  });
});

test("network scan failures retain the primary closed category facts", () => {
  // Arrange
  const error = new NetworkScanEvidenceError("hardware_blocked", "safe failure", {
    stage: "network_scan_capture",
    recovery_complete: true,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "network_scan_capture",
    recovery_complete: true,
  });
});
