import { lstat, readFile, readdir } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { requireCadenceTask } from "./cadence-contract.mjs";
import { validateCadenceContext } from "./cadence-preflight.mjs";
import { canonicalDirectory, digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { parseSamples } from "./sample-seal.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

const CLOSURE = "unissued-closure.json";
const FORBIDDEN = ["issued.json", "consumed.json", "result.json", "sample-seal-intent.json", "iterative.fault.json",
  "cadence-idle-arm.json", "cadence-usb-arm.json", "cadence-mining-arm.json", "cadence-mining.json", "cadence-idle.json", "cadence-usb.json", "cadence-observer.jsonl",
  "cadence-observer-result.json", "cadence-probes.jsonl", "cadence-mining-measurement-end.json"];
const REMEDIATION = { schema: "cadence-preparation-remediation-v1",
  original_boundary: { failure: "connect_failed", stage: "permission", serial: "operation_failed" },
  remediation: "foreground_native_accessibility_click", native_chooser_observed: true, exact_firmware_admitted: true,
  baseline_confirmed: true, ledger_read_method: "workerAcceptance.reviewQualificationAttempts", original_budget_read_method: "workerAcceptance.reviewBudget",
  private_values_exported: false, qualification_claimed: false };

function validateCleanup(value) {
  exactObject(value, ["schema", "source", "browser_closed", "supervisor_exited", "supervisor_exit_code", "listener_absent", "owned_children_absent", "serial_holders_absent"]);
  requireCondition(value.schema === "worker-unissued-cleanup-v1" && value.source === "parent-observed" &&
    value.supervisor_exit_code === 0 && ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(key => value[key] === true),
  "cadence_unissued_cleanup");
}

async function inventory(root, directory = root) {
  await protectedPath(directory, true);
  const rows = [];
  for (const name of (await readdir(directory)).sort()) {
    if (directory === root && name === CLOSURE) continue;
    const path = resolve(directory, name), info = await lstat(path), pathName = relative(root, path);
    requireCondition(!info.isSymbolicLink(), "cadence_unissued_inventory_type");
    if (info.isDirectory()) {
      await protectedPath(path, true);
      rows.push({ path: pathName, type: "directory", mode: 0o700 }, ...await inventory(root, path));
    } else {
      await protectedPath(path);
      rows.push({ path: pathName, type: "file", mode: 0o600, length: info.size, sha256: await fileDigest(path) });
    }
  }
  return rows;
}

async function inspectUnissued(root, input, operations) {
  exactObject(input, ["ledger", "original_budget", "cleanup"]); validateCleanup(input.cleanup);
  await protectedPath(root, true); await protectedPath(resolve(root, "context.json"));
  const saved = await readJson(resolve(root, "context.json"));
  exactObject(saved, ["context", "sha256"]);
  const context = saved.context;
  requireCondition(saved.sha256 === digest(JSON.stringify(context)), "cadence_unissued_context");
  await validateCadenceContext(root, context, { historical: true, operations });
  await (operations.verifyArtifactSnapshot ?? verifyArtifactSnapshot)(root, context);
  for (const name of FORBIDDEN) await missing(resolve(root, name));
  requireIdleLedger(input.ledger, context.qualification_attempt.ordinal, context.expected_charged_ms);
  requireExhaustedOriginal(input.original_budget);
  const path = resolve(root, "iterative.samples.jsonl"); await protectedPath(path);
  const bytes = await readFile(path), records = parseSamples(bytes, context);
  requireCondition(records.every(({ state }) => !state.running && !["running", "stopping", "window_loaded"].includes(state.status) && state.renewalsConfirmed === 0 &&
    (state.qualification?.attempt === undefined || state.qualification.attempt.ordinal < context.qualification_attempt.ordinal) &&
    (!state.cadence || (state.cadence.firstWorkObservedAtMs === undefined && !state.cadence.latestWork && !state.cadence.review && !state.cadence.suppressionRequested))), "cadence_unissued_work_observed");
  const final = records.at(-1), state = final.state, preservation = state.preservation;
  requireCondition(state.status === "closed" && !state.connected && state.deviceBaselineConfirmed === true &&
    state.deviceLeaseInactive && state.serialOwnershipReleased && preservation?.device_identity_match === true &&
    preservation.settings_match === true && preservation.authorization_high_water_match === true && preservation.mine_on_boot === false,
  "cadence_unissued_final_state");
  const earliest = records.find(record => record.state.failure);
  requireCondition(earliest?.state.failure === "connect_failed" && earliest.state.admissionFailureStage === "permission", "cadence_unissued_first_failure");
  const failurePath = resolve(root, "first-failure.json"); await protectedPath(failurePath);
  const details = { schema: "worker-iterative-first-failure-v1", ordinal: context.qualification_attempt.ordinal, sequence: earliest.sequence,
    browser: earliest.state.failure, serial: earliest.state.serialFailureCategory ?? null, admission: earliest.state.admissionFailureStage ?? null };
  requireCondition(isDeepStrictEqual(await readJson(failurePath), details), "cadence_unissued_first_failure");
  const remediationPath = resolve(root, "native-gesture-remediation.json"); await protectedPath(remediationPath);
  requireCondition(isDeepStrictEqual(await readJson(remediationPath), REMEDIATION) && details.serial === "operation_failed", "cadence_unissued_remediation");
  return { context, context_sha256: saved.sha256, samples_sha256: digest(bytes), final_sequence: final.sequence,
    first_failure: { details, sha256: await fileDigest(failurePath) }, remediation_sha256: await fileDigest(remediationPath),
    artifact_snapshot_sha256: await fileDigest(resolve(root, "artifact-snapshot.json")) };
}

/** Seal a permission-stage failure only when its existing allowance was never issued. */
export async function closeUnissued(root, input, operations = {}) {
  root = await canonicalDirectory(root); await missing(resolve(root, CLOSURE));
  const inspected = await inspectUnissued(root, input, operations);
  await requireCadenceTask(inspected.context.firmware_root);
  const receipt = { schema: "worker-cadence-unissued-v1", result: "unissued", outcome: "continue_after_manual_remediation", hardware_pass: false, root,
    ...inspected, ...input, inventory: await inventory(root) };
  await writeNew(resolve(root, CLOSURE), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return { result: "unissued", outcome: "continue_after_manual_remediation", ordinal: inspected.context.qualification_attempt.ordinal, hardware_pass: false, device_effects: false };
}

/** Revalidate every sealed source without granting live authority to the old preparation. */
export async function readUnissued(path, operations = {}) {
  path = resolve(path); const root = await canonicalDirectory(dirname(path));
  const ancestors = operations.unissuedAncestors ?? [];
  requireCondition(!ancestors.includes(root), "cadence_unissued_recursive_lineage");
  operations = { ...operations, unissuedAncestors: [...ancestors, root] };
  requireCondition(path === resolve(root, CLOSURE), "cadence_unissued_closure_path"); await protectedPath(path);
  const saved = await readJson(path); exactObject(saved, ["receipt", "sha256"]);
  const receipt = saved.receipt;
  exactObject(receipt, ["schema", "result", "outcome", "hardware_pass", "root", "context", "context_sha256", "samples_sha256", "final_sequence", "first_failure", "remediation_sha256",
    "artifact_snapshot_sha256", "ledger", "original_budget", "cleanup", "inventory"]);
  requireCondition(saved.sha256 === digest(JSON.stringify(receipt)) && receipt.schema === "worker-cadence-unissued-v1" &&
    receipt.result === "unissued" && receipt.outcome === "continue_after_manual_remediation" && receipt.hardware_pass === false && receipt.root === root, "cadence_unissued_closure_integrity");
  requireCondition(isDeepStrictEqual(receipt.inventory, await inventory(root)), "cadence_unissued_inventory_changed");
  const inspected = await inspectUnissued(root, { ledger: receipt.ledger, original_budget: receipt.original_budget, cleanup: receipt.cleanup }, operations);
  requireCondition(Object.keys(inspected).every(key => isDeepStrictEqual(inspected[key], receipt[key])), "cadence_unissued_evidence_changed");
  return receipt;
}
