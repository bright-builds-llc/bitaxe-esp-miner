/** Null is idle-only on firmware. A confirmed Start must be queried by its known id. */
export async function readRecoveryStatus(gate, attemptId, statusMode) {
  if (statusMode === 'unknown') throw Error('startup_status_unknown_start');
  if (statusMode !== 'confirmed' && statusMode !== 'not_invoked') throw Error('startup_status_mode');
  if (statusMode === 'confirmed' && typeof attemptId !== 'string') throw Error('startup_status_attempt_missing');
  const binding = await gate.stratumV2Possession();
  return gate.stratumV2Status('share', statusMode === 'confirmed' ? attemptId : null, binding);
}
