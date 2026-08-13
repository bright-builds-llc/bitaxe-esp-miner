import assert from "node:assert/strict";
import test from "node:test";

import { ThemeDurabilityError } from "./theme-durability.js";
import { AsicInitializationEvidenceError } from "./asic-initialization-evidence.js";
import { AsicPowerInitializationEvidenceError } from "./asic-power-initialization-evidence.js";
import { CoreVoltageControlEvidenceError } from "./core-voltage-control-evidence.js";
import { Ina260EvidenceError } from "./ina260-evidence.js";
import { Emc2101ThermalEvidenceError } from "./emc2101-thermal-evidence.js";
import { Emc2101ThermalFaultEvidenceError } from "./emc2101-thermal-fault-evidence.js";
import { AsicWorkSendEvidenceError } from "./asic-work-send-evidence.js";
import { AsicResultParsingEvidenceError } from "./asic-result-parsing-evidence.js";
import { AsicSerialTransportEvidenceError } from "./asic-serial-transport-evidence.js";
import { AsicFrequencyTransitionEvidenceError } from "./asic-frequency-transition-evidence.js";
import { StratumSocketEvidenceError } from "./stratum-socket-evidence.js";
import { ProtocolCoordinatorEvidenceError } from "./protocol-coordinator-evidence.js";
import { MiningCriteriaEvidenceError } from "./mining-criteria-evidence.js";
import { NetworkScanEvidenceError } from "./network-scan-evidence.js";
import { maybeTypedFailureCategory, maybeTypedFailurePublicValue } from "./typed-failure.js";

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

test("ASIC initialization failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicInitializationEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_initialization_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_initialization_projection",
    hardware_rerun_used: false,
  });
});

test("ASIC power initialization failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicPowerInitializationEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_asic_power_initialization_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_asic_power_initialization_projection",
    hardware_rerun_used: false,
  });
});

test("core-voltage-control failures retain only closed projection facts", () => {
  // Arrange
  const error = new CoreVoltageControlEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_core_voltage_control_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_core_voltage_control_projection",
    hardware_rerun_used: false,
  });
});

test("INA260 failures retain only closed projection facts", () => {
  // Arrange
  const error = new Ina260EvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_ina260_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_ina260_projection",
    hardware_rerun_used: false,
  });
});

test("EMC2101 thermal failures retain only closed capture facts", () => {
  // Arrange
  const error = new Emc2101ThermalEvidenceError("hardware_blocked", "safe failure", {
    stage: "emc2101_thermal_capture",
    projection_published: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "emc2101_thermal_capture",
    projection_published: false,
  });
});

test("EMC2101 thermal fault failures retain category and recovery facts", () => {
  // Arrange
  const error = new Emc2101ThermalFaultEvidenceError("evidence_invalid", "safe failure", {
    stage: "emc2101_thermal_fault_capture",
    projection_published: false,
    recovery_complete: true,
    recovery_flash_used: true,
    secondary_recovery_failure: false,
  });

  // Act / Assert
  assert.equal(maybeTypedFailureCategory(error), "evidence_invalid");
  assert.deepEqual(maybeTypedFailurePublicValue(error), error.publicValue);
});

test("ASIC work-send failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicWorkSendEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_work_send_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_work_send_projection",
    hardware_rerun_used: false,
  });
});

test("ASIC result-parsing failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicResultParsingEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_result_parsing_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_result_parsing_projection",
    hardware_rerun_used: false,
  });
});

test("ASIC serial-transport failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicSerialTransportEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_serial_transport_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_serial_transport_projection",
    hardware_rerun_used: false,
  });
});

test("ASIC frequency-transition failures retain only closed projection facts", () => {
  // Arrange
  const error = new AsicFrequencyTransitionEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_frequency_transition_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_frequency_transition_projection",
    hardware_rerun_used: false,
  });
});

test("Stratum socket failures retain only closed projection facts", () => {
  // Arrange
  const error = new StratumSocketEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_stratum_socket_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_stratum_socket_projection",
    hardware_rerun_used: false,
  });
});

test("protocol coordinator failures retain only closed projection facts", () => {
  // Arrange
  const error = new ProtocolCoordinatorEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_protocol_coordinator_projection",
    hardware_rerun_used: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_protocol_coordinator_projection",
    hardware_rerun_used: false,
  });
});

test("mining criteria failures retain only closed projection facts", () => {
  // Arrange
  const error = new MiningCriteriaEvidenceError("evidence_invalid", "safe failure", {
    stage: "sealed_mining_criteria_projection",
    hardware_rerun_used: false,
    terminal_attempt_reopened: false,
  });

  // Act
  const publicValue = maybeTypedFailurePublicValue(error);

  // Assert
  assert.deepEqual(publicValue, {
    stage: "sealed_mining_criteria_projection",
    hardware_rerun_used: false,
    terminal_attempt_reopened: false,
  });
});
