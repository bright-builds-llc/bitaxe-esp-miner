import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("statistics history evidence requires the closed detector-gated surface", () => {
  // Arrange
  const complete = [
    "capture-statistics-history-evidence",
    "--private-root", "scratch/stat002-statistics-history/attempt-002",
    "--package-manifest", "package.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/stat002-statistics-history/wrapper-002/detector.stdout",
    "--projection", "docs/evidence/statistics-history.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-statistics-history-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
