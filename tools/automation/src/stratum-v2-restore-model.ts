import { createHash } from "node:crypto";

export const restoreBundleSchema = "bitaxe-stratum-v2-restore-bundle-v1" as const;
export const restoreProjectionSchema = "bitaxe-stratum-v2-restore-readiness-v1" as const;

export type InstalledIdentity = {
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly app_elf_sha256: string;
  readonly build_timestamp_utc: string;
  readonly semantic_version: string;
  readonly build_label: string;
  readonly build_channel: "dev" | "release";
  readonly source_dirty: boolean;
  readonly release_tag: string | null;
  readonly idf_version: string;
  readonly running_partition: "factory" | "ota_0" | "ota_1";
};

export type SnapshotRangeName =
  | "bootloader"
  | "partition_table"
  | "phy_init"
  | "factory"
  | "www"
  | "ota_0"
  | "ota_1"
  | "otadata";

export type SnapshotRange = {
  readonly name: SnapshotRangeName;
  readonly address: number;
  readonly size: number;
  readonly path: string;
  readonly sha256: string;
};

export type PackageRestoreBundle = {
  readonly schema_version: typeof restoreBundleSchema;
  readonly kind: "package_v3";
  readonly board: 205;
  readonly installed_identity: InstalledIdentity;
  readonly package_manifest: string;
  readonly package_manifest_sha256: string;
  readonly factory_sha256: string;
  readonly capture_source_commit: string;
  readonly plan_sha256: string;
};

export type SnapshotRestoreBundle = {
  readonly schema_version: typeof restoreBundleSchema;
  readonly kind: "flash_snapshot_v1";
  readonly board: 205;
  readonly installed_identity: InstalledIdentity;
  readonly ranges: readonly SnapshotRange[];
  readonly capture_source_commit: string;
  readonly plan_sha256: string;
};

export type RestoreBundle = PackageRestoreBundle | SnapshotRestoreBundle;

export type RestoreReadinessProjection = {
  readonly schema_version: typeof restoreProjectionSchema;
  readonly status: "accepted";
  readonly board: 205;
  readonly bundle_kind: RestoreBundle["kind"];
  readonly installed_identity_sha256: string;
  readonly restore_bundle_sha256: string;
  readonly artifact_search_count: number;
  readonly rebuild_attempted: boolean;
  readonly snapshot_range_count: 0 | 8;
  readonly runtime_unchanged: true;
  readonly private_modes_valid: true;
  readonly independent_validation: true;
  readonly redaction_status: "passed";
  readonly exact_non_claims: readonly [
    "raw_nvs",
    "coredump",
    "new_baseline",
    "external_pool",
    "attempt_005",
  ];
};

const commitPattern = /^[0-9a-f]{40}$/u;
const digestPattern = /^[0-9a-f]{64}$/u;
const timestampPattern = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/u;
const buildLabelPattern = /^[0-9a-f]{12}(?:-dirty)?(?:-dev)?$/u;
const semanticVersionPattern = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/u;

const snapshotSpecifications = [
  { name: "bootloader", address: 0x000000, size: 0x008000 },
  { name: "partition_table", address: 0x008000, size: 0x001000 },
  { name: "phy_init", address: 0x00f000, size: 0x001000 },
  { name: "factory", address: 0x010000, size: 0x400000 },
  { name: "www", address: 0x410000, size: 0x300000 },
  { name: "ota_0", address: 0x710000, size: 0x400000 },
  { name: "ota_1", address: 0xb10000, size: 0x400000 },
  { name: "otadata", address: 0xf10000, size: 0x002000 },
] as const satisfies readonly Omit<SnapshotRange, "path" | "sha256">[];

const forbiddenRanges = [
  { start: 0x009000, end: 0x00f000 },
  { start: 0xf12000, end: 0x1000000 },
] as const;

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function string(value: Record<string, unknown>, key: string): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw new Error(`${key} is unavailable`);
  }
  return candidate;
}

