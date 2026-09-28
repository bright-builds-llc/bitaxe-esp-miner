/** Poll actual diagnostic observations; elapsed time alone never qualifies a fault. */
export async function awaitSelfTestReady({ exportDiagnostics, readiness, now = () => performance.now(), wait = ms => new Promise(resolve => setTimeout(resolve, ms)), timeoutMs = 30000, cadenceMs = 250 }) {
  const started = now(), deadline = started + Math.min(timeoutMs, 30000);
  async function bounded(operation) {
    const remaining = deadline - now();
    if (remaining <= 0) throw Error('panic_self_test_readiness_timeout');
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('panic_self_test_readiness_timeout')), remaining);
    })]); } finally { clearTimeout(timer); }
  }
  while (now() < deadline) {
    await bounded(exportDiagnostics);
    const result = await bounded(readiness);
    if (now() >= deadline) throw Error('panic_self_test_readiness_timeout');
    if (result?.state === 'ready') return result;
    if (result?.state !== 'pending') throw Error('panic_self_test_readiness_rejected');
    const remaining = deadline - now();
    if (remaining <= 0) break;
    await bounded(() => wait(Math.min(cadenceMs, remaining)));
  }
  throw Error('panic_self_test_readiness_timeout');
}

/** A preclaim failure owns cleanup and never consumes a device fault nonce. */
export async function runReadySelfTest({ gate, published, readiness, claim, fault, exportEvidence, saveEvidence, saveFailure, now = () => performance.now(), wait,
  timeoutMs = 30000, operationMs = 30000, cleanupMs = 150000 }) {
  let firstFailure, claimed = false, phase = 'readiness';
  const bounded = async (operation, limit = operationMs) => {
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => { timer = setTimeout(() => reject(Error('panic_self_test_operation_timeout')), limit); })]); }
    finally { clearTimeout(timer); }
  };
  try {
    await awaitSelfTestReady({ exportDiagnostics: () => gate.exportDiagnostics(), readiness, now, wait, timeoutMs });
    phase = 'claim';
    const request = await bounded(claim); claimed = true; phase = 'self_test';
    await bounded(() => fault(request));
  } catch (error) {
    firstFailure = { phase, category: ['panic_self_test_readiness_timeout', 'panic_self_test_operation_timeout'].includes(error?.message) ? 'timeout' : 'rejected' };
  } finally {
    if (claimed) {
      try { const evidence = exportEvidence(); if (!evidence) throw Error('panic_self_test_evidence_missing'); await bounded(() => saveEvidence(evidence)); } catch { firstFailure ??= { phase: 'evidence', category: 'rejected' }; }
      try { await bounded(() => gate.exportDiagnostics()); } catch { firstFailure ??= { phase: 'diagnostics', category: 'rejected' }; }
    }
    let stopComplete = false, closeComplete = false;
    try { await bounded(() => gate.stop(), cleanupMs); stopComplete = true; } catch { firstFailure ??= { phase: 'stop', category: 'rejected' }; }
    try { await bounded(() => gate.close(), cleanupMs); closeComplete = true; } catch { firstFailure ??= { phase: 'close', category: 'rejected' }; }
    let maybeClosed = null; try { maybeClosed = published(); } catch { /* No invented closure observation. */ }
    if (firstFailure) await bounded(() => saveFailure({ schema: 'str005-self-test-readiness-failure-v1', first_failure: firstFailure,
      claim_created: claimed, stop_complete: stopComplete, close_complete: closeComplete, closed: maybeClosed }));
  }
  return { complete: firstFailure === undefined, claim_created: claimed, first_failure: firstFailure ?? null };
}
