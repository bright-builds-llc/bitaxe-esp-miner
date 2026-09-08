import { constants } from "node:fs";
import { lstat, mkdir, open } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { authorityCall } from "./authority.mjs";
import { admitTrust, canonicalDirectory, cleanPushed, digest, exactObject, fileDigest, git, hex,
  missing, protectedPath, readJson, requireCondition, within, writeNew } from "./contract.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

const DRIVER_PROFILE = "fixed-usb-retained-runtime-driver-v1";
const ALLOWED = new Set(["TASKS.md", "docs/adr/0028-continue-unreserved-qualification.md",
  "docs/hardware/native-usb-ownership.md", "docs/project/project-decisions.md", "scripts/BUILD.bazel",
  ...["runtime-source.mjs", "runtime-source.test.mjs", "unreserved.mjs", "unreserved.test.mjs",
    "iterative-preflight.mjs", "iterative-server.mjs", "iterative.test.mjs", "preflight.mjs", "main.mjs", "server.mjs"]
    .map(name => `scripts/fixed-usb-qualification/${name}`)]);
const SNAPSHOT_FIELDS = ["manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments",
  "firmware_commit", "gate_commit", "gate_bundle_sha256", "gate_page_relative_path", "gate_page_sha256",
  "trust_sha256", "authority_trust_sha256", "supervisor_client_sha256"];
export const retainedManifestPath = root => resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json");

/** Admits only the published host driver; this never establishes device execution. */
export function checkRetainedDriverSource(context, driverCommit, operations = {}) {
  requireCondition(hex(context.firmware_commit, 40) && hex(context.gate_commit, 40) && hex(driverCommit, 40), "retained_source_commit");
  const check = operations.cleanPushed ?? cleanPushed, readGit = operations.git ?? git;
  check(context.firmware_root, driverCommit);
  check(context.gate_root, context.gate_commit);
  requireCondition(readGit(context.firmware_root, ["merge-base", context.firmware_commit, driverCommit]) === context.firmware_commit, "retained_source_ancestry");
  const changes = readGit(context.firmware_root, ["diff", "--name-status", "--no-renames", context.firmware_commit, driverCommit]);
  for (const line of changes.split("\n").filter(Boolean)) {
    const [status, path, extra] = line.split("\t");
    requireCondition(["A", "M"].includes(status) && extra === undefined && ALLOWED.has(path), "retained_source_forbidden");
    requireCondition(readGit(context.firmware_root, ["ls-tree", driverCommit, "--", path]).startsWith("100644 blob "), "retained_source_mode");
  }
}
async function sourceRecord(root) {
  await canonicalDirectory(root); await protectedPath(root, true);
  const path = resolve(root, "context.json"); await protectedPath(path);
  const record = await readJson(path); exactObject(record, ["context", "sha256"]);
  requireCondition(hex(record.sha256, 64) && record.sha256 === digest(JSON.stringify(record.context)), "retained_context_integrity");
  requireCondition(["fixed-usb-iterative-context-v3", "fixed-usb-iterative-context-v4"].includes(record.context.schema), "retained_context_profile");
  return record;
}
async function verifyPublicInputs(context, authorityDirectory, bun, operations) {
  await canonicalDirectory(context.firmware_root); await canonicalDirectory(context.gate_root);
  const trustPath = resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json");
  const clientPath = resolve(context.firmware_root, "scripts/fixed-usb-qualification/client.mjs");
  for (const path of [trustPath, clientPath]) {
    const stat = await lstat(path);
    requireCondition(stat.isFile() && !stat.isSymbolicLink(), "retained_public_input_type");
  }
  requireCondition(await fileDigest(trustPath) === context.trust_sha256 && await fileDigest(clientPath) === context.supervisor_client_sha256, "retained_public_input_drift");
  await protectedPath(authorityDirectory, true);
  const observed = await (operations.authorityCall ?? authorityCall)(context.gate_root, authorityDirectory, "public-trust", undefined, bun);
  admitTrust(await readJson(trustPath), observed);
  requireCondition(digest(JSON.stringify(observed)) === context.authority_trust_sha256, "retained_authority_drift");
}

