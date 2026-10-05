import { check, object } from '../str005-v2-serial/values.mjs';
import { CONTROL_REJECTIONS } from '../str005-v2-serial/safety-diagnostics.mjs';
const phases = ['begin', 'state', 'ledger', 'original_budget', 'diagnostics', 'status', 'stop', 'closed', 'errors', 'finished'];
const categories = ['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed',
  'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active', 'operation_failed'];
export { RECOVERY_ERRORS_V2, recoveryErrorRow } from './recovery-error-row.mjs';
/** Closed diagnostic vocabulary only; never persist Error.message or device payloads. v1 rows have no rejection. */
export function validateRecoveryErrors(value, prefix = 'startup_recovery') {
  object(value, ['schema', 'firstFailure', 'errors']);
  const v2 = value.schema === 'str005-recovery-errors-v2';
  check((v2 || value.schema === 'str005-recovery-errors-v1') && Array.isArray(value.errors) && value.errors.length <= 24, `${prefix}_errors`);
  for (const row of value.errors) {
    object(row, ['phase', 'category', ...(v2 ? ['rejection'] : [])]);
    check(phases.includes(row.phase) && categories.includes(row.category) &&
      (!v2 || row.rejection === null || CONTROL_REJECTIONS.includes(row.rejection)), `${prefix}_errors`);
  }
  check(JSON.stringify(value.firstFailure) === JSON.stringify(value.errors[0] ?? null), `${prefix}_first_failure`);
  return structuredClone(value);
}

/** v1 has no rejection; v2 adds the Worker's closed control rejection when the Gate exposed one. */
export function validateClientFailure(value) {
  const v2 = value?.schema === 'str005-client-failure-v2';
  object(value, ['schema', 'phase', 'category', 'observedAtMs', ...(v2 ? ['rejection'] : [])]);
  check((v2 || value.schema === 'str005-client-failure-v1') && ['prepare', 'start', 'dispatch', 'share', 'status'].includes(value.phase) &&
    categories.includes(value.category) && Number.isFinite(value.observedAtMs) && value.observedAtMs >= 0 &&
    (!v2 || value.rejection === null || CONTROL_REJECTIONS.includes(value.rejection)), 'startup_client_failure');
  return structuredClone(value);
}

const BASELINE_OPERATIONS = ['context', 'ledger', 'original_budget', 'possession', 'status', 'diagnostics', 'baseline', 'close', 'configure'];
/** Which read-only baseline operation failed, in the closed serial vocabulary; unknown Gate categories become operation_failed. */
export function validateBaselineFailure(value) {
  object(value, ['schema', 'operation', 'category']);
  check(value.schema === 'str005-baseline-failure-v1' && BASELINE_OPERATIONS.includes(value.operation) && typeof value.category === 'string',
    'startup_baseline_failure');
  return { ...value, category: categories.includes(value.category) ? value.category : 'operation_failed' };
}
