import assert from "node:assert/strict";
import test from "node:test";

import { anchoredPath, assertWithinWorkspace } from "./workspace.js";

test("relative projector paths are anchored before admission", () => {
  // Arrange
  const root = "/workspace/project";

  // Act / Assert
  assert.equal(anchoredPath(root, "scratch/projection.json"), "/workspace/project/scratch/projection.json");
  assert.throws(() => assertWithinWorkspace(root, "../outside.json"));
});
