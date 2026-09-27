import { constants } from "node:fs";
import { lstat, open, readdir, realpath } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { canonical, inventory } from "../str005-noise-serial/files.mjs";
import { validateWriter } from "./native-writer.mjs";
import { verifyPublishedSources } from "./preflight-corpus-git.mjs";
import { check, object, sha256 } from "./values.mjs";
export const FIXED_PREFLIGHT = Object.freeze({
  root: "scratch/usb-bootstrap-measure/attempt-001", assignment: "attempt-ordinal-1.json",
  commit: "29cf472dc50e6763cd30f525efc2b013eea5dfba",
  contextSha256: "71a82b212b63e7d3023138bb569f756e5415907d401c6dbf815cc3387ce91632",
  contextFileSha256: "dd0084021071b22c4b2c465a0be5fa93dad68434f81081350ff387f887905dea",
  assignmentSha256: "759685dc765117c4bef70d5a3888f980104b4677cc30095e8f4760c158219980",
  inventorySha256: "a481daa922ec5d96565225c1b597a09304c2a246cb4fbc524c6b3c578345f9ff",
  commandLog: "scratch/usb-bootstrap-measure-software-20260926/preflight-corrected.log",
  commandLogSha256: "ff6af63a7d7f7b619ca9df84a204a0750ba5abd530c68255a55fc11509267999", commandLogLength: 1438,
  fileCount: 1773, directoryCount: 216, sourceCount: 1758,
  reservedSource: "scripts/phase28.1.1.1-synthetic-pool-credentials.mjs",
});
const forbidden = /(?:credentials|private[-_]key|authority[-_]directory)/iu;
const nonClaims = ["no_hardware_execution_proof", "no_fresh_device_accounting", "no_qualification_credit", "no_continuation_authority"];
async function protectedEntry(path, directory) {
  const stat = await lstat(path);
  check(stat.uid === process.getuid() && !stat.isSymbolicLink() && await realpath(path) === path && (directory ? stat.isDirectory() : stat.isFile() && stat.nlink === 1) &&
    (stat.mode & 0o777) === (directory ? 0o700 : 0o600), "bootstrap_corpus_path"); return stat;
}
async function protectedBytes(path) {
  const before = await protectedEntry(path, false);
  const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const opened = await handle.stat();
    check(opened.uid === process.getuid() && opened.isFile() && opened.nlink === 1 && opened.dev === before.dev && opened.ino === before.ino && opened.size <= 64 * 1024 * 1024, "bootstrap_corpus_changed");
    const bytes = await handle.readFile(), after = await protectedEntry(path, false);
    check(before.dev === after.dev && before.ino === after.ino && bytes.length === after.size, "bootstrap_corpus_changed"); return bytes;
  } finally { await handle.close(); }
}
const validPath = path => typeof path === "string" && path.length > 0 && !path.startsWith("/") && !/[\r\n\\\0]/u.test(path) && path.split("/").every(part => part !== "" && part !== "." && part !== "..");
async function absent(path) { try { await lstat(path); return false; } catch (error) { if (error.code === "ENOENT") return true; throw error; } }
async function mappedInventory(root, expected, anchors) {
  const files = [], directories = new Set([""]);
  for (const path of expected) { let parent = dirname(path); while (parent !== ".") { directories.add(parent); parent = dirname(parent); } }
  const visited = new Set();
  async function visit(path) {
    const logical = relative(root, path), stat = await lstat(path);
    if (stat.isDirectory()) {
      await protectedEntry(path, true); check(directories.has(logical), "bootstrap_corpus_membership"); visited.add(logical);
      for (const name of (await readdir(path)).sort()) await visit(resolve(path, name));
      return;
    }
    check(expected.has(logical) && (!logical.split("/").some(part => forbidden.test(part)) || logical === `snapshot/source/${anchors.reservedSource}`), "bootstrap_corpus_membership");
    const bytes = await protectedBytes(path); files.push({ path: logical, sha256: sha256(bytes), length: bytes.length });
  }
  await visit(root);
  check(files.length === expected.size && files.length === anchors.fileCount && visited.size === directories.size && visited.size === anchors.directoryCount, "bootstrap_corpus_membership");
  check(sha256(canonical(files)) === anchors.inventorySha256, "bootstrap_corpus_inventory"); return files;
}

