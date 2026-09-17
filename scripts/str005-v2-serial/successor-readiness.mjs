import { link, unlink } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { inspectFailedChannel, recheckFailedEvidence } from "./successor-evidence.mjs";
import { collectCurrentOwnership, validateOwnershipObservation } from "./successor-ownership.mjs";
import { inspectFailedInstallChannel } from "./install-successor-evidence.mjs";
import { collectCurrentInstallOwnership, validateInstallOwnershipObservation } from "./install-successor-ownership.mjs";
import { createCheckerIdentity, inspectCheckerIdentity } from "./successor-sources.mjs";
import { CLEANUP_AMENDMENT_SHA256, INSTALL_REVIEW_AMENDMENT_SHA256, check, object } from "./values.mjs";

const NONCLAIMS = ["historical-pool-port-absence", "historical-cleanup-complete", "accepted-channel", "mining-acceptance", "cycle-or-baseline-transfer", "effect-authority"];
const INSTALL_NONCLAIMS = ["post-install-baseline", "post-install-accounting", "historical-cleanup-complete", "accepted-channel", "mining-acceptance", "cycle-or-baseline-transfer", "effect-authority"];
const inspectedCapabilities = new WeakMap();
export const successorReceiptPath = root => `${root}.successor-readiness.json`;
function failureClass(root) {
  check(["channel-002", "channel-003"].includes(basename(root)), "v2_successor_failed_root");
  const installed = basename(root) === "channel-003";
  return { installed, inspect: installed ? inspectFailedInstallChannel : inspectFailedChannel,
    collect: installed ? collectCurrentInstallOwnership : collectCurrentOwnership,
    validate: installed ? validateInstallOwnershipObservation : validateOwnershipObservation,
    schema: installed ? "str005-v2-channel-successor-readiness-v2" : "str005-v2-channel-successor-readiness-v1",
    amendment: installed ? INSTALL_REVIEW_AMENDMENT_SHA256 : CLEANUP_AMENDMENT_SHA256,
    nonClaims: installed ? INSTALL_NONCLAIMS : NONCLAIMS };
}
function accountingFacts(observed, installed) {
  return installed ? { afterBaselineObserved: false, initialAccounting: observed.initialAccounting } :
    { ledger: observed.ledger, original: observed.original };
}
function result(observed, path, sha256, checkerIdentity) {
  const value = structuredClone({ root: observed.root, context: observed.context, contextSha256: observed.contextSha256, resultSha256: observed.resultSha256, sealSha256: observed.sealSha256,
    receiptPath: path, receiptSha256: sha256, classification: "ready_for_fresh_channel", status: "unverified", hardwareQualified: false, historicalCleanupComplete: false,
    beforeSource: observed.beforeSource, predecessor: observed.predecessor,
    ...accountingFacts(observed, failureClass(observed.root).installed), checkerIdentity });
  inspectedCapabilities.set(value, { observed: structuredClone(observed), path, sha256 });
  return Object.freeze(value);
}

/** Fresh readiness is an observation, not an effect permit or a repaired historical verdict. */
export async function prepareChannelSuccessor(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_successor_failed_root");
  await privateRoot(dirname(root)); const path = successorReceiptPath(root), pending = `${path}.pending`;
  await missing(path); await missing(pending);
  const kind = failureClass(root), observed = await kind.inspect(root, operations), checkerIdentity = await createCheckerIdentity(observed.context, operations);
  const currentOwnership = await kind.collect(observed.ownership, operations);
  await recheckFailedEvidence(observed);
  const value = { schema: kind.schema, amendmentSha256: kind.amendment,
    failedRoot: root, failedContextSha256: observed.contextSha256, failedResultSha256: observed.resultSha256, failedSealSha256: observed.sealSha256,
    status: "unverified", classification: "ready_for_fresh_channel", hardwareQualified: false, historicalCleanupComplete: false,
    beforeSource: observed.beforeSource, predecessor: observed.predecessor, ...accountingFacts(observed, kind.installed),
    inspectedInputs: observed.inspectedInputs, currentOwnership, checkerIdentity, nonClaims: kind.nonClaims };
  await writeNew(pending, value); await (operations.beforeReadinessPublish ?? (() => {}))();
  await link(pending, path); await unlink(pending);
  const stored = await proof(dirname(path), path);
  check(canonical(stored.value) === canonical(value), "v2_successor_receipt_drift");
  await recheckFailedEvidence(observed);
  return result(observed, path, stored.sha256, checkerIdentity);
}

/** No process, listener, serial or device inspection occurs during historical receipt review. */
export async function inspectChannelSuccessor(path, operations = {}) {
  check(typeof path === "string" && path === resolve(path) && path.endsWith(".successor-readiness.json"), "v2_successor_receipt_path");
  const root = path.slice(0, -".successor-readiness.json".length);
  await privateRoot(dirname(root)); await missing(`${path}.pending`);
  const stored = await proof(dirname(path), path), value = stored.value, kind = failureClass(root);
  object(value, ["schema", "amendmentSha256", "failedRoot", "failedContextSha256", "failedResultSha256", "failedSealSha256", "status", "classification",
    "hardwareQualified", "historicalCleanupComplete", "beforeSource", "predecessor",
    ...(kind.installed ? ["afterBaselineObserved", "initialAccounting"] : ["ledger", "original"]),
    "inspectedInputs", "currentOwnership", "checkerIdentity", "nonClaims"]);
  const observed = await kind.inspect(root, operations);
  check(value.schema === kind.schema && value.amendmentSha256 === kind.amendment && value.failedRoot === root &&
    value.failedContextSha256 === observed.contextSha256 && value.failedResultSha256 === observed.resultSha256 && value.failedSealSha256 === observed.sealSha256 &&
    value.status === "unverified" && value.classification === "ready_for_fresh_channel" && value.hardwareQualified === false && value.historicalCleanupComplete === false &&
    canonical(value.beforeSource) === canonical(observed.beforeSource) && canonical(value.predecessor) === canonical(observed.predecessor) &&
    Object.entries(accountingFacts(observed, kind.installed)).every(([key, expected]) => canonical(value[key]) === canonical(expected)) &&
    canonical(value.inspectedInputs) === canonical(observed.inspectedInputs) && canonical(value.nonClaims) === canonical(kind.nonClaims), "v2_successor_receipt_binding");
  kind.validate(value.currentOwnership, observed.ownership);
  await inspectCheckerIdentity(value.checkerIdentity, observed.context, operations);
  await recheckFailedEvidence(observed);
  check((await proof(dirname(path), path)).sha256 === stored.sha256, "v2_successor_receipt_drift");
  return result(observed, path, stored.sha256, value.checkerIdentity);
}
export async function reviewChannelSuccessor(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_successor_failed_root");
  return inspectChannelSuccessor(successorReceiptPath(root), operations);
}

/** Recollect only current ownership at preassignment/serve; do not retrieve the old private pool port. */
export async function checkCurrentSuccessorOwnership(inspected, operations = {}) {
  const capability = inspectedCapabilities.get(inspected);
  check(capability, "v2_successor_inspection_required");
  await recheckFailedEvidence(capability.observed);
  await missing(`${capability.path}.pending`);
  check((await proof(dirname(capability.path), capability.path)).sha256 === capability.sha256, "v2_successor_receipt_drift");
  return failureClass(capability.observed.root).collect(capability.observed.ownership, operations);
}
