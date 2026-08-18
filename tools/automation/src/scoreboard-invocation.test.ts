import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("scoreboard capture requires the exact detector-gated 600-second surface", () => {
  // Arrange
  const complete = [
    "capture-scoreboard-evidence",
    "--private-root", "scratch/stat003-scoreboard/attempt-002",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--pool-credentials", "pool-credentials.json",
    "--detector-output", "scratch/stat003-scoreboard/wrapper-002/detector.stdout",
    "--projection", "docs/evidence/scoreboard.json",
    "--duration-seconds", "600",
    "--capture-timeout-seconds", "1800",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-scoreboard-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete.slice(0, -3), "599", ...complete.slice(-2)]));
  assert.throws(() => parseInvocation([...complete.slice(0, -1), "1500"]));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
