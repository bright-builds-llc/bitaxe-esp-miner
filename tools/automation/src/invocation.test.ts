import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("parser rejects legacy and unsupported monitor syntax", () => {
  // Arrange
  const cases = [
    ["observe-serial", "port=/dev/cu.test"],
    ["observe-serial", "--evidence_mode", "dual"],
    ["observe-serial", "--evidence-mode", "dual"],
  ];

  // Act / Assert
  for (const args of cases) assert.throws(() => parseInvocation(args));
});

test("parser rejects duplicate, unknown, invalid enum, and missing options", () => {
  // Arrange
  const cases = [
    ["observe-serial", "--port", "a", "--port", "b"],
    ["doctor", "--port", "a"],
    ["verify-hardware-surface", "--surface", "overclock", "--request", "request.json"],
    ["capture-version-evidence", "--private-root", "scratch/attempt"],
    ["verify-flash-durability", "--image", "firmware.bin"],
  ];

  // Act / Assert
  for (const args of cases) assert.throws(() => parseInvocation(args));
});

test("parser accepts a complete version evidence request", () => {
  // Act
  const invocation = parseInvocation([
    "capture-version-evidence",
    "--private-root", "scratch/attempt",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--port", "/dev/cu.test",
    "--projection", "scratch/version.json",
    "--capture-timeout-seconds", "45",
  ]);

  // Assert
  assert.equal(invocation.command, "capture-version-evidence");
});