/** Returns snapshot metadata separately from historical attempt/accounting context. */
export async function inspectRetainedSources(sourceRoot, options, operations = {}) {
  sourceRoot = resolve(sourceRoot);
  const record = await sourceRecord(sourceRoot), context = record.context;
  requireCondition(await canonicalDirectory(options.firmwareRoot) === context.firmware_root &&
    await canonicalDirectory(options.gateRoot) === context.gate_root && options.firmwareCommit === context.firmware_commit &&
    options.gateCommit === context.gate_commit && resolve(options.manifest) === retainedManifestPath(sourceRoot), "retained_identity_mismatch");
  checkRetainedDriverSource(context, options.qualificationSourceCommit, operations);
  const snapshotProof = await verifyArtifactSnapshot(sourceRoot, context);
  await verifyPublicInputs(context, options.authorityDirectory, options.bun, operations);
  const snapshot = {};
  for (const key of SNAPSHOT_FIELDS) {
    requireCondition(Object.hasOwn(context, key), "retained_snapshot_metadata");
    snapshot[key] = structuredClone(context[key]);
  }
  return { snapshot, sourceContext: context, sourceContextSha256: record.sha256, artifactSnapshotSha256: snapshotProof.receipt_sha256 };
}

/** Copies only verified runtime artifacts; the caller owns context, cycles and lineage. */
export async function copyRetainedArtifacts(sourceRoot, targetRoot, targetContext) {
  sourceRoot = await canonicalDirectory(sourceRoot); targetRoot = await canonicalDirectory(targetRoot);
  const relation = relative(sourceRoot, targetRoot);
  requireCondition(relation !== "" && (relation.startsWith("../") || relation === ".."), "retained_copy_overlap");
  await protectedPath(targetRoot, true);
  requireCondition(resolve(targetContext.manifest) === retainedManifestPath(targetRoot), "retained_manifest_path");
  const record = await sourceRecord(sourceRoot);
  const proof = await verifyArtifactSnapshot(sourceRoot, record.context);
  for (const key of SNAPSHOT_FIELDS) requireCondition(JSON.stringify(record.context[key]) === JSON.stringify(targetContext[key]), "retained_copy_identity");
  const receiptPath = resolve(sourceRoot, "artifact-snapshot.json"), receipt = await readJson(receiptPath);
  requireCondition(await fileDigest(receiptPath) === proof.receipt_sha256, "retained_snapshot_changed");
  const destination = resolve(targetRoot, "qualified-artifacts");
  await missing(destination); await missing(resolve(targetRoot, "artifact-snapshot.json"));
  await mkdir(destination, { mode: 0o700 });
  for (const entry of receipt.files) {
    const source = within(resolve(sourceRoot, "qualified-artifacts"), resolve(sourceRoot, "qualified-artifacts", entry.path));
    const target = within(destination, resolve(destination, entry.path));
    await mkdir(dirname(target), { recursive: true, mode: 0o700 });
    let parent = dirname(target);
    while (parent !== destination) { await protectedPath(parent, true); parent = dirname(parent); }
    const input = await open(source, constants.O_RDONLY | constants.O_NOFOLLOW);
    let bytes;
    try { const stat = await input.stat(); requireCondition(stat.isFile() && (stat.mode & 0o777) === 0o600, "retained_copy_source_type"); bytes = await input.readFile(); }
    finally { await input.close(); }
    requireCondition(bytes.length === entry.length && digest(bytes) === entry.sha256, "retained_copy_source_changed");
    const output = await open(target, "wx", 0o600);
    try { await output.writeFile(bytes); await output.sync(); } finally { await output.close(); }
  }
  await writeNew(resolve(targetRoot, "artifact-snapshot.json"), { schema: "fixed-usb-qualified-artifacts-v1", context_sha256: digest(JSON.stringify(targetContext)), files: receipt.files });
  await verifyArtifactSnapshot(targetRoot, targetContext);
  requireCondition((await sourceRecord(sourceRoot)).sha256 === record.sha256 && await fileDigest(receiptPath) === proof.receipt_sha256, "retained_source_changed");
}

/** Verifies the current host driver while serving only the retained exact runtime. */
export async function verifyRetainedRuntime(root, context, authorityDirectory, bun, operations = {}) {
  await canonicalDirectory(root); await protectedPath(root, true);
  exactObject(context.qualification_driver, ["profile", "source_commit"]);
  requireCondition(context.schema === "fixed-usb-iterative-context-v4" && context.qualification_driver.profile === DRIVER_PROFILE &&
    resolve(context.manifest) === retainedManifestPath(root), "retained_driver_profile");
  checkRetainedDriverSource(context, context.qualification_driver.source_commit, operations);
  const record = await sourceRecord(root);
  requireCondition(record.sha256 === digest(JSON.stringify(context)), "retained_context_changed");
  const snapshot = await verifyArtifactSnapshot(root, context);
  await verifyPublicInputs(context, authorityDirectory, bun, operations);
  return { gate_root: snapshot.gate_root };
}
