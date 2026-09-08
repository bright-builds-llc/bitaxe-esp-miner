import test from "node:test";
import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, realpath, rm, symlink, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { BUNDLE, PAGE, digest, inspectPackage, QualificationError, writeNew } from "./contract.mjs";
import { checkRetainedDriverSource, copyRetainedArtifacts, inspectRetainedSources, retainedManifestPath, verifyRetainedRuntime } from "./runtime-source.mjs";
const FW = "a".repeat(40), GATE = "b".repeat(40), DRIVER = "c".repeat(40);
function sourceOperations(changes = "M\tTASKS.md", mode = "100644", ancestor = FW) {
  return {
    cleanPushed(root, commit) { assert.equal(commit, root.endsWith("gate") ? GATE : DRIVER); },
    git(_root, args) {
      if (args[0] === "merge-base") return ancestor;
      if (args[0] === "diff") return changes;
      if (args[0] === "ls-tree") return `${mode} blob ${"d".repeat(40)}\t${args.at(-1)}`;
      throw new Error("unexpected git request");
    },
  };
}
function publicTrust() {
  const key = { kid: "fixture", kty: "OKP", crv: "Ed25519", x: Buffer.alloc(32, 1).toString("base64url"), alg: "Ed25519", use: "sig", key_ops: ["verify"] };
  return { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "fixture-update", audience: "fixture-update", keys: [key] }, workLeaseAuthority: { issuer: "fixture-lease", audience: "fixture-lease", keys: [key] } };
}
async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "retained-runtime-test-")));
  await chmod(base, 0o700); t.after(() => rm(base, { recursive: true, force: true }));
  const origin = resolve(base, "origin"), target = resolve(base, "target"), firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), authorityDirectory = resolve(base, "authority");
  for (const path of [origin, target, firmwareRoot, gateRoot, authorityDirectory]) await mkdir(path, { mode: 0o700 });
  const trust = publicTrust(), files = [], artifacts = [];
  const snapshot = resolve(origin, "qualified-artifacts"); await mkdir(snapshot, { mode: 0o700 });
  async function add(path, bytes) {
    const out = resolve(snapshot, path); await mkdir(dirname(out), { recursive: true, mode: 0o700 }); await writeFile(out, bytes, { mode: 0o600 });
    files.push({ path, sha256: digest(bytes), length: Buffer.byteLength(bytes) });
  }
  for (const kind of ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"]) {
    const path = `${kind}.bin`, bytes = Buffer.from(`synthetic-${kind}`);
    await add(`firmware/${path}`, bytes); artifacts.push({ kind, path, sha256: digest(bytes) });
  }
  const manifest = { schema_version: 4, build_identity: { source_dirty: false }, source_commit: FW, reference_commit: "d".repeat(40), app_elf_sha256: artifacts[0].sha256, artifacts,
    update_segments: [["bootloader", 0], ["partition_table_binary", 0x8000], ["firmware_ota_image", 0x10000], ["www_spiffs_image", 0x410000], ["otadata_initial", 0xf10000]].map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`synthetic-${artifact_kind}`) })) };
  await add("firmware/bitaxe-ultra205-package.json", JSON.stringify(manifest));
  await add("firmware/docs/release/license-inventory.md", "synthetic license"); await add("firmware/docs/release/provenance-manifest.md", "synthetic provenance");
  await add(`gate/${BUNDLE}`, `synthetic bundle ${GATE}`); await add(`gate/${PAGE}`, "synthetic page");
  const trustPath = resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), clientPath = resolve(firmwareRoot, "scripts/fixed-usb-qualification/client.mjs");
  for (const path of [trustPath, clientPath]) await mkdir(dirname(path), { recursive: true });
  await writeFile(trustPath, JSON.stringify(trust)); await writeFile(clientPath, "synthetic client");
  const packaged = await inspectPackage(retainedManifestPath(origin), FW, () => resolve(snapshot, "firmware"));
  const context = { ...packaged, schema: "fixed-usb-iterative-context-v3", firmware_commit: FW, gate_commit: GATE, firmware_root: firmwareRoot, gate_root: gateRoot,
    manifest: resolve(firmwareRoot, "bazel-bin/old-manifest.json"), gate_bundle_sha256: digest(`synthetic bundle ${GATE}`), gate_page_relative_path: PAGE, gate_page_sha256: digest("synthetic page"),
    trust_sha256: digest(JSON.stringify(trust)), authority_trust_sha256: digest(JSON.stringify(trust)), supervisor_client_sha256: digest("synthetic client"), qualification_attempt: { ordinal: 99 }, suggested_difficulty: 1000 };
  await writeNew(resolve(origin, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await writeNew(resolve(origin, "artifact-snapshot.json"), { schema: "fixed-usb-qualified-artifacts-v1", context_sha256: digest(JSON.stringify(context)), files });
  const operations = { ...sourceOperations(), authorityCall: async () => trust };
  const options = { firmwareRoot, gateRoot, firmwareCommit: FW, gateCommit: GATE, manifest: retainedManifestPath(origin), qualificationSourceCommit: DRIVER, authorityDirectory };
  const next = { ...context, schema: "fixed-usb-iterative-context-v4", manifest: retainedManifestPath(target), qualification_driver: { profile: "fixed-usb-retained-runtime-driver-v1", source_commit: DRIVER }, qualification_attempt: { ordinal: 99, id: "new" } };
  return { base, origin, target, context, next, files, operations, options, trustPath, clientPath };
}

test("host driver allowlist rejects runtime edits, deletions, renames and non-regular modes", () => {
  const context = { firmware_root: "/firmware", gate_root: "/gate", firmware_commit: FW, gate_commit: GATE };
  checkRetainedDriverSource(context, DRIVER, sourceOperations("M\tTASKS.md\nA\tscripts/fixed-usb-qualification/runtime-source.mjs"));
  for (const path of ["MODULE.bazel", "crates/core.rs", "firmware/bitaxe/src/main.rs", "scripts/fixed-usb-qualification/client.mjs", "scripts/fixed-usb-qualification/contract.mjs", "scripts/fixed-usb-qualification/judge.mjs"]) assert.throws(() => checkRetainedDriverSource(context, DRIVER, sourceOperations(`M\t${path}`)), /retained_source_forbidden/u);
  for (const status of ["D", "R100", "T"]) assert.throws(() => checkRetainedDriverSource(context, DRIVER, sourceOperations(`${status}\tTASKS.md`)), /retained_source_forbidden/u);
  for (const mode of ["100755", "120000", "160000"]) assert.throws(() => checkRetainedDriverSource(context, DRIVER, sourceOperations("M\tTASKS.md", mode)), /retained_source_mode/u);
});
test("ancestry, current publication and exact Gate identity are mandatory", () => {
  const context = { firmware_root: "/firmware", gate_root: "/gate", firmware_commit: FW, gate_commit: GATE };
  assert.throws(() => checkRetainedDriverSource(context, DRIVER, sourceOperations("", "100644", "e".repeat(40))), /retained_source_ancestry/u);
  for (const failRoot of ["/firmware", "/gate"]) assert.throws(() => checkRetainedDriverSource(context, DRIVER, { ...sourceOperations(), cleanPushed(root) { if (root === failRoot) throw new QualificationError("source_not_pushed"); } }), /source_not_pushed/u);
});
test("retained inspection and exclusive copying preserve exact runtime but not old accounting metadata", async t => {
  const f = await fixture(t), original = await readFile(resolve(f.origin, "context.json"));
  const inspected = await inspectRetainedSources(f.origin, f.options, f.operations);
  assert.equal(inspected.snapshot.qualification_attempt, undefined); assert.equal(inspected.snapshot.schema, undefined);
  await copyRetainedArtifacts(f.origin, f.target, f.next);
  await writeNew(resolve(f.target, "context.json"), { context: f.next, sha256: digest(JSON.stringify(f.next)) });
  assert.deepEqual(await verifyRetainedRuntime(f.target, f.next, f.options.authorityDirectory, undefined, f.operations), { gate_root: resolve(f.target, "qualified-artifacts/gate") });
  assert.deepEqual(await readFile(resolve(f.origin, "context.json")), original);
  const receipt = JSON.parse(await readFile(resolve(f.target, "artifact-snapshot.json")));
  assert.equal(receipt.files.length, 13); assert.equal(receipt.context_sha256, digest(JSON.stringify(f.next)));
  await assert.rejects(copyRetainedArtifacts(f.origin, f.target, f.next), /private_path_exists/u);
});
test("source/context, package and caller identity tampering fail before copying", async t => {
  const f = await fixture(t);
  await assert.rejects(inspectRetainedSources(f.origin, { ...f.options, firmwareCommit: DRIVER }, f.operations), /retained_identity_mismatch/u);
  await assert.rejects(inspectRetainedSources(f.origin, { ...f.options, manifest: f.context.manifest }, f.operations), /retained_identity_mismatch/u);
  await assert.rejects(copyRetainedArtifacts(f.origin, f.target, { ...f.next, app_elf_sha256: "f".repeat(64) }), /retained_copy_identity/u);
  await writeFile(resolve(f.origin, "context.json"), JSON.stringify({ context: { ...f.context, gate_commit: DRIVER }, sha256: digest(JSON.stringify(f.context)) }));
  await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /retained_context_integrity/u);
});
test("snapshot bytes, symlink artifacts and permissive modes are rejected", async t => {
  const f = await fixture(t), file = resolve(f.origin, "qualified-artifacts", f.files[0].path), original = await readFile(file);
  await writeFile(file, "changed"); await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /snapshot_file_integrity/u);
  await writeFile(file, original); await chmod(file, 0o644); await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /private_path_policy/u);
  await unlink(file); const outside = resolve(f.base, "outside"); await writeFile(outside, original, { mode: 0o600 }); await symlink(outside, file);
  await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /private_path_policy/u);
});
test("current client, deployment trust and authority trust cannot drift", async t => {
  const f = await fixture(t);
  await assert.rejects(inspectRetainedSources(f.origin, f.options, { ...f.operations, authorityCall: async () => ({ profile: "wrong" }) }), /trust_profile/u);
  await writeFile(f.clientPath, "changed"); await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /retained_public_input_drift/u);
  await writeFile(f.clientPath, "synthetic client"); await writeFile(f.trustPath, "{}"); await assert.rejects(inspectRetainedSources(f.origin, f.options, f.operations), /retained_public_input_drift/u);
});
