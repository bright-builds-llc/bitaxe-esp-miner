import { basename, resolve } from "node:path";
import { canonical } from "../str005-noise-serial/files.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { check, digest, object, sha256, uint } from "./values.mjs";

export function installSupersessionMetadata(value) {
  object(value, ["failedRoot", "failedContextSha256", "failedResultSha256", "failedSealSha256", "receiptPath", "receiptSha256"]);
  for (const key of ["failedContextSha256", "failedResultSha256", "failedSealSha256", "receiptSha256"]) digest(value[key]);
  check(typeof value.failedRoot === "string" && value.failedRoot === resolve(value.failedRoot) && basename(value.failedRoot) === "channel-003" &&
    value.receiptPath === `${value.failedRoot}.successor-readiness.json`, "v2_install_successor_path");
}
/** Explicitly historical pre-write accounting; it cannot supply the fresh successor baseline. */
export function initialSuccessorAccounting(value) {
  object(value, ["path", "sha256", "observedSequence", "ledger", "original"]);
  check(value.path === "accounting-before-install.json" && uint(value.observedSequence) > 0, "v2_install_successor_accounting");
  digest(value.sha256); requireIdleLedger(value.ledger, 18, 1560000); requireExhaustedOriginal(value.original);
}
export function installSupersessionBinding(context, inspected) {
  const expectedRoot = resolve(context.firmware_root, "scratch/str005-v2-serial/channel-003");
  check(inspected.status === "unverified" && inspected.classification === "ready_for_fresh_channel" && inspected.hardwareQualified === false &&
    inspected.historicalCleanupComplete === false && inspected.afterBaselineObserved === false &&
    !Object.hasOwn(inspected, "ledger") && !Object.hasOwn(inspected, "original") &&
    inspected.root === expectedRoot && inspected.receiptPath === `${expectedRoot}.successor-readiness.json` &&
    inspected.context.schema === "str005-v2-serial-context-v3" && inspected.context.scope === "channel" && inspected.context.hostOrdinal === 3 &&
    inspected.contextSha256 === sha256(JSON.stringify(inspected.context)) && canonical(inspected.predecessor) === canonical(context.predecessor) &&
    canonical(inspected.context.predecessor) === canonical(context.predecessor) &&
    inspected.context.original_campaign_id === context.original_campaign_id, "v2_install_successor_predecessor");
  object(inspected.beforeSource, ["firmware_commit", "app_elf_sha256"]);
  check(canonical(context.before_source) === canonical(inspected.beforeSource) && inspected.beforeSource.firmware_commit === inspected.context.firmware_commit &&
    inspected.beforeSource.app_elf_sha256 === inspected.context.app_elf_sha256, "v2_install_successor_before_source");
  initialSuccessorAccounting(inspected.initialAccounting);
  object(inspected.checkerIdentity, ["firmwareCommit", "sources"]);
  check(inspected.checkerIdentity.firmwareCommit === context.firmware_commit && Array.isArray(inspected.checkerIdentity.sources) &&
    inspected.checkerIdentity.sources.length > 0 && new Set(inspected.checkerIdentity.sources.map(row => row.path)).size === inspected.checkerIdentity.sources.length &&
    inspected.checkerIdentity.sources.every(row => context.evaluator.some(entry => canonical(entry) === canonical(row))), "v2_cleanup_checker_changed");
  check(context.firmware_commit !== inspected.context.firmware_commit && context.manifest_sha256 !== inspected.context.manifest_sha256 &&
    canonical(context.evaluator) !== canonical(inspected.context.evaluator), "v2_cleanup_correction_unchanged");
  const binding = { failedRoot: inspected.root, failedContextSha256: inspected.contextSha256, failedResultSha256: inspected.resultSha256,
    failedSealSha256: inspected.sealSha256, receiptPath: inspected.receiptPath, receiptSha256: inspected.receiptSha256 };
  installSupersessionMetadata(binding); check(canonical(binding) === canonical(context.cleanupSupersession), "v2_cleanup_receipt_changed");
}
