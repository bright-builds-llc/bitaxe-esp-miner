import { lstat, readFile, readdir, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileDigest, protectedPath, within } from "../fixed-usb-qualification/contract.mjs";
import { readPrevious } from "../fixed-usb-qualification/iterative-preflight.mjs";
import { check, digest, proof, verifyInventory } from "./files.mjs";
export const CADENCE_RESULT = "75f62388727078793177d549e1fcc1a9eadef9fdbbdba8fcc4b0f47147b911c0";
export const CADENCE_SEAL = "fdc6db7219f6c3898b16cc63206da544e0120ed2d3b9b24dd72fd2a2222e48a9";

/** Exact accepted CPU0 baseline; structural result validation alone is not provenance. */
export async function inspectPredecessor(path) {
  check(await realpath(path) === resolve(path), "noise_predecessor_alias");
  await protectedPath(path); const root = dirname(path); await protectedPath(root, true);
  check(await fileDigest(path) === CADENCE_RESULT, "noise_predecessor_anchor");
  const sealed = await proof(root, "sealed-inventory.json");
  check(sealed.sha256 === CADENCE_SEAL && sealed.value.schema === "cpu0-cadence-retention-ordinal17-inventory-v1" &&
    sealed.value.result_sha256 === CADENCE_RESULT && Array.isArray(sealed.value.inventory), "noise_predecessor_seal");
  const expected = new Map(sealed.value.inventory.map((item) => [item.path, item]));
  check(expected.size === sealed.value.inventory.length, "noise_predecessor_membership");
  async function visit(dir, prefix) {
    for (const name of await readdir(dir)) {
      if (!prefix && name === "sealed-inventory.json") continue;
      const relative = `${prefix}${name}`, entry = expected.get(relative);
      check(entry, "noise_predecessor_membership"); expected.delete(relative);
      const current = within(root, resolve(root, relative)), stat = await lstat(current);
      check(!stat.isSymbolicLink() && (stat.mode & 0o777) === entry.mode, "noise_predecessor_mode");
      if (stat.isDirectory()) { check(entry.type === "directory" && entry.mode === 0o700, "noise_predecessor_directory"); await visit(current, `${relative}/`); }
      else { check(entry.type === "file" && entry.mode === 0o600 && stat.size === entry.length && await fileDigest(current) === entry.sha256, "noise_predecessor_changed"); }
    }
  }
  await visit(root, ""); check(expected.size === 0, "noise_predecessor_membership");
  const previous = await readPrevious(path);
  return { previous, resultSha256: CADENCE_RESULT, inventorySha256: CADENCE_SEAL };
}

export const RECOVERY_RESULT = "609698fb42c694def0e6f736281d2a31558453707ce1bbc2931fe3e5b5f4aac9";
export const RECOVERY_SEAL = "08b70903f7129ab435945871746d853dd4a297a2205e8c0660b2eb373a158d56";

/** Exact sealed safety-recovery006: a current-safe-recovery basis, never a qualification pass. */
export async function inspectRecoveryPredecessor(path) {
  check(await realpath(path) === resolve(path), "noise_predecessor_alias");
  await protectedPath(path); const root = dirname(path); await protectedPath(root, true);
  check(await fileDigest(path) === RECOVERY_RESULT, "noise_predecessor_anchor");
  const sealed = await proof(root, "sealed-inventory.json");
  check(sealed.sha256 === RECOVERY_SEAL && Array.isArray(sealed.value.files), "noise_predecessor_seal");
  await verifyInventory(root, sealed.value.files, new Set(["sealed-inventory.json"]));
  const result = (await proof(root, "result.json")).value, context = (await proof(root, "context.json")).value;
  const ledger = (await proof(root, "baseline-ledger.json")).value, original = (await proof(root, "baseline-original_budget.json")).value;
  check(result.schema === "str005-panic-baseline-result-v1" && result.current_safe_recovery === true &&
    result.host_resources_released === true && result.first_failure === null && result.next_ordinal === ledger.next_ordinal &&
    context.stage === "safety-recovery" && context.recoveryOnly === true &&
    context.before_source?.firmware_commit === context.firmware_commit && context.before_source?.app_elf_sha256 === context.app_elf_sha256,
  "noise_predecessor_recovery");
  check(ledger.schema === "worker-qualification-ledger-v1" && ledger.pending === false &&
    original.schema === "worker-budget-review-v1" && original.campaign_match === true && original.pending === false &&
    original.reserved_mask === original.completed_mask, "noise_predecessor_ledger");
  const previous = { basis: "current_safe_recovery", cleanup_confirmed: true, next_ordinal: ledger.next_ordinal,
    last_ordinal: ledger.last_completed_ordinal, total_charged_ms: ledger.total_charged_ms,
    context: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 },
    original_campaign_id: context.original_campaign_id };
  return { previous, resultSha256: RECOVERY_RESULT, inventorySha256: RECOVERY_SEAL };
}
