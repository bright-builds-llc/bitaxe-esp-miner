import { inspectRestoredPredecessor, recheckRestoredPredecessor } from "./restored-predecessor.mjs";
import { inspectMeasurementPredecessor, recheckMeasurement } from "./measurement-predecessor.mjs";
import { runReaderCheck } from "./reader-correction-check.mjs";
import { writeSnapshot, verifySnapshot } from "./snapshot.mjs";
import { validateBinding, recheckBinding } from "./preflight-binding.mjs";
import { packageShape, nativeShape, sourceShape, hex } from "./shapes.mjs";
import { inspectWriter, validateWriter } from "./native-writer.mjs";
import { mkdir, mkdtemp, readFile, rename, rmdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { PAGE, nonce, missing, ignored } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireIdleLedger, requireExhaustedOriginal } from "../fixed-usb-qualification/iterative-contract.mjs";
import { requireOperatorStopped } from "../str005-v2-serial/operator-evidence.mjs";
import { requireHostStopped } from "../str005-v2-serial/cleanup.mjs";
import { inspectSources, verifyCurrent } from "./sources.mjs";
import { CONTEXT_V4, ACCOUNTING_AMENDMENT, BEFORE_V4, MEASUREMENT_003, BEFORE, CONTRACT, TASK, PREDECESSOR, CONTEXT_V1, CONTEXT_V2, CONTEXT_V3, CORRECTION_CONTRACT, BEFORE_V3, MEASUREMENT_002, PREFLIGHT_AMENDMENT, check, object, schema, sha256 } from "./values.mjs";

