import { basename, dirname, resolve } from "node:path";
import { review as reviewNoise } from "../str005-noise-serial/finalize.mjs";
import { canonical, privateRoot, proof } from "../str005-noise-serial/files.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { check, object } from "./values.mjs";

export const ACCEPTED_NOISE_RESULT_SHA256 = "9fe8f1ed6755e2c9a8573fa6dc5ae867fca044f6a79636cc584415c60744c40a";
export const ACCEPTED_NOISE_SEAL_SHA256 = "cc8e278543447e026b908b2a0e9e5b626ff1f7d47ec245060325ccf91d49f4d2";

/** Finite lineage only: Share -> accepted Channel -> accepted Noise. No retry authority. */
export async function inspectPredecessor(path, scope, operations = {}) {
  path = resolve(path); check(basename(path) === "final-result.json", "v2_predecessor_path");
  const root = await privateRoot(dirname(path)), before = await proof(root, "final-result.json");
  if (scope === "channel") {
    check(before.sha256 === ACCEPTED_NOISE_RESULT_SHA256, "v2_noise_predecessor_anchor");
    check((await proof(root, "sealed-inventory.json")).sha256 === ACCEPTED_NOISE_SEAL_SHA256, "v2_noise_predecessor_seal");
  }
  const stored = await proof(root, "context.json"), previous = stored.value.context;
  let reviewed;
  if (scope === "channel") {
    check(previous?.schema === "noise-serial-context-v2", "v2_noise_predecessor_required");
    reviewed = await reviewNoise(root);
  } else {
    check(scope === "share" && ["str005-v2-serial-context-v1", "str005-v2-serial-context-v2", "str005-v2-serial-context-v3"].includes(previous?.schema) && previous.scope === "channel", "v2_channel_predecessor_required");
    const reader = operations.reviewChannel ?? (await import("./finalize.mjs")).review;
    reviewed = await reader(root);
    check(reviewed.scope === "channel", "v2_channel_predecessor_scope");
  }
  const seal = await proof(root, "sealed-inventory.json");
  check(reviewed.status === "passed" && reviewed.outcome === "complete" && reviewed.hardware_qualified === true &&
    reviewed.result_sha256 === before.sha256 && reviewed.sealed_inventory_sha256 === seal.sha256 &&
    (await proof(root, "final-result.json")).sha256 === before.sha256 &&
    (await proof(root, "context.json")).sha256 === stored.sha256, "v2_predecessor_unverified");
  let ledger, original;
  if (scope === "channel") {
    const accounting = (await proof(root, "accounting-after.json")).value;
    ledger = accounting.ledger; original = accounting.original_budget;
  } else {
    // V2 finalization binds this closed after-accounting receipt into its inventory.
    const accounting = (await proof(root, "accounting-after.json")).value;
    check(accounting.schema === "str005-v2-accounting-v1" && accounting.contextSha256 === stored.value.sha256 && accounting.stage === "after", "v2_predecessor_accounting");
    ledger = accounting.ledger; original = accounting.original_budget;
  }
  requireIdleLedger(ledger, 18, 1560000); requireExhaustedOriginal(original);
  return { root, context: previous, resultSha256: before.sha256, sealSha256: seal.sha256, ledger, original };
}
export function requirePredecessorBinding(context, predecessor) {
  object(context.predecessor, ["root", "resultSha256", "sealSha256"]);
  check(context.predecessor.root === predecessor.root && context.predecessor.resultSha256 === predecessor.resultSha256 &&
    context.predecessor.sealSha256 === predecessor.sealSha256 && context.original_campaign_id === predecessor.context.original_campaign_id,
    "v2_predecessor_changed");
  // v3 Channel's actual installed image is independently joined to its failed
  // successor receipt. Older contexts and Share retain the exact ancestor rule.
  if (!(context.schema === "str005-v2-serial-context-v3" && context.scope === "channel"))
    check(context.before_source.firmware_commit === predecessor.context.firmware_commit &&
      context.before_source.app_elf_sha256 === predecessor.context.app_elf_sha256, "v2_before_source_changed");
  if (context.scope === "share") {
    for (const key of ["firmware_commit", "gate_commit", "app_elf_sha256", "manifest_sha256", "artifacts", "update_segments", "trust_sha256",
      "gate_bundle_sha256", "gate_page_sha256", "cadence_observer", "observer_build_receipt_sha256", "client_sha256", "operator_sha256", "fixture_sha256", "fixture_build_receipt_sha256", "sdkconfig_sha256", "evaluator", "contracts", "contractSha256"])
      check(canonical(context[key]) === canonical(predecessor.context[key]), "v2_share_pair_changed");
  }
}
