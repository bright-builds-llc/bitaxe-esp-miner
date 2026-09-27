import { requirePredecessorBinding } from "./predecessor.mjs";
import { link, unlink } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { canonical, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { check, digest, object, sha256 } from "./values.mjs";
import { initialSuccessorAccounting } from "./context-install-binding.mjs";
import { missing } from "../fixed-usb-qualification/contract.mjs";

export function shareMetadata(value) {
  object(value, ["failedRoot", "failedContextSha256", "failedResultSha256", "failedSealSha256", "receiptPath", "receiptSha256"]);
  for (const key of ["failedContextSha256", "failedResultSha256", "failedSealSha256", "receiptSha256"]) digest(value[key]);
  check(value.failedRoot === resolve(value.failedRoot) && basename(value.failedRoot) === "share-001" &&
    value.receiptPath === `${value.failedRoot}.share-successor-readiness.json`, "v2_share_successor_path");
}
export const shareBinding = observed => ({ failedRoot: observed.root, failedContextSha256: observed.contextSha256,
  failedResultSha256: observed.resultSha256, failedSealSha256: observed.sealSha256, receiptPath: observed.receiptPath, receiptSha256: observed.receiptSha256 });
export async function reviewedShare(path, operations) {
  return (operations.inspectShareSuccessor ?? (await import("./share-successor-readiness.mjs")).inspectShareSuccessor)(path, operations);
}
export async function currentShareOwnership(inspected, operations) {
  return (operations.checkCurrentShareSuccessorOwnership ?? (await import("./share-successor-readiness.mjs")).checkCurrentShareSuccessorOwnership)(inspected, operations);
}
export function requireShareBinding(context, inspected) {
  shareMetadata(context.shareSupersession);
  check(inspected.root === resolve(context.firmware_root, "scratch/str005-v2-serial/share-001") &&
    canonical(context.shareSupersession) === canonical(shareBinding(inspected)) && inspected.status === "unverified" &&
    inspected.classification === "ready_for_fresh_channel" && inspected.hardwareQualified === false && inspected.historicalCleanupComplete === false &&
    inspected.afterBaselineObserved === false && inspected.context.schema === "str005-v2-serial-context-v4" && inspected.context.scope === "share" &&
    inspected.context.hostOrdinal === 1 && inspected.contextSha256 === sha256(JSON.stringify(inspected.context)) &&
    inspected.context.original_campaign_id === context.original_campaign_id, "v2_share_successor_binding");
  check(!Object.hasOwn(inspected, "ledger") && !Object.hasOwn(inspected, "original"), "v2_share_successor_accounting");
  initialSuccessorAccounting(inspected.initialAccounting);
  check(canonical(inspected.beforeSource) === canonical({ firmware_commit: inspected.context.firmware_commit, app_elf_sha256: inspected.context.app_elf_sha256 }) &&
    canonical(inspected.predecessor) === canonical(inspected.context.predecessor), "v2_share_successor_binding");
  requireIdleLedger(inspected.initialAccounting.ledger, 18, 1560000); requireExhaustedOriginal(inspected.initialAccounting.original);
  if (context.scope === "channel" && context.schema !== "str005-v2-serial-context-v6") check(canonical(context.before_source) === canonical(inspected.beforeSource), "v2_share_successor_before_source");
  check((context.schema === "str005-v2-serial-context-v6" || inspected.checkerIdentity.firmwareCommit === context.firmware_commit && inspected.checkerIdentity.sources.length > 0 &&
    inspected.checkerIdentity.sources.every(row => context.evaluator.some(entry => canonical(row) === canonical(entry)))) &&
    context.firmware_commit !== inspected.context.firmware_commit && context.manifest_sha256 !== inspected.context.manifest_sha256 &&
    canonical(context.evaluator) !== canonical(inspected.context.evaluator), "v2_share_successor_correction");
}
export async function verifySharePins(context) {
  if (context.scope === "share") {
    const previous = context.predecessor, [stored, result, seal] = await Promise.all([proof(previous.root, "context.json"),
      proof(previous.root, "final-result.json"), proof(previous.root, "sealed-inventory.json")]);
    const prior = stored.value.context;
    check(prior.schema === context.schema && prior.scope === "channel" && prior.hostOrdinal === (context.schema === "str005-v2-serial-context-v6" ? 6 : 5) &&
      stored.value.sha256 === sha256(JSON.stringify(prior)) && result.sha256 === previous.resultSha256 && seal.sha256 === previous.sealSha256 &&
      seal.value.files?.some(row => row.path === "context.json" && row.sha256 === stored.sha256 && row.length === stored.bytes.length) &&
      canonical(prior.shareSupersession) === canonical(context.shareSupersession) && (context.schema !== "str005-v2-serial-context-v6" || canonical(prior.bootstrapCorrection) === canonical(context.bootstrapCorrection)), "v2_channel_successor_changed");
    requirePredecessorBinding(context, { ...previous, context: prior });
  }
  const pin = context.shareSupersession; shareMetadata(pin); await missing(`${pin.receiptPath}.pending`);
  const [stored, failed, result, seal] = await Promise.all([proof(dirname(pin.receiptPath), pin.receiptPath), proof(pin.failedRoot, "context.json"),
    proof(pin.failedRoot, "final-result.json"), proof(pin.failedRoot, "sealed-inventory.json")]);
  const value = stored.value;
  check(stored.sha256 === pin.receiptSha256 && value.schema === "str005-v2-share-successor-readiness-v1" &&
    failed.value.sha256 === pin.failedContextSha256 && sha256(JSON.stringify(failed.value.context)) === pin.failedContextSha256 &&
    result.sha256 === pin.failedResultSha256 && seal.sha256 === pin.failedSealSha256 && value.failedRoot === pin.failedRoot &&
    value.failedContextSha256 === pin.failedContextSha256 && value.failedResultSha256 === pin.failedResultSha256 && value.failedSealSha256 === pin.failedSealSha256,
    "v2_share_successor_pin_changed");
  for (const [path, file] of [["context.json", failed], ["final-result.json", result]]) {
    const rows = value.inspectedInputs?.filter(entry => entry.path === path);
    check(rows?.length === 1 && rows[0].sha256 === file.sha256 && rows[0].length === file.bytes.length, "v2_share_successor_pin_changed");
  }
  const accounting = await proof(pin.failedRoot, "accounting-before-install.json");
  const row = value.inspectedInputs?.filter(entry => entry.path === "accounting-before-install.json");
  check(row?.length === 1 && row[0].sha256 === accounting.sha256 && row[0].length === accounting.bytes.length &&
    accounting.sha256 === value.initialAccounting?.sha256 && accounting.value.observedSequence === value.initialAccounting.observedSequence &&
    canonical(accounting.value.ledger) === canonical(value.initialAccounting.ledger) &&
    canonical(accounting.value.original_budget) === canonical(value.initialAccounting.original), "v2_share_successor_pin_changed");
  const ordinal = await proof(dirname(pin.failedRoot), "qualification-ordinal-18.json");
  check(canonical(value.ordinalMarker) === canonical({ path: "qualification-ordinal-18.json", sha256: ordinal.sha256, length: ordinal.bytes.length }),
    "v2_share_successor_ordinal_changed");
  requireShareBinding(context, { ...value, root: pin.failedRoot, context: failed.value.context, contextSha256: pin.failedContextSha256,
    resultSha256: pin.failedResultSha256, sealSha256: pin.failedSealSha256, receiptPath: pin.receiptPath, receiptSha256: stored.sha256 });
  return value;
}
export const successorOrdinalPath = root => resolve(dirname(root), "qualification-ordinal-18.successor-2.json");
export async function ordinalSuccessor(context, root) {
  const receipt = await proof(dirname(context.shareSupersession.receiptPath), context.shareSupersession.receiptPath);
  const marker = await proof(dirname(root), "qualification-ordinal-18.json");
  const originalMarker = { path: "qualification-ordinal-18.json", sha256: marker.sha256, length: marker.bytes.length };
  check(canonical(originalMarker) === canonical(receipt.value.ordinalMarker) && marker.value.context_sha256 === context.shareSupersession.failedContextSha256 &&
    marker.value.root === context.shareSupersession.failedRoot && marker.value.scope === "share", "v2_share_successor_ordinal_changed");
  return { schema: "str005-v2-qualification-ordinal-successor-v1", ordinal: 18, originalMarker,
    failedContextSha256: context.shareSupersession.failedContextSha256, failedResultSha256: context.shareSupersession.failedResultSha256,
    failedSealSha256: context.shareSupersession.failedSealSha256, receiptSha256: context.shareSupersession.receiptSha256,
    contextSha256: sha256(JSON.stringify(context)), attemptId: context.attemptId };
}

export async function publishOrdinalSuccessor(context, root) {
  const path = successorOrdinalPath(root), pending = `${path}.pending`;
  await missing(path); await missing(pending); await writeNew(pending, await ordinalSuccessor(context, root));
  await link(pending, path); await unlink(pending);
}
