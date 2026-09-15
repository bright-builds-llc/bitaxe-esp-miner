import { RESTART_NETWORK_FAILURE_SHA256 } from "./reset-origin-restart-network-failure.mjs";
import { RESTART_INSTALL_FAILURE_SHA256 } from "./reset-origin-restart-install-failure.mjs";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, requireCondition as check } from "./contract.mjs";
import { baseline, proof } from "./cadence-premining-evidence.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { readRestartResult } from "./reset-origin-restart-judge.mjs";
import { readResetOrigin } from "./reset-origin-judge.mjs";
import { RESET_ORIGIN_SOURCE_SEAL } from "./reset-origin-runtime-source.mjs";
import { STARTUP_FAILURE_SHA256 } from "./cadence-startup-failure.mjs";

export const CADENCE_RESTART_STAGE_A_SHA256 = "8847afd051a0a17ad3703dd3a5e6986b13e0c6ba395f705160095ad9f1543817";
const PAIR_FIELDS = [
  "firmware_commit",
  "gate_commit",
  "manifest_sha256",
  "app_elf_sha256",
  "reference_commit",
  "artifacts",
  "update_segments",
  "gate_bundle_sha256",
  "gate_page_relative_path",
  "gate_page_sha256",
  "trust_sha256",
];

/** Revalidate Stage B and its exact accepted Stage-A lineage. It cannot itself reserve mining authority. */
export async function readCadenceRestart(path, operations = {}) {
  path = resolve(path);
  let stagePath, stagePromise;
  const readStageA = (value) => {
    const canonical = resolve(value);
    check(stagePath === undefined || stagePath === canonical, "cadence_restart_stage_a_path");
    if (!stagePromise) {
      stagePath = canonical;
      stagePromise = (operations.readStageA ?? ((value) => readResetOrigin(value, operations)))(canonical);
    }
    return stagePromise;
  };
  const receipt = await readRestartResult(path, Object.assign(Object.create(operations), { readStageA }));
  const accepted = await readStageA(receipt.context.stage_a.path),
    stageFile = await proof(stagePath);
  // readRestartResult/readResetOrigin already revalidate every immutable ancestor. Bind these extracted contexts to those same hashes.
  const recoveryFile = await proof(resolve(accepted.context.runtime_source.root, "context.json")),
    recovery = recoveryFile.value.context;
  check(
    recoveryFile.value.sha256 === digest(JSON.stringify(recovery)) &&
      recoveryFile.value.sha256 === accepted.context.runtime_source.context_sha256 &&
      recovery.failed_inventory_sha256 === STARTUP_FAILURE_SHA256,
    "cadence_restart_recovery_binding",
  );
  const failedSeal = await proof(resolve(recovery.failed_root, "failed-inventory.json")),
    failedFile = await proof(resolve(recovery.failed_root, "context.json")),
    failed = failedFile.value.context;
  check(
    failedSeal.value.context_sha256 === failedFile.value.sha256 && failedFile.value.sha256 === digest(JSON.stringify(failed)),
    "cadence_restart_failed_context_changed",
  );
  validateCadenceRestartEvidence(receipt, accepted, recovery, failed, stageFile.sha256, failedSeal.sha256);
  const resultFile = await proof(path);
  check(
    equal(resultFile.value.receipt, receipt) && resultFile.value.sha256 === digest(JSON.stringify(receipt)),
    "cadence_restart_receipt_changed",
  );
  return {
    root: dirname(path),
    receipt,
    failed,
    failed_root: recovery.failed_root,
    stage_a: accepted,
    binding: { root: dirname(path), receipt_sha256: resultFile.sha256 },
  };
}

/** Validate domain joins after readers have verified the supplied immutable evidence. */
export function validateCadenceRestartEvidence(receipt, accepted, recovery, failed, stageSha256, failedSealSha256) {
  check(
    receipt.result === "controlled_restart_verified" &&
      receipt.device_recovered === true &&
      receipt.cleanup_confirmed === true &&
      receipt.mining_authorized === false &&
      receipt.qualification_pass === false &&
      receipt.cadence_admission_authorized === false,
    "cadence_restart_result_required",
  );
  check(
    receipt.context.restart_attempt === undefined ||
      ([2, 3].includes(receipt.context.restart_attempt) &&
        receipt.context.statistics_startup_required === true &&
        receipt.context.install_failure_predecessor?.failed_inventory_sha256 ===
          (receipt.context.restart_attempt === 2 ? RESTART_INSTALL_FAILURE_SHA256 : RESTART_NETWORK_FAILURE_SHA256)),
    "cadence_restart_successor_class",
  );
  requireIdleLedger(receipt.ledger, 17, 1380000);
  requireExhaustedOriginal(receipt.original_budget);
  baseline(receipt.final_state, true);
  check(
    stageSha256 === CADENCE_RESTART_STAGE_A_SHA256 &&
      receipt.context.stage_a.sha256 === stageSha256 &&
      accepted.context.observation_attempt === 5 &&
      accepted.result === "observed_stable" &&
      accepted.observation_qualified === true &&
      accepted.context.runtime_source.failed_inventory_sha256 === RESET_ORIGIN_SOURCE_SEAL,
    "cadence_restart_stage_a_anchor",
  );
  check(
    failedSealSha256 === STARTUP_FAILURE_SHA256 &&
      recovery.failed_inventory_sha256 === failedSealSha256 &&
      failed.qualification_attempt.ordinal === 17 &&
      failed.preparation_attempt === undefined &&
      failed.expected_charged_ms === 1380000 &&
      failed.previous_receipt === accepted.context.previous_receipt &&
      failed.previous_receipt_sha256 === accepted.context.previous_receipt_sha256 &&
      recovery.previous_receipt === failed.previous_receipt &&
      recovery.previous_receipt_sha256 === failed.previous_receipt_sha256,
    "cadence_restart_original_preparation",
  );
}

/** Admit only a fresh preparation of the unused ordinal, on the exact accepted restart runtime. */
export function requireCadenceRestartLineage(root, context, prior) {
  root = resolve(root);
  exactObject(context.restart_predecessor, ["root", "receipt_sha256"]);
  check(
    equal(context.restart_predecessor, prior.binding) &&
      dirname(root) === dirname(prior.root) &&
      root !== prior.root &&
      dirname(root) === dirname(prior.failed_root) &&
      root !== prior.failed_root,
    "cadence_restart_sibling_required",
  );
  check(
    context.preparation_attempt === 2 &&
      context.qualification_attempt.ordinal === 17 &&
      context.qualification_attempt.purpose === "normal" &&
      context.qualification_attempt.maximumActiveMilliseconds === 180000 &&
      context.qualification_attempt.id !== prior.failed.qualification_attempt.id &&
      context.previous_receipt === prior.stage_a.context.previous_receipt &&
      context.previous_receipt_sha256 === prior.stage_a.context.previous_receipt_sha256 &&
      context.original_campaign_id === prior.failed.original_campaign_id &&
      context.expected_charged_ms === 1380000,
    "cadence_restart_ledger_lineage",
  );
  check(
    PAIR_FIELDS.every((key) => equal(context[key], prior.receipt.context[key])),
    "cadence_restart_pair_changed",
  );
}
