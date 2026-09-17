import { link, mkdir, open, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, inventory, proof, verifyInventory, writeNew } from "../str005-noise-serial/files.mjs";
import { loadContext, recheckNative } from "./context.mjs";
import { requireHostStopped } from "./cleanup.mjs";
import { collectInputs, snapshotCleanup } from "./inputs.mjs";
import { firstFailure, failureOutcome, judgmentCode } from "./disposition.mjs";
import { judge, inspectRestoredCleanup } from "./judge.mjs";
import { projection } from "./projection.mjs";
import { check, object, sha256 } from "./values.mjs";

const SEAL_EXCLUSIONS = new Set(["sealed-inventory.json", "projection.json"]);
const summary = (result, resultSha256, sealSha256) => ({ status: result.status, outcome: result.outcome,
  hardware_qualified: result.status === "passed", scope: result.scope, result_sha256: resultSha256, sealed_inventory_sha256: sealSha256 });

async function classify(root, context, cleanupPath, operations) {
  try { return { maybeAccepted: await judge(root, context, cleanupPath, operations), maybeCode: null }; }
  catch (error) { return { maybeAccepted: null, maybeCode: judgmentCode(error) }; }
}
async function publish(context, value) {
  const path = resolve(context.firmware_root, "docs/parity/evidence/str005-v2-serial", `${value.scope}-${String(value.hostOrdinal).padStart(3, "0")}.json`);
  await mkdir(dirname(path), { recursive: true });
  const pending = `${path}.pending`, handle = await open(pending, "wx", 0o644);
  try { await handle.writeFile(`${JSON.stringify(value, null, 2)}\n`); await handle.sync(); }
  finally { await handle.close(); }
  // Atomic creation does not replace previously published evidence.
  await link(pending, path); await unlink(pending);
}
async function publishEligible(root, context, cleanupPath, maybeAccepted, operations) {
  if (context.scope !== "share") return null;
  try { await inspectRestoredCleanup(root, context, cleanupPath, operations); }
  catch (error) { return { publishedScopes: [], blockedBy: judgmentCode(error) }; }
  const predecessor = await review(context.predecessor.root, operations);
  check(predecessor.status === "passed" && predecessor.scope === "channel" &&
    predecessor.result_sha256 === context.predecessor.resultSha256 && predecessor.sealed_inventory_sha256 === context.predecessor.sealSha256,
  "v2_publication_predecessor");
  await publish(context, (await proof(context.predecessor.root, "projection.json")).value);
  if (maybeAccepted) await publish(context, (await proof(root, "projection.json")).value);
  return { publishedScopes: maybeAccepted ? ["channel", "share"] : ["channel"], blockedBy: null };
}

/** Seal once, after actual host writers stop; Channel acceptance remains entirely private. */
export async function finalize(root, cleanupPath, operations = {}) {
  const context = await loadContext(root, { historical: true, operations });
  await requireHostStopped(root, context, operations);
  for (const name of ["final-result.json", "sealed-inventory.json", "projection.json", "judgment-failure.json", "native/final-readiness.json", "final-inputs"])
    await missing(resolve(root, name));
  const initial = await inventory(root);
  let maybeNativeCode = null, maybeNative;
  try { maybeNative = await recheckNative(context, operations); }
  catch (error) { maybeNativeCode = judgmentCode(error); }
  await verifyInventory(root, initial);
  if (maybeNative) await writeNew(resolve(root, "native/final-readiness.json"), maybeNative);
  await snapshotCleanup(root, cleanupPath);
  const options = { ...operations, cleanupSnapshot: true };
  const stable = await inventory(root);
  const classified = maybeNativeCode ? { maybeAccepted: null, maybeCode: maybeNativeCode } : await classify(root, context, cleanupPath, options);
  const { maybeAccepted, maybeCode } = classified;
  const maybeFailure = maybeAccepted ? null : await firstFailure(root, context, maybeCode);
  const inputs = await collectInputs(root, { cleanupSnapshot: true });
  await requireHostStopped(root, context, operations);
  await verifyInventory(root, stable);
  if (!maybeAccepted) await writeNew(resolve(root, "judgment-failure.json"), { schema: "str005-v2-judgment-failure-v1", code: maybeCode, nativeCode: maybeNativeCode });
  const result = { schema: "str005-v2-serial-result-v1", contextSha256: sha256(JSON.stringify(context)),
    status: maybeAccepted ? "passed" : "unverified", outcome: maybeAccepted ? "complete" : failureOutcome(maybeFailure, maybeCode), firstFailure: maybeFailure, inputs, scope: context.scope };
  await writeNew(resolve(root, "final-result.json"), result);
  await writeNew(resolve(root, "sealed-inventory.json"), { schema: "str005-v2-serial-seal-v1", contextSha256: result.contextSha256,
    files: await inventory(root, SEAL_EXCLUSIONS) });
  const resultSha256 = (await proof(root, "final-result.json")).sha256, sealSha256 = (await proof(root, "sealed-inventory.json")).sha256;
  if (maybeAccepted) await writeNew(resolve(root, "projection.json"), projection(context, maybeAccepted, sealSha256, resultSha256));
  const maybePublication = await publishEligible(root, context, cleanupPath, maybeAccepted, options);
  return { ...summary(result, resultSha256, sealSha256), ...(maybePublication ? { publication: maybePublication } : {}) };
}

