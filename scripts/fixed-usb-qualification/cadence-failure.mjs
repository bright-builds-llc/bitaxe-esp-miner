import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, exactObject, protectedPath, requireCondition as check } from "./contract.mjs";
import { validateCadenceContext } from "./cadence-preflight.mjs";
import { inspectCadenceFailureEvidence } from "./cadence-failure-evidence.mjs";

export const CADENCE_FAILURE_SHA256 = "2003dd26039daa1edad78057e88215a07fffab4883d955648b8a61ad12f8aacb";
export const CADENCE_FAILURE_PRODUCER = "f894edb393c176edc115e5408530a6ebe1045709f1b17543ca91450e19395c78";
const UNCHANGED = [
  "gate_commit",
  "gate_bundle_sha256",
  "gate_page_relative_path",
  "gate_page_sha256",
  "trust_sha256",
  "authority_trust_sha256",
  "reference_commit",
];

/** Evidence-only classification. The public reader additionally requires the immutable accepted seal. */
export async function inspectCadenceFailure(root, context, operations = {}) {
  check(
    context.schema === "fixed-usb-cadence-context-v1" &&
      context.preparation_attempt === 2 &&
      context.qualification_attempt?.ordinal === 17 &&
      context.restart_predecessor &&
      context.cadence_failure_predecessor === undefined &&
      context.premining_predecessor === undefined &&
      context.unissued_predecessor === undefined &&
      context.startup_predecessor === undefined,
    "cadence_failure_class",
  );
  await validateCadenceContext(root, context, { historical: true, operations });
  // Context validation already revalidated the complete restart receipt and ancestry.
  // Re-read only its bound bytes; a changed file cannot replace that verified receipt.
  const restartPath = resolve(context.restart_predecessor.root, "result.json");
  await protectedPath(restartPath);
  const restartBytes = await readFile(restartPath);
  const restartHash = digest(restartBytes);
  check(restartHash === context.restart_predecessor.receipt_sha256, "cadence_failure_restart_changed");
  const restartValue = JSON.parse(restartBytes.toString("utf8"));
  exactObject(restartValue, ["receipt", "sha256"]);
  check(
    restartValue.sha256 === digest(JSON.stringify(restartValue.receipt)) &&
      restartValue.receipt.result === "controlled_restart_verified" &&
      restartValue.receipt.context.restart_attempt === 4,
    "cadence_failure_restart_ancestry",
  );
  const restart = { sha256: restartHash, value: restartValue };
  const expected = await inspectCadenceFailureEvidence(root, context, restart, CADENCE_FAILURE_PRODUCER);
  const sealPath = resolve(root, "failed-inventory.json");
  await protectedPath(sealPath);
  const bytes = await readFile(sealPath),
    seal = JSON.parse(bytes.toString("utf8"));
  check(equal(seal, expected), "cadence_failure_evidence_changed");
  return { root, context, seal, binding: { root, failed_inventory_sha256: digest(bytes) } };
}
/** Read only the exact accepted USB-percentile failure, preserving all historical outcomes. */
export async function readCadenceFailure(root, operations = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  const sealPath = resolve(root, "failed-inventory.json");
  await protectedPath(sealPath);
  const bytes = await readFile(sealPath);
  check(digest(bytes) === CADENCE_FAILURE_SHA256, "cadence_failure_anchor");
  const seal = JSON.parse(bytes.toString("utf8")),
    contextPath = resolve(root, "context.json");
  await protectedPath(contextPath);
  const contextBytes = await readFile(contextPath),
    entry = seal.files?.find((value) => value.path === "context.json" && value.type === "file");
  check(entry && entry.sha256 === digest(contextBytes), "cadence_failure_context_changed");
  const saved = JSON.parse(contextBytes.toString("utf8"));
  exactObject(saved, ["context", "sha256"]);
  check(saved.sha256 === digest(JSON.stringify(saved.context)), "cadence_failure_context_changed");
  const result = await inspectCadenceFailure(root, saved.context, operations);
  check(result.binding.failed_inventory_sha256 === CADENCE_FAILURE_SHA256, "cadence_failure_anchor");
  return result;
}
export function requireCadenceFailureLineage(root, context, prior, progress) {
  exactObject(context.cadence_failure_predecessor, ["root", "failed_inventory_sha256"]);
  const old = prior.context;
  check(
    prior.binding.failed_inventory_sha256 === CADENCE_FAILURE_SHA256 &&
      equal(context.cadence_failure_predecessor, prior.binding) &&
      old.preparation_attempt === 2 &&
      old.qualification_attempt.ordinal === 17 &&
      old.cadence_failure_predecessor === undefined &&
      dirname(root) === dirname(prior.root) &&
      root !== prior.root,
    "cadence_failure_lineage",
  );
  check(
    context.preparation_attempt === 3 &&
      context.restart_predecessor === undefined &&
      context.premining_predecessor === undefined &&
      context.unissued_predecessor === undefined &&
      context.startup_predecessor === undefined &&
      context.qualification_attempt.ordinal === 17 &&
      context.qualification_attempt.purpose === "normal" &&
      context.qualification_attempt.maximumActiveMilliseconds === 180000 &&
      context.qualification_attempt.id !== old.qualification_attempt.id &&
      context.previous_receipt === old.previous_receipt &&
      context.previous_receipt_sha256 === old.previous_receipt_sha256 &&
      context.original_campaign_id === old.original_campaign_id &&
      context.expected_charged_ms === 1380000,
    "cadence_failure_accounting_lineage",
  );
  check(
    context.firmware_commit !== old.firmware_commit &&
      context.app_elf_sha256 !== old.app_elf_sha256 &&
      UNCHANGED.every((key) => equal(context[key], old[key])),
    "cadence_failure_runtime_delta",
  );
  check(
    progress.reason === "software_correction" &&
      progress.review === "verified" &&
      progress.evidence_sha256.includes(CADENCE_FAILURE_SHA256) &&
      progress.evidence_sha256.some((value) => value !== CADENCE_FAILURE_SHA256),
    "cadence_failure_progress",
  );
}
export async function reviewCadenceFailure(root, operations = {}) {
  const value = await readCadenceFailure(root, operations);
  return {
    reviewed: true,
    result: "unverified",
    failed_inventory_sha256: value.binding.failed_inventory_sha256,
    first_review_sequence: value.seal.first_rejected_review_sequence,
    final_sequence: value.seal.final_sequence,
    observations: value.seal.observations,
    post_failure_ledger_observed: false,
    qualification_pass: false,
    continuation_authority: false,
  };
}
