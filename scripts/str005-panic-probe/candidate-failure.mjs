import { resolve } from 'node:path';
import { writeNew, inventory } from '../str005-noise-serial/files.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
export function validateCandidateFailure(value) {
  object(value, ['schema', 'first_failure', 'stop_complete', 'close_complete', 'closed']);
  object(value.first_failure, ['phase', 'category']);
  check(value.schema === 'str005-candidate-recovery-failure-v1' &&
    ['possession', 'status', 'begin', 'collection', 'state', 'ledger', 'original_budget', 'diagnostics', 'stop', 'close'].includes(value.first_failure.phase) &&
    ['timeout', 'operation_failed', 'v2_retained_evidence'].includes(value.first_failure.category) &&
    typeof value.stop_complete === 'boolean' && typeof value.close_complete === 'boolean', 'panic_candidate_failure_shape');
  return value;
}
export function applyCandidateFailure(result, value) {
  object(value, ['schema', 'first_failure', 'stop_complete', 'close_complete', 'closed_state_valid']);
  validateCandidateFailure({ schema: value.schema, first_failure: value.first_failure, stop_complete: value.stop_complete,
    close_complete: value.close_complete, closed: null });
  check(typeof value.closed_state_valid === 'boolean', 'panic_candidate_failure_shape');
  result.complete = false;
  result.blockers.push('candidate_recovery_failed');
  result.first_failure = value.first_failure;
  result.candidate_close_confirmed = value.close_complete && value.closed_state_valid;
  return result;
}

/** Successful and partial probe outcomes use the same immutable finalization path. */
export async function sealProbeResult(root, result) {
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return result;
}
