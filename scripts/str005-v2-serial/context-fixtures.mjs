import { chmod, mkdir, readFile, readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { fixture as noiseFixture, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { NATIVE_AUDITOR_SOURCES } from "../noise-native-readiness.mjs";
import { sha256, PERMISSION_AMENDMENT_PATH, CLEANUP_AMENDMENT_PATH } from "./values.mjs";
import { nonce } from "../fixed-usb-qualification/contract.mjs";
import { canonical, writeNew } from "../str005-noise-serial/files.mjs";
import { createSnapshot } from "./snapshot.mjs";
import { inspectSources, nativeInterface } from "./context-sources.mjs";
import { AMENDMENT_PATH } from "./context-sources.mjs";
import { ACCEPTED_NOISE_RESULT_SHA256, ACCEPTED_NOISE_SEAL_SHA256 } from "./predecessor.mjs";
import { checkPermissionCorrection } from "./permission-correction.mjs";
import { loadContext, preflight } from "./context.mjs";
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const NATIVE = ["firmware/bitaxe/src/noise_serial_runtime.rs", "firmware/bitaxe/src/production_mining_session.rs",
  "firmware/bitaxe/src/production_mining_session/transport.rs", "firmware/bitaxe/src/production_mining_session/transport/borrow.rs"];
const AUDITORS = ["scripts/v2-native-readiness.mjs", ...NATIVE_AUDITOR_SOURCES];

/** Explicit synthetic filesystem/reader/auditor seam; never usable as live evidence. */
export async function contextFixture(t, { scope = "channel", prepare = true, legacy = false, permission = false, beforeLegacySnapshot, maybeFixtureBytes } = {}) {
  const base = await noiseFixture(t, { prepare: false });
  const { firmwareRoot, gateRoot } = base.options;
  await base.put(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-str005-v2-serial-qualification | synthetic qualification\n");
  await base.put(resolve(firmwareRoot, CLEANUP_AMENDMENT_PATH), await readFile(resolve(REPO, CLEANUP_AMENDMENT_PATH)));
  await base.put(resolve(firmwareRoot, PERMISSION_AMENDMENT_PATH), await readFile(resolve(REPO, PERMISSION_AMENDMENT_PATH)));
  await base.put(resolve(firmwareRoot, AMENDMENT_PATH), await readFile(resolve(REPO, AMENDMENT_PATH)));
  await base.put(resolve(firmwareRoot, "docs/hardware/str005-v2-serial-qualification.md"), await readFile(resolve(REPO, "docs/hardware/str005-v2-serial-qualification.md")));
  await base.put(resolve(firmwareRoot, "scripts/v2-native-readiness.mjs"), "// Explicit synthetic auditor; never a native proof.\n");
  for (const name of await readdir(resolve(REPO, "scripts/str005-v2-serial"))) if (name.endsWith(".mjs"))
    await base.put(resolve(firmwareRoot, "scripts/str005-v2-serial", name), await readFile(resolve(REPO, "scripts/str005-v2-serial", name)));
  for (const name of ["client", "operator", "observer-build-identity"]) {
    const path = resolve(firmwareRoot, `scripts/str005-v2-serial/${name}.mjs`);
    try { await readFile(path); }
    catch (error) { if (error.code !== "ENOENT") throw error; await base.put(path, "// Explicit synthetic fixture code; no device effects.\n"); }
  }
  await base.put(resolve(gateRoot, "dist/worker-serial-acceptance/worker-serial-acceptance.js"), `${"b".repeat(40)} bwg-worker-stratum-v2-standard/0.1 worker-stratum-v2-status-v1 stratumV2ChannelStart stratumV2Status stratumV2Scope explicit synthetic bundle`);
  const observerPath = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer");
  await base.put(observerPath, "explicit synthetic observer binary"); await chmod(observerPath, 0o700);
  await base.put(resolve(firmwareRoot, "tools/http-transport/src/lib.rs"), "// explicit synthetic observer source\n");
  await base.put(resolve(dirname(observerPath), "v2-observer-build-identity.json"), JSON.stringify({ schema: "str005-v2-observer-build-v1",
    sourceCommit: "a".repeat(40), sourceDirty: false, observerSha256: sha256(await readFile(observerPath)),
    writerSha256: sha256(await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/observer-build-identity.mjs"))) }));
  const fixtureBinary = base.options.fixtureBinary;
  if (maybeFixtureBytes !== undefined) {
    if (!Buffer.isBuffer(maybeFixtureBytes)) throw Error("fixture bytes must be an explicit Buffer");
    await base.put(fixtureBinary, maybeFixtureBytes);
  }
  await base.put(resolve(dirname(fixtureBinary), "v2-serial-build-identity.json"), JSON.stringify({ schema: "str005-v2-fixture-build-v1",
    sourceCommit: "a".repeat(40), sourceDirty: false, fixtureSha256: sha256(await readFile(fixtureBinary)),
    writerSha256: sha256(await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/build-identity.mjs"))) }));
  const parent = resolve(firmwareRoot, "scratch/str005-v2-serial");
  await base.put(resolve(parent, ".synthetic-parent"), "explicit synthetic namespace");
  const previousRoot = resolve(base.base, "accepted-noise");
  await base.put(resolve(previousRoot, "final-result.json"), JSON.stringify({ synthetic: true }));
  const previous = { root: previousRoot, resultSha256: ACCEPTED_NOISE_RESULT_SHA256, sealSha256: ACCEPTED_NOISE_SEAL_SHA256, ledger: structuredClone(ledger), original: structuredClone(original),
    context: { schema: "noise-serial-context-v2", firmware_commit: "d".repeat(40), app_elf_sha256: "e".repeat(64), original_campaign_id: Buffer.alloc(16, 4).toString("base64url") } };
  const operations = { ...base.operations, nativeSourceFiles: [...NATIVE], nativeAuditorSources: [...AUDITORS],
    inspectNative: async (input) => ({ schema: "str005-v2-native-readiness-v1", result: "selected_native_checks_passed",
      firmwareCommit: input.expectedSourceCommit, elfSha256: input.expectedElfSha256, hardwareQualified: false,
      sdkconfigSha256: sha256(await readFile(resolve(dirname(input.manifestPath), "bitaxe-firmware.sdkconfig"))),
      sourceFiles: await Promise.all(NATIVE.map(async (path) => ({ path, sha256: sha256(await readFile(resolve(firmwareRoot, path))) }))),
      auditorSources: await Promise.all(AUDITORS.map(async (path) => ({ path, sha256: sha256(await readFile(resolve(firmwareRoot, path))) }))) }),
    inspectPredecessor: async () => previous,
    async readNoiseAnchorProof(root, name) {
      if (root !== previousRoot || !["final-result.json", "sealed-inventory.json"].includes(name)) throw Error("synthetic Noise anchor request drift");
      return { sha256: name === "final-result.json" ? ACCEPTED_NOISE_RESULT_SHA256 : ACCEPTED_NOISE_SEAL_SHA256 };
    },
    spawnSync(program, args, options) {
      const gate = program === "bun" && args.join(" ") === "test ./web/worker-qualification-gesture.test.ts ./web/worker-serial-admission.test.ts" && options.timeout === 30000;
      const host = program === "node" && args.join(" ") === "--test scripts/str005-v2-serial/host-resources.test.mjs scripts/str005-v2-serial/continuity-baseline.test.mjs scripts/str005-v2-serial/cleanup-rehearsal.test.mjs" && options.timeout === 180000;
      if (!gate && !host) throw Error("synthetic correction command drift");
      return { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) };
    }, correctionNow: () => 0, hostCorrectionNow: () => 0, hostPlatform: "darwin" };
  const failedRoot = resolve(parent, "channel-001"), closurePath = `${failedRoot}.permission-closure.json`;
  if (!legacy) {
    // Explicit classifier seam; actual closure parsing has separate immutable
    // failure fixtures. This object never claims real hardware observations.
    const failedContext = { schema: "str005-v2-serial-context-v1", scope: "channel", hostOrdinal: 1,
      firmware_commit: "f".repeat(40), gate_commit: "e".repeat(40), gate_bundle_sha256: "f".repeat(64),
      predecessor: { root: previous.root, resultSha256: previous.resultSha256, sealSha256: previous.sealSha256 } };
    await base.put(resolve(failedRoot, "context.json"), JSON.stringify({ context: failedContext, sha256: sha256(JSON.stringify(failedContext)) }));
    const failedBytes = await readFile(resolve(failedRoot, "context.json"));
    await base.put(closurePath, JSON.stringify({ schema: "str005-v2-permission-closure-v1", failedRoot,
      failedContextSha256: sha256(JSON.stringify(failedContext)), synthetic: "permission classifier test seam",
      inputs: { attempt: [{ path: "context.json", sha256: sha256(failedBytes), length: failedBytes.length }] } }));
    await writeNew(resolve(parent, "channel-ordinal-1.json"), { schema: "str005-v2-serial-assignment-v1", root: failedRoot,
      scope: "channel", context_sha256: sha256(JSON.stringify(failedContext)) });
    operations.inspectPermissionClosure = async path => {
      if (path !== closurePath) throw Object.assign(Error("wrong closure"), { code: "v2_permission_path" });
      return { root: failedRoot, closurePath, closureSha256: sha256(await readFile(closurePath)), context: failedContext,
        contextSha256: sha256(JSON.stringify(failedContext)), predecessor: failedContext.predecessor,
        classification: "closed_no_device_admission", hardware_qualified: false };
    };
  }
  let maybeSuccessor;
  if (!legacy && !permission) {
    const failedRoot = resolve(parent, "channel-002"), receiptPath = `${failedRoot}.successor-readiness.json`;
    const failedContext = { schema: "str005-v2-serial-context-v2", scope: "channel", hostOrdinal: 2,
      firmware_commit: "f".repeat(40), app_elf_sha256: "f".repeat(64), manifest_sha256: "e".repeat(64), gate_commit: "b".repeat(40),
      evaluator: [{ path: "synthetic-old-source", sha256: "e".repeat(64), length: 1 }], original_campaign_id: previous.context.original_campaign_id,
      predecessor: { root: previous.root, resultSha256: previous.resultSha256, sealSha256: previous.sealSha256 },
      permissionSupersession: { failedRoot: resolve(parent, "channel-001"), closurePath } };
    await base.put(resolve(failedRoot, "context.json"), JSON.stringify({ context: failedContext, sha256: sha256(JSON.stringify(failedContext)) }));
    await base.put(resolve(failedRoot, "final-result.json"), JSON.stringify({ synthetic: "unverified Channel002" }));
    const inputs = await Promise.all(["context.json", "final-result.json"].map(async path => {
      const data = await readFile(resolve(failedRoot, path)); return { path, sha256: sha256(data), length: data.length };
    }));
    await base.put(resolve(failedRoot, "sealed-inventory.json"), JSON.stringify({ files: inputs }));
    const resultSha256 = sha256(await readFile(resolve(failedRoot, "final-result.json"))), sealSha256 = sha256(await readFile(resolve(failedRoot, "sealed-inventory.json")));
    const checkerBytes = await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/context.mjs"));
    const checkerIdentity = { firmwareCommit: "a".repeat(40), sources: [{ path: "scripts/str005-v2-serial/context.mjs", sha256: sha256(checkerBytes), length: checkerBytes.length }] };
    const value = { schema: "str005-v2-channel-successor-readiness-v1", failedRoot, failedContextSha256: sha256(JSON.stringify(failedContext)),
      failedResultSha256: resultSha256, failedSealSha256: sealSha256, inspectedInputs: inputs, checkerIdentity,
      status: "unverified", classification: "ready_for_fresh_channel", hardwareQualified: false, historicalCleanupComplete: false,
      beforeSource: { firmware_commit: failedContext.firmware_commit, app_elf_sha256: failedContext.app_elf_sha256 },
      predecessor: failedContext.predecessor, ledger: structuredClone(ledger), original: structuredClone(original),
      synthetic: "readiness classifier test seam" };
    await base.put(receiptPath, JSON.stringify(value));
    const receiptSha256 = sha256(await readFile(receiptPath));
    await writeNew(resolve(parent, "channel-ordinal-2.json"), { schema: "str005-v2-serial-assignment-v1", root: failedRoot,
      scope: "channel", context_sha256: value.failedContextSha256 });
    operations.inspectChannelSuccessor = async path => {
      if (path !== receiptPath || sha256(await readFile(path)) !== receiptSha256) throw Object.assign(Error("changed readiness"), { code: "v2_cleanup_receipt_changed" });
      for (const input of inputs) if (sha256(await readFile(resolve(failedRoot, input.path))) !== input.sha256)
        throw Object.assign(Error("changed failed input"), { code: "v2_cleanup_receipt_changed" });
      if (sha256(await readFile(resolve(failedRoot, "sealed-inventory.json"))) !== sealSha256)
        throw Object.assign(Error("changed failed seal"), { code: "v2_cleanup_receipt_changed" });
      return { root: failedRoot, receiptPath, receiptSha256, context: failedContext, contextSha256: value.failedContextSha256, resultSha256, sealSha256,
        classification: "ready_for_fresh_channel", status: "unverified", hardwareQualified: false, historicalCleanupComplete: false,
        beforeSource: { firmware_commit: failedContext.firmware_commit, app_elf_sha256: failedContext.app_elf_sha256 },
        predecessor: failedContext.predecessor, ledger: structuredClone(ledger), original: structuredClone(original),
        checkerIdentity: structuredClone(checkerIdentity) };
    };
    operations.checkCurrentSuccessorOwnership = async () => {};
    maybeSuccessor = receiptPath;
  }
  if (beforeLegacySnapshot) {
    if (!legacy && !permission) throw Error("historical fixture hook requires historical schema");
    await beforeLegacySnapshot({ ...base, parent, operations, previous });
  }
  execFileSync("git", ["-C", firmwareRoot, "add", "."]);
  const ordinal = legacy ? 1 : permission ? 2 : 3;
  let predecessorReceipt = resolve(previousRoot, "final-result.json");
  if (scope === "share") {
    const channelOptions = { ...base.options, scope: "channel", privateRoot: resolve(parent, `channel-${String(ordinal).padStart(3, "0")}`), predecessorReceipt,
      ...(permission ? { supersedePermission: closurePath } : { supersedeChannel: maybeSuccessor }) };
    if (permission) await historicalPreparation(channelOptions, operations, previous, "str005-v2-serial-context-v2");
    else await preflight(channelOptions, operations);
    const channel = await loadContext(channelOptions.privateRoot, { operations, historical: permission });
    await base.put(resolve(channelOptions.privateRoot, "final-result.json"), JSON.stringify({ synthetic: "accepted Channel prerequisite" }));
    const contextBytes = await readFile(resolve(channelOptions.privateRoot, "context.json"));
    await base.put(resolve(channelOptions.privateRoot, "sealed-inventory.json"), JSON.stringify({ files: [{ path: "context.json", sha256: sha256(contextBytes), length: contextBytes.length }] }));
    const channelPrevious = { ...previous, root: channelOptions.privateRoot, context: channel,
      resultSha256: sha256(await readFile(resolve(channelOptions.privateRoot, "final-result.json"))),
      sealSha256: sha256(await readFile(resolve(channelOptions.privateRoot, "sealed-inventory.json"))) };
    operations.inspectPredecessor = async (_path, requestedScope) => requestedScope === "share" ? channelPrevious : previous;
    predecessorReceipt = resolve(channelOptions.privateRoot, "final-result.json");
  }
  const options = { ...base.options, scope, privateRoot: resolve(parent, `${scope}-${String(scope === "channel" ? ordinal : 1).padStart(3, "0")}`), predecessorReceipt,
    ...(scope === "channel" ? (permission ? { supersedePermission: closurePath } : !legacy ? { supersedeChannel: maybeSuccessor } : {}) : {}) };
  delete options.attemptOrdinal;
  if (prepare) {
    if (legacy || permission) await historicalPreparation(options, operations,
      await operations.inspectPredecessor(predecessorReceipt, scope), legacy ? "str005-v2-serial-context-v1" : "str005-v2-serial-context-v2");
    else await preflight(options, operations);
  }
  return { ...base, parent, root: options.privateRoot, options, operations, previous,
    context: prepare ? await loadContext(options.privateRoot, { operations, historical: legacy || permission }) : null };
}

/** Fixture-only assembly of historical schemas. The live CLI cannot select this path. */
async function historicalPreparation(options, operations, predecessor, schema) {
  const native = await nativeInterface(operations), source = await inspectSources(options, native, operations);
  delete source.contracts.cleanup;
  if (schema === "str005-v2-serial-context-v1") delete source.contracts.permission;
  source.contractSha256 = sha256(canonical(source.contracts));
  const maybeClosure = schema === "str005-v2-serial-context-v2" && options.scope === "channel" ? await operations.inspectPermissionClosure(options.supersedePermission) : null;
  const context = { schema, ...source, scope: options.scope, attemptId: nonce(), hostOrdinal: options.scope === "channel" && schema.endsWith("v2") ? 2 : 1,
    predecessor: { root: predecessor.root, resultSha256: predecessor.resultSha256, sealSha256: predecessor.sealSha256 },
    before_source: { firmware_commit: predecessor.context.firmware_commit, app_elf_sha256: predecessor.context.app_elf_sha256 },
    original_campaign_id: predecessor.context.original_campaign_id, install_indices: options.scope === "channel" ? [0, 1, 2, 3, 4] : [1, 2, 3, 4],
    ...(schema.endsWith("v2") ? { permissionSupersession: maybeClosure ? { failedRoot: maybeClosure.root, failedContextSha256: maybeClosure.contextSha256,
      closurePath: maybeClosure.closurePath, closureSha256: maybeClosure.closureSha256 } : null } : {}) };
  if (options.scope === "share") {
    context.qualificationAttempt = { schema: "worker-qualification-attempt-v1", id: context.attemptId, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 };
    context.expectedLedgerBefore = predecessor.ledger;
  }
  context.native_readiness = await native.inspect({ firmwareRoot: context.firmware_root, manifestPath: context.manifest,
    expectedSourceCommit: context.firmware_commit, expectedElfSha256: context.app_elf_sha256 });
  const hash = sha256(JSON.stringify(context));
  const marker = { schema: "str005-v2-serial-assignment-v1", root: options.privateRoot, scope: options.scope, context_sha256: hash };
  await writeNew(resolve(dirname(options.privateRoot), `${options.scope}-ordinal-${context.hostOrdinal}.json`), marker);
  if (options.scope === "share") await writeNew(resolve(dirname(options.privateRoot), "qualification-ordinal-18.json"), marker);
  await mkdir(options.privateRoot, { mode: 0o700 });
  await writeNew(resolve(options.privateRoot, "context.json"), { context, sha256: hash });
  const permissionCorrection = schema.endsWith("v2") ? await checkPermissionCorrection(context, operations) : undefined;
  await createSnapshot(options.privateRoot, context, { permissionCorrection });
}
export function legacyContextFixture(t, options = {}) { return contextFixture(t, { ...options, scope: "channel", legacy: true }); }
export function permissionContextFixture(t, options = {}) { return contextFixture(t, { ...options, permission: true }); }