/** Read-only historical validation never appends missing evidence or upgrades a failed result. */
export async function review(root, operations = {}) {
  const context = await loadContext(root, { historical: true, operations });
  const resultProof = await proof(root, "final-result.json"), sealProof = await proof(root, "sealed-inventory.json");
  const result = resultProof.value, seal = sealProof.value, hash = sha256(JSON.stringify(context));
  object(seal, ["schema", "contextSha256", "files"]);
  check(seal.schema === "str005-v2-serial-seal-v1" && seal.contextSha256 === hash, "v2_seal_context");
  await verifyInventory(root, seal.files, SEAL_EXCLUSIONS);
  object(result, ["schema", "contextSha256", "status", "outcome", "firstFailure", "inputs", "scope"]);
  check(result.schema === "str005-v2-serial-result-v1" && result.contextSha256 === hash && result.scope === context.scope &&
    ["passed", "unverified"].includes(result.status), "v2_result_shape");
  check(canonical(result.inputs) === canonical(await collectInputs(root, { cleanupSnapshot: true })), "v2_result_input_drift");
  const options = { ...operations, checkKernel: false, cleanupSnapshot: true }, cleanupPath = `${root}.cleanup/receipt.json`;
  if (result.status === "passed") {
    check(result.firstFailure === null && result.outcome === "complete", "v2_pass_disposition");
    await missing(resolve(root, "judgment-failure.json"));
    const accepted = await judge(root, context, cleanupPath, options);
    check(canonical((await proof(root, "projection.json")).value) === canonical(projection(context, accepted, sealProof.sha256, resultProof.sha256)),
      "v2_projection_changed");
  } else {
    await missing(resolve(root, "projection.json"));
    const judgment = (await proof(root, "judgment-failure.json")).value;
    object(judgment, ["schema", "code", "nativeCode"]);
    check(judgment.schema === "str005-v2-judgment-failure-v1" && judgment.code === judgmentCode(judgment), "v2_failure_schema");
    const original = await firstFailure(root, context, judgment.code);
    check(canonical(original) === canonical(result.firstFailure) && result.outcome === failureOutcome(original, judgment.code), "v2_failure_disposition");
    if (judgment.nativeCode === null) {
      const checked = await classify(root, context, cleanupPath, options);
      check(checked.maybeAccepted === null && checked.maybeCode === judgment.code, "v2_failed_result_reclassified");
    } else {
      check(judgment.nativeCode === judgment.code, "v2_native_failure_binding");
      await missing(resolve(root, "native/final-readiness.json"));
    }
  }
  await verifyInventory(root, seal.files, SEAL_EXCLUSIONS);
  check((await proof(root, "sealed-inventory.json")).sha256 === sealProof.sha256 &&
    (await proof(root, "final-result.json")).sha256 === resultProof.sha256, "v2_result_changed_during_review");
  return summary(result, resultProof.sha256, sealProof.sha256);
}
