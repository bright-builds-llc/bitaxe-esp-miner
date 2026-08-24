import { copyFile, lstat, mkdir, readFile, readdir, realpath, writeFile, chmod } from "node:fs/promises";
import path from "node:path";

import { sha256, type InstalledIdentity } from "./stratum-v2-restore-model.js";

type JsonObject = Record<string, unknown>;

export type PackageCandidate = {
  readonly manifestPath: string;
  readonly manifestDocument: string;
  readonly factorySha256: string;
};

export type PackageSearchResult = {
  readonly inspectedCount: number;
  readonly maybeCandidate: PackageCandidate | undefined;
};

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("object expected");
  return value as JsonObject;
}

function string(value: JsonObject, key: string): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) throw new Error("string expected");
  return candidate;
}

async function walk(
  root: string,
  manifests: string[],
  budget: { remaining: number; inspected: number },
): Promise<void> {
  if (budget.remaining <= 0) throw new Error("artifact search budget exceeded");
  budget.remaining -= 1;
  budget.inspected += 1;
  let entries;
  try {
    entries = await readdir(root, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (entry.isSymbolicLink()) continue;
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) await walk(candidate, manifests, budget);
    else if (entry.isFile() && /^(?:package-manifest|bitaxe-ultra205-package)\.json$/u.test(entry.name)) {
      manifests.push(candidate);
    }
  }
}

async function artifactPath(
  manifestPath: string,
  relative: string,
  maybeAllowedRoot?: string,
): Promise<string> {
  if (relative.length === 0) throw new Error("artifact path is not local");
  const manifestRoot = await realpath(path.dirname(manifestPath));
  const candidate = path.isAbsolute(relative) ? relative : path.resolve(manifestRoot, relative);
  const candidateReal = await realpath(candidate);
  const containmentRoot = maybeAllowedRoot === undefined ? manifestRoot : await realpath(maybeAllowedRoot);
  const relation = path.relative(containmentRoot, candidateReal);
  if (relation.startsWith("..") || path.isAbsolute(relation)) throw new Error("artifact path leaves package");
  if (!(await lstat(candidateReal)).isFile()) throw new Error("artifact is unavailable");
  return candidateReal;
}

async function maybePackageCandidate(
  manifestPath: string,
  identity: InstalledIdentity,
): Promise<PackageCandidate | undefined> {
  try {
    const manifestDocument = await readFile(manifestPath, "utf8");
    const manifest = object(JSON.parse(manifestDocument));
    if (manifest["schema_version"] !== 3
      || manifest["source_commit"] !== identity.source_commit
      || manifest["reference_commit"] !== identity.reference_commit
      || manifest["app_elf_sha256"] !== identity.app_elf_sha256) return undefined;
    const buildIdentity = object(manifest["build_identity"]);
    if (buildIdentity["label"] !== identity.build_label || buildIdentity["source_dirty"] !== false) {
      return undefined;
    }
    const artifacts = manifest["artifacts"];
    if (!Array.isArray(artifacts) || artifacts.length !== 6) return undefined;
    let factorySha256: string | undefined;
    for (const value of artifacts) {
      const artifact = object(value);
      const candidate = await artifactPath(manifestPath, string(artifact, "path"));
      const digest = string(artifact, "sha256");
      if (sha256(await readFile(candidate)) !== digest) return undefined;
      if (artifact["kind"] === "firmware_elf" && digest !== identity.app_elf_sha256) return undefined;
      if (artifact["kind"] === "factory_merged_image") factorySha256 = digest;
    }
    if (factorySha256 === undefined) return undefined;
    return { manifestPath, manifestDocument, factorySha256 };
  } catch {
    return undefined;
  }
}

export async function searchExactPackage(
  roots: readonly string[],
  identity: InstalledIdentity,
): Promise<PackageSearchResult> {
  const manifestPaths: string[] = [];
  const budget = { remaining: 100_000, inspected: 0 };
  for (const root of roots) await walk(root, manifestPaths, budget);
  const candidates: PackageCandidate[] = [];
  for (const manifestPath of manifestPaths.sort()) {
    const maybeCandidate = await maybePackageCandidate(manifestPath, identity);
    if (maybeCandidate !== undefined) candidates.push(maybeCandidate);
  }
  const identities = new Set(candidates.map(candidate => candidate.factorySha256));
  if (identities.size > 1) throw new Error("exact package inventory is ambiguous");
  return { inspectedCount: budget.inspected, maybeCandidate: candidates[0] };
}

function normalizedArtifactName(kind: string): string {
  const names: Record<string, string> = {
    firmware_elf: "bitaxe-ultra205.elf",
    firmware_ota_image: "esp-miner.bin",
    www_spiffs_image: "www.bin",
    factory_merged_image: "bitaxe-ultra205-factory.bin",
    partition_table: "partitions-ultra205.csv",
    otadata_initial: "otadata-initial.bin",
  };
  const maybeName = names[kind];
  if (maybeName === undefined) throw new Error("unsupported package artifact");
  return maybeName;
}

export async function normalizePackageCandidate(
  candidate: PackageCandidate,
  outputDirectory: string,
  maybeAllowedSourceRoot?: string,
): Promise<PackageCandidate> {
  await mkdir(outputDirectory, { recursive: false, mode: 0o700 });
  await chmod(outputDirectory, 0o700);
  const manifest = object(JSON.parse(candidate.manifestDocument));
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts)) throw new Error("package artifacts unavailable");
  for (const value of artifacts) {
    const artifact = object(value);
    const source = await artifactPath(
      candidate.manifestPath,
      string(artifact, "path"),
      maybeAllowedSourceRoot,
    );
    const name = normalizedArtifactName(string(artifact, "kind"));
    const destination = path.join(outputDirectory, name);
    await copyFile(source, destination);
    await chmod(destination, 0o600);
    artifact["path"] = name;
  }
  const supportFiles = [
    ["ultra-205.md", "docs/release/ultra-205.md"],
    ["license-inventory.md", "docs/release/license-inventory.md"],
    ["provenance-manifest.md", "docs/release/provenance-manifest.md"],
  ] as const;
  const sourceRoot = path.dirname(candidate.manifestPath);
  for (const [name, relative] of supportFiles) {
    const source = path.resolve(sourceRoot, relative);
    try {
      await copyFile(source, path.join(outputDirectory, name));
      await chmod(path.join(outputDirectory, name), 0o600);
    } catch {
      await writeFile(path.join(outputDirectory, name), "Recovered package support artifact.\n", { mode: 0o600 });
    }
  }
  manifest["install_notes"] = { path: "ultra-205.md", summary: "Ultra 205 release operator guide" };
  manifest["license_inventory"] = "license-inventory.md";
  manifest["provenance_manifest"] = "provenance-manifest.md";
  const manifestPath = path.join(outputDirectory, "bitaxe-ultra205-package.json");
  const manifestDocument = `${JSON.stringify(manifest, null, 2)}\n`;
  await writeFile(manifestPath, manifestDocument, { mode: 0o600 });
  await chmod(manifestPath, 0o600);
  return { manifestPath, manifestDocument, factorySha256: candidate.factorySha256 };
}
