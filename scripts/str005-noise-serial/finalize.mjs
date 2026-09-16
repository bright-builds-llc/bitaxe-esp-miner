import { mkdir, readFile, open, link, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { exactObject, missing } from "../fixed-usb-qualification/contract.mjs";
import { loadContext, recheckNative } from "./context.mjs";
import { requireHostStopped } from "./cleanup.mjs";
import { collectInputs, failureOutcome } from "./inputs.mjs";
import { snapshotCleanup } from "./final-inputs.mjs";
import { judge } from "./judge.mjs";
import { check, digest, inventory, proof, verifyInventory, writeNew } from "./files.mjs";
import { cause } from "./contract-v2.mjs";
import { parseProjection } from "./evidence.mjs";

export function parseProjectionV2(value) {
  check(value.schema_version === "bitaxe-stratum-v2-noise-serial-projection-v2" && Number.isSafeInteger(value.timings_ms?.device_cleanup) &&
    value.timings_ms.device_cleanup >= 0 && value.timings_ms.device_cleanup <= 120000, "noise_projection_v2");
  parseProjection({ ...value, schema_version: "bitaxe-stratum-v2-noise-serial-projection-v1",
    timings_ms: { ...value.timings_ms, device_cleanup: Math.min(5000, value.timings_ms.device_cleanup) } });
  return structuredClone(value);
}
function projection(context, accepted, sealSha256, resultSha256) {
  return parseProjectionV2({ schema_version: "bitaxe-stratum-v2-noise-serial-projection-v2", status: "accepted", board: 205,
    diagnostic_ordinal: context.ordinal, source_commit: context.firmware_commit, gate_commit: context.gate_commit, reference_commit: context.reference_commit,
    provenance: { app_elf: context.app_elf_sha256, package_manifest: context.manifest_sha256, contract: context.contracts.sha256,
      fixture: context.fixture_sha256, evaluator: digest(JSON.stringify(context.evaluator)), sealed_inventory: sealSha256, private_result: resultSha256 },
    criteria: Object.fromEntries(["identity", "continuity", "authority", "tcp_delivery", "noise_authentication", "encrypted_proof", "no_new_work",
      "preservation", "accounting", "restoration", "cleanup", "privacy"].map((key) => [key, true])),
    timings_ms: { ...accepted.protocol.timings, ...accepted.hostTiming },
    counts: { exact_peer_connections: 1, unexpected_peer_connections: 0, act_one_written: 64, act_one_received: 64,
      proof_written: 22, proof_received: 22, new_work: 0, new_shares: 0 }, redaction_status: "passed" });
}

export async function finalize(root, cleanupPath, operations = {}) {
  const context = await loadContext(root, { historical: true, operations });
  await requireHostStopped(root, operations);
  await missing(resolve(root, "final-result.json")); await missing(resolve(root, "sealed-inventory.json"));
  const native = await recheckNative(context, operations);
  await writeNew(resolve(root, "native/final-readiness.json"), native);
  await snapshotCleanup(root);
  operations = { ...operations, cleanupSnapshot: true };
  let accepted = null, maybeFailure = null;
  try { accepted = await judge(root, context, cleanupPath, operations); }
  catch (error) { maybeFailure = error.code ?? "noise_evidence_rejected"; }
  let firstFailure = null;
  if (!accepted) {
    try { firstFailure = (await proof(root, "failure.json")).value.cause; }
    catch (error) { if (error.code !== "ENOENT") throw error; }
    firstFailure ??= { stage: "evidence", category: "evidence_incomplete", detail: "missing" };
    await writeNew(resolve(root, "judgment-failure.json"), { schema: "noise-serial-judgment-failure-v2", code: maybeFailure });
  }
  const inputs = await collectInputs(root, true);
  const result = { schema: "noise-serial-result-v2", contextSha256: digest(JSON.stringify(context)), status: accepted ? "passed" : "unverified",
    firstFailure, outcome: accepted ? "complete" : failureOutcome(firstFailure, maybeFailure), inputs };
  await writeNew(resolve(root, "final-result.json"), result);
  await writeNew(resolve(root, "sealed-inventory.json"), { schema: "noise-serial-seal-v2", contextSha256: result.contextSha256,
    files: await inventory(root, new Set(["sealed-inventory.json"])) });
  if (accepted) {
    const publicValue = projection(context, accepted, (await proof(root, "sealed-inventory.json")).sha256, (await proof(root, "final-result.json")).sha256);
    const publicPath = resolve(context.firmware_root, "docs/parity/evidence/str005-noise-serial", `attempt-${String(context.ordinal).padStart(3, "0")}.json`);
    await mkdir(dirname(publicPath), { recursive: true });
    const temporary = `${publicPath}.pending`;
    const handle = await open(temporary, "wx", 0o644);
    try { await handle.writeFile(`${JSON.stringify(publicValue, null, 2)}\n`); await handle.sync(); } finally { await handle.close(); }
    // Hard-link creation is atomic and never overwrites an existing published artifact.
    await link(temporary, publicPath); await unlink(temporary);
  }
  return { status: result.status, outcome: result.outcome, hardware_qualified: accepted !== null,
    result_sha256: (await proof(root, "final-result.json")).sha256, sealed_inventory_sha256: (await proof(root, "sealed-inventory.json")).sha256 };
}
export async function review(root, operations = {}) {
  const context = await loadContext(root, { historical: true, operations });
  const sealed = await proof(root, "sealed-inventory.json"), result = await proof(root, "final-result.json");
  check(sealed.value.schema === "noise-serial-seal-v2" && sealed.value.contextSha256 === digest(JSON.stringify(context)), "noise_seal_context");
  await verifyInventory(root, sealed.value.files, new Set(["sealed-inventory.json"]));
  check(JSON.stringify((await proof(root, "native/final-readiness.json")).value) === JSON.stringify(context.native_readiness), "noise_native_final_drift");
  exactObject(result.value, ["schema", "contextSha256", "status", "firstFailure", "outcome", "inputs"]);
  exactObject(result.value.inputs, ["deviceJournal", "fixtureReady", "fixtureTerminal", "preservation", "accounting", "cleanup", "recovery"]);
  if (result.value.firstFailure !== null) cause(result.value.firstFailure);
  check(result.value.schema === "noise-serial-result-v2" && result.value.contextSha256 === sealed.value.contextSha256 &&
    ["passed", "unverified"].includes(result.value.status), "noise_result_schema");
  check(JSON.stringify(result.value.inputs) === JSON.stringify(await collectInputs(root, true)), "noise_result_input_drift");
  if (result.value.status === "passed") {
    check(result.value.firstFailure === null && result.value.outcome === "complete", "noise_pass_disposition");
    const accepted = await judge(root, context, `${root}.cleanup/receipt.json`, { ...operations, checkKernel: false, cleanupSnapshot: true });
    check(JSON.stringify(accepted.inputs) === JSON.stringify(result.value.inputs), "noise_result_join");
    const published = JSON.parse(await readFile(resolve(context.firmware_root, "docs/parity/evidence/str005-noise-serial", `attempt-${String(context.ordinal).padStart(3, "0")}.json`), "utf8"));
    check(JSON.stringify(published) === JSON.stringify(projection(context, accepted, sealed.sha256, result.sha256)), "noise_projection_changed");
  }
  if (result.value.status === "unverified") {
    check(result.value.firstFailure !== null, "noise_failure_missing");
    const failure = (await proof(root, "judgment-failure.json")).value;
    check(result.value.outcome === failureOutcome(result.value.firstFailure, failure.code), "noise_failure_disposition");
    let rejected = false;
    try { await judge(root, context, `${root}.cleanup/receipt.json`, { ...operations, checkKernel: false, cleanupSnapshot: true }); }
    catch (error) { check((error.code ?? "noise_evidence_rejected") === failure.code, "noise_failure_reclassification"); rejected = true; }
    check(rejected, "noise_failed_result_promoted");
  }
  return { status: result.value.status, outcome: result.value.outcome, hardware_qualified: result.value.status === "passed",
    result_sha256: result.sha256, sealed_inventory_sha256: sealed.sha256 };
}
