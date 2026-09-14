import { resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { canonicalDirectory, digest, requireCondition as check } from "./contract.mjs";
import { loadStartupRecoveryContext } from "./cadence-startup-context.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
export const RESET_ORIGIN_SOURCE_SEAL = "0ed37434d6bb8587ea51dec6b1e3cf41128ee555635058bd74fd2c84b702e834";
export const RUNTIME_KEYS = [
  "manifest_sha256",
  "app_elf_sha256",
  "reference_commit",
  "artifacts",
  "update_segments",
  "firmware_commit",
  "gate_commit",
  "gate_bundle_sha256",
  "gate_page_relative_path",
  "gate_page_sha256",
  "trust_sha256",
  "supervisor_client_sha256",
];
export async function runtimeSource(root, operations, runtimeCache) {
  root = await canonicalDirectory(root);
  if (!runtimeCache.has(root)) runtimeCache.set(root, readRuntimeSource(root, operations));
  return runtimeCache.get(root);
}
async function readRuntimeSource(root, operations) {
  root = await canonicalDirectory(root);
  const seal = await proof(resolve(root, "failed-inventory.json"));
  check(
    seal.sha256 === (operations.expectedResetOriginSourceSeal ?? RESET_ORIGIN_SOURCE_SEAL) &&
      seal.value.schema === "cpu0-cadence-startup-recovery-failed-inventory-v1" &&
      seal.value.outcome === "stop_impossible_contract" &&
      seal.value.qualification_pass === false &&
      seal.value.device_recovery_claimed === false &&
      seal.value.new_hardware_authorized === false,
    "reset_origin_source_anchor",
  );
  const context = await loadStartupRecoveryContext(root, { historical: true, operations });
  const snapshot = await verifyArtifactSnapshot(root, context);
  check(
    seal.value.context_sha256 === digest(JSON.stringify(context)) &&
      seal.value.artifact_snapshot_sha256 === snapshot.receipt_sha256 &&
      isDeepStrictEqual(seal.value.inventory, await inventory(root)),
    "reset_origin_source_changed",
  );
  return {
    context,
    binding: {
      root,
      failed_inventory_sha256: seal.sha256,
      context_sha256: digest(JSON.stringify(context)),
      artifact_snapshot_sha256: snapshot.receipt_sha256,
    },
  };
}
