// Browser-safe: Gate pages load this module directly. Every V2 status read in the recovery and loop
// owners goes through `readStatusFor`, so the firmware's rule lives in one place: a null query
// succeeds only while no record is retained; a retained record is read only by its device record
// attempt (see `start-record.mjs`). Any other query is rejected, and USB then revokes the session.

/** How status must be read for a lineage: by id while a record is retained, otherwise discovered. */
export function statusModeFor(attempt) {
  return attempt?.recordRetained === true ? 'confirmed' : 'discover_current';
}

/** The single status read for a known mode. `unknown` (a Start that may or may not exist) is refused. */
export async function readStatusFor(gate, { statusMode, attemptId }, binding) {
  if (statusMode === 'confirmed') {
    if (typeof attemptId !== 'string') throw Error('startup_status_attempt_missing');
    return gate.stratumV2Status('share', attemptId, binding);
  }
  if (statusMode === 'not_invoked') return gate.stratumV2Status('share', null, binding);
  if (statusMode === 'discover_current') return discoverCurrentStatus(gate, attemptId, binding);
  if (statusMode === 'unknown') throw Error('startup_status_unknown_start');
  throw Error('startup_status_mode');
}

/** Null is idle-only on firmware. A confirmed Start must be queried by its known id. */
export async function readRecoveryStatus(gate, attemptId, statusMode) {
  if (statusMode === 'unknown') throw Error('startup_status_unknown_start');
  if (statusMode !== 'confirmed' && statusMode !== 'not_invoked') throw Error('startup_status_mode');
  if (statusMode === 'confirmed' && typeof attemptId !== 'string') throw Error('startup_status_attempt_missing');
  return readStatusFor(gate, { statusMode, attemptId }, await gate.stratumV2Possession());
}

/** New current-session discovery never retries a rejected wire command. Use it only when no Start is known on
 * this boot: firmware rejects a null query while a record is retained (`command_rejected`) and revokes the session,
 * so the attempt-id fallback below runs only after a local idle-correlation failure. */
export async function discoverCurrentStatus(gate, attemptId, binding) {
  try { return await gate.stratumV2Status('share', null, binding); }
  catch (error) {
    // This is a local typed correlation result, not a firmware command rejection.
    if (error?.category !== 'v2_idle_correlation' || typeof attemptId !== 'string') throw error;
    return gate.stratumV2Status('share', attemptId, binding);
  }
}
