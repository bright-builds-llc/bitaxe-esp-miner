import { packageShape, nativeShape, sourceShape, hex } from "./shapes.mjs";
import { inspectWriter, validateWriter } from "./native-writer.mjs";
import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE, PAGE, nonce, missing, ignored } from "../fixed-usb-qualification/contract.mjs";
import { canonical, inventory, privateRoot, proof, protectedPath, retain, verifyInventory, writeNew } from "../str005-noise-serial/files.mjs";
import { requireIdleLedger, requireExhaustedOriginal } from "../fixed-usb-qualification/iterative-contract.mjs";
import { requireOperatorStopped } from "../str005-v2-serial/operator-evidence.mjs";
import { requireHostStopped } from "../str005-v2-serial/cleanup.mjs";
import { inspectSources, verifyCurrent, validateTrust } from "./sources.mjs";
import { BEFORE, CONTRACT, TASK, PREDECESSOR, check, object, schema, sha256 } from "./values.mjs";

const KEYS = ["schema", "taskId", "contract", "attempt", "firmwareRoot", "gateRoot", "package", "beforeSource", "gate", "trustSha256", "predecessor", "sourceInventory", "nativeReadiness", "hostTools", "originalCampaign", "expectedAccounting", "installIndices", "miningAuthorized"];
export const contextHash = context => sha256(JSON.stringify(context));
export const effectRoot = context => resolve(context.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-001");
export function validate(context) {
  object(context, KEYS); packageShape(context.package); sourceShape(context.sourceInventory); nativeShape(context.nativeReadiness, context);
  hex(context.trustSha256); object(context.attempt, ["id", "ordinal"]); object(context.predecessor, ["root", ...Object.keys(PREDECESSOR)]);
  check(context.schema === schema("context") && context.taskId === TASK && canonical(context.contract) === canonical(CONTRACT) &&
    context.attempt.ordinal === 1 && /^[A-Za-z0-9_-]{22}$/u.test(context.attempt.id) && canonical(context.beforeSource) === canonical(BEFORE) &&
    canonical(context.installIndices) === "[0]" && context.miningAuthorized === false, "bootstrap_context_policy");
  check(context.firmwareRoot === resolve(context.firmwareRoot) && context.gateRoot === resolve(context.gateRoot) && context.package.manifest === resolve(context.firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"), "bootstrap_context_paths");
  object(context.gate, ["commit", "bundleSha256", "pageSha256", "pageRelativePath"]);
  hex(context.gate.commit, 40); hex(context.gate.bundleSha256); hex(context.gate.pageSha256);
  check(context.gate.pageRelativePath === PAGE, "bootstrap_gate_page");
  object(context.originalCampaign, ["id", "record"]); object(context.originalCampaign.record, ["path", "sha256", "length"]);
  check(/^[A-Za-z0-9_-]{22}$/u.test(context.originalCampaign.id) && context.originalCampaign.record.path === "accounting-before-install.json", "bootstrap_original_campaign");
  hex(context.originalCampaign.record.sha256);
  object(context.expectedAccounting, ["ledger", "original"]); requireIdleLedger(context.expectedAccounting.ledger, 18, 1560000); requireExhaustedOriginal(context.expectedAccounting.original);
  check(context.predecessor.root === resolve(context.firmwareRoot, "scratch/str005-v2-serial/channel-005") &&
    Object.entries(PREDECESSOR).every(([key, value]) => context.predecessor[key] === value), "bootstrap_predecessor_anchor");
  object(context.hostTools, ["espflash", "managedEsptool", "node"]);
  for (const tool of Object.values(context.hostTools)) { object(tool, ["path", "version", "sha256"]); check(tool.path === resolve(tool.path) && /^[a-f0-9]{64}$/u.test(tool.sha256), "bootstrap_tool"); }
  for (const file of context.sourceInventory) { object(file, ["path", "sha256", "length"]); check(!file.path.startsWith("/") && !file.path.split("/").includes("..") && /^[a-f0-9]{64}$/u.test(file.sha256), "bootstrap_source_inventory"); }
}
export async function predecessor(root, operations = {}, current = false) {
  root = await privateRoot(root);
  const [stored, result, seal] = await Promise.all([proof(root, "context.json"), proof(root, "final-result.json"), proof(root, "sealed-inventory.json")]);
  check(stored.value.sha256 === PREDECESSOR.contextSha256 && contextHash(stored.value.context) === PREDECESSOR.contextSha256 &&
    result.sha256 === PREDECESSOR.resultSha256 && seal.sha256 === PREDECESSOR.sealSha256, "bootstrap_predecessor_anchor");
  const reviewed = await (operations.reviewPredecessor ?? (await import("../str005-v2-serial/finalize.mjs")).review)(root);
  check(reviewed.status === "unverified" && reviewed.hardware_qualified === false && reviewed.result_sha256 === result.sha256 && reviewed.sealed_inventory_sha256 === seal.sha256,
    "bootstrap_predecessor_verdict");
  if (current) { await requireOperatorStopped(root, stored.value.context, operations); await requireHostStopped(root, stored.value.context, operations); }
  const accounting = await proof(root, "accounting-before-install.json");
  requireIdleLedger(accounting.value.ledger, 18, 1560000); requireExhaustedOriginal(accounting.value.original_budget);
  return { context: stored.value.context, binding: { root, ...PREDECESSOR }, originalCampaign: { id: stored.value.context.original_campaign_id,
    record: { path: "accounting-before-install.json", sha256: accounting.sha256, length: accounting.bytes.length } },
    expectedAccounting: { ledger: accounting.value.ledger, original: accounting.value.original_budget } };
}
async function snapshot(root, context, writer) {
  await retain(resolve(root, "snapshot/manifest.json"), await readFile(context.package.manifest));
  await writeNew(resolve(root, "snapshot/native-writer.json"), writer);
  const manifest = JSON.parse(await readFile(context.package.manifest));
  for (const file of manifest.artifacts) await retain(resolve(root, "snapshot/package", file.kind), await readFile(resolve(file.kind === "partition_table" ? context.firmwareRoot : dirname(context.package.manifest), file.path)));
  for (const [name, path] of [["page", context.gate.pageRelativePath], ["bundle", BUNDLE]]) await retain(resolve(root, "snapshot/gate", name), await readFile(resolve(context.gateRoot, path)));
  await retain(resolve(root, "snapshot/trust.json"), await readFile(resolve(context.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json")));
  await writeNew(resolve(root, "snapshot/native.json"), context.nativeReadiness);
  for (const file of context.sourceInventory) await retain(resolve(root, "snapshot/source", file.path), await readFile(resolve(context.firmwareRoot, file.path)));
  await writeNew(resolve(root, "preflight-inventory.json"), { schema: schema("preflight-inventory"), files: await inventory(root) });
}
export async function preflight(options, operations = {}) {
  const root = resolve(options.privateRoot), parent = await privateRoot(dirname(root)); await missing(root);
  const source = await inspectSources(options, operations);
  check(root === resolve(source.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-001"), "bootstrap_namespace");
  (operations.ignored ?? ignored)(source.firmwareRoot, root);
  check(options.predecessorReceipt === resolve(source.firmwareRoot, "scratch/str005-v2-serial/channel-005/final-result.json"), "bootstrap_predecessor_path");
  const prior = await (operations.predecessor ?? predecessor)(dirname(options.predecessorReceipt), operations, true);
  const context = { schema: schema("context"), taskId: TASK, contract: CONTRACT, attempt: { id: nonce(), ordinal: 1 }, ...source,
    beforeSource: BEFORE, predecessor: prior.binding, originalCampaign: prior.originalCampaign, expectedAccounting: prior.expectedAccounting, installIndices: [0], miningAuthorized: false };
  validate(context); await verifyCurrent(context, operations);
  const writer = validateWriter(await inspectWriter(context, operations), context);
  await writeNew(resolve(parent, "attempt-ordinal-1.json"), { schema: schema("assignment"), root, contextSha256: contextHash(context), attemptId: context.attempt.id });
  await operations.beforeCreate?.(); await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: contextHash(context) }); await snapshot(root, context, writer);
  return { ready: true, contextSha256: contextHash(context), mining_authorized: false, hardware_qualified: false };
}
export async function load(root, { historical = false, operations = {}, ancestry = false } = {}) {
  root = await privateRoot(root); const stored = (await proof(root, "context.json")).value; object(stored, ["context", "sha256"]);
  const context = stored.context; validate(context); check(root === effectRoot(context) && stored.sha256 === contextHash(context), "bootstrap_context_changed");
  const marker = (await proof(dirname(root), "attempt-ordinal-1.json")).value;
  check(canonical(marker) === canonical({ schema: schema("assignment"), root, contextSha256: stored.sha256, attemptId: context.attempt.id }), "bootstrap_assignment_changed");
  const frozen = (await proof(root, "preflight-inventory.json")).value;
  check(frozen.schema === schema("preflight-inventory"), "bootstrap_snapshot");
  for (const file of frozen.files) {
    check(typeof file.path === "string" && !file.path.startsWith("/") && !file.path.split("/").includes(".."), "bootstrap_snapshot_path");
    await protectedPath(resolve(root, file.path));
    const data = await readFile(resolve(root, file.path)); check(data.length === file.length && sha256(data) === file.sha256, "bootstrap_snapshot_changed");
  }
  const expectedSnapshot = frozen.files.filter(file => file.path.startsWith("snapshot/")).map(file => ({ ...file, path: file.path.slice("snapshot/".length) }));
  await verifyInventory(resolve(root, "snapshot"), expectedSnapshot);
  for (const file of context.sourceInventory) {
    const data = await readFile(resolve(root, "snapshot/source", file.path));
    check(data.length === file.length && sha256(data) === file.sha256, "bootstrap_source_snapshot_changed");
  }
  check((await proof(root, "snapshot/manifest.json")).sha256 === context.package.manifest_sha256 &&
    sha256(JSON.stringify((await proof(root, "snapshot/trust.json")).value)) === context.trustSha256 &&
    canonical((await proof(root, "snapshot/native.json")).value) === canonical(context.nativeReadiness), "bootstrap_snapshot_binding");
  for (const file of context.package.artifacts) check(sha256(await readFile(resolve(root, "snapshot/package", file.kind))) === file.sha256, "bootstrap_package_snapshot");
  validateTrust((await proof(root, "snapshot/trust.json")).value);
  validateWriter((await proof(root, "snapshot/native-writer.json")).value, context);
  if (ancestry) {
    const prior = await (operations.predecessor ?? predecessor)(context.predecessor.root, operations, !historical);
    check(canonical(prior.originalCampaign) === canonical(context.originalCampaign) && canonical(prior.expectedAccounting) === canonical(context.expectedAccounting), "bootstrap_predecessor_accounting_changed");
  }
  if (!historical) { await missing(resolve(root, "final-result.json")); await missing(resolve(root, "sealed-inventory.json")); await verifyCurrent(context, operations); }
  return context;
}
/** Shared low-level validators receive a fixed view, never a different authority context. */
export function legacyView(context, phase = "candidate") {
  return { ...context.package, ...(phase === "before" ? context.beforeSource : {}), firmware_root: context.firmwareRoot,
    gate_root: context.gateRoot, gate_commit: context.gate.commit, original_campaign_id: context.originalCampaign.id, scope: "channel", install_indices: [0] };
}
export const loadOperatorContext = (root, operations = {}) => load(root, { operations });
export async function loadEffectContext(root, operations = {}) {
  const context = await load(root, { operations }); await (await import("./install.mjs")).requireSupervisor(root, context, operations); return context;
}
export const verifyCleanupInputs = (context, operations = {}) => verifyCurrent(context, operations);
