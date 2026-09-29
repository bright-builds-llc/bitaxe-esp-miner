const recoveryFailure = (phase, error) => ({ phase, category: error?.category === 'timeout' ? 'timeout' : 'operation_failed' });

/** Browser-only one-shot Start/status coordinator with an independent Stop timer. */
export function createCoordinator({ gate, prepare, record, recover, release, post, recordFailure = async () => {},
  now = () => performance.now(), limits = {} }) {
  const bounds = { replyMs: 30000, statusMs: 10000, stopFromReplyMs: 15000, stopFromInvocationMs: 45000,
    cleanupMs: 150000, releaseMs: 10000, ...limits };
  let consumed = false;
  const bounded = async (operation, ms) => {
    if (!(ms > 0)) throw Error('status_repro_deadline');
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Object.assign(Error('status_repro_timeout'), { category: 'timeout' })), ms);
    })]); } finally { clearTimeout(timer); }
  };
  return async () => {
    if (consumed) throw Error('status_repro_consumed'); consumed = true;
    const failures = []; let firstFailure = null, phase = 'prepare', clientFailure;
    let startInvokedAt = null, startRepliedAt = null, stopRequestedAt = null, observedStart = false, proof = null;
    let requestSettled = false, timedOut = false, stopPromise, hardStop, replyStop;
    const collect = async (stage, operation, ms = bounds.cleanupMs) => {
      try { await bounded(operation, ms); } catch { failures.push(stage); }
    };
    const stop = () => {
      if (!stopPromise) { stopRequestedAt = now(); stopPromise = Promise.resolve().then(() => gate.stop()); }
      return stopPromise;
    };
    try {
      const admission = await bounded(prepare, bounds.cleanupMs);
      phase = 'start'; startInvokedAt = now();
      hardStop = setTimeout(() => { void stop().catch(() => {}); }, bounds.stopFromInvocationMs);
      const started = Promise.resolve().then(() => gate.startWindow()).then(async value => {
        requestSettled = true;
        if (timedOut) {
          let stopComplete = false, closeComplete = false;
          try { await gate.stop(); stopComplete = true; } catch { /* Preserve the independent late cleanup receipt. */ }
          try { await gate.close(); closeComplete = true; } catch { /* Preserve the independent late cleanup receipt. */ }
          await post('/status-repro/late-completion', { stopComplete, closeComplete }).catch(() => {});
        }
        return value;
      }, error => { requestSettled = true; throw error; });
      await bounded(() => started, bounds.replyMs);
      startRepliedAt = now();
      if (startRepliedAt - startInvokedAt > bounds.replyMs || startRepliedAt - startInvokedAt >= bounds.stopFromInvocationMs)
        throw Error('status_repro_late_reply');
      replyStop = setTimeout(() => { void stop().catch(() => {}); }, bounds.stopFromReplyMs);
      const state = gate.state();
      if (!state.running || state.failure || state.heartbeatSuppressed || state.renewalsConfirmed !== 0 ||
        state.qualification?.generation !== admission.generation) throw Error('status_repro_start_unverified');
      observedStart = true; phase = 'status';
      const deadline = startRepliedAt + bounds.statusMs;
      const status = await bounded(() => gate.stratumV2Status('share', admission.attemptId, admission.binding), deadline - now());
      const receipt = await bounded(() => post('/status-repro/observe', { status, state: gate.state() }), deadline - now());
      if (!receipt.observed || receipt.generation !== admission.generation) throw Error('status_repro_observation_unverified');
      proof = { generation: receipt.generation, observedAtMs: now() };
    } catch (error) {
      firstFailure = phase; timedOut = !requestSettled && startInvokedAt !== null;
      clientFailure = { schema: 'str005-client-failure-v1', ...recoveryFailure(phase, error), observedAtMs: now() };
    } finally {
      clearTimeout(hardStop); clearTimeout(replyStop);
      await collect('stop', stop);
      if (clientFailure) await collect('client-failure', () => recordFailure(clientFailure));
      await collect('result', () => record({ firstFailure, observedStart, startInvokedAt, startRepliedAt, stopRequestedAt, proof }));
      await collect('recovery', recover);
      await collect('close', () => gate.close());
      await collect('closed-state', () => record({ closed: gate.state() }));
      await collect('release', release, bounds.releaseMs);
    }
    return { complete: firstFailure === null && failures.length === 0, firstFailure, failures, observedStart,
      statusObserved: proof !== null, lateReplyPending: !requestSettled && startInvokedAt !== null, parityPromotion: false };
  };
}
