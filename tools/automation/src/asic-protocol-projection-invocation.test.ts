import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("ASIC initialization projection accepts only sealed source inputs", () => {
  // Arrange
  const complete = [
    "project-asic-initialization-evidence",
    "--attempt-root", "scratch/accepted/attempt-007",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-init.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-initialization-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC power initialization projection accepts only the committed source proof", () => {
  // Arrange
  const complete = [
    "project-asic-power-initialization-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-power-initialization.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-power-initialization-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("core-voltage projection accepts only the committed power proof", () => {
  // Arrange
  const complete = [
    "project-core-voltage-control-evidence",
    "--source-projection", "docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/core-voltage-control.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-core-voltage-control-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC work-send projection accepts only the committed source proof", () => {
  // Arrange
  const complete = [
    "project-asic-work-send-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-work-send.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-work-send-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC result-parsing projection accepts only the committed work-send proof", () => {
  // Arrange
  const complete = [
    "project-asic-result-parsing-evidence",
    "--source-projection", "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-result-parsing.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-result-parsing-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC serial-transport projection accepts only both committed source proofs", () => {
  // Arrange
  const complete = [
    "project-asic-serial-transport-evidence",
    "--work-send-projection", "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
    "--result-parsing-projection", "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-serial-transport.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-serial-transport-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC frequency-transition projection accepts only the initialization proof", () => {
  // Arrange
  const complete = [
    "project-asic-frequency-transition-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-frequency-transition.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-frequency-transition-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("Stratum socket projection accepts only the initialization proof", () => {
  // Arrange
  const complete = [
    "project-stratum-socket-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/stratum-socket.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-stratum-socket-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("protocol coordinator projection requires the closed four-proof join", () => {
  // Arrange
  const complete = [
    "project-protocol-coordinator-evidence",
    "--initialization-projection", "docs/evidence/initialization.json",
    "--work-send-projection", "docs/evidence/work-send.json",
    "--result-parsing-projection", "docs/evidence/result-parsing.json",
    "--socket-projection", "docs/evidence/socket.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/protocol-coordinator.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-protocol-coordinator-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
