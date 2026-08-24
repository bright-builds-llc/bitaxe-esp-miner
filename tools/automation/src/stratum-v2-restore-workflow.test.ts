import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  normalizePackageCandidate,
  searchExactPackage,
} from "./stratum-v2-restore-artifacts.js";
import {
  parseInstalledIdentity,
  projectRestoreReadiness,
  restoreBundleSchema,
  sha256,
  snapshotRangeTemplates,
  type InstalledIdentity,
  type RestoreBundle,
} from "./stratum-v2-restore-model.js";
import { validateRestoreReadiness } from "./stratum-v2-restore-validator.js";
import { parseRestoreRecoveryArgs } from "./stratum-v2-restore-recovery.js";
import { restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import {
  prepareSnapshotTarget,
  snapshotReadArgs,
} from "./stratum-v2-restore-snapshot.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appDigest = sha256(Buffer.from("elf"));
const captureCommit = "d".repeat(40);
const planDigest = "e".repeat(64);

function identity(): InstalledIdentity {
  return parseInstalledIdentity({
    sourceCommit,
    referenceCommit,
    appElfSha256: appDigest,
    buildTimestampUtc: "2026-08-22T06:39:26Z",
    semanticVersion: "0.1.0",
    version: `${sourceCommit.slice(0, 12)}-dev`,
    buildChannel: "dev",
    sourceDirty: false,
    releaseTag: null,
    idfVersion: "v5.5.4",
    runningPartition: "factory",
  });
}

async function packageFixture(root: string, name: string, factoryContents: string): Promise<string> {
  const directory = path.join(root, name);
  await mkdir(directory, { recursive: true });
  const specifications = [
    ["firmware_elf", "bitaxe-ultra205.elf", "elf"],
    ["firmware_ota_image", "esp-miner.bin", "ota"],
    ["www_spiffs_image", "www.bin", "www"],
    ["factory_merged_image", "bitaxe-ultra205-factory.bin", factoryContents],
    ["partition_table", "partitions-ultra205.csv", "partition"],
    ["otadata_initial", "otadata-initial.bin", "otadata"],
  ] as const;
  const artifacts = [];
  for (const [kind, file, contents] of specifications) {
    const bytes = kind === "firmware_elf" ? Buffer.from("elf") : Buffer.from(contents);
    await writeFile(path.join(directory, file), bytes);
    artifacts.push({
      kind,
      path: file,
      offset: "Unavailable",
      sha256: kind === "firmware_elf" ? appDigest : sha256(bytes),
    });
  }
  const manifest = {
    schema_version: 3,
    semantic_version: "0.1.0",
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appDigest,
    build_identity: {
      label: `${sourceCommit.slice(0, 12)}-dev`,
      channel: "dev",
      source_dirty: false,
      release_tag: null,
    },
    default_flash_image: "bitaxe-ultra205.elf",
    artifacts,
  };
  const manifestPath = path.join(directory, "bitaxe-ultra205-package.json");
  await writeFile(manifestPath, `${JSON.stringify(manifest)}\n`);
  return manifestPath;
}

test("artifact search selects and normalizes one exact historical package", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "restore-search-"));
  await packageFixture(root, "candidate", "factory");

  // Act
  const result = await searchExactPackage([root], identity());
  assert(result.maybeCandidate !== undefined);
  const normalized = await normalizePackageCandidate(
    result.maybeCandidate,
    path.join(root, "normalized"),
  );

  // Assert
  assert.equal((await stat(normalized.manifestPath)).mode & 0o777, 0o600);
  const manifest = JSON.parse(await readFile(normalized.manifestPath, "utf8")) as Record<string, unknown>;
  assert.equal(manifest["license_inventory"], "license-inventory.md");
});

test("restore recovery parser admits only the immutable protected command", () => {
  // Arrange
  const exact = [
    "--board", "205",
    "--port", "/dev/cu.usbmodem101",
    "--private-root", "scratch/str005-installed-package-recovery/recovery-002",
    "--projection", "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-002.json",
    "--redact-evidence",
  ];

  // Act / Assert
  assert.equal(parseRestoreRecoveryArgs(exact).board, "205");
  assert.throws(() => parseRestoreRecoveryArgs([...exact, "--unknown", "value"]));
  assert.throws(() => parseRestoreRecoveryArgs(exact.filter(value => value !== "--redact-evidence")));
});