/** Immutable preparation classifier only. It cannot create a receipt or acquire effect authority. */
export async function verifyPreflightCorpus(failedRoot, operations = {}) {
  const anchors = operations.anchors ?? FIXED_PREFLIGHT, root = resolve(failedRoot);
  await protectedEntry(root, true);
  const contextBytes = await protectedBytes(resolve(root, "context.json"));
  check(sha256(contextBytes) === anchors.contextFileSha256, "bootstrap_corpus_context");
  const wrapper = JSON.parse(contextBytes); object(wrapper, ["context", "sha256"]); const context = wrapper.context;
  check(wrapper.sha256 === anchors.contextSha256 && sha256(JSON.stringify(context)) === wrapper.sha256 &&
    context.schema === "usb-bootstrap-measure-context-v1" && context.package.firmware_commit === anchors.commit &&
    root === resolve(context.firmwareRoot, anchors.root) && context.attempt.ordinal === 1 && context.miningAuthorized === false, "bootstrap_corpus_context");
  const assignmentPath = resolve(dirname(root), anchors.assignment), logPath = resolve(context.firmwareRoot, anchors.commandLog);
  await protectedEntry(dirname(root), true); await protectedEntry(dirname(logPath), true);
  const assignmentBytes = await protectedBytes(assignmentPath), logBytes = await protectedBytes(logPath);
  check(sha256(assignmentBytes) === anchors.assignmentSha256 && sha256(logBytes) === anchors.commandLogSha256 && logBytes.length === anchors.commandLogLength, "bootstrap_corpus_external");
  check(canonical(JSON.parse(assignmentBytes)) === canonical({ schema: "usb-bootstrap-measure-assignment-v1", root, contextSha256: wrapper.sha256, attemptId: context.attempt.id }), "bootstrap_corpus_assignment");
  const sources = context.sourceInventory;
  check(Array.isArray(sources) && sources.length === anchors.sourceCount && sources.some(row => row.path === anchors.reservedSource), "bootstrap_corpus_sources");
  for (const [i, row] of sources.entries()) { object(row, ["path", "sha256", "length"]); check(validPath(row.path) && /^[a-f0-9]{64}$/u.test(row.sha256) && Number.isSafeInteger(row.length) && row.length >= 0 &&
    (i === 0 || sources[i - 1].path < row.path) && (!row.path.split("/").some(part => forbidden.test(part)) || row.path === anchors.reservedSource), "bootstrap_corpus_sources"); }
  const fixed = ["context.json", "snapshot/manifest.json", "snapshot/native-writer.json", "snapshot/native.json", "snapshot/trust.json", "snapshot/gate/page", "snapshot/gate/bundle"];
  check(context.package.artifacts.length === 8 && context.package.artifacts.every(row => /^[a-z_]+$/u.test(row.kind)), "bootstrap_corpus_artifacts");
  const expected = new Set([...fixed, ...context.package.artifacts.map(row => `snapshot/package/${row.kind}`), ...sources.map(row => `snapshot/source/${row.path}`)]);
  const files = await mappedInventory(root, expected, anchors), byPath = new Map(files.map(row => [row.path, row]));
  for (const [prefix, entries] of [["snapshot/source", sources], ["snapshot/package", context.package.artifacts.map(row => ({ ...row, path: row.kind }))]])
    for (const row of entries) { const item = byPath.get(`${prefix}/${row.path}`); check(item.sha256 === row.sha256 && item.length === row.length, "bootstrap_corpus_bytes"); }
  await (operations.verifyPublishedSources ?? verifyPublishedSources)(context.firmwareRoot, anchors.commit, sources);
  const json = async path => JSON.parse(await protectedBytes(resolve(root, path)));
  const manifest = await json("snapshot/manifest.json");
  check(byPath.get("snapshot/manifest.json").sha256 === context.package.manifest_sha256 && manifest.source_commit === anchors.commit && manifest.app_elf_sha256 === context.package.app_elf_sha256 &&
    manifest.artifacts.length === 8 && manifest.artifacts.every(row => context.package.artifacts.some(item => item.kind === row.kind && item.sha256 === row.sha256)), "bootstrap_corpus_manifest");
  check(canonical(await json("snapshot/native.json")) === canonical(context.nativeReadiness) && context.nativeReadiness.firmwareCommit === anchors.commit && context.nativeReadiness.elfSha256 === context.package.app_elf_sha256 &&
    context.nativeReadiness.result === "selected_native_checks_passed" && context.nativeReadiness.hardwareQualified === false, "bootstrap_corpus_native");
  validateWriter(await json("snapshot/native-writer.json"), context);
  check(byPath.get("snapshot/gate/page").sha256 === context.gate.pageSha256 && byPath.get("snapshot/gate/bundle").sha256 === context.gate.bundleSha256 &&
    sha256(JSON.stringify(await json("snapshot/trust.json"))) === context.trustSha256, "bootstrap_corpus_gate_trust");
  check(await absent(`${root}.operator`), "bootstrap_corpus_effects");
  let reproduced = false;
  try { await inventory(root); } catch (error) { if (error.message !== "noise_forbidden_inventory_name" && error.code !== "noise_forbidden_inventory_name") throw error; reproduced = true; }
  check(reproduced, "bootstrap_corpus_failure_class");
  check(canonical(files) === canonical(await mappedInventory(root, expected, anchors)) && assignmentBytes.equals(await protectedBytes(assignmentPath)) && logBytes.equals(await protectedBytes(logPath)), "bootstrap_corpus_changed");
  return { root, context, contextSha256: wrapper.sha256, assignment: { path: anchors.assignment, sha256: sha256(assignmentBytes), length: assignmentBytes.length },
    failedSourceCommit: anchors.commit, inspectedInputs: files, commandLog: { path: anchors.commandLog, sha256: sha256(logBytes), length: logBytes.length },
    inventorySha256: anchors.inventorySha256, diagnosedCause: { source: "read-only-reproduction", code: "noise_forbidden_inventory_name", repositoryPath: anchors.reservedSource }, nonClaims: [...nonClaims] };
}