function optionalString(value: Record<string, unknown>, key: string): string | null {
  const candidate = value[key];
  if (candidate === null) return null;
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw new Error(`${key} is malformed`);
  }
  return candidate;
}

export function parseInstalledIdentity(value: unknown): InstalledIdentity {
  const source = object(value, "installed identity");
  const sourceCommit = string(source, "sourceCommit");
  const referenceCommit = string(source, "referenceCommit");
  const appElfSha256 = string(source, "appElfSha256");
  const buildTimestampUtc = string(source, "buildTimestampUtc");
  const semanticVersion = string(source, "semanticVersion");
  const buildLabel = string(source, "version");
  const buildChannel = string(source, "buildChannel");
  const sourceDirty = source["sourceDirty"];
  const releaseTag = optionalString(source, "releaseTag");
  const idfVersion = string(source, "idfVersion");
  const runningPartition = string(source, "runningPartition");
  if (!commitPattern.test(sourceCommit) || !commitPattern.test(referenceCommit)) {
    throw new Error("installed commit identity is malformed");
  }
  if (!digestPattern.test(appElfSha256) || !timestampPattern.test(buildTimestampUtc)) {
    throw new Error("installed build identity is malformed");
  }
  if (!semanticVersionPattern.test(semanticVersion) || !buildLabelPattern.test(buildLabel)) {
    throw new Error("installed version identity is malformed");
  }
  if (buildChannel !== "dev" && buildChannel !== "release") {
    throw new Error("installed build channel is unsupported");
  }
  if (typeof sourceDirty !== "boolean") throw new Error("installed dirty state is malformed");
  if (releaseTag !== null && !/^v\d+\.\d+(?:\.\d+)?$/u.test(releaseTag)) {
    throw new Error("installed release tag is malformed");
  }
  if (!/^v\d+\.\d+\.\d+$/u.test(idfVersion)) throw new Error("installed IDF version is malformed");
  if (runningPartition !== "factory" && runningPartition !== "ota_0" && runningPartition !== "ota_1") {
    throw new Error("installed running partition is unsupported");
  }
  const expectedLabel = `${sourceCommit.slice(0, 12)}${sourceDirty ? "-dirty" : ""}${buildChannel === "dev" ? "-dev" : ""}`;
  if (buildLabel !== expectedLabel || (buildChannel === "release") !== (releaseTag !== null)) {
    throw new Error("installed build identity is contradictory");
  }
  return {
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    build_timestamp_utc: buildTimestampUtc,
    semantic_version: semanticVersion,
    build_label: buildLabel,
    build_channel: buildChannel,
    source_dirty: sourceDirty,
    release_tag: releaseTag,
    idf_version: idfVersion,
    running_partition: runningPartition,
  };
}

export function formattedDateStatus(timestamp: string): string {
  if (!timestampPattern.test(timestamp)) throw new Error("build timestamp is malformed");
  const date = new Date(timestamp);
  if (!Number.isFinite(date.getTime()) || date.toISOString().replace(".000Z", "Z") !== timestamp) {
    throw new Error("build timestamp is invalid");
  }
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;
  const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;
  const month = months[date.getUTCMonth()];
  const weekday = weekdays[date.getUTCDay()];
  if (month === undefined || weekday === undefined) throw new Error("build timestamp is invalid");
  return `BUILD_TIMESTAMP ${String(Math.floor(date.getTime() / 1000))}\nFORMATTED_DATE ${String(date.getUTCFullYear())} ${month} ${String(date.getUTCDate()).padStart(2, "0")} ${String(date.getUTCHours()).padStart(2, "0")} ${String(date.getUTCMinutes()).padStart(2, "0")} ${String(date.getUTCSeconds()).padStart(2, "0")} ${weekday}\n`;
}