test("restored runtime requires every installed package identity and disabled boot mining", () => {
  // Arrange
  const bundle = {
    schema_version: restoreBundleSchema,
    kind: "package_v3" as const,
    board: 205 as const,
    installed_identity: identity(),
    package_manifest: "package.json",
    package_manifest_sha256: "1".repeat(64),
    factory_sha256: "2".repeat(64),
    capture_source_commit: captureCommit,
    plan_sha256: planDigest,
  };
  const runtime = {
    sourceCommit,
    appElfSha256: appDigest,
    buildTimestampUtc: bundle.installed_identity.build_timestamp_utc,
    version: bundle.installed_identity.build_label,
    runningPartition: "factory",
    startMiningOnBoot: false,
  };

  // Act / Assert
  assert(restoreRuntimeMatches(bundle, runtime));
  assert(!restoreRuntimeMatches(bundle, { ...runtime, runningPartition: "ota_0" }));
  assert(!restoreRuntimeMatches(bundle, { ...runtime, startMiningOnBoot: true }));
});

test("artifact search rejects distinct factory identities for one installed ELF", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "restore-ambiguous-"));
  await packageFixture(root, "one", "factory-one");
  await packageFixture(root, "two", "factory-two");

  // Act / Assert
  await assert.rejects(searchExactPackage([root], identity()), /ambiguous/u);
});

test("snapshot readiness validator accepts only protected exact ranges", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "restore-snapshot-"));
  await chmod(root, 0o700);
  await mkdir(path.join(root, "snapshot"), { mode: 0o700 });
  const ranges = [];
  for (const template of snapshotRangeTemplates("snapshot")) {
    const candidate = path.join(root, template.path);
    const bytes = Buffer.alloc(template.size, 0x5a);
    await writeFile(candidate, bytes, { mode: 0o600 });
    await chmod(candidate, 0o600);
    ranges.push({ ...template, sha256: sha256(bytes) });
  }
  const bundle: RestoreBundle = {
    schema_version: restoreBundleSchema,
    kind: "flash_snapshot_v1",
    board: 205,
    installed_identity: identity(),
    ranges,
    capture_source_commit: captureCommit,
    plan_sha256: planDigest,
  };
  const bundlePath = path.join(root, "restore-bundle.private.json");
  const bundleDocument = `${JSON.stringify(bundle, null, 2)}\n`;
  await writeFile(bundlePath, bundleDocument, { mode: 0o600 });
  await chmod(bundlePath, 0o600);
  const projection = projectRestoreReadiness(bundle, bundleDocument, 71, true);
  const projectionPath = path.join(root, "projection.json");
  await writeFile(projectionPath, `${JSON.stringify(projection)}\n`, { mode: 0o600 });
  await chmod(projectionPath, 0o600);

  // Act / Assert
  await assert.doesNotReject(validateRestoreReadiness(
    bundlePath,
    projectionPath,
    captureCommit,
    planDigest,
  ));
  await chmod(path.join(root, ranges[0]?.path ?? "missing"), 0o644);
  await assert.rejects(validateRestoreReadiness(
    bundlePath,
    projectionPath,
    captureCommit,
    planDigest,
  ));
});

test("snapshot capture protects targets and renders the bounded fast read", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "restore-snapshot-target-"));
  const candidate = path.join(root, "factory.bin");

  // Act
  await prepareSnapshotTarget(candidate);
  const args = snapshotReadArgs("/dev/cu.usbmodem101", 0x10000, 0x400000, candidate);

  // Assert
  assert.equal((await stat(candidate)).mode & 0o777, 0o600);
  assert.deepEqual(args, [
    "read-flash",
    "--chip", "esp32s3",
    "--port", "/dev/cu.usbmodem101",
    "--baud", "460800",
    "--non-interactive",
    "--before", "usb-reset",
    "--after", "hard-reset",
    "--skip-update-check",
    "0x10000",
    "0x400000",
    candidate,
  ]);
});
