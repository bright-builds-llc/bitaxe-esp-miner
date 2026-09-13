import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, exactObject, hex, requireCondition } from "./contract.mjs";

export const CADENCE_SCHEMA = "fixed-usb-cadence-context-v1";
export const CADENCE_TASK = "task-cpu0-telemetry-cadence-qualification";
export const CADENCE_PHASES = ["idle", "usb", "mining"];
export const CADENCE_LIMITS = Object.freeze({ duration_ms: 60000, minimum_intervals: 60,
  percent_within_750_ms: 95, maximum_interval_ms: 1500, maximum_execution_ms: 500,
  maximum_storage_bytes: 2048, observer_lifetime_ms: 360000, usb_probes: 12, probe_period_ms: 5000 });

export async function requireCadenceTask(root) {
  const text = await readFile(resolve(root, "TASKS.md"), "utf8");
  let active = false, count = 0, total = 0;
  for (const line of text.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (line.startsWith(`### ${CADENCE_TASK} |`)) { total += 1; if (active) count += 1; }
  }
  requireCondition(count === 1 && total === 1, "cadence_active_task_required");
}

export function validateCadencePolicy(context) {
  requireCondition(context.schema === CADENCE_SCHEMA && context.required_no_mining_cycles === 4 &&
    context.owner_stack_minimum_bytes === 4096 && context.suggested_difficulty === 1000 &&
    context.qualification_attempt?.purpose === "normal" && context.qualification_attempt.maximumActiveMilliseconds === 180000 &&
    JSON.stringify(context.cadence_limits) === JSON.stringify(CADENCE_LIMITS), "cadence_policy");
  requireCondition(context.recovery_phase === undefined && context.cycle_source === undefined &&
    context.qualification_driver === undefined && context.unreserved_continuation === undefined, "cadence_profile_mixing");
  exactObject(context.cadence_observer, ["path", "sha256"]);
  requireCondition(typeof context.cadence_observer.path === "string" && hex(context.cadence_observer.sha256, 64) &&
    hex(context.cadence_validator_sha256, 64), "cadence_observer_identity");
}

/** Bind all runtime helpers and membership, including transitive imported validators. */
export async function cadenceValidatorDigest(root) {
  const directory = resolve(root, "scripts/fixed-usb-qualification");
  const names = (await readdir(directory)).filter(name => name.endsWith(".mjs") && !name.endsWith(".test.mjs")).sort();
  const rows = [];
  for (const name of names) rows.push([name, digest(await readFile(resolve(directory, name)))]);
  return digest(JSON.stringify(rows));
}

export function validateCadenceProgress(progress) {
  exactObject(progress, ["schema", "review", "reason", "evidence_sha256"]);
  requireCondition(progress.schema === "worker-qualification-progress-v1" && progress.review === "verified" &&
    progress.reason === "software_correction" && Array.isArray(progress.evidence_sha256) &&
    progress.evidence_sha256.length > 0 && progress.evidence_sha256.length <= 16 &&
    progress.evidence_sha256.every(value => hex(value, 64)), "cadence_verified_progress_required");
}
