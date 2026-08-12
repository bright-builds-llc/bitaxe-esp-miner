import assert from "node:assert/strict";
import test from "node:test";

import { flashCommand, flashMonitorCommand, monitorCommand, parseAutomationResult } from "./contracts.generated.js";

test("monitor builder exposes only supported monitor flags", () => {
  // Arrange
  const program = "/tmp/flash";

  // Act
  const command = monitorCommand(program, {
    board: 205,
    port: "/dev/test",
    captureTimeoutSeconds: 360,
  });

  // Assert
  assert.deepEqual(command.args, [
    "monitor",
    "--board",
    "205",
    "--port",
    "/dev/test",
    "--capture-timeout-seconds",
    "360",
  ]);
  assert.equal(command.args.includes("--evidence-mode"), false);
});

test("dual evidence is representable only on flash-monitor", () => {
  // Arrange / Act
  const command = flashMonitorCommand("/tmp/flash", {
    evidenceMode: "dual",
    evidenceDir: "/tmp/evidence",
  });

  // Assert
  assert.deepEqual(command.args.slice(-4), [
    "--evidence-dir",
    "/tmp/evidence",
    "--evidence-mode",
    "dual",
  ]);
});

test("result validator rejects open string categories", () => {
  // Act / Assert
  assert.throws(() => parseAutomationResult({
    schema_version: "bitaxe-automation-result-v1",
    command: "doctor",
    status: "succeeded",
    category: "invented",
  }));
});

test("result validator accepts the closed ASIC result-parsing command", () => {
  // Act
  const result = parseAutomationResult({
    schema_version: "bitaxe-automation-result-v1",
    command: "project-asic-result-parsing-evidence",
    status: "succeeded",
    category: "complete",
  });

  // Assert
  assert.equal(result.command, "project-asic-result-parsing-evidence");
});

test("result validator accepts the closed ASIC serial-transport command", () => {
  // Act
  const result = parseAutomationResult({
    schema_version: "bitaxe-automation-result-v1",
    command: "project-asic-serial-transport-evidence",
    status: "succeeded",
    category: "complete",
  });

  // Assert
  assert.equal(result.command, "project-asic-serial-transport-evidence");
});

test("result validator accepts the closed ASIC frequency-transition command", () => {
  // Act
  const result = parseAutomationResult({
    schema_version: "bitaxe-automation-result-v1",
    command: "project-asic-frequency-transition-evidence",
    status: "succeeded",
    category: "complete",
  });

  // Assert
  assert.equal(result.command, "project-asic-frequency-transition-evidence");
});

// @ts-expect-error monitor intentionally has no evidenceMode option.
monitorCommand("/tmp/flash", { evidenceMode: "dual" });

// @ts-expect-error dual evidence requires an evidence directory.
flashMonitorCommand("/tmp/flash", { evidenceMode: "dual" });

// @ts-expect-error explicit images require their exact package manifest.
flashCommand("/tmp/flash", { image: "/tmp/firmware.bin" });

// @ts-expect-error flash intentionally has no monitor timeout option.
flashCommand("/tmp/flash", { captureTimeoutSeconds: 30 });
