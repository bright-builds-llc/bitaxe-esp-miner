/** Explicit accepted-share prerequisite; historical panic contracts remain independent. */
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { isDeepStrictEqual as equal } from 'node:util';
import { privateRoot, proof, verifyInventory, canonical } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateRecoveryParts, recoveryConclusion, projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
export const RENEW_ENABLED = false;
export const RENEW_TASK = 'task-str005-v2-accepted-share-probe';
export const RENEW_CONTRACT = 'docs/hardware/str005-renew-installation.md';
export const STARTUP_SEAL = 'cc5bab7bd34da6f3b50388c31f5e1fc4aefb6a7faf12bd14678a6dc08593e7a1';
export function renewScope(tasks, enabled = RENEW_ENABLED) {
  const blocks = tasks.split(`### ${RENEW_TASK} |`);
  check(enabled && blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${RENEW_TASK} |`), 'renew_image_disabled');
  const lines = blocks[1].split(/^### /mu)[0].split(/\r?\n/u).map(line => line.trim());
  check(lines.includes('Renew image qualification: baseline enabled.'), 'renew_image_disabled');
  const installEnabled = lines.includes('Renew image qualification: installation enabled.');
  const selfTestEnabled = lines.includes('Renew image qualification: self-test enabled.');
  const clearEnabled = lines.includes('Renew image qualification: core clear enabled.');
  check([installEnabled, selfTestEnabled, clearEnabled].filter(Boolean).length <= 1, 'renew_image_effect_scope');
  return { ownerTask: RENEW_TASK, renewSuccessor: true, installEnabled, selfTestEnabled, clearEnabled, storeAuditRequired: true, cutoffUserRegionRequired: true };
}
export async function renewSource(root) {
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8'), scope = renewScope(tasks);
  const block = tasks.split(`### ${RENEW_TASK} |`)[1]?.split(/^### /mu)[0] ?? '';
  const pins = [...block.matchAll(/^Renew image capture seal: ([a-f0-9]{64})\.$/gmu)];
  if (scope.clearEnabled) check(pins.length === 1, 'renew_capture_pin');
  return { ...scope, ...(scope.clearEnabled ? { captureSealSha256: pins[0][1] } : {}),
    contractSha256: sha256(await readFile(resolve(root, RENEW_CONTRACT))) };
}
export async function startupPredecessor(root) {
  const directory = resolve(root, 'scratch/str005-startup/startup003/attempt');
  await privateRoot(directory); const seal = await proof(directory, 'sealed-inventory.json');
  check(seal.sha256 === STARTUP_SEAL, 'renew_startup_seal');
  await verifyInventory(directory, seal.value.files, new Set(['sealed-inventory.json']));
  const saved = await proof(directory, 'context.json'), result = (await proof(directory, 'result.json')).value;
  const before = (await proof(directory, 'before.json')).value, run = (await proof(directory, 'run.json')).value;
  const status = (await proof(directory, 'recovery-1-status.json')).value;
  const ledger = (await proof(directory, 'recovery-1-ledger.json')).value;
  const original = (await proof(directory, 'recovery-1-original_budget.json')).value;
  check(result.schema === 'str005-startup-result-v1' && result.complete === true && result.blockers?.length === 0 &&
    result.start_proven === true && result.dispatch_increased === true && result.core_capture_verified === true &&
    run.observedStart === true && run.firstFailure === null && status.record?.attemptId === before.attempt.id &&
    status.record.workerGeneration === run.proof.generation && status.observation.bootOrdinal === 9, 'renew_startup_unqualified');
  validateLedger(ledger); requireExhaustedOriginal(original);
  check(!ledger.pending && ledger.next_ordinal === 20 && ledger.last_completed_ordinal === 19 && ledger.total_charged_ms === 1920000, 'renew_startup_accounting');
  const context = saved.value;
  return { seal: seal.sha256, contextDigest: saved.sha256, before: { ...context, attemptId: before.attempt.id }, detector: context.detector,
    retainedBaseline: { attemptId: before.attempt.id, generation: run.proof.generation, bootOrdinal: 9, ledger, originalBudget: original } };
}
export function retainedConclusion(parts, context) {
  const projected = { ...parts, ...(parts.finished ? { finished: { failures: parts.finished.failures } } : {}), ...(parts.diagnostics ? { diagnostics: projectRecoveryPart('diagnostics', parts.diagnostics, context) } : {}) };
  validateRecoveryParts(projected, { ...context, ...context.before_source });
  const result = recoveryConclusion(projected), bound = context.retainedBaseline;
  check(context.renewSuccessor === true && context.ownerTask === RENEW_TASK && bound, 'renew_retained_owner');
  const blockers = [...result.blockers], record = parts.status?.record;
  if (parts.status && !(record?.attemptId === bound.attemptId && record.workerGeneration === bound.generation &&
    parts.status.observation.bootOrdinal === bound.bootOrdinal && record.bootOrdinal === bound.bootOrdinal)) blockers.push('retained_identity_mismatch');
  if (parts.ledger && !equal(parts.ledger, bound.ledger)) blockers.push('retained_accounting_changed');
  if (parts.original_budget && !equal(parts.original_budget, bound.originalBudget)) blockers.push('retained_original_budget_changed');
  // The installation keeps one same-page authorization/preservation baseline across reset.
  for (const key of ['state', 'closed']) if (parts[key]?.preservation?.authorization_high_water_match !== true) blockers.push(`${key}_authorization_unconfirmed`);
  return { schema: 'str005-panic-baseline-result-v1', complete: blockers.length === 0, blockers,
    next_ordinal: parts.ledger?.next_ordinal ?? null, historical_resource_proof: false, parity_promotion: false };
}
export function releasedProofFields(context, parts) {
  check(retainedConclusion(parts, context).complete, 'renew_retained_incomplete');
  return { schema: 'str005-current-recovery-proof-v2', current_v2_released: true,
    retained_attempt_id: context.retainedBaseline.attemptId, retained_status_sha256: sha256(canonical(parts.status)) };
}
export function admitRetainedProof(value, context) {
  check(context.renewSuccessor === true && context.ownerTask === RENEW_TASK && context.retainedBaseline &&
    value.schema === 'str005-current-recovery-proof-v2' && value.current_v2_released === true &&
    !Object.hasOwn(value, 'current_v2_idle') && value.retained_attempt_id === context.retainedBaseline.attemptId &&
    /^[0-9a-f]{64}$/u.test(value.retained_status_sha256) && equal(value.ledger, context.retainedBaseline.ledger) &&
    equal(value.original_budget, context.retainedBaseline.originalBudget), 'renew_retained_proof');
}

/** Rebind the proof to the actual one-use collection and retained status before reset. */
export async function verifyRetainedInputs(root, context, value) {
  if (!context.retainedBaseline) return;
  admitRetainedProof(value, context);
  const begin = (await proof(root, 'baseline-begin.json')).value;
  check(begin.schema === 'str005-renew-baseline-begin-v1' && begin.startedAtUnixMs === value.observed_at_unix_ms, 'renew_collection_binding');
  const parts = {};
  for (const stage of ['state', 'ledger', 'original_budget', 'diagnostics', 'status', 'closed', 'finished'])
    parts[stage] = (await proof(root, `baseline-${stage}.json`)).value;
  check(retainedConclusion(parts, context).complete && value.retained_status_sha256 === sha256(canonical(parts.status)), 'renew_status_binding');
}
