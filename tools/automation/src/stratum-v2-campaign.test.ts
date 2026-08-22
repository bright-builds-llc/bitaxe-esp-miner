import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseStratumV2CampaignArgs,
  selectRestorePackage,
  StratumV2CampaignError,
} from "./stratum-v2-campaign.js";

const exactArgs = [
  "--board", "205",
  "--port", "/dev/cu.usbmodem101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--wifi-credentials", "wifi-credentials.json",
  "--private-root", "scratch/str005-stratum-v2/attempt-001",
  "--projection", "docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json",
  "--duration-seconds", "180",
  "--redact-evidence",
] as const;

test("campaign parser admits only the immutable attempt-001 command", () => {
  // Arrange
  const changed: string[] = [...exactArgs];
  const durationIndex = changed.indexOf("180");
  changed[durationIndex] = "181";

  // Act
  const admitted = parseStratumV2CampaignArgs(exactArgs);
  let rejected: unknown;
  try {
    parseStratumV2CampaignArgs(changed);
  } catch (error) {
    rejected = error;
  }

  // Assert
  assert.equal(admitted.durationSeconds, 180);
  assert.equal(admitted.redactEvidence, true);
  assert.ok(rejected instanceof StratumV2CampaignError);
  assert.equal(rejected.category, "invalid_invocation");
});

test("restore package discovery requires one exact identity with existing factory bytes", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "str005-restore-"));
  const artifacts = path.join(workspace, "scratch", "attempt", "artifacts");
  await mkdir(artifacts, { recursive: true });
  await writeFile(path.join(artifacts, "bitaxe-ultra205-factory.bin"), "factory");
  await writeFile(path.join(artifacts, "package-manifest.json"), JSON.stringify({
    source_commit: "a".repeat(40),
    app_elf_sha256: "b".repeat(64),
    artifacts: [{
      kind: "factory_merged_image",
      path: "bitaxe-ultra205-factory.bin",
      sha256: "c".repeat(64),
    }],
  }));

  try {
    // Act
    const selected = await selectRestorePackage(workspace, "b".repeat(64));

    // Assert
    assert.equal(selected.sourceCommit, "a".repeat(40));
    assert.equal(selected.factorySha256, "c".repeat(64));
    assert.equal(selected.manifestPath, path.join(artifacts, "package-manifest.json"));
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("restore package discovery rejects conflicting identities", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "str005-ambiguous-"));
  for (const [name, source] of [["one", "a"], ["two", "d"]] as const) {
    const artifacts = path.join(workspace, "scratch", name, "artifacts");
    await mkdir(artifacts, { recursive: true });
    await writeFile(path.join(artifacts, "bitaxe-ultra205-factory.bin"), name);
    await writeFile(path.join(artifacts, "package-manifest.json"), JSON.stringify({
      source_commit: source.repeat(40),
      app_elf_sha256: "b".repeat(64),
      artifacts: [{
        kind: "factory_merged_image",
        path: "bitaxe-ultra205-factory.bin",
        sha256: source.repeat(64),
      }],
    }));
  }

  try {
    // Act
    let failure: unknown;
    try {
      await selectRestorePackage(workspace, "b".repeat(64));
    } catch (error) {
      failure = error;
    }

    // Assert
    assert.ok(failure instanceof StratumV2CampaignError);
    assert.equal(failure.category, "hardware_blocked");
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});
