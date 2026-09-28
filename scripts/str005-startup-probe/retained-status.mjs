/** Null is idle-only on firmware. A confirmed Start must be queried by its known id. */
export async function readRecoveryStatus(gate, attemptId, statusMode) {
  if (statusMode === 'unknown') throw Error('startup_status_unknown_start');
  if (statusMode !== 'confirmed' && statusMode !== 'not_invoked') throw Error('startup_status_mode');
  if (statusMode === 'confirmed' && typeof attemptId !== 'string') throw Error('startup_status_attempt_missing');
  const binding = await gate.stratumV2Possession();
  return gate.stratumV2Status('share', statusMode === 'confirmed' ? attemptId : null, binding);
}

/** New current-session discovery never retries a rejected wire command. */
export async function discoverCurrentStatus(gate, attemptId, binding) {
  try { return await gate.stratumV2Status('share', null, binding); }
  catch (error) {
    // This is a local typed correlation result, not a firmware command rejection.
    if (error?.category !== 'v2_idle_correlation' || typeof attemptId !== 'string') throw error;
    return gate.stratumV2Status('share', attemptId, binding);
  }
}