const KEYS = ["schema", "taskId", "contract", "attempt", "firmwareRoot", "gateRoot", "package", "beforeSource", "gate", "trustSha256", "predecessor", "sourceInventory", "nativeReadiness", "hostTools", "originalCampaign", "expectedAccounting", "installIndices", "miningAuthorized"];
export const contextHash = context => sha256(JSON.stringify(context));
export const effectRoot = context => resolve(context.firmwareRoot, `scratch/usb-bootstrap-measure/attempt-${String(context.attempt.ordinal).padStart(3, "0")}`);
export function validate(context) {
  object(context, [...KEYS, ...(context?.schema === CONTEXT_V2 ? ["preflightAmendment", "preflightSupersession"] : context?.schema === CONTEXT_V4 ? ["correctionAmendment", "accountingAmendment"] : context?.schema === CONTEXT_V3 ? ["correctionAmendment"] : [])]); packageShape(context.package); sourceShape(context.sourceInventory); nativeShape(context.nativeReadiness, context);
  hex(context.trustSha256); object(context.attempt, ["id", "ordinal"]); object(context.predecessor, ["root", ...Object.keys(PREDECESSOR)]);
  check([CONTEXT_V1, CONTEXT_V2, CONTEXT_V3, CONTEXT_V4].includes(context.schema) && context.taskId === TASK && canonical(context.contract) === canonical(CONTRACT) &&
    context.attempt.ordinal === (context.schema === CONTEXT_V4 ? 4 : context.schema === CONTEXT_V3 ? 3 : context.schema === CONTEXT_V2 ? 2 : 1) && /^[A-Za-z0-9_-]{22}$/u.test(context.attempt.id) && canonical(context.beforeSource) === canonical(context.schema === CONTEXT_V4 ? BEFORE_V4 : context.schema === CONTEXT_V3 ? BEFORE_V3 : BEFORE) &&
    canonical(context.installIndices) === "[0]" && context.miningAuthorized === false, "bootstrap_context_policy");
  if (context.schema === CONTEXT_V2) validateBinding(context);
  if ([CONTEXT_V3, CONTEXT_V4].includes(context.schema)) check(canonical(context.correctionAmendment) === canonical(CORRECTION_CONTRACT), "bootstrap_correction_contract");
  if (context.schema === CONTEXT_V4) check(canonical(context.accountingAmendment) === canonical(ACCOUNTING_AMENDMENT), "bootstrap_accounting_contract");
  check(context.firmwareRoot === resolve(context.firmwareRoot) && context.gateRoot === resolve(context.gateRoot) && context.package.manifest === resolve(context.firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"), "bootstrap_context_paths");
  object(context.gate, ["commit", "bundleSha256", "pageSha256", "pageRelativePath"]);
  hex(context.gate.commit, 40); hex(context.gate.bundleSha256); hex(context.gate.pageSha256);
  check(context.gate.pageRelativePath === PAGE, "bootstrap_gate_page");
  object(context.originalCampaign, ["id", "record"]); object(context.originalCampaign.record, ["path", "sha256", "length"]);
  check(/^[A-Za-z0-9_-]{22}$/u.test(context.originalCampaign.id) && context.originalCampaign.record.path === (context.schema === CONTEXT_V4 ? "accounting-before.json" : context.schema === CONTEXT_V3 ? "accounting-after.json" : "accounting-before-install.json"), "bootstrap_original_campaign");
  hex(context.originalCampaign.record.sha256);
  object(context.expectedAccounting, ["ledger", "original"]); requireIdleLedger(context.expectedAccounting.ledger, 18, 1560000); requireExhaustedOriginal(context.expectedAccounting.original);
  check(context.predecessor.root === resolve(context.firmwareRoot, context.schema === CONTEXT_V4 ? "scratch/usb-bootstrap-measure/attempt-003" : context.schema === CONTEXT_V3 ? "scratch/usb-bootstrap-measure/attempt-002" : "scratch/str005-v2-serial/channel-005") &&
    Object.entries(context.schema === CONTEXT_V4 ? Object.fromEntries(Object.entries(MEASUREMENT_003).filter(([key]) => key !== "failureSha256")) : context.schema === CONTEXT_V3 ? MEASUREMENT_002 : PREDECESSOR).every(([key, value]) => context.predecessor[key] === value), "bootstrap_predecessor_anchor");
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
/** Validate prospective bindings without assignment, regression execution or device effects. */
export async function prepareContext(options, operations = {}) {
  const root = resolve(options.privateRoot);
  const source = await inspectSources(options, operations);
  check(root === resolve(source.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-004"), "bootstrap_namespace");
  (operations.ignored ?? ignored)(source.firmwareRoot, root);
  check(options.predecessorReceipt === resolve(source.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-003/final-result.json"), "bootstrap_predecessor_path");
  const prior = await (operations.inspectRestoredPredecessor ?? inspectRestoredPredecessor)(dirname(options.predecessorReceipt), operations);
  const context = { schema: CONTEXT_V4, taskId: TASK, contract: CONTRACT, attempt: { id: nonce(), ordinal: 4 }, ...source,
    correctionAmendment: CORRECTION_CONTRACT, accountingAmendment: ACCOUNTING_AMENDMENT, beforeSource: prior.beforeSource, predecessor: prior.binding, originalCampaign: prior.originalCampaign,
    expectedAccounting: prior.expectedAccounting, installIndices: [0], miningAuthorized: false };
  validate(context); await recheckRestoredPredecessor(context, operations); await verifyCurrent(context, operations);
  return context;
}
export async function preflight(options, operations = {}) {
  const root = resolve(options.privateRoot), parent = await privateRoot(dirname(root)); await missing(root);
  check(options.supersedePreflight === undefined, "bootstrap_correction_supersession_forbidden");
  const context = await prepareContext(options, operations);
  const writer = validateWriter(await inspectWriter(context, operations), context), correction = await runReaderCheck(context, operations);
  await missing(resolve(parent, "attempt-ordinal-4.json"));
  const stage = await mkdtemp(resolve(parent, "attempt-004.preparation-")); await privateRoot(stage);
  await writeNew(resolve(stage, "context.json"), { context, sha256: contextHash(context) });
  await writeSnapshot(stage, context, writer, correction);
  await operations.beforeSnapshotReview?.(stage);
  await verifySnapshot(stage, context); await recheckRestoredPredecessor(context, operations); await verifyCurrent(context, operations);
  await writeNew(resolve(parent, "attempt-ordinal-4.json"), { schema: schema("assignment"), root, contextSha256: contextHash(context), attemptId: context.attempt.id });
  await operations.beforeCreate?.(); await mkdir(root, { mode: 0o700 });
  // Exclusive final-directory creation prevents replacement. The last rename is
  // the admission-completion boundary; a partial publication cannot load.
  await rename(resolve(stage, "snapshot"), resolve(root, "snapshot"));
  await rename(resolve(stage, "context.json"), resolve(root, "context.json"));
  await operations.beforeInventoryPublish?.(stage, root);
  await rename(resolve(stage, "preflight-inventory.json"), resolve(root, "preflight-inventory.json")); await rmdir(stage);
  return { ready: true, contextSha256: contextHash(context), mining_authorized: false, hardware_qualified: false };
}
export async function load(root, { historical = false, operations = {}, ancestry = false } = {}) {
  root = await privateRoot(root); const stored = (await proof(root, "context.json")).value; object(stored, ["context", "sha256"]);
  const context = stored.context; validate(context);
  if (!historical) check(context.schema === CONTEXT_V4, "bootstrap_prior_context_read_only");
  check(root === effectRoot(context) && stored.sha256 === contextHash(context), "bootstrap_context_changed");
  const marker = (await proof(dirname(root), `attempt-ordinal-${context.attempt.ordinal}.json`)).value;
  check(canonical(marker) === canonical({ schema: schema("assignment"), root, contextSha256: stored.sha256, attemptId: context.attempt.id }), "bootstrap_assignment_changed");
  await verifySnapshot(root, context);
  if (context.schema === CONTEXT_V2) { await recheckBinding(context, operations); await verifyPredecessorPins(context, operations); }
  if (context.schema === CONTEXT_V3) await recheckMeasurement(context, operations);
  if (context.schema === CONTEXT_V4) await recheckRestoredPredecessor(context, operations);
  if (ancestry && context.schema === CONTEXT_V1) {
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

/** Prior deep judgment remains bound by immutable exact root pins, not replayed inside hot deadlines. */
export async function verifyPredecessorPins(context, operations = {}) {
  if (operations.verifyPredecessorPins) return operations.verifyPredecessorPins(context);
  const root = context.predecessor.root;
  const [stored, result, seal, accounting] = await Promise.all([proof(root, "context.json"), proof(root, "final-result.json"), proof(root, "sealed-inventory.json"), proof(root, "accounting-before-install.json")]);
  check(stored.value.sha256 === PREDECESSOR.contextSha256 && contextHash(stored.value.context) === PREDECESSOR.contextSha256 && result.sha256 === PREDECESSOR.resultSha256 &&
    seal.sha256 === PREDECESSOR.sealSha256 && stored.value.context.original_campaign_id === context.originalCampaign.id && accounting.sha256 === context.originalCampaign.record.sha256 &&
    accounting.bytes.length === context.originalCampaign.record.length && canonical({ ledger: accounting.value.ledger, original: accounting.value.original_budget }) === canonical(context.expectedAccounting), "bootstrap_predecessor_changed");
}
