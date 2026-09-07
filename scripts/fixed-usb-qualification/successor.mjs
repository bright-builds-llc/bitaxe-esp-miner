import { lstat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateCycle, validateState } from "./judge.mjs";

const EVIDENCE = ["context.json", "window-0.consumed.json", "window-0.first-failure.json"];
export function validateBudgetReview(report, window = 1) {
  exactObject(report, ["schema", "campaign_match", "reserved_mask", "completed_mask", "charged_ms", "pending"]);
  const mask = window === 1 ? 1 : window === 2 ? 3 : -1;
  requireCondition(report.schema === "worker-budget-review-v1" && report.campaign_match === true &&
    report.reserved_mask === mask && report.completed_mask === mask && report.pending === false &&
    report.charged_ms === (window === 1 ? 180000 : 210000), "successor_budget_state");
  return report;
}
export async function requireSuccessorBaseline(root, context, state) {
  validateState(state, context);
  requireCondition(state.connected && !state.running && state.deviceRestorationConfirmed && state.deviceLeaseInactive &&
    !state.serialOwnershipReleased && !state.failure && state.preservation?.device_identity_match === true &&
    state.preservation.settings_match === true && state.preservation.mine_on_boot === false, "successor_baseline");
  let previous;
  for (let index = 1; index <= 4; index += 1) {
    const path = resolve(root, `cycle-${index}.json`);
    await protectedPath(path);
    previous = validateCycle(await readJson(path), context, previous);
  }
  requireCondition(previous.baseline_id === state.preservation.baseline_id, "successor_baseline_continuity");
}
async function sourceEvidence(root) {
  await protectedPath(root, true);
  const hashes = {};
  for (const name of EVIDENCE) {
    const path = resolve(root, name);
    await protectedPath(path);
    requireCondition((await lstat(path)).size <= 65536, "successor_evidence_bound");
    hashes[name] = await fileDigest(path);
  }
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.sha256 === digest(JSON.stringify(record.context)), "successor_predecessor_context");
  const consumed = await readJson(resolve(root, "window-0.consumed.json"));
  requireCondition(consumed.window === 0 && consumed.delivery_attempted === true, "successor_window_not_consumed");
  const failure = await readJson(resolve(root, "window-0.first-failure.json"));
  requireCondition(failure.schema === "fixed-usb-live-failure-review-v1" && failure.window === 0 &&
    failure.first_failure === "start_failed" && failure.active_milliseconds === "unverified" && failure.mining_retry === false,
  "successor_failure_evidence");
  await missing(resolve(root, "window-0.result.json"));
  return { context: record.context, hashes };
}
export async function createSuccessor(root, context, predecessorRoot, reviewPath) {
  predecessorRoot = resolve(predecessorRoot);
  requireCondition(root !== predecessorRoot && dirname(root) === dirname(predecessorRoot), "successor_campaign_directory");
  const original = await sourceEvidence(predecessorRoot);
  requireCondition(original.context.campaign_id === context.campaign_id &&
    original.context.firmware_commit !== context.firmware_commit, "successor_campaign_identity");
  await protectedPath(reviewPath);
  const review = await readJson(reviewPath);
  requireCondition(review.schema === "fixed-usb-budget-review-v1" && review.context_sha256 === digest(JSON.stringify(context)), "successor_review_context");
  validateBudgetReview(review.report);
  await requireSuccessorBaseline(root, context, review.state);
  for (let index = 0; index < 3; index += 1) await missing(resolve(root, `window-${index}.issued.json`));
  const value = { schema: "fixed-usb-remaining-campaign-v1", context_sha256: digest(JSON.stringify(context)),
    predecessor_root: predecessorRoot, predecessor_hashes: original.hashes,
    review_path: resolve(reviewPath), review_sha256: await fileDigest(reviewPath),
    remaining_windows: [1, 2], window_limits_ms: [180000, 30000, 30000], renewal_ms: 5000,
    original_normal_window: "consumed_unverified", no_refund: true };
  await writeNew(resolve(root, "successor.json"), value);
  return { successor_created: true, remaining_windows: [1, 2], original_normal_window: "consumed_unverified", device_effects: false };
}
export async function loadSuccessor(root, context) {
  const path = resolve(root, "successor.json");
  try { await lstat(path); } catch (error) { if (error.code === "ENOENT") return undefined; throw error; }
  await protectedPath(path);
  const value = await readJson(path);
  exactObject(value, ["schema", "context_sha256", "predecessor_root", "predecessor_hashes", "review_path", "review_sha256",
    "remaining_windows", "window_limits_ms", "renewal_ms", "original_normal_window", "no_refund"]);
  requireCondition(value.schema === "fixed-usb-remaining-campaign-v1" && value.context_sha256 === digest(JSON.stringify(context)) &&
    value.original_normal_window === "consumed_unverified" && value.no_refund === true && value.renewal_ms === 5000 &&
    JSON.stringify(value.remaining_windows) === "[1,2]" && JSON.stringify(value.window_limits_ms) === "[180000,30000,30000]" &&
    value.predecessor_root !== root && dirname(value.predecessor_root) === dirname(root), "successor_integrity");
  for (const suffix of ["issued", "consumed", "result"]) await missing(resolve(root, `window-0.${suffix}.json`));
  const original = await sourceEvidence(value.predecessor_root);
  requireCondition(original.context.campaign_id === context.campaign_id &&
    JSON.stringify(original.hashes) === JSON.stringify(value.predecessor_hashes), "successor_evidence_changed");
  await protectedPath(value.review_path);
  requireCondition(await fileDigest(value.review_path) === value.review_sha256, "successor_review_changed");
  const review = await readJson(value.review_path);
  requireCondition(review.context_sha256 === value.context_sha256, "successor_review_context");
  validateBudgetReview(review.report);
  await requireSuccessorBaseline(root, context, review.state);
  return value;
}
