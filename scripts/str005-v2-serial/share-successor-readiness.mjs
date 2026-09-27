import { link, unlink } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { recheckFailedEvidence } from "./successor-evidence.mjs";
import { inspectFailedShare } from "./share-successor-evidence.mjs";


import { collectCurrentShareOwnership, validateShareOwnershipObservation } from "./share-successor-ownership.mjs";
import { createCheckerIdentity, inspectCheckerIdentity } from "./share-successor-sources.mjs";
import { OPERATOR_AMENDMENT_SHA256, check, object } from "./values.mjs";

const NONCLAIMS = ["historical-parent-exit", "historical-supervisor-exit", "post-install-baseline", "post-install-accounting", "historical-cleanup-complete", "mining-acceptance", "cycle-or-baseline-transfer", "effect-authority"];
const inspectedCapabilities = new WeakMap();
export const shareSuccessorReceiptPath = root => `${root}.share-successor-readiness.json`;
function failureClass(root) {
  check(basename(root) === "share-001", "v2_share_successor_root");
  return { inspect: inspectFailedShare, collect: collectCurrentShareOwnership, validate: validateShareOwnershipObservation,
    schema: "str005-v2-share-successor-readiness-v1", amendment: OPERATOR_AMENDMENT_SHA256, nonClaims: NONCLAIMS };
}
function accountingFacts(observed) { return { afterBaselineObserved: false, initialAccounting: observed.initialAccounting, ordinalMarker: observed.ordinalMarker }; }
function result(observed, path, sha256, checkerIdentity) {
  const value = structuredClone({ root: observed.root, context: observed.context, contextSha256: observed.contextSha256, resultSha256: observed.resultSha256, sealSha256: observed.sealSha256,
    receiptPath: path, receiptSha256: sha256, classification: "ready_for_fresh_channel", status: "unverified", hardwareQualified: false, historicalCleanupComplete: false,
    beforeSource: observed.beforeSource, predecessor: observed.predecessor,
    ...accountingFacts(observed), checkerIdentity });
  inspectedCapabilities.set(value, { observed: structuredClone(observed), path, sha256 });
  return Object.freeze(value);
}

/** Fresh readiness is an observation, not an effect permit or a repaired historical verdict. */
export async function prepareShareSuccessor(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_successor_failed_root");
  await privateRoot(dirname(root)); const path = shareSuccessorReceiptPath(root), pending = `${path}.pending`;
  await missing(path); await missing(pending);
  const kind = failureClass(root), observed = await kind.inspect(root, operations), checkerIdentity = await createCheckerIdentity(observed.context, operations);
  const currentOwnership = await kind.collect(observed.ownership, operations);
  await recheckFailedEvidence(observed);
  const value = { schema: kind.schema, amendmentSha256: kind.amendment,
    failedRoot: root, failedContextSha256: observed.contextSha256, failedResultSha256: observed.resultSha256, failedSealSha256: observed.sealSha256,
    status: "unverified", classification: "ready_for_fresh_channel", hardwareQualified: false, historicalCleanupComplete: false,
    beforeSource: observed.beforeSource, predecessor: observed.predecessor, ...accountingFacts(observed),
    inspectedInputs: observed.inspectedInputs, currentOwnership, checkerIdentity, nonClaims: kind.nonClaims };
  await writeNew(pending, value); await (operations.beforeReadinessPublish ?? (() => {}))();
  await link(pending, path); await unlink(pending);
  const stored = await proof(dirname(path), path);
  check(canonical(stored.value) === canonical(value), "v2_successor_receipt_drift");
  await recheckFailedEvidence(observed);
  return result(observed, path, stored.sha256, checkerIdentity);
}

/** No process, listener, serial or device inspection occurs during historical receipt review. */
export async function inspectShareSuccessor(path, operations = {}) {
  check(typeof path === "string" && path === resolve(path) && path.endsWith(".share-successor-readiness.json"), "v2_successor_receipt_path");
  const root = path.slice(0, -".share-successor-readiness.json".length);
  await privateRoot(dirname(root)); await missing(`${path}.pending`);
  const stored = await proof(dirname(path), path), value = stored.value, kind = failureClass(root);
  object(value, ["schema", "amendmentSha256", "failedRoot", "failedContextSha256", "failedResultSha256", "failedSealSha256", "status", "classification",
    "hardwareQualified", "historicalCleanupComplete", "beforeSource", "predecessor",
    "afterBaselineObserved", "initialAccounting", "ordinalMarker",
    "inspectedInputs", "currentOwnership", "checkerIdentity", "nonClaims"]);
  const observed = await kind.inspect(root, operations);
  check(value.schema === kind.schema && value.amendmentSha256 === kind.amendment && value.failedRoot === root &&
    value.failedContextSha256 === observed.contextSha256 && value.failedResultSha256 === observed.resultSha256 && value.failedSealSha256 === observed.sealSha256 &&
    value.status === "unverified" && value.classification === "ready_for_fresh_channel" && value.hardwareQualified === false && value.historicalCleanupComplete === false &&
    canonical(value.beforeSource) === canonical(observed.beforeSource) && canonical(value.predecessor) === canonical(observed.predecessor) &&
    Object.entries(accountingFacts(observed)).every(([key, expected]) => canonical(value[key]) === canonical(expected)) &&
    canonical(value.inspectedInputs) === canonical(observed.inspectedInputs) && canonical(value.nonClaims) === canonical(kind.nonClaims), "v2_successor_receipt_binding");
  kind.validate(value.currentOwnership, observed.ownership);
  await inspectCheckerIdentity(value.checkerIdentity, observed.context, operations);
  await recheckFailedEvidence(observed);
  check((await proof(dirname(path), path)).sha256 === stored.sha256, "v2_successor_receipt_drift");
  return result(observed, path, stored.sha256, value.checkerIdentity);
}
export async function reviewShareSuccessor(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_successor_failed_root");
  return inspectShareSuccessor(shareSuccessorReceiptPath(root), operations);
}

/** Recollect only current ownership at preassignment/serve; do not retrieve the old private pool port. */
export async function checkCurrentShareSuccessorOwnership(inspected, operations = {}) {
  const capability = inspectedCapabilities.get(inspected);
  check(capability, "v2_successor_inspection_required");
  await recheckFailedEvidence(capability.observed);
  await missing(`${capability.path}.pending`);
  check((await proof(dirname(capability.path), capability.path)).sha256 === capability.sha256, "v2_successor_receipt_drift");
  return failureClass(capability.observed.root).collect(capability.observed.ownership, operations);
}
