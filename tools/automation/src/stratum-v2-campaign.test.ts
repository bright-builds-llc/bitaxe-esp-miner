import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, realpath, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  campaignWorkspaceRoot,
  parseStratumV2CampaignArgs,
  runCampaignProcess,
  selectRestorePackage,
  type StratumV2CampaignCheckpoint,
  StratumV2CampaignError,
  stratumV2CampaignFailureResult,
} from "./stratum-v2-campaign.js";
import { prepareStratumV2Campaign } from "./stratum-v2-campaign-preflight.js";
import { singleRuntimeOrigin } from "./stratum-v2-runtime-admission.js";

const exactArgs = [
  "--board", "205",
  "--port", "/dev/cu.usbmodem101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--wifi-credentials", "wifi-credentials.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-001/restore-bundle.private.json",
  "--private-root", "scratch/str005-stratum-v2/attempt-004",
  "--projection", "docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json",
  "--duration-seconds", "180",
  "--redact-evidence",
] as const;

async function requireGit(workspace: string, args: readonly string[]): Promise<string> {
  const outcome = await runCampaignProcess(workspace, "git", args, 5_000);
  assert.equal(outcome.exitCode, 0, outcome.stderr);
  return outcome.stdout.trim();
}

async function createPreflightWorkspace(): Promise<string> {
  const workspace = await mkdtemp(path.join(os.tmpdir(), "str005-preflight-"));
  await writeFile(path.join(workspace, ".gitignore"), [
    "bazel-bin/",
    "pool-credentials.json",
    "scratch/",
    "wifi-credentials.json",
    "",
  ].join("\n"));
  await writeFile(path.join(workspace, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await writeFile(path.join(workspace, "wifi-credentials.json"), "{}\n", { mode: 0o600 });
  await writeFile(path.join(workspace, "pool-credentials.json"), "{}\n", { mode: 0o600 });
  await chmod(path.join(workspace, "wifi-credentials.json"), 0o600);
  await chmod(path.join(workspace, "pool-credentials.json"), 0o600);
  await requireGit(workspace, ["init", "--quiet"]);
  await requireGit(workspace, ["add", ".gitignore", "MODULE.bazel"]);
  await requireGit(workspace, [
    "-c", "commit.gpgsign=false",
    "-c", "user.name=Bitaxe Test",
    "-c", "user.email=bitaxe-test@example.invalid",
    "commit", "--quiet", "-m", "fixture",
  ]);
  const head = await requireGit(workspace, ["rev-parse", "HEAD"]);
  const manifestDirectory = path.join(workspace, "bazel-bin", "firmware", "bitaxe");
  await mkdir(manifestDirectory, { recursive: true });
  await writeFile(
    path.join(manifestDirectory, "bitaxe-ultra205-package.json"),
    `${JSON.stringify({ source_commit: head })}\n`,
  );
  return workspace;
}

test("workspace discovery prefers an explicit workspace with a Bazel module", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "str005-workspace-"));
  await writeFile(path.join(workspace, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(workspace, ".git"));
  try {
    // Act
    const selected = campaignWorkspaceRoot(
      { BUILD_WORKSPACE_DIRECTORY: workspace },
      path.join(workspace, "ignored"),
    );

    // Assert
    assert.equal(selected, await realpath(workspace));
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("campaign parser admits only the immutable attempt-004 command", () => {
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
  assert.equal(rejected.checkpoint, "invocation");
});

test("runtime origin requires exactly one origin-only candidate", () => {
  // Arrange
  const one = "runtime ready http://192.0.2.1:80";
  const fail = (
    category: string,
    message: string,
    checkpoint: StratumV2CampaignCheckpoint,
  ): never => {
    throw new StratumV2CampaignError(category, message, checkpoint);
  };

  // Act / Assert
  assert.equal(singleRuntimeOrigin(one, fail).origin, "http://192.0.2.1");
  for (const value of ["runtime ready", "http://192.0.2.1 http://192.0.2.2"]) {
    assert.throws(
      () => singleRuntimeOrigin(value, fail),
      (error: unknown) => error instanceof StratumV2CampaignError
        && error.checkpoint === "runtime_origin",
    );
  }
});

test("campaign failure output exposes only the closed pre-effect discriminator", () => {
  // Arrange
  const error = new StratumV2CampaignError(
    "evidence_invalid",
    "sensitive internal detail",
    "private_path_ignored",
  );

  // Act
  const result = stratumV2CampaignFailureResult(error);

  // Assert
  assert.deepEqual(result, {
    schema_version: "bitaxe-stratum-v2-campaign-result-v1",
    status: "failed",
    category: "evidence_invalid",
    checkpoint: "private_path_ignored",
    projection_published: false,
  });
  assert(!JSON.stringify(result).includes("sensitive"));
});

test("software preflight proves read-only source predicates without creating the private root", async () => {
  // Arrange
  const workspace = await createPreflightWorkspace();
  try {
    const args = parseStratumV2CampaignArgs(exactArgs);

    // Act
    await prepareStratumV2Campaign(workspace, args, {
      runProcess: runCampaignProcess,
      fail: (category, message, checkpoint) => {
        throw new StratumV2CampaignError(category, message, checkpoint);
      },
    });

    // Assert
    await assert.rejects(stat(path.join(workspace, "scratch/str005-stratum-v2/attempt-004")));
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
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
