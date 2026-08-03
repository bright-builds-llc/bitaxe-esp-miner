import assert from "node:assert/strict";
import test from "node:test";

import { monitorCommand } from "./contracts.generated.js";
import { createFakeProcessPort } from "./process.js";
import { executeCommandSpec } from "./workflow.js";

test("public workflow interface exercises the fake process adapter", async () => {
  // Arrange
  const observed: string[][] = [];
  const fake = createFakeProcessPort(async (spec) => {
    observed.push([...spec.args]);
    return { exitCode: 0, stdout: "captured", stderr: "", timedOut: false };
  });

  // Act
  const outcome = await executeCommandSpec(monitorCommand("flash", { board: 205 }), fake);

  // Assert
  assert.equal(outcome.stdout, "captured");
  assert.deepEqual(observed, [["monitor", "--board", "205"]]);
});
