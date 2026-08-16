import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("hashrate capture requires the exact detector-gated 600-second surface", () => {
  // Arrange
  const complete = [
    "capture-hashrate-monitor-evidence",
    "--private-root", "scratch/stat001-hashrate-monitor/attempt-001",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--pool-credentials", "pool-credentials.json",
    "--detector-output", "scratch/stat001-hashrate-monitor/wrapper-001/detector.stdout",
    "--projection", "docs/evidence/hashrate.json",
    "--duration-seconds", "600",
    "--capture-timeout-seconds", "1500",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-hashrate-monitor-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete.slice(0, -3), "599", ...complete.slice(-2)]));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
