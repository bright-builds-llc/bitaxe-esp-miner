import { RECOVERY_SCHEMA } from "./recovery-judge.mjs";
import { lstat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateCycle, validateState } from "./judge.mjs";

const SCHEMAS = ["fixed-usb-remaining-campaign-v1", "fixed-usb-remaining-campaign-v2"];
export function validateBudgetReview(report, window = 1) {
  exactObject(report, ["schema", "campaign_match", "reserved_mask", "completed_mask", "charged_ms", "pending"]);
  const mask = window === 1 ? 1 : window === 2 ? 3 : -1;
  requireCondition(report.schema === "worker-budget-review-v1" && report.campaign_match === true &&
    report.reserved_mask === mask && report.completed_mask === mask && report.pending === false &&
    report.charged_ms === (window === 1 ? 180000 : 210000), "successor_budget_state");
  return report;
}
export function validateCoolingReview(value) {
  exactObject(value, ["proof", "restoration", "budget_before", "budget_after", "state"]);
  const { proof, restoration } = value;
  exactObject(proof, ["schema", "fan_duty_percent", "fan_rpm", "post_command_fan_proven", "asic_effects", "budget_reserved"]);
  exactObject(restoration, ["schema", "fan_duty_percent", "cooling_proven", "asic_effects", "budget_reserved"]);
  requireCondition(proof.schema === "worker-cooling-proof-v1" && proof.fan_duty_percent === 100 &&
    Number.isInteger(proof.fan_rpm) && proof.fan_rpm > 0 && proof.fan_rpm <= 65535 &&
    proof.post_command_fan_proven === true && proof.asic_effects === false && proof.budget_reserved === false,
  "successor_fan_proof");
  requireCondition(restoration.schema === "worker-cooling-baseline-v1" && restoration.fan_duty_percent === 30 &&
    restoration.cooling_proven === true && restoration.asic_effects === false && restoration.budget_reserved === false,
  "successor_cooling_restoration");
  validateBudgetReview(value.budget_before, 2);
  validateBudgetReview(value.budget_after, 2);
  return value;
}
async function readCoolingReview(root, context, path) {
  requireCondition(typeof path === "string" && dirname(resolve(path)) === root, "successor_cooling_required");
  await protectedPath(path);
  requireCondition((await lstat(path)).size <= 65536, "successor_cooling_bound");
  const receipt = await readJson(path);
  exactObject(receipt, ["schema", "context_sha256", "proof", "restoration", "budget_before", "budget_after", "state"]);
  requireCondition(receipt.schema === "fixed-usb-cooling-review-v1" &&
    receipt.context_sha256 === digest(JSON.stringify(context)), "successor_cooling_context");
  const { schema, context_sha256, ...review } = receipt;
  validateCoolingReview(review);
  await requireSuccessorBaseline(root, context, receipt.state);
  return receipt;
}
export async function requireSuccessorBaseline(root, context, state) {
  return requireBaselineContinuity(root, context, state, false);
}
async function requireBaselineContinuity(root, context, state, released) {
  validateState(state, context);
  const baselineConfirmed = context.schema === RECOVERY_SCHEMA && !released ? state.deviceBaselineConfirmed === true : state.deviceRestorationConfirmed;
  requireCondition(state.connected === !released && !state.running && baselineConfirmed && state.deviceLeaseInactive &&
    state.serialOwnershipReleased === released && !state.failure && state.preservation?.device_identity_match === true &&
    state.preservation.settings_match === true && state.preservation.mine_on_boot === false, "successor_baseline");
  let previous;
  for (let index = 1; index <= 4; index += 1) {
    const path = resolve(root, `cycle-${index}.json`);
    await protectedPath(path);
    previous = validateCycle(await readJson(path), context, previous);
  }
  requireCondition(previous.baseline_id === state.preservation.baseline_id, "successor_baseline_continuity");
}
async function sourceEvidence(root, window = 0) {
  await protectedPath(root, true);
  const hashes = {};
  const names = ["context.json", `window-${window}.consumed.json`, `window-${window}.first-failure.json`];
  if (window === 1) names.push("window-1.issued.json", "successor.json", "window-1.recovery.json", "window-1.budget-review.json");
  for (const name of names) {
    const path = resolve(root, name);
    await protectedPath(path);
    requireCondition((await lstat(path)).size <= 65536, "successor_evidence_bound");
    hashes[name] = await fileDigest(path);
  }
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.sha256 === digest(JSON.stringify(record.context)), "successor_predecessor_context");
  const consumed = await readJson(resolve(root, `window-${window}.consumed.json`));
  requireCondition(consumed.window === window && consumed.delivery_attempted === true, "successor_window_not_consumed");
  const failure = await readJson(resolve(root, `window-${window}.first-failure.json`));
  requireCondition(failure.schema === "fixed-usb-live-failure-review-v1" && failure.window === window &&
    failure.first_failure === "start_failed" && failure.active_milliseconds === "unverified" && (window === 0 ? failure.mining_retry === false :
      failure.foreground_fault_triggered === false && failure.running_observed === false),
  "successor_failure_evidence");
  await missing(resolve(root, `window-${window}.result.json`));
  if (window === 1) await validateRetiredForeground(root, record);
  return { context: record.context, hashes };
}

