import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("screen-flow projection accepts only the two committed public proofs", () => {
  // Arrange
  const complete = [
    "project-screen-flow-evidence",
    "--source-display-uat",
    "docs/parity/evidence/api009-command-effects/display-uat-projection-attempt-005.json",
    "--source-command-effects",
    "docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json",
    "--attempt-source-commit",
    "a".repeat(40),
    "--projection",
    "docs/parity/evidence/ui002-screen-flow/screen-flow-projection.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-screen-flow-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
