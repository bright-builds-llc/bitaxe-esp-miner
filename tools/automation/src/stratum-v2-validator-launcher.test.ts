import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  parseInstalledIdentity,
  projectRestoreReadiness,
  restoreBundleSchema,
  sha256,
  type PackageRestoreBundle,
} from "./stratum-v2-restore-model.js";
import {
  runValidatorChild,
  validateValidatorChildReceipt,
} from "./stratum-v2-validator-child.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const planSha256 = "c".repeat(64);

function launcherContext(): { readonly workspace: string; readonly program: string } {
  const workspace = process.cwd();
  const maybeRunfiles = process.env["RUNFILES_DIR"] ?? process.env["JS_BINARY__RUNFILES"];
  assert(maybeRunfiles !== undefined);
  return {
    workspace,
    program: path.join(
      maybeRunfiles,
      "_main/tools/automation/stratum_v2_restore_validator_/stratum_v2_restore_validator",
    ),
  };
}

async function writeProtected(candidate: string, contents: string | Buffer): Promise<void> {
  await writeFile(candidate, contents, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function fixture(root: string): Promise<{
  readonly bundle: string;
  readonly projection: string;
}> {
  const packageRoot = path.join(root, "package");
  await mkdir(packageRoot, { mode: 0o700 });
  await chmod(packageRoot, 0o700);
  const specifications = [
    ["firmware_elf", "firmware.elf", "elf"],
    ["firmware_ota_image", "firmware.bin", "ota"],
    ["www_spiffs_image", "www.bin", "www"],
    ["factory_merged_image", "factory.bin", "factory"],
    ["partition_table", "partitions.csv", "partition"],
    ["otadata_initial", "otadata.bin", "otadata"],
  ] as const;
  const artifacts = [];
  for (const [kind, filename, contents] of specifications) {
    const bytes = Buffer.from(contents);
    await writeProtected(path.join(packageRoot, filename), bytes);
    artifacts.push({ kind, path: filename, offset: "Unavailable", sha256: sha256(bytes) });
  }
  const identity = parseInstalledIdentity({
    sourceCommit,
    referenceCommit,
    appElfSha256: sha256("elf"),
    buildTimestampUtc: "2026-08-24T21:49:20Z",
    semanticVersion: "0.1.0",
    version: `${sourceCommit.slice(0, 12)}-dev`,
    buildChannel: "dev",
    sourceDirty: false,
    releaseTag: null,
    idfVersion: "v5.5.4",
    runningPartition: "factory",
  });
  const manifest = {
    schema_version: 3,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: identity.app_elf_sha256,
    artifacts,
  };
  const manifestDocument = `${JSON.stringify(manifest)}\n`;
  await writeProtected(path.join(packageRoot, "manifest.json"), manifestDocument);
  const factory = artifacts.find(value => value.kind === "factory_merged_image");
  assert(factory !== undefined);
  const bundleValue: PackageRestoreBundle = {
    schema_version: restoreBundleSchema,
    kind: "package_v3",
    board: 205,
    installed_identity: identity,
    package_manifest: "package/manifest.json",
    package_manifest_sha256: sha256(manifestDocument),
    factory_sha256: factory.sha256,
    capture_source_commit: sourceCommit,
    plan_sha256: planSha256,
  };
  const bundleDocument = `${JSON.stringify(bundleValue, null, 2)}\n`;
  const bundle = path.join(root, "bundle.json");
  const projection = path.join(root, "projection.json");
  await writeProtected(bundle, bundleDocument);
  await writeProtected(
    projection,
    `${JSON.stringify(projectRestoreReadiness(bundleValue, bundleDocument, 1, true), null, 2)}\n`,
  );
  return { bundle, projection };
}

test("real Bazel validator child accepts a protected fixture with sanitized launcher state", async () => {
  // Arrange
  const { workspace, program } = launcherContext();
  await mkdir(path.join(workspace, "scratch"), { recursive: true });
  const root = await mkdtemp(path.join(workspace, "scratch/str005-validator-launcher-"));
  await chmod(root, 0o700);
  const values = await fixture(root);
  const canary = "validator-launcher-secret-canary";
  assert(Object.keys(process.env).some(key => key.startsWith("JS_BINARY__")));
  process.env["VALIDATOR_SECRET_CANARY"] = canary;
  try {
    // Act
    const receiptPath = path.join(root, "accepted-receipt.json");
    const receipt = await runValidatorChild({
      workspace,
      program,
      args: [values.bundle, values.projection, sourceCommit, planSha256],
      receiptPath,
      sourceCommit,
      planSha256,
    });

    // Assert
    assert(receipt.validation_accepted);
    assert.equal((await stat(receiptPath)).mode & 0o777, 0o600);
    assert(!(await readFile(receiptPath, "utf8")).includes(canary));
    await assert.doesNotReject(validateValidatorChildReceipt(
      receiptPath,
      sourceCommit,
      planSha256,
    ));
  } finally {
    delete process.env["VALIDATOR_SECRET_CANARY"];
    await rm(root, { recursive: true, force: true });
  }
});

test("real Bazel validator child records rejection without publishing output text", async () => {
  // Arrange
  const { workspace, program } = launcherContext();
  await mkdir(path.join(workspace, "scratch"), { recursive: true });
  const root = await mkdtemp(path.join(workspace, "scratch/str005-validator-reject-"));
  await chmod(root, 0o700);
  const values = await fixture(root);
  const projection = JSON.parse(await readFile(values.projection, "utf8")) as Record<string, unknown>;
  projection["runtime_unchanged"] = false;
  await writeProtected(values.projection, `${JSON.stringify(projection)}\n`);
  try {
    // Act
    const receiptPath = path.join(root, "rejected-receipt.json");
    const receipt = await runValidatorChild({
      workspace,
      program,
      args: [values.bundle, values.projection, sourceCommit, planSha256],
      receiptPath,
      sourceCommit,
      planSha256,
    });

    // Assert
    assert(!receipt.validation_accepted);
    assert.equal(receipt.exit_code, 1);
    assert(!(await readFile(receiptPath, "utf8")).includes("restore_readiness=rejected"));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
