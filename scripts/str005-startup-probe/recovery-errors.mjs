import { check, object } from '../str005-v2-serial/values.mjs';
const phases = ['begin', 'state', 'ledger', 'original_budget', 'diagnostics', 'status', 'stop', 'closed', 'errors', 'finished'];
const categories = ['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed',
  'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active', 'operation_failed'];
/** Closed diagnostic vocabulary only; never persist Error.message or device payloads. */
export function validateRecoveryErrors(value) {
  object(value, ['schema', 'firstFailure', 'errors']);
  check(value.schema === 'str005-recovery-errors-v1' && Array.isArray(value.errors) && value.errors.length <= 24, 'startup_recovery_errors');
  for (const row of value.errors) { object(row, ['phase', 'category']); check(phases.includes(row.phase) && categories.includes(row.category), 'startup_recovery_errors'); }
  check(JSON.stringify(value.firstFailure) === JSON.stringify(value.errors[0] ?? null), 'startup_recovery_first_failure');
  return structuredClone(value);
}

export function validateClientFailure(value) {
  object(value, ['schema', 'phase', 'category', 'observedAtMs']);
  check(value.schema === 'str005-client-failure-v1' && ['prepare', 'start', 'dispatch', 'share'].includes(value.phase) &&
    categories.includes(value.category) && Number.isFinite(value.observedAtMs) && value.observedAtMs >= 0, 'startup_client_failure');
  return structuredClone(value);
}
