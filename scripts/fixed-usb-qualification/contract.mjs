import { createHash, randomBytes } from "node:crypto";
import { createReadStream } from "node:fs";
import { lstat, open, readFile, realpath } from "node:fs/promises";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { execFileSync } from "node:child_process";

export const WINDOW_MS = Object.freeze([180000, 30000, 30000]);
export const PAGE = "conformance/bwg-worker-serial-0.1/acceptance.html";
export const BUNDLE = "dist/worker-serial-acceptance/worker-serial-acceptance.js";
export class QualificationError extends Error {
  constructor(code) { super(code); this.code = code; }
}
export function requireCondition(condition, code) {
  if (!condition) throw new QualificationError(code);
}
export const hex = (value, length) => typeof value === "string" &&
  new RegExp(`^[0-9a-f]{${length}}$`, "u").test(value);
export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const nonce = () => randomBytes(16).toString("base64url");
export function canonicalBase64(value, bytes) {
  if (typeof value !== "string") return false;
  const decoded = Buffer.from(value, "base64url");
  return decoded.length === bytes && decoded.toString("base64url") === value;
}
export function exactObject(value, required, optional = []) {
  requireCondition(value !== null && typeof value === "object" && !Array.isArray(value), "object_shape");
  requireCondition(required.every((key) => Object.hasOwn(value, key)) &&
    Object.keys(value).every((key) => required.includes(key) || optional.includes(key)), "object_fields");
}
export function within(root, path) {
  const resolved = resolve(path);
  const suffix = relative(resolve(root), resolved);
  requireCondition(suffix !== "" && !suffix.startsWith("..") && !isAbsolute(suffix), "path_outside_root");
  return resolved;
}
export async function protectedPath(path, directory = false) {
  const stat = await lstat(path);
  requireCondition(!stat.isSymbolicLink() && (directory ? stat.isDirectory() : stat.isFile()) &&
    (stat.mode & 0o777) === (directory ? 0o700 : 0o600), "private_path_policy");
}
export async function missing(path) {
  try { await lstat(path); } catch (error) {
    if (error.code === "ENOENT") return;
    throw new QualificationError("private_path_unavailable");
  }
  throw new QualificationError("private_path_exists");
}
export async function writeNew(path, value) {
  const file = await open(path, "wx", 0o600);
  try { await file.writeFile(`${JSON.stringify(value, null, 2)}\n`); await file.sync(); }
  finally { await file.close(); }
}
export async function readJson(path) { return JSON.parse(await readFile(path, "utf8")); }
export async function fileDigest(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}
export function git(root, args) {
  try { return execFileSync("git", ["-C", root, ...args], { encoding: "utf8", maxBuffer: 1048576, timeout: 10000, stdio: ["ignore", "pipe", "pipe"] }).trim(); }
  catch { throw new QualificationError("repository_check_failed"); }
}
export function cleanPushed(root, expected) {
  requireCondition(hex(expected, 40) && git(root, ["rev-parse", "HEAD"]) === expected, "source_mismatch");
  requireCondition(git(root, ["status", "--porcelain", "--untracked-files=normal"]) === "", "source_dirty");
  requireCondition(git(root, ["rev-parse", "@{upstream}"]) === expected, "source_not_pushed");
}
export function ignored(root, path) {
  try { execFileSync("git", ["-C", root, "check-ignore", "--quiet", "--", path], { timeout: 10000, stdio: "ignore" }); }
  catch { throw new QualificationError("private_path_not_ignored"); }
}

export async function packageSnapshot(firmwareRoot, manifestPath, expectedSource) {
  const expectedPath = resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json");
  requireCondition(resolve(manifestPath) === expectedPath, "manifest_path");
  const manifest = await readJson(manifestPath);
  requireCondition(manifest.schema_version === 4 && manifest.build_identity?.source_dirty === false && manifest.source_commit === expectedSource && hex(manifest.app_elf_sha256, 64) &&
    hex(manifest.reference_commit, 40), "manifest_identity");
  const kinds = ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"];
  requireCondition(Array.isArray(manifest.artifacts) && manifest.artifacts.length === kinds.length, "artifact_set");
  const artifacts = [];
  for (const kind of kinds) {
    const matches = manifest.artifacts.filter((artifact) => artifact.kind === kind);
    requireCondition(matches.length === 1 && hex(matches[0].sha256, 64), "artifact_identity");
    const artifact = matches[0];
    requireCondition(typeof artifact.path === "string" && !isAbsolute(artifact.path), "artifact_path");
    const root = kind === "partition_table" ? firmwareRoot : dirname(manifestPath);
    const path = within(root, resolve(root, artifact.path));
    const stat = await lstat(path);
    requireCondition(stat.isFile() && !stat.isSymbolicLink(), "artifact_file");
    requireCondition(await fileDigest(path) === artifact.sha256, "artifact_digest");
    artifacts.push({ kind, sha256: artifact.sha256, length: stat.size });
  }
  requireCondition(artifacts.find((entry) => entry.kind === "firmware_elf").sha256 === manifest.app_elf_sha256, "elf_digest");
  const geometry = [["bootloader", 0, 0x8000], ["partition_table_binary", 0x8000, 0x1000],
    ["firmware_ota_image", 0x10000, 0x400000], ["www_spiffs_image", 0x410000, 0x300000], ["otadata_initial", 0xf10000, 0x2000]];
  requireCondition(Array.isArray(manifest.update_segments) && manifest.update_segments.length === 5, "update_segments_missing");
  for (const [index, [kind, offset, capacity]] of geometry.entries()) {
    const segment = manifest.update_segments[index];
    exactObject(segment, ["artifact_kind", "offset", "length"]);
    requireCondition(segment.artifact_kind === kind && segment.offset === offset && Number.isInteger(segment.length) &&
      segment.length > 0 && Math.ceil(segment.length / 4096) * 4096 <= capacity &&
      segment.length === artifacts.find((artifact) => artifact.kind === kind).length, "update_geometry");
  }
  return { manifest_sha256: await fileDigest(manifestPath), app_elf_sha256: manifest.app_elf_sha256,
    reference_commit: manifest.reference_commit, artifacts, update_segments: manifest.update_segments };
}

export function admitTrust(trust, authorityTrust) {
  requireCondition(trust.profile === "bwg-worker-deployment-trust/0.2" &&
    authorityTrust.profile === "bwg-worker-deployment-trust/0.2", "trust_profile");
  for (const role of ["updateAuthority", "workLeaseAuthority"]) {
    const deployed = trust[role], authority = authorityTrust[role];
    requireCondition(deployed?.issuer === authority?.issuer && deployed?.audience === authority?.audience &&
      Array.isArray(deployed?.keys) && Array.isArray(authority?.keys) && authority.keys.length > 0, "trust_role");
    for (const key of authority.keys) {
      requireCondition(deployed.keys.some((candidate) => candidate.kid === key.kid && candidate.x === key.x &&
        candidate.alg === "Ed25519" && candidate.crv === "Ed25519"), "authority_not_deployed");
    }
    for (const key of deployed.keys) {
      exactObject(key, ["kid", "kty", "crv", "x", "alg", "use", "key_ops"]);
      requireCondition(key.kty === "OKP" && key.crv === "Ed25519" && key.alg === "Ed25519" && key.use === "sig" &&
        canonicalBase64(key.x, 32) && Array.isArray(key.key_ops) && key.key_ops.length === 1 && key.key_ops[0] === "verify", "public_key_shape");
    }
  }
}

export async function canonicalDirectory(path) {
  const canonical = await realpath(path);
  requireCondition(canonical === resolve(path), "directory_alias");
  return canonical;
}
