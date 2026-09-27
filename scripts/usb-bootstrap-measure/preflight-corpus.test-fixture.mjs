// Synthetic preparation only: no real device, credentials, authority or failure receipt.
import { mkdtemp, realpath, mkdir, writeFile, rm, readFile, readdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { promisify } from "node:util";
import { execFile } from "node:child_process";
import { USB_STACK_AUDIT_SOURCES } from "./native-stack.mjs";
import { FIXED_PREFLIGHT } from "./preflight-corpus.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
const run = promisify(execFile);
export async function makeCorpusFixture(t) {
  const firmwareRoot = await realpath(await mkdtemp(resolve(tmpdir(), "bootstrap-corpus-test-")));
  t.after(() => rm(firmwareRoot, { recursive: true, force: true }));
  const root = resolve(firmwareRoot, FIXED_PREFLIGHT.root), anchors = { ...FIXED_PREFLIGHT };
  const retained = new Map();
  async function put(path, value) { await mkdir(dirname(path), { recursive: true, mode: 0o700 }); await writeFile(path, value, { mode: 0o600, flag: "wx" }); }
  const entries = [...new Set([...USB_STACK_AUDIT_SOURCES, anchors.reservedSource])].sort();
  const sourceInventory = [];
  for (const path of entries) { const bytes = Buffer.from(`// published synthetic source ${path}\n`); await put(resolve(firmwareRoot, path), bytes);
    sourceInventory.push({ path, sha256: sha256(bytes), length: bytes.length }); retained.set(`snapshot/source/${path}`, bytes); }
  const git = args => run("git", args, { cwd: firmwareRoot, timeout: 10000 });
  await git(["init", "-q"]); await git(["add", "scripts", "firmware"]);
  await git(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "Synthetic published sources"]);
  anchors.commit = (await git(["rev-parse", "HEAD"])).stdout.trim(); await git(["update-ref", "refs/remotes/origin/main", anchors.commit]);
  const kinds = ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"];
  const artifacts = kinds.map(kind => { const bytes = Buffer.from(`synthetic ${kind}`); retained.set(`snapshot/package/${kind}`, bytes); return { kind, sha256: sha256(bytes), length: bytes.length }; });
  const app = artifacts[0].sha256, tool = sha256("fixture tool");
  const manifest = { source_commit: anchors.commit, app_elf_sha256: app, artifacts: artifacts.map(({ kind, sha256: hash }) => ({ kind, sha256: hash })) };
  const encode = value => Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
  retained.set("snapshot/manifest.json", encode(manifest)); retained.set("snapshot/gate/page", Buffer.from("fixture page")); retained.set("snapshot/gate/bundle", Buffer.from("fixture bundle"));
  const trust = { fixturePublicTrust: true }; retained.set("snapshot/trust.json", encode(trust));
  const nativeReadiness = { firmwareCommit: anchors.commit, elfSha256: app, objdumpSha256: tool, result: "selected_native_checks_passed", hardwareQualified: false };
  retained.set("snapshot/native.json", encode(nativeReadiness));
  const context = { schema: "usb-bootstrap-measure-context-v1", firmwareRoot, package: { firmware_commit: anchors.commit, app_elf_sha256: app, artifacts, manifest_sha256: sha256(retained.get("snapshot/manifest.json")) },
    attempt: { ordinal: 1, id: "a".repeat(32) }, miningAuthorized: false, sourceInventory, nativeReadiness,
    gate: { pageSha256: sha256(retained.get("snapshot/gate/page")), bundleSha256: sha256(retained.get("snapshot/gate/bundle")) }, trustSha256: sha256(JSON.stringify(trust)) };
  const writer = { schema: "usb-bootstrap-writer-stack-v1", stackBytes: 8192, requiredMarginBytes: 512, selectedFrameSumBytes: 128, remainingBytes: 8064,
    frames: [{ address: 4096, symbol: "bitaxe_firmware::bwg_worker_usb::writer::run", entryBytes: 128 }], roles: { run: "standalone_measured", emit: "absent_or_inlined", write: "absent_or_inlined", marker: "absent_or_inlined", record: "absent_or_inlined" },
    completeCallgraphBound: false, hardwareFitVerified: false, elfSha256: app, objdumpSha256: tool, supplementalDecodeRanges: 0, unresolvedJumps: [], decodeComplete: true,
    sources: sourceInventory.filter(row => USB_STACK_AUDIT_SOURCES.includes(row.path)) };
  retained.set("snapshot/native-writer.json", encode(writer)); anchors.contextSha256 = sha256(JSON.stringify(context));
  retained.set("context.json", encode({ context, sha256: anchors.contextSha256 })); anchors.contextFileSha256 = sha256(retained.get("context.json"));
  for (const [path, bytes] of retained) await put(resolve(root, path), bytes);
  const assignment = encode({ schema: "usb-bootstrap-measure-assignment-v1", root, contextSha256: anchors.contextSha256, attemptId: context.attempt.id });
  await put(resolve(dirname(root), anchors.assignment), assignment); anchors.assignmentSha256 = sha256(assignment);
  const log = Buffer.from("bootstrap_operation_failed\n"); await put(resolve(firmwareRoot, anchors.commandLog), log); anchors.commandLogSha256 = sha256(log); anchors.commandLogLength = log.length;
  const dirs = new Set([""]); for (const path of retained.keys()) { let parent = dirname(path); while (parent !== ".") { dirs.add(parent); parent = dirname(parent); } }
  // Recursive lexical traversal, matching the historical inventory ordering.
  const inventory = [];
  async function walk(path, prefix = "") { for (const item of (await readdir(path, { withFileTypes: true })).sort((a,b) => a.name < b.name ? -1 : 1)) {
    const logical = `${prefix}${item.name}`; if (item.isDirectory()) await walk(resolve(path, item.name), `${logical}/`);
    else { const bytes = await readFile(resolve(path, item.name)); inventory.push({ path: logical, sha256: sha256(bytes), length: bytes.length }); }
  } }
  await walk(root); anchors.inventorySha256 = sha256(canonical(inventory)); anchors.sourceCount = sourceInventory.length; anchors.fileCount = inventory.length; anchors.directoryCount = dirs.size;
  return { root, firmwareRoot, context, anchors, operations: { anchors }, inventory };
}
