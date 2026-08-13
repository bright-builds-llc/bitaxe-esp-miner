import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("UI workflow projection accepts only its closed private source surface", () => {
  // Arrange
  const complete = [
    "project-ui-workflow-evidence",
    "--private-root", "scratch/ui004-live-workflows/attempt-001",
    "--package-manifest", "bazel-bin/package.json",
    "--operator-snapshot-projection", "scratch/operator.private.json",
    "--browser-attestation", "output/playwright/ui004-attempt-001/browser.private.json",
    "--projection", "docs/evidence/ui-workflow.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-ui-workflow-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
