import assert from "node:assert/strict";
import { realpath, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  campaignWorkspaceRoot,
  runCampaignProcess,
} from "./stratum-v2-campaign.js";

test("real Bazel launcher resolves the repository workspace before paths", async () => {
  // Arrange
  const configured = process.env["BUILD_WORKSPACE_DIRECTORY"];

  // Act
  const workspace = campaignWorkspaceRoot();

  // Assert
  assert.ok((await stat(path.join(workspace, "MODULE.bazel"))).isFile());
  if (configured !== undefined) assert.equal(workspace, await realpath(configured));
});

test("real Bazel launcher runs campaign children in the repository workspace", async () => {
  // Arrange
  const workspace = campaignWorkspaceRoot();

  // Act
  const child = await runCampaignProcess(
    workspace,
    "git",
    ["rev-parse", "--show-toplevel"],
    5_000,
  );

  // Assert
  assert.equal(child.exitCode, 0);
  assert.equal(await realpath(child.stdout.trim()), workspace);
});

test("real Bazel launcher admits the ignored private campaign path", async () => {
  // Arrange
  const workspace = campaignWorkspaceRoot();

  // Act
  const child = await runCampaignProcess(
    workspace,
    "git",
    ["check-ignore", "-q", "scratch/str005-stratum-v2/attempt-006"],
    5_000,
  );

  // Assert
  assert.equal(child.exitCode, 0);
});
