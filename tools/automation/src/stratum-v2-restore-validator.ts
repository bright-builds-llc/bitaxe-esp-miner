import { lstat, readFile, realpath } from "node:fs/promises";
import path from "node:path";

import {
  projectRestoreReadiness,
  sha256,
  type RestoreBundle,
  type RestoreReadinessProjection,
  validateRestoreBundle,
} from "./stratum-v2-restore-model.js";

type JsonObject = Record<string, unknown>;

function object(value: unknown, label: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as JsonObject;
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink()
    || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw new Error("protected recovery mode is invalid");
  }
}

async function containedFile(root: string, relative: string): Promise<string> {
  if (relative.length === 0 || path.isAbsolute(relative)) throw new Error("recovery path is invalid");
  const candidate = path.resolve(root, relative);
  const rootReal = await realpath(root);
  const parentReal = await realpath(path.dirname(candidate));
  const parentRelative = path.relative(rootReal, parentReal);
  if (parentRelative.startsWith("..") || path.isAbsolute(parentRelative)) {
    throw new Error("recovery path leaves the private root");
  }
  await requireMode(candidate, 0o600, false);
  return candidate;
}

async function validatePackageBundle(root: string, bundle: Extract<RestoreBundle, { kind: "package_v3" }>): Promise<void> {
  const manifestPath = await containedFile(root, bundle.package_manifest);
  const manifestDocument = await readFile(manifestPath, "utf8");
  if (sha256(manifestDocument) !== bundle.package_manifest_sha256) {
    throw new Error("recovery package manifest digest mismatch");
  }
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  if (manifest["schema_version"] !== 3
    || manifest["source_commit"] !== bundle.installed_identity.source_commit
    || manifest["reference_commit"] !== bundle.installed_identity.reference_commit
    || manifest["app_elf_sha256"] !== bundle.installed_identity.app_elf_sha256) {
    throw new Error("recovery package identity mismatch");
  }
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts) || artifacts.length !== 6) {
    throw new Error("recovery package artifact inventory is invalid");
  }
  const kinds = new Set<string>();
  let factorySeen = false;
  for (const value of artifacts) {
    const artifact = object(value, "package artifact");
    const kind = artifact["kind"];
    const artifactPath = artifact["path"];
    const digest = artifact["sha256"];
    if (typeof kind !== "string" || typeof artifactPath !== "string" || typeof digest !== "string") {
      throw new Error("recovery package artifact is malformed");
    }
    if (kinds.has(kind)) throw new Error("recovery package artifact is duplicated");
    kinds.add(kind);
    const candidate = await containedFile(path.dirname(manifestPath), artifactPath);
    if (sha256(await readFile(candidate)) !== digest) throw new Error("recovery package artifact digest mismatch");
    if (kind === "firmware_elf" && digest !== bundle.installed_identity.app_elf_sha256) {
      throw new Error("recovery package ELF mismatch");
    }
    if (kind === "factory_merged_image") {
      factorySeen = true;
      if (digest !== bundle.factory_sha256) throw new Error("recovery factory digest mismatch");
    }
  }
  if (!factorySeen) throw new Error("recovery factory artifact is missing");
}

async function validateSnapshotBundle(root: string, bundle: Extract<RestoreBundle, { kind: "flash_snapshot_v1" }>): Promise<void> {
  for (const range of bundle.ranges) {
    const candidate = await containedFile(root, range.path);
    const bytes = await readFile(candidate);
    if (bytes.length !== range.size || sha256(bytes) !== range.sha256) {
      throw new Error("recovery snapshot bytes mismatch");
    }
  }
}

export async function validateRestoreReadiness(
  bundlePath: string,
  projectionPath: string,
  expectedSourceCommit: string,
  expectedPlanSha256: string,
): Promise<RestoreReadinessProjection> {
  const root = path.dirname(bundlePath);
  await requireMode(root, 0o700, true);
  await requireMode(bundlePath, 0o600, false);
  await requireMode(projectionPath, 0o600, false);
  const bundleDocument = await readFile(bundlePath, "utf8");
  const bundle = JSON.parse(bundleDocument) as RestoreBundle;
  validateRestoreBundle(bundle);
  if (bundle.capture_source_commit !== expectedSourceCommit || bundle.plan_sha256 !== expectedPlanSha256) {
    throw new Error("recovery bundle source binding mismatch");
  }
  if (bundle.kind === "package_v3") await validatePackageBundle(root, bundle);
  else await validateSnapshotBundle(root, bundle);
  const projection = JSON.parse(await readFile(projectionPath, "utf8")) as RestoreReadinessProjection;
  const expected = projectRestoreReadiness(
    bundle,
    bundleDocument,
    projection.artifact_search_count,
    projection.rebuild_attempted,
  );
  if (JSON.stringify(projection) !== JSON.stringify(expected)) {
    throw new Error("recovery readiness projection mismatch");
  }
  return projection;
}