export function stableStatus(identity: InstalledIdentity): string {
  return [
    `STABLE_BITAXE_SOURCE_COMMIT ${identity.source_commit}`,
    `STABLE_BITAXE_SOURCE_DIRTY ${String(identity.source_dirty)}`,
    `STABLE_BITAXE_RELEASE_TAG ${identity.release_tag ?? "unavailable"}`,
    `STABLE_BITAXE_SEMANTIC_VERSION ${identity.semantic_version}`,
    `STABLE_BITAXE_REFERENCE_COMMIT ${identity.reference_commit}`,
    "",
  ].join("\n");
}

export function snapshotRangeTemplates(directory: string): readonly Omit<SnapshotRange, "sha256">[] {
  return snapshotSpecifications.map(specification => ({
    ...specification,
    path: `${directory}/${specification.name}.bin`,
  }));
}

export function validateSnapshotRanges(ranges: readonly SnapshotRange[]): void {
  if (ranges.length !== snapshotSpecifications.length) throw new Error("snapshot range count is invalid");
  for (const [index, expected] of snapshotSpecifications.entries()) {
    const range = ranges[index];
    if (range === undefined
      || range.name !== expected.name
      || range.address !== expected.address
      || range.size !== expected.size
      || !digestPattern.test(range.sha256)
      || range.path.length === 0) {
      throw new Error("snapshot range contract is invalid");
    }
    const rangeEnd = range.address + range.size;
    if (forbiddenRanges.some(forbidden => range.address < forbidden.end && rangeEnd > forbidden.start)) {
      throw new Error("snapshot range overlaps prohibited storage");
    }
  }
}

export function validateRestoreBundle(bundle: RestoreBundle): void {
  if (bundle.schema_version !== restoreBundleSchema || bundle.board !== 205) {
    throw new Error("restore bundle schema is invalid");
  }
  if (!commitPattern.test(bundle.capture_source_commit) || !digestPattern.test(bundle.plan_sha256)) {
    throw new Error("restore bundle provenance is invalid");
  }
  parseInstalledIdentity({
    sourceCommit: bundle.installed_identity.source_commit,
    referenceCommit: bundle.installed_identity.reference_commit,
    appElfSha256: bundle.installed_identity.app_elf_sha256,
    buildTimestampUtc: bundle.installed_identity.build_timestamp_utc,
    semanticVersion: bundle.installed_identity.semantic_version,
    version: bundle.installed_identity.build_label,
    buildChannel: bundle.installed_identity.build_channel,
    sourceDirty: bundle.installed_identity.source_dirty,
    releaseTag: bundle.installed_identity.release_tag,
    idfVersion: bundle.installed_identity.idf_version,
    runningPartition: bundle.installed_identity.running_partition,
  });
  if (bundle.kind === "package_v3") {
    if (bundle.package_manifest.length === 0
      || !digestPattern.test(bundle.package_manifest_sha256)
      || !digestPattern.test(bundle.factory_sha256)) {
      throw new Error("package restore bundle is invalid");
    }
    return;
  }
  validateSnapshotRanges(bundle.ranges);
}

export function projectRestoreReadiness(
  bundle: RestoreBundle,
  bundleDocument: string,
  artifactSearchCount: number,
  rebuildAttempted: boolean,
): RestoreReadinessProjection {
  validateRestoreBundle(bundle);
  if (!Number.isSafeInteger(artifactSearchCount) || artifactSearchCount < 0 || artifactSearchCount > 100_000) {
    throw new Error("artifact search count is invalid");
  }
  return {
    schema_version: restoreProjectionSchema,
    status: "accepted",
    board: 205,
    bundle_kind: bundle.kind,
    installed_identity_sha256: sha256(JSON.stringify(bundle.installed_identity)),
    restore_bundle_sha256: sha256(bundleDocument),
    artifact_search_count: artifactSearchCount,
    rebuild_attempted: rebuildAttempted,
    snapshot_range_count: bundle.kind === "flash_snapshot_v1" ? 8 : 0,
    runtime_unchanged: true,
    private_modes_valid: true,
    independent_validation: true,
    redaction_status: "passed",
    exact_non_claims: ["raw_nvs", "coredump", "new_baseline", "external_pool", "attempt_005"],
  };
}
