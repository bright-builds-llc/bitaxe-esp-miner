import { recoveryFailure } from './recovery-collection.mjs';
/** One Start; independent timers request normal Stop even while an observation is blocked. */
export function createShareCoordinator({ gate, prepare, post, record, recover, release, recordFailure = async () => {},
  now = () => performance.now(), limits = {} }) {
  const bounds = { replyMs: 30000, observeMs: 45000, readMs: 30000, cleanupMs: 150000, pollMs: 200, ...limits };
  let consumed = false;
  const bounded = async (operation, ms) => {
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Object.assign(Error('share_timeout'), { category: 'timeout' })), Math.max(0, ms));
    })]); } finally { clearTimeout(timer); }
  };
  return async () => {
    if (consumed) throw Error('share_consumed'); consumed = true;
    let firstFailure = null, phase = 'prepare', startInvokedAt = null, startRepliedAt = null, stopRequestedAt = null;
    let observedStart = false, proof = null, timedOut = false, settled = false, deadlineTimer, stopPromise;
    const failures = []; let clientFailure;
    const collect = async (stage, operation, ms = bounds.cleanupMs) => {
      try { await bounded(operation, ms); } catch { failures.push(stage); }
    };
    const stop = () => {
      stopRequestedAt ??= now();
      stopPromise ??= collect('stop', () => gate.stop());
      return stopPromise;
    };
    try {
      const admission = await bounded(prepare, bounds.cleanupMs);
      phase = 'start'; startInvokedAt = now();
      const started = Promise.resolve().then(() => gate.startWindow()).then(async value => {
        settled = true;
        if (timedOut) { await collect('late-stop', () => gate.stop()); await collect('late-close', () => gate.close()); }
        return value;
      }, error => { settled = true; throw error; });
      await bounded(() => started, bounds.replyMs); startRepliedAt = now();
      if (startRepliedAt - startInvokedAt > bounds.replyMs) throw Error('share_late_reply');
      const deadline = startRepliedAt + bounds.observeMs;
      deadlineTimer = setTimeout(() => { void stop(); }, bounds.observeMs);
      observedStart = gate.state().running === true;
      if (!observedStart) throw Error('share_not_running');
      phase = 'share';
      while (now() < deadline && stopRequestedAt === null) {
        const state = gate.state();
        if (!state.running || state.failure || state.heartbeatSuppressed || state.qualification?.generation !== admission.generation ||
          !Number.isInteger(state.renewalsConfirmed) || state.renewalsConfirmed < 0 || state.renewalsConfirmed > 2) throw Error('share_state_changed');
        const selected = await bounded(async () => {
          const status = await gate.stratumV2Status('share', admission.attemptId, admission.binding);
          if (stopRequestedAt !== null || now() >= deadline) throw Error('share_observation_late');
          return post('/share/observe', { status, state: gate.state() });
        }, Math.min(bounds.readMs, deadline - now()));
        if (stopRequestedAt !== null || now() >= deadline) throw Error('share_observation_late');
        if (selected.selected) { proof = { generation: admission.generation, selectionSha256: selected.selectionSha256,
          renewalsConfirmed: gate.state().renewalsConfirmed }; break; }
        await new Promise(resolve => setTimeout(resolve, Math.min(bounds.pollMs, Math.max(0, deadline - now()))));
      }
      if (!proof) throw Error('share_not_observed');
    } catch (error) { clientFailure = { schema: 'str005-client-failure-v1', ...recoveryFailure(phase, error), observedAtMs: now() }; firstFailure = phase; timedOut = startInvokedAt !== null && !settled; }
    finally {
      clearTimeout(deadlineTimer); const stopping = stop();
      if (clientFailure) await collect('client-failure', () => recordFailure(clientFailure), bounds.readMs);
      await stopping;
      await collect('result', () => record({ firstFailure, observedStart, startInvokedAt, startRepliedAt, stopRequestedAt, proof }), bounds.readMs);
      await collect('recovery', recover); await collect('close', () => gate.close());
      await collect('closed-state', () => record({ closed: gate.state() }), bounds.readMs);
      await collect('release', release, 10000);
    }
    return { complete: firstFailure === null && failures.length === 0, firstFailure, failures, observedStart,
      acceptedShare: proof !== null, lateReplyPending: timedOut && !settled, parityPromotion: false };
  };
}
