import assert from "node:assert/strict";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { anchoredPath, assertWithinWorkspace, sourceWorkspaceRoot } from "./workspace.js";

test("relative projector paths are anchored before admission", () => {
  // Arrange
  const root = "/workspace/project";

  // Act / Assert
  assert.equal(anchoredPath(root, "scratch/projection.json"), "/workspace/project/scratch/projection.json");
  assert.throws(() => assertWithinWorkspace(root, "../outside.json"));
});

test("source workspace discovery skips nested Bazel outputs without Git identity", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "workspace-root-"));
  const output = path.join(workspace, "bazel-out", "bin");
  await mkdir(path.join(workspace, ".git"));
  await mkdir(output, { recursive: true });
  await writeFile(path.join(workspace, "MODULE.bazel"), "module(name = \"source\")\n");
  await writeFile(path.join(output, "MODULE.bazel"), "module(name = \"output\")\n");
  try {
    // Act
    const selected = sourceWorkspaceRoot([output]);

    // Assert
    assert.equal(selected, await realpath(workspace));
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});
