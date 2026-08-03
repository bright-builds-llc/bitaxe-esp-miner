import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

test("redaction CI uses the host toolchain and bare semantic command", async () => {
  // Arrange
  const root = runfileRoot();
  const [bazelConfig, workflow] = await Promise.all([
    readFile(path.join(root, ".bazelrc"), "utf8"),
    readFile(path.join(root, ".github/workflows/evidence-redaction.yml"), "utf8"),
  ]);

  // Act
  const removedSurface = ["--base", "--head", "--new-branch-base", "BASE_REF", "HEAD_REF", "NEW_BRANCH_BASE_REF"];

  // Assert
  assert.match(
    bazelConfig,
    /^build --workspace_status_command="cargo \+stable run --quiet -p xtask -- build-identity-status"$/mu,
  );
  assert.match(workflow, /^\s*run: just verify-redaction$/mu);
  for (const legacy of removedSurface) assert.equal(workflow.includes(legacy), false);
});
