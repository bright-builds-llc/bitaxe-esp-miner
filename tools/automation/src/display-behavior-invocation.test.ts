import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("display projection accepts only the two committed public proofs", () => {
  // Arrange
  const complete = [
    "project-display-behavior-evidence",
    "--source-display-uat", "docs/parity/evidence/api009-command-effects/display-uat-projection-attempt-005.json",
    "--source-command-effects", "docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/display-behavior.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-display-behavior-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
