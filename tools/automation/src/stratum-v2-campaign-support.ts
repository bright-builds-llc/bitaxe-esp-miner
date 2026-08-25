import { readFile, readdir, stat } from "node:fs/promises";
import { networkInterfaces, type NetworkInterfaceInfo } from "node:os";
import path from "node:path";

type JsonObject = Record<string, unknown>;

export type RestorePackage = {
  readonly manifestPath: string;
  readonly sourceCommit: string;
  readonly appElfSha256: string;
  readonly factorySha256: string;
};

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("object required");
  return value as JsonObject;
}

function requiredString(value: JsonObject, key: string): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) throw new Error("identity required");
  return candidate;
}

async function walkManifests(
  root: string,
  output: string[],
  budget: { remaining: number },
): Promise<void> {
  if (budget.remaining <= 0) throw new Error("restore-package inventory exceeded bound");
  budget.remaining -= 1;
  const entries = await readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isSymbolicLink()) continue;
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) await walkManifests(candidate, output, budget);
    else if (entry.isFile() && /(?:package-manifest|bitaxe-ultra205-package)\.json$/u.test(entry.name)) {
      output.push(candidate);
    }
  }
}

export async function selectRestorePackage(
  workspace: string,
  appElfSha256: string,
): Promise<RestorePackage> {
  const manifestPaths: string[] = [];
  await walkManifests(path.join(workspace, "scratch"), manifestPaths, { remaining: 10_000 });
  const candidates: RestorePackage[] = [];
  for (const manifestPath of manifestPaths.sort()) {
    try {
      const manifest = object(JSON.parse(await readFile(manifestPath, "utf8")));
      if (manifest["app_elf_sha256"] !== appElfSha256) continue;
      const artifacts = manifest["artifacts"];
      if (!Array.isArray(artifacts)) continue;
      const factory = artifacts.map(object).find(value => value["kind"] === "factory_merged_image");
      if (factory === undefined) continue;
      const factoryPath = path.resolve(path.dirname(manifestPath), requiredString(factory, "path"));
      if (!(await stat(factoryPath)).isFile()) continue;
      candidates.push({
        manifestPath,
        sourceCommit: requiredString(manifest, "source_commit"),
        appElfSha256,
        factorySha256: requiredString(factory, "sha256"),
      });
    } catch {
      continue;
    }
  }
  const identities = new Set(candidates.map(candidate =>
    `${candidate.sourceCommit}:${candidate.appElfSha256}:${candidate.factorySha256}`));
  if (candidates.length === 0 || identities.size !== 1 || candidates[0] === undefined) {
    throw new Error("exact prior package is unavailable or ambiguous");
  }
  return candidates[0];
}

function ipv4(value: string): number | undefined {
  const parts = value.split(".").map(Number);
  if (parts.length !== 4 || parts.some(part => !Number.isInteger(part) || part < 0 || part > 255)) {
    return undefined;
  }
  return parts.reduce((total, part) => ((total << 8) | part) >>> 0, 0);
}

export function sameSubnetFixtureAddress(
  origin: URL,
  interfaces: NodeJS.Dict<NetworkInterfaceInfo[]> = networkInterfaces(),
): string {
  const device = ipv4(origin.hostname);
  if (device === undefined) throw new Error("device origin is not IPv4");
  const tunnel = /^(?:utun|tun|tap|wg|ppp|ipsec|tailscale)/u;
  const candidates = Object.entries(interfaces).flatMap(([name, values]) =>
    (values ?? []).flatMap(value => {
      const address = ipv4(value.address);
      const mask = ipv4(value.netmask);
      if (value.family !== "IPv4" || value.internal || tunnel.test(name)
        || address === undefined || mask === undefined || (address & mask) !== (device & mask)) {
        return [];
      }
      return [value.address];
    }));
  const unique = [...new Set(candidates)];
  if (unique.length !== 1 || unique[0] === undefined) {
    throw new Error("same-subnet fixture route is ambiguous");
  }
  return unique[0];
}
