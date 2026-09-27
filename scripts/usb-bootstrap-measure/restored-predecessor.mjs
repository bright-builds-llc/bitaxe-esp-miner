import { resolve } from "node:path";
import { canonical, privateRoot, proof } from "../str005-noise-serial/files.mjs";
import { BEFORE_V4, MEASUREMENT_003, CONTEXT_V3, check, sha256 } from "./values.mjs";
import { maybeProof, validateCleanup, validateResourceCleanup, requireHostStopped, operatorAbsent } from "./cleanup.mjs";
import { readJournal, baseline } from "./journal.mjs";
import { judgeOperator } from "./operator-disposition.mjs";
export const restoredBinding = root => ({ root, ...Object.fromEntries(Object.entries(MEASUREMENT_003).filter(([key]) => key !== "failureSha256")) });
/** The exact sealed negative result provides continuity expectations, never a fabricated after-ledger. */
export function validateRestoredFacts({ root, context, result, accounting, failure, last, rows }) {
  check(context.schema === CONTEXT_V3 && root === resolve(context.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-003") &&
    context.package.firmware_commit === BEFORE_V4.firmware_commit && context.package.app_elf_sha256 === BEFORE_V4.app_elf_sha256, "bootstrap_restored_source");
  check(result.status === "unverified" && result.correction.accepted === false && result.cleanup.complete === false && result.restoration.confirmed === false &&
    result.capture.qualified === true && result.capture.exitCode === 0 && result.hardware_qualified === false && result.mining_authorized === false && result.qualification_credit === "none", "bootstrap_restored_result");
  check(["captureQualified", "hostTimingComplete", "readerHeadroom", "nativeBootstrapComplete", "zeroTxFailures"].every(key => result.correction.checks[key] === true) &&
    result.correction.checks.preservationAndAccounting === false && result.correction.checks.cleanupComplete === false, "bootstrap_restored_result");
  check(canonical(result.firstFailure) === canonical({ source: "browser", stage: "browser", code: "bootstrap_client_failed", observationSha256: null }) &&
    failure.code === "bootstrap_client_failed" && failure.source === "browser" && failure.stage === "browser" && failure.observationSha256 === null, "bootstrap_restored_failure");
  check(accounting.stage === "before" && accounting.contextSha256 === MEASUREMENT_003.contextSha256 &&
    canonical({ ledger: accounting.ledger, original: accounting.original }) === canonical(context.expectedAccounting), "bootstrap_restored_accounting");
  const row = rows[accounting.observedSequence - 1];
  check(row?.phase === "before" && canonical(row.state) === canonical(accounting.state), "bootstrap_restored_accounting"); baseline(accounting.state);
  baseline(last.state, true); check(last.state.deviceRestorationConfirmed === true, "bootstrap_restored_baseline");
  const states = rows.filter(row => row.phase === "candidate").map(row => row.state.status);
  let index = -1; for (const status of ["ready", "stopping", "baseline_confirmed", "closed"]) { index = states.indexOf(status, index + 1); check(index >= 0, "bootstrap_restored_order"); }
  for (const row of rows) {
    check(row.state.renewalsConfirmed === 0 && !row.state.running && !row.state.heartbeatSuppressed && !["window_loaded", "running"].includes(row.state.status), "bootstrap_restored_effect");
    for (const key of ["budget_reserved_ms", "submitted", "accepted", "rejected", "work_dispatched", "nonce_work_correlations"])
      check(row.state.qualification?.[key] === undefined || row.state.qualification[key] === accounting.state.qualification?.[key], "bootstrap_restored_effect");
  }
}
export async function restoredPins(root) {
  root = await privateRoot(root);
  const [stored, result, seal, accounting, failure] = await Promise.all(["context.json", "final-result.json", "sealed-inventory.json", "accounting-before.json", "failure.json"].map(name => proof(root, name)));
  check(stored.value.sha256 === MEASUREMENT_003.contextSha256 && sha256(JSON.stringify(stored.value.context)) === MEASUREMENT_003.contextSha256 && result.sha256 === MEASUREMENT_003.resultSha256 &&
    seal.sha256 === MEASUREMENT_003.sealSha256 && failure.sha256 === MEASUREMENT_003.failureSha256, "bootstrap_restored_anchor");
  for (const [path, item] of [["accounting-before.json", accounting], ["failure.json", failure]]) {
    const matching = seal.value.files.filter(row => row.path === path); check(matching.length === 1 && matching[0].sha256 === item.sha256 && matching[0].length === item.bytes.length, "bootstrap_restored_inventory");
  }
  check(await maybeProof(root, "accounting-after.json") === null && await maybeProof(root, "restoration.json") === null, "bootstrap_restored_missing_boundary");
  const context = stored.value.context, rows = await readJournal(root, context);
  validateRestoredFacts({ root, context, result: result.value, accounting: accounting.value, failure: failure.value, rows, last: rows.at(-1) });
  return { context, binding: restoredBinding(root), beforeSource: BEFORE_V4, originalCampaign: { id: context.originalCampaign.id,
    record: { path: "accounting-before.json", sha256: accounting.sha256, length: accounting.bytes.length } }, expectedAccounting: { ledger: accounting.value.ledger, original: accounting.value.original } };
}
export async function inspectRestoredPredecessor(root, operations = {}) {
  const prior = await restoredPins(root), reviewed = await (await import("./finalize.mjs")).review(root, operations);
  check(reviewed.status === "unverified" && reviewed.correction_accepted === false && reviewed.resultSha256 === MEASUREMENT_003.resultSha256 && reviewed.sealSha256 === MEASUREMENT_003.sealSha256, "bootstrap_restored_review");
  const cleanup = (await proof(root, "cleanup.json")).value;
  await validateCleanup(root, prior.context, cleanup, resolve(root, "final-inputs/operator"));
  await validateResourceCleanup(root, prior.context, cleanup, resolve(root, "final-inputs/operator"));
  await judgeOperator(root, prior.context, resolve(root, "final-inputs/operator"));
  await requireHostStopped(root, prior.context, operations); await operatorAbsent(root, prior.context, operations); return prior;
}
export async function recheckRestoredPredecessor(context, operations = {}) {
  const prior = await (operations.restoredPins ?? restoredPins)(context.predecessor.root);
  check(["binding", "beforeSource", "originalCampaign", "expectedAccounting"].every(key => canonical(prior[key]) === canonical(context[key === "binding" ? "predecessor" : key])) &&
    context.package.firmware_commit !== prior.context.package.firmware_commit && context.package.manifest_sha256 !== prior.context.package.manifest_sha256 && canonical(context.sourceInventory) !== canonical(prior.context.sourceInventory), "bootstrap_restored_pair");
  return prior;
}
