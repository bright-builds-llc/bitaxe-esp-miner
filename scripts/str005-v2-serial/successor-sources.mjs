import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { cleanPushed, git } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { sourceInventory, validateSourcePath } from "./context-sources.mjs";
import { AMENDMENT_PATH, CONTRACT_PATH, PERMISSION_AMENDMENT_PATH, CLEANUP_AMENDMENT_PATH, CLEANUP_AMENDMENT_SHA256, check, object, sha256 } from "./values.mjs";

export const SUCCESSOR_MODULES = ["successor-readiness.mjs", "successor-evidence.mjs", "successor-continuity.mjs", "successor-ownership.mjs", "successor-sources.mjs"]
  .map(name => `scripts/str005-v2-serial/${name}`);
// Fixed v1 dependency domain matches the qualification evaluator. Enumerate at the recorded
// commit, so later HEAD changes cannot alter the historical membership or require a rerun.
const ROOTS = ["scripts/str005-v2-serial", "scripts/str005-noise-serial", "scripts/fixed-usb-qualification", "scripts/host-stalls",
  "tools/stratum-v2-fixture", "tools/http-transport", "crates/bitaxe-stratum", "crates/bitaxe-worker-control", "firmware/bitaxe/src"];
const FILES = [CONTRACT_PATH, AMENDMENT_PATH, PERMISSION_AMENDMENT_PATH, CLEANUP_AMENDMENT_PATH, "Cargo.toml", "Cargo.lock", "MODULE.bazel",
  "firmware/bitaxe/bwg/deployment-trust.json", "tools/automation/src/redaction.ts", "scripts/str005-v2-serial/client.mjs",
  "scripts/str005-v2-serial/operator.mjs", "scripts/str005-v2-serial/observer-build-identity.mjs", "scripts/str005-v2-serial/permission-correction.mjs"];
const HERE = dirname(fileURLToPath(import.meta.url));
function required(context) { return [...context.native_source_files, ...context.native_auditor_sources, ...SUCCESSOR_MODULES]; }

function treePaths(root, commit, context) {
  try { return execFileSync("git", ["-C", root, "ls-tree", "-r", "--name-only", commit, "--", ...ROOTS, ...FILES, ...required(context)],
    { encoding: "utf8", timeout: 10000, maxBuffer: 1048576, stdio: ["ignore", "pipe", "pipe"] }).trim().split("\n").filter(Boolean).sort(); }
  catch { check(false, "v2_successor_checker_tree"); }
}
function treeSources(root, commit, paths) {
  let output;
  try { output = execFileSync("git", ["-C", root, "cat-file", "--batch"], { input: paths.map(path => `${commit}:${path}\n`).join(""),
    timeout: 30000, maxBuffer: 134217728, stdio: ["pipe", "pipe", "pipe"] }); }
  catch { check(false, "v2_successor_checker_blobs"); }
  const rows = []; let offset = 0;
  for (const path of paths) {
    const end = output.indexOf(10, offset); check(end >= offset, "v2_successor_checker_blob_header");
    const match = /^[a-f0-9]{40,64} blob ([0-9]+)$/u.exec(output.subarray(offset, end).toString("utf8"));
    check(match, "v2_successor_checker_blob_header"); const length = Number(match[1]);
    check(Number.isSafeInteger(length) && length >= 0 && end + 1 + length < output.length && output[end + 1 + length] === 10, "v2_successor_checker_blob_bound");
    const bytes = output.subarray(end + 1, end + 1 + length); rows.push({ path, sha256: sha256(bytes), length }); offset = end + 2 + length;
  }
  check(offset === output.length, "v2_successor_checker_blob_tail"); return rows;
}

export async function inspectCheckerIdentity(value, context, operations = {}) {
  object(value, ["firmwareCommit", "sources"]);
  check(/^[a-f0-9]{40}$/u.test(value.firmwareCommit) && value.firmwareCommit !== context.firmware_commit && Array.isArray(value.sources), "v2_successor_checker_identity");
  for (const row of value.sources) { object(row, ["path", "sha256", "length"]); validateSourcePath(row.path); }
  const paths = await (operations.publishedCheckerPaths ?? treePaths)(context.firmware_root, value.firmwareCommit, context);
  check(paths.length > 0 && new Set(paths).size === paths.length && [...FILES, ...required(context)].every(path => paths.includes(path)) &&
    canonical(paths) === canonical(value.sources.map(row => row.path)), "v2_successor_checker_membership");
  paths.forEach(validateSourcePath);
  const sources = await (operations.publishedCheckerSources ?? treeSources)(context.firmware_root, value.firmwareCommit, paths);
  check(canonical(sources) === canonical(value.sources) && sources.find(row => row.path === CLEANUP_AMENDMENT_PATH)?.sha256 === CLEANUP_AMENDMENT_SHA256,
    "v2_successor_checker_source");
  return value;
}

export async function createCheckerIdentity(context, operations = {}) {
  const commit = (operations.git ?? git)(context.firmware_root, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(context.firmware_root, commit);
  const sources = await sourceInventory(context.firmware_root, required(context));
  for (const path of SUCCESSOR_MODULES) check(sources.find(row => row.path === path)?.sha256 ===
    sha256(await readFile(resolve(HERE, path.split("/").at(-1)))), "v2_successor_running_checker_changed");
  const value = { firmwareCommit: commit, sources }; await inspectCheckerIdentity(value, context, operations);
  (operations.cleanPushed ?? cleanPushed)(context.firmware_root, commit); return value;
}
