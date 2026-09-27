import { inspectSyntheticClosure } from "./preflight-binding.fixture.mjs";
import { USB_STACK_AUDIT_SOURCES } from "./native-stack.mjs";
// Explicit synthetic device/native/publication prerequisites; never a production CLI seam.
import { chmod, mkdir, readFile, readdir, realpath, writeFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { contextFixture } from "../str005-v2-serial/context-fixtures.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { qualification } from "../str005-v2-serial/completed-fixture.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { preflight, load, legacyView } from "./context.mjs";
import { BEFORE, BEFORE_V3, MEASUREMENT_002, CORRECTION_CONTRACT, PREDECESSOR, CONTRACT, PREFLIGHT_AMENDMENT, TASK, sha256, check } from "./values.mjs";
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export async function fixture(t, { prepare = true } = {}) {
  const f = await contextFixture(t, { prepare: false }), firmwareRoot = f.options.firmwareRoot;
  for (const name of await readdir(resolve(REPO, "scripts/usb-bootstrap-measure"))) if (name.endsWith(".mjs")) await f.put(resolve(firmwareRoot, "scripts/usb-bootstrap-measure", name), await readFile(resolve(REPO, "scripts/usb-bootstrap-measure", name)));
  for (const path of USB_STACK_AUDIT_SOURCES) await f.put(resolve(firmwareRoot, path), await readFile(resolve(REPO, path)));
  for (const publicFixture of ["scripts/phase28.1.1.1-synthetic-pool-credentials.mjs", "scripts/fixed-usb-qualification/contract.mjs"])
    await f.put(resolve(firmwareRoot, publicFixture), await readFile(resolve(REPO, publicFixture)));
  await f.put(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), await readFile(resolve(REPO, "firmware/bitaxe/bwg/deployment-trust.json")));
  await f.put(resolve(firmwareRoot, PREFLIGHT_AMENDMENT.path), await readFile(resolve(REPO, PREFLIGHT_AMENDMENT.path)));
  await f.put(resolve(firmwareRoot, CORRECTION_CONTRACT.path), await readFile(resolve(REPO, CORRECTION_CONTRACT.path)));
  await f.put(resolve(firmwareRoot, CONTRACT.path), await readFile(resolve(REPO, CONTRACT.path)));
  await f.put(resolve(firmwareRoot, "TASKS.md"), `## Active\n### ${TASK} | synthetic test only\n`);
  const parent = resolve(firmwareRoot, "scratch/usb-bootstrap-measure"); await mkdir(parent, { mode: 0o700 });
  const root = resolve(parent, "attempt-003"), previousRoot = resolve(parent, "attempt-002");
  await f.put(resolve(previousRoot, "install-0.claim.json"), JSON.stringify({ detector: { physical: "c".repeat(64) } }));
  const native = f.operations.inspectNative;
  const tools = { path: await realpath(process.execPath), version: process.version, sha256: sha256(await readFile(process.execPath)) };
  const predecessor = { context: { package: { firmware_commit: BEFORE_V3.firmware_commit, manifest_sha256: "e".repeat(64) }, sourceInventory: [] },
    beforeSource: BEFORE_V3, binding: { root: previousRoot, ...MEASUREMENT_002 }, originalCampaign: { id: Buffer.alloc(16, 4).toString("base64url"),
    record: { path: "accounting-after.json", sha256: "c".repeat(64), length: 1 } }, expectedAccounting: { ledger, original } };
  await f.put(resolve(firmwareRoot, "bazel-bin/tools/device-session/tests"), "synthetic regression binary"); await chmod(resolve(firmwareRoot, "bazel-bin/tools/device-session/tests"), 0o700);
  await f.put(resolve(firmwareRoot, "bazel-bin/tools/device-session/reader-test-build.json"), JSON.stringify({ schema: "usb-bootstrap-reader-test-identity-v1", sourceCommit: "a".repeat(40), sourceDirty: false,
    writerSha256: sha256(await readFile(resolve(firmwareRoot, "scripts/usb-bootstrap-measure/test-provenance.mjs"))) }));
  const operations = { ...f.operations, inspectMeasurementPredecessor: async () => structuredClone(predecessor), measurementPins: async () => structuredClone(predecessor),
    runReaderRegression(program, args) {
      const identity = { schema: "usb-bootstrap-reader-test-identity-v1", sourceCommit: "a".repeat(40), sourceDirty: false };
      const stdout = args[0] === "--exact" ? JSON.stringify(identity) + "\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n" : args[0] === "--test" ? "# pass 2\n# fail 0\n# skipped 0\n# todo 0\n# cancelled 0\n" : "test result: ok. 2 passed; 0 failed; 0 ignored;\n";
      return { status: 0, signal: null, stdout: Buffer.from(stdout), stderr: Buffer.alloc(0) };
    }, inspectPreflightClosure: inspectSyntheticClosure, verifyPredecessorPins: async () => {},
    runSourceLayoutCheck: () => ({ status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) }), predecessor: async () => structuredClone(predecessor), hostTools: async () => ({ espflash: tools, managedEsptool: tools, node: tools }), inspectNative: async input => syntheticNative(await native(input), input, tools.sha256), inspectUsbWriterStack: async input => ({ schema: "usb-bootstrap-writer-stack-v1", stackBytes: 8192,
    requiredMarginBytes: 512, selectedFrameSumBytes: 512, remainingBytes: 7680, frames: [{ address: 4096, symbol: "bitaxe_firmware::bwg_worker_usb::writer::run", entryBytes: 512 }], roles: { run: "standalone_measured", emit: "absent_or_inlined", write: "absent_or_inlined", marker: "absent_or_inlined", record: "absent_or_inlined" }, completeCallgraphBound: false, hardwareFitVerified: false,
    elfSha256: input.expectedElfSha256, objdumpSha256: input.expectedObjdumpSha256, supplementalDecodeRanges: [], unresolvedJumps: [], decodeComplete: true, sources: await Promise.all(USB_STACK_AUDIT_SOURCES.map(async path => { const bytes = await readFile(resolve(firmwareRoot, path)); return { path, sha256: sha256(bytes), length: bytes.length }; })) }) };
  const options = { ...f.options, privateRoot: root, predecessorReceipt: resolve(previousRoot, "final-result.json") };
  execFileSync("git", ["-C", firmwareRoot, "add", "."]);

  if (prepare) await preflight(options, operations);
  const context = prepare ? await load(root, { operations }) : null;
  return { ...f, options, root, parent, context, operations, predecessor, state(phase = "before", closed = false) {
    const value = state(legacyView(context, phase), "candidate", closed);
    value.qualification = { ...qualification(), budget_reserved_ms: 240000 }; return value;
  } };
}
export async function writeTestSeam(f, captureMode = "healthy") {
  const value = { schema: "bootstrap-test-only-v1", contextSha256: sha256(JSON.stringify(f.context)), predecessor: f.predecessor, captureMode };
  await f.put(resolve(f.root, "test-seam.json"), JSON.stringify(value));
}
export async function testOperations(root) {
  const canonical = await realpath(root); check(canonical === root && /\/noise-v2-[A-Za-z0-9]+\/firmware\/scratch\/usb-bootstrap-measure\/attempt-003$/u.test(root), "bootstrap_test_root");
  const stored = (await proof(root, "context.json")).value, seam = (await proof(root, "test-seam.json")).value;
  check(seam.schema === "bootstrap-test-only-v1" && seam.contextSha256 === stored.sha256 && stored.context.package.firmware_commit === "a".repeat(40), "bootstrap_test_context");
  return { cleanPushed() {}, measurementPins: async () => seam.predecessor, inspectPreflightClosure: inspectSyntheticClosure, verifyPredecessorPins: async () => {}, predecessor: async () => seam.predecessor,
    execFileSync(program, args, options) {
      if (program === "/usr/sbin/lsof" && args[0] === "-t" && /^\/dev\/(?:cu|tty)\.synthetic$/u.test(args[1])) throw Object.assign(Error("synthetic_serial_absence"), { status: 1, signal: null, stdout: "", stderr: "" });
      return execFileSync(program, args, options);
    } };
}

