// Browser-safe: the Gate page loads this module directly, so it imports nothing.
const CATEGORIES = ['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed',
  'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active'];
/** The Worker's closed control rejections; a test pins this list to the v2-serial vocabulary. */
export const WORKER_REJECTIONS = Object.freeze(['invalid_frame', 'invalid_request', 'admission_required', 'invalid_proof',
  'authentication_failed', 'invalid_transition', 'persistence_failed', 'monotonic_reset', 'session_failed',
  'restoration_pending', 'stale_response', 'encoding_failed']);
export const RECOVERY_ERRORS_V2 = 'str005-recovery-errors-v2';
/** One collection error row. v2 keeps the Worker's closed rejection (for example `invalid_transition`)
 * that the Gate exposes beside `command_rejected`, so a rejected read explains itself. */
export function recoveryErrorRow(phase, error) {
  return { phase, category: CATEGORIES.includes(error?.category) ? error.category : 'operation_failed',
    rejection: WORKER_REJECTIONS.includes(error?.rejection) ? error.rejection : null };
}
