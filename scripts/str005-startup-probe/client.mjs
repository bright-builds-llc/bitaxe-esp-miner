/** Browser-only one-shot coordinator. Timers bound requests, never claim physical shutdown. */
export function createCoordinator({ gate, prepare, record, recover, release, now = () => performance.now(),
  sleep = ms => new Promise(resolve => setTimeout(resolve, ms)), limits = {} }) {
  const bounds = { replyMs: 30000, observeMs: 5000, stopRequestMs: 35000, readMs: 30000, stopMs: 150000, closeMs: 150000, ...limits };
  let consumed = false;
  async function bounded(operation, ms) {
    if (!(ms > 0)) throw Error('startup_deadline');
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('startup_timeout')), ms);
    })]); } finally { clearTimeout(timer); }
  }
  return async () => {
    if (consumed) throw Error('startup_consumed'); consumed = true;
    let firstFailure = null, phase = 'prepare', startInvokedAt = null, startRepliedAt = null, stopRequestedAt = null, timedOut = false;
    let maybeProof = null, observedStart = false, requestSettled = false;
    const failures = [];
    const collect = async (stage, operation, limit = bounds.readMs) => {
      try { await bounded(operation, limit); } catch { failures.push(stage); }
    };
    const stop = async () => { stopRequestedAt ??= now(); await collect('stop', () => gate.stop(), bounds.stopMs); };
    try {
      const admission = await bounded(prepare, bounds.stopMs);
      phase = 'start'; startInvokedAt = now();
      const deadline = startInvokedAt + bounds.stopRequestMs;
      const started = Promise.resolve().then(() => gate.startWindow()).then(async value => {
        requestSettled = true;
        if (timedOut) { await stop(); await collect('late-close', () => gate.close(), bounds.closeMs); }
        return value;
      }, error => { requestSettled = true; throw error; });
      await bounded(() => started, Math.min(bounds.replyMs, deadline - now()));
      startRepliedAt = now();
      if (startRepliedAt - startInvokedAt > bounds.replyMs || startRepliedAt >= deadline) throw Error('startup_late_reply');
      const initial = gate.state();
      if (!initial.running || initial.failure || initial.heartbeatSuppressed || initial.renewalsConfirmed !== 0 ||
        initial.qualification?.generation !== admission.generation) throw Error('startup_response_not_running');
      observedStart = true; phase = 'dispatch';
      const initialCount = initial.qualification.work_dispatched;
      if (!Number.isSafeInteger(initialCount) || initialCount < 0) throw Error('startup_counter');
      const observeDeadline = Math.min(deadline, startRepliedAt + bounds.observeMs);
      while (now() < observeDeadline) {
        await bounded(() => gate.refresh(), Math.min(bounds.readMs, observeDeadline - now()));
        if (now() > observeDeadline) throw Error('startup_observation_late');
        const state = gate.state(), q = state.qualification;
        if (!state.running || state.failure || state.heartbeatSuppressed || state.renewalsConfirmed !== 0 || q?.generation !== admission.generation ||
          !Number.isSafeInteger(q.work_dispatched) || q.work_dispatched < initialCount) throw Error('startup_state_changed');
        if (q.work_dispatched > initialCount) {
          maybeProof = { generation: q.generation, initialWorkDispatched: initialCount, workDispatched: q.work_dispatched, state };
          break;
        }
        await sleep(Math.min(100, Math.max(0, observeDeadline - now())));
      }
      if (!maybeProof) throw Error('startup_no_dispatch_increase');
    } catch {
      firstFailure = phase; timedOut = !requestSettled && startInvokedAt !== null;
    } finally {
      // Stop is requested before disk/network persistence and before recovery reads.
      await stop();
      await collect('result', () => record({ firstFailure, observedStart, startInvokedAt, startRepliedAt, stopRequestedAt, proof: maybeProof }));
      await collect('recovery', recover, bounds.stopMs);
      await collect('close', () => gate.close(), bounds.closeMs);
      await collect('closed-state', () => record({ closed: gate.state() }));
      await collect('release', release, 10000);
    }
    return { firstFailure, failures, observedStart, dispatchIncreased: maybeProof !== null, lateReplyPending: !requestSettled && startInvokedAt !== null,
      complete: firstFailure === null && failures.length === 0, parityPromotion: false };
  };
}

/** Each read has its own failure boundary; a missing record never skips Close. */
export async function collectRecovery({ gate, campaignId, attemptId, save, limitMs = 30000 }) {
  const failures = [];
  for (const [stage, operation] of [
    ['state', async () => { await gate.refresh(); return gate.state(); }],
    ['ledger', () => gate.reviewQualificationAttempts()], ['original_budget', () => gate.reviewBudget(campaignId)],
    ['status', async () => { const binding = await gate.stratumV2Possession();
      try { return await gate.stratumV2Status('share', null, binding); }
      catch (error) { if (error?.category !== 'v2_idle_correlation') throw error; return gate.stratumV2Status('share', attemptId, binding); } }],
    ['diagnostics', () => gate.exportDiagnostics()],
  ]) {
    let timer; let active = true;
    try { await Promise.race([Promise.resolve().then(operation).then(value => { if (!active) return; return stage === 'diagnostics' ? value : save(stage, value); }),
      new Promise((_, reject) => { timer = setTimeout(() => reject(Error('recovery_timeout')), limitMs); })]); }
    catch { failures.push(stage); }
    finally { active = false; clearTimeout(timer); }
  }
  if (failures.length) { const error = Error('startup_recovery_incomplete'); error.failures = failures; throw error; }
}