async function syntheticNative(base, input, toolSha256) {
  const manifest = JSON.parse(await readFile(input.manifestPath)), app = manifest.artifacts.find(row => row.kind === "firmware_ota_image");
  const path = { symbol: "synthetic", ownership: "test-only", stackBytes: 8192, addedStackBytes: 0, selectedPathBytes: 512, requiredMarginBytes: 512,
    remainingStackBytes: 7680, selectedFunctions: 1, nodes: [{ symbol: "synthetic", entryBytes: 512 }], completeCallgraphBound: false };
  return { ...base, packageManifestSha256: sha256(await readFile(input.manifestPath)), appImageSha256: app.sha256, appImageBytes: 1, imageSlotBytes: 4194304,
    objdumpSha256: toolSha256, noise: { ...path, entryBytes: 512, entryBudgetBytes: 1024, callbackBinding: "synthetic", threadEntryBinding: "synthetic", supplementalRanges: 0, unresolvedJumps: [] },
    production: { schema: "production-owner-stack-audit-v1", stack_bytes: 8192, owner_entry_bytes: 512, owner_entry_budget_bytes: 1024, required_measured_free_bytes: 512 },
    telemetry: { schema: "telemetry-stack-audit-v1", result: "targeted_path_fits", main_stack_budget_bytes: 8192, targeted_path_bytes: 512, matched_paths: 1, complete_callgraph_bound: false, hardware_safety_verified: false, nodes: [], edges: [] },
    v2: { channel: path, share: path, supplementalRanges: 0, unresolvedJumps: [] }, startupHeapQualified: false, completeCallgraphBound: false };
}
