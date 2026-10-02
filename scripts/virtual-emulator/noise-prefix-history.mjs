// Pure readers for prior diagnostic results. Active telemetry never calls these.
/** First-boundary safety and post-drop safety remain separate, with missing snapshots unknown. */
export function judgePrefixSnapshots(decoded, stop) {
  if (decoded?.schema !== 'bitaxe-noise-live-snapshots-v1' || decoded.cutoff_phase !== stop || !Array.isArray(decoded.snapshots)) return { prefix_state: 'unknown', cleanup_state: 'unknown' };
  const safe = (snapshot, expected) => snapshot?.registers_available === true && snapshot.checkpoint_memory_available === true
    && snapshot.scope_violation === false && snapshot.task_bounds?.filter(task => task.task_label === 'main' && task.bounds_available === true && task.captured_sp_inside_bounds === true && task.saved_tcb_top_inside_bounds === true).length === 1
    && snapshot.records?.length === expected.length && snapshot.records.every((record, i) => record.phase === expected[i] && record.stage === 2
      && record.heap_integrity === true && record.heap_observation_available === true && record.stack_pointer_inside_configured_span === true
      && record.configured_stack_span_bytes === 16384 && Number.isSafeInteger(record.stack_low_water_bytes) && record.stack_low_water_bytes >= 2048);
  const before = decoded.snapshots[0], after = decoded.snapshots[1], phases = Array.from({ length: stop - 100 }, (_, i) => 101 + i);
  const state = (snapshot, kind, expected) => {
    if (!snapshot) return 'unknown';
    if (['panic', 'assert'].includes(snapshot.kind) || snapshot.scope_violation === true) return 'failed';
    if (snapshot.kind !== kind) return 'unknown';
    if (snapshot.records?.some(record => record.heap_integrity === false || record.stack_pointer_inside_configured_span === false
      || (Number.isSafeInteger(record.stack_low_water_bytes) && record.stack_low_water_bytes < 2048))) return 'failed';
    if (snapshot.task_bounds?.some(task => task.task_label === 'main' && task.bounds_available === true
      && (task.captured_sp_inside_bounds === false || task.saved_tcb_top_inside_bounds === false))) return 'failed';
    if (snapshot.prefix_flags_available !== true) return 'unknown';
    return safe(snapshot, expected) ? 'safe' : 'unknown';
  };
  const prefixState = state(before, 'prefix', phases);
  const cleanupState = state(after, 'released', [...phases, 114]);
  return { prefix_state: prefixState === 'safe' && (before.prefix_argument_matches !== true || before.prefix_reached_phase !== stop || before.prefix_released_flag !== 0 || before.prefix_flags_available !== true) ? 'failed' : prefixState,
    cleanup_state: cleanupState === 'safe' && (after.prefix_reached_phase !== stop || after.prefix_released_flag !== 1 || after.prefix_argument_matches !== true || after.prefix_flags_available !== true) ? 'failed' : cleanupState };
}

export function qualificationFailure(result) {
  if (result.first_target_fault) return { phase: result.first_target_fault.phase, category: result.first_target_fault.category };
  return result.first_host_failure ?? result.collection_failures?.[0] ?? null;
}


/** SDK panic observation is independent of damaged or truncated structured guest output. */
export function prefixLogFacts(log) {
  const unexpectedPanic = /Guru Meditation|assert failed|Stack canary|stack overflow|abort\(\)/.test(log);
  const events = [];
  let invalidGuestJson = false;
  for (const line of log.split(/\r?\n/).filter(line => line.startsWith('VIRTUAL_U205 '))) {
    try { events.push(JSON.parse(line.slice(13))); }
    catch { invalidGuestJson = true; }
  }
  return { unexpected_panic: unexpectedPanic, events, invalid_guest_json: invalidGuestJson };
}
