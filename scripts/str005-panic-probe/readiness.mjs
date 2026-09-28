import { proof } from '../str005-noise-serial/files.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { requireCurrentCaptureReview } from './capture-existing.mjs';
import { requireStoreReady } from './store-diagnostics.mjs';
/** Only absence of a current-boot receipt is pending; every contradictory row rejects. */
export function evaluateReadiness(diagnostics, context, status, review, observedAt, now) {
  if (context.captureExisting) requireCurrentCaptureReview(review, status, now);
  check(Number.isSafeInteger(observedAt) && now >= observedAt && now - observedAt <= 30000, 'panic_store_diagnostics_stale');
  const rows = diagnostics?.observations;
  check(Array.isArray(rows), 'panic_store_diagnostics_missing');
  const boots = rows.filter(row => row.category === 'boot'), identities = rows.filter(row => row.category === 'runtime_identity');
  check(boots.length === 1 && boots[0].boot_ordinal === status.observation.bootOrdinal, 'panic_store_boot_correlation');
  check(identities.length === 1 && identities[0].firmware_commit === context.firmware_commit && identities[0].app_elf_sha256 === context.app_elf_sha256, 'panic_store_runtime_identity');
  const current = rows.filter(row => row.category === 'core_dump_store_receipt' && row.origin !== 'previous_boot');
  if (current.length === 0) return { state: 'pending' };
  check(current.length === 1, 'panic_store_receipt_conflicting');
  requireStoreReady(diagnostics, context, status.observation.bootOrdinal);
  return { state: 'ready' };
}

/** Preserve a preclaim failure in the sealed outcome independently of fault evidence. */
export async function applyReadinessFailure(root, result) {
  let value;
  try { value = (await proof(root, 'self-test-readiness-failure.json')).value; }
  catch (error) { if (error.code !== 'ENOENT') throw error; return; }
  object(value, ['schema', 'first_failure', 'claim_created', 'claim_acknowledged', 'stop_complete', 'close_complete']);
  object(value.first_failure, ['phase', 'category']);
  check(value.schema === 'str005-self-test-readiness-failure-v1' &&
    ['readiness', 'claim', 'self_test', 'evidence', 'diagnostics', 'stop', 'close'].includes(value.first_failure.phase) &&
    ['timeout', 'rejected'].includes(value.first_failure.category) &&
    ['claim_created', 'claim_acknowledged', 'stop_complete', 'close_complete'].every(key => typeof value[key] === 'boolean'), 'panic_readiness_failure_shape');
  result.complete = false; result.blockers.push('self_test_readiness_failed'); result.first_failure = value.first_failure;
}