async function validateRetiredForeground(root, record) {
  // Only one predecessor generation is permitted; v2 cannot recursively mint successors.
  const predecessor = await readJson(resolve(root, "successor.json"));
  requireCondition(predecessor.schema === SCHEMAS[0], "successor_ancestry_bound");
  await loadSuccessor(root, record.context);
  for (const suffix of ["issued", "consumed", "result"]) await missing(resolve(root, `window-2.${suffix}.json`));
  const recovery = await readJson(resolve(root, "window-1.recovery.json"));
  exactObject(recovery, ["schema", "window", "context_sha256", "restoration_confirmed", "lease_inactive", "mine_on_boot",
    "serial_ownership_released", "device_identity_match", "settings_match", "active_milliseconds", "work_dispatched",
    "hardware_preparation_started", "readiness_mask", "first_device_failure", "fan_rpm"]);
  requireCondition(recovery.schema === "fixed-usb-recovery-review-v1" && recovery.window === 1 &&
    recovery.context_sha256 === record.sha256 && recovery.mine_on_boot === false &&
    recovery.active_milliseconds === 0 && recovery.work_dispatched === 0 && recovery.hardware_preparation_started === false &&
    recovery.readiness_mask === 55 && recovery.first_device_failure === "readiness" && recovery.fan_rpm === 0 &&
    ["restoration_confirmed", "lease_inactive", "serial_ownership_released", "device_identity_match", "settings_match"]
      .every((key) => recovery[key] === true), "successor_recovery_evidence");
  const budget = await readJson(resolve(root, "window-1.budget-review.json"));
  requireCondition(budget.schema === "fixed-usb-budget-review-v1" && budget.context_sha256 === record.sha256, "successor_retired_budget_context");
  validateBudgetReview(budget.report, 2);
  await requireBaselineContinuity(root, record.context, budget.state, true);
}

