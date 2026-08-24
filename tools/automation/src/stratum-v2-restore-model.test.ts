import assert from "node:assert/strict";
import test from "node:test";

import {
  formattedDateStatus,
  parseInstalledIdentity,
  projectRestoreReadiness,
  restoreBundleSchema,
  snapshotRangeTemplates,
  stableStatus,
  type InstalledIdentity,
  type RestoreBundle,
  validateRestoreBundle,
  validateSnapshotRanges,
} from "./stratum-v2-restore-model.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const digest = "c".repeat(64);
const planDigest = "d".repeat(64);

function wireIdentity(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    sourceCommit,
    referenceCommit,
    appElfSha256: digest,
    buildTimestampUtc: "2026-08-22T06:39:26Z",
    semanticVersion: "0.1.0",
    version: `${sourceCommit.slice(0, 12)}-dev`,
    buildChannel: "dev",
    sourceDirty: false,
    releaseTag: null,
    idfVersion: "v5.5.4",
    runningPartition: "factory",
    ...overrides,
  };
}

function identity(): InstalledIdentity {
  return parseInstalledIdentity(wireIdentity());
}

function packageBundle(): RestoreBundle {
  return {
    schema_version: restoreBundleSchema,
    kind: "package_v3",
    board: 205,
    installed_identity: identity(),
    package_manifest: "package/bitaxe-ultra205-package.json",
    package_manifest_sha256: digest,
    factory_sha256: "e".repeat(64),
    capture_source_commit: "f".repeat(40),
    plan_sha256: planDigest,
  };
}

test("installed identity parses the exact clean development shape", () => {
  // Arrange
  const value = wireIdentity();

  // Act
  const parsed = parseInstalledIdentity(value);

  // Assert
  assert.equal(parsed.source_commit, sourceCommit);
  assert.equal(parsed.build_channel, "dev");
  assert.equal(parsed.release_tag, null);
  assert.equal(parsed.running_partition, "factory");
});

test("installed identity rejects dirty-label and release contradictions", () => {
  // Arrange / Act / Assert
  assert.throws(() => parseInstalledIdentity(wireIdentity({ sourceDirty: true })));
  assert.throws(() => parseInstalledIdentity(wireIdentity({ buildChannel: "release" })));
  assert.throws(() => parseInstalledIdentity(wireIdentity({ runningPartition: "test" })));
});

test("status inputs reproduce the installed timestamp and provenance", () => {
  // Arrange
  const installed = identity();

  // Act
  const stable = stableStatus(installed);
  const volatile = formattedDateStatus(installed.build_timestamp_utc);

  // Assert
  assert(stable.includes(`STABLE_BITAXE_SOURCE_COMMIT ${sourceCommit}`));
  assert(stable.includes("STABLE_BITAXE_SOURCE_DIRTY false"));
  assert(volatile.includes("FORMATTED_DATE 2026 Aug 22 06 39 26 Sat"));
});

test("snapshot templates contain only the eight approved nonsecret ranges", () => {
  // Arrange
  const ranges = snapshotRangeTemplates("snapshot").map(range => ({
    ...range,
    sha256: digest,
  }));

  // Act / Assert
  assert.doesNotThrow(() => validateSnapshotRanges(ranges));
  assert.equal(ranges.length, 8);
  assert(!ranges.some(range => range.address < 0xf000 && range.address + range.size > 0x9000));
});

test("snapshot validation rejects overlap drift and reordered ranges", () => {
  // Arrange
  const ranges = snapshotRangeTemplates("snapshot").map(range => ({ ...range, sha256: digest }));
  const overlap = ranges.map((range, index) => index === 2
    ? { ...range, address: 0xe000, size: 0x2000 }
    : range);
  const reordered = [ranges[1], ranges[0], ...ranges.slice(2)].filter(
    (range): range is NonNullable<typeof range> => range !== undefined,
  );

  // Act / Assert
  assert.throws(() => validateSnapshotRanges(overlap));
  assert.throws(() => validateSnapshotRanges(reordered));
});

test("restore bundle rejects malformed package and snapshot variants", () => {
  // Arrange
  const valid = packageBundle();
  const malformed = { ...valid, package_manifest_sha256: "bad" } as RestoreBundle;

  // Act / Assert
  assert.doesNotThrow(() => validateRestoreBundle(valid));
  assert.throws(() => validateRestoreBundle(malformed));
});

test("readiness projection contains only closed aggregate recovery facts", () => {
  // Arrange
  const bundle = packageBundle();
  const document = `${JSON.stringify(bundle)}\n`;

  // Act
  const projection = projectRestoreReadiness(bundle, document, 71, true);
  const serialized = JSON.stringify(projection);

  // Assert
  assert.equal(projection.bundle_kind, "package_v3");
  assert.equal(projection.snapshot_range_count, 0);
  assert(!serialized.includes(sourceCommit));
  assert(!serialized.includes("package/"));
  assert(!serialized.includes("password"));
});