export async function createSuccessor(root, context, predecessorRoot, reviewPath, maybeCoolingPath) {
  predecessorRoot = resolve(predecessorRoot);
  requireCondition(root !== predecessorRoot && dirname(root) === dirname(predecessorRoot), "successor_campaign_directory");
  await protectedPath(reviewPath);
  const review = await readJson(reviewPath);
  const nextWindow = review.report?.charged_ms === 210000 ? 2 : 1;
  const original = await sourceEvidence(predecessorRoot, nextWindow - 1);
  requireCondition(original.context.campaign_id === context.campaign_id &&
    original.context.firmware_commit !== context.firmware_commit, "successor_campaign_identity");
  requireCondition(review.schema === "fixed-usb-budget-review-v1" && review.context_sha256 === digest(JSON.stringify(context)), "successor_review_context");
  validateBudgetReview(review.report, nextWindow);
  await requireSuccessorBaseline(root, context, review.state);
  if (nextWindow === 2) await readCoolingReview(root, context, maybeCoolingPath);
  else requireCondition(maybeCoolingPath === undefined, "successor_cooling_scope");
  for (let index = 0; index < 3; index += 1) await missing(resolve(root, `window-${index}.issued.json`));
  const value = { schema: SCHEMAS[nextWindow - 1], context_sha256: digest(JSON.stringify(context)),
    predecessor_root: predecessorRoot, predecessor_hashes: original.hashes,
    review_path: resolve(reviewPath), review_sha256: await fileDigest(reviewPath),
    remaining_windows: nextWindow === 1 ? [1, 2] : [2], window_limits_ms: [180000, 30000, 30000], renewal_ms: 5000,
    original_normal_window: "consumed_unverified",
    ...(nextWindow === 2 ? { original_foreground_loss_window: "consumed_unverified",
      cooling_review_path: resolve(maybeCoolingPath), cooling_review_sha256: await fileDigest(maybeCoolingPath) } : {}), no_refund: true };
  await writeNew(resolve(root, "successor.json"), value);
  return { successor_created: true, remaining_windows: value.remaining_windows, original_normal_window: "consumed_unverified", device_effects: false };
}
export async function loadSuccessor(root, context) {
  const path = resolve(root, "successor.json");
  try { await lstat(path); } catch (error) { if (error.code === "ENOENT") return undefined; throw error; }
  await protectedPath(path);
  const value = await readJson(path);
  exactObject(value, ["schema", "context_sha256", "predecessor_root", "predecessor_hashes", "review_path", "review_sha256",
    "remaining_windows", "window_limits_ms", "renewal_ms", "original_normal_window", "no_refund"], ["original_foreground_loss_window", "cooling_review_path", "cooling_review_sha256"]);
  const nextWindow = value.schema === SCHEMAS[1] ? 2 : 1;
  requireCondition(SCHEMAS.includes(value.schema) && value.context_sha256 === digest(JSON.stringify(context)) &&
    value.original_normal_window === "consumed_unverified" && value.no_refund === true && value.renewal_ms === 5000 &&
    JSON.stringify(value.remaining_windows) === (nextWindow === 1 ? "[1,2]" : "[2]") && JSON.stringify(value.window_limits_ms) === "[180000,30000,30000]" &&
    value.predecessor_root !== root && dirname(value.predecessor_root) === dirname(root), "successor_integrity");
  requireCondition(nextWindow === 1 ? value.original_foreground_loss_window === undefined && value.cooling_review_path === undefined && value.cooling_review_sha256 === undefined :
    value.original_foreground_loss_window === "consumed_unverified", "successor_original_claims");
  for (let index = 0; index < nextWindow; index += 1) {
    for (const suffix of ["issued", "consumed", "result"]) await missing(resolve(root, `window-${index}.${suffix}.json`));
  }
  const original = await sourceEvidence(value.predecessor_root, nextWindow - 1);
  requireCondition(original.context.campaign_id === context.campaign_id &&
    JSON.stringify(original.hashes) === JSON.stringify(value.predecessor_hashes), "successor_evidence_changed");
  await protectedPath(value.review_path);
  requireCondition(await fileDigest(value.review_path) === value.review_sha256, "successor_review_changed");
  const review = await readJson(value.review_path);
  requireCondition(review.context_sha256 === value.context_sha256, "successor_review_context");
  validateBudgetReview(review.report, nextWindow);
  await requireSuccessorBaseline(root, context, review.state);
  if (nextWindow === 2) {
    await readCoolingReview(root, context, value.cooling_review_path);
    requireCondition(await fileDigest(value.cooling_review_path) === value.cooling_review_sha256, "successor_cooling_changed");
  }
  return value;
}
