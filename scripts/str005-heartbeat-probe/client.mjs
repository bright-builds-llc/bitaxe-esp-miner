import { LIMITS } from './limits.mjs';
/** One invocation; suppression uses Gate's real fault checkpoint, never a fabricated status. */
export function createHeartbeatCoordinator({ gate, prepare, readStatus, record, recover, release, observer, claimStart = async () => {}, recordSuppression = async () => {},
  now = () => performance.now(), sleep = ms => new Promise(resolve => setTimeout(resolve, ms)), limits = {} }) {
  const bounds = { ...LIMITS, ...limits };
  let consumed = false;
  async function bounded(operation, milliseconds) {
    if (!(milliseconds > 0)) throw Error('heartbeat_deadline');
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('heartbeat_timeout')), milliseconds);
    })]); } finally { clearTimeout(timer); }
  }
  return async () => {
    if (consumed) throw Error('heartbeat_consumed'); consumed = true;
    const failures = [], run = { firstFailure: null, observedStart: false, startInvokedAt: null, startRepliedAt: null,
      suppressionRequestedAt: null, suppressionConfirmedAt: null, stopRequestedAt: null,
      dispatchStatus: null, headroom: null, suppressedState: null };
    let phase = 'prepare', settled = false, timedOut = false;
    async function collect(stage, operation, limit) { try { await bounded(operation, limit); } catch { failures.push(stage); } }
    const stop = async () => { run.stopRequestedAt ??= now(); await collect('stop', () => gate.stop(), bounds.stopMs); };
    try {
      const admission = await bounded(prepare, bounds.stopMs);
      // The observer must already own its connected passive stream before Start.
      await bounded(() => observer.requireArmed(admission), bounds.readMs);
      await bounded(() => claimStart(admission), bounds.readMs);
      phase = 'start'; run.startInvokedAt = now();
      const started = Promise.resolve().then(() => gate.startWindow()).then(async value => {
        settled = true;
        if (timedOut) { await stop(); await collect('late-close', () => gate.close(), bounds.closeMs); }
        return value;
      }, error => { settled = true; throw error; });
      await bounded(() => started, bounds.replyMs); run.startRepliedAt = now();
      if (run.startRepliedAt - run.startInvokedAt > bounds.replyMs) throw Error('heartbeat_late_start');
      const admittedState = gate.state();
      if (!admittedState.running || admittedState.failure || admittedState.qualification?.generation !== admission.generation) throw Error('heartbeat_start_state');
      run.observedStart = true; phase = 'dispatch';
      const deadline = Math.min(run.startRepliedAt + bounds.dispatchMs, run.startInvokedAt + bounds.suppressAfterInvocationMs);
      while (now() < deadline) {
        await bounded(() => gate.refresh(), deadline - now());
        const state = gate.state(), q = state.qualification;
        if (!state.running || state.failure || state.heartbeatSuppressed || state.renewalsConfirmed !== 0 || q?.generation !== admission.generation)
          throw Error('heartbeat_start_state');
        if (q.work_dispatched > 0) {
          const status = await bounded(readStatus, deadline - now());
          if (status.record?.workerGeneration !== admission.generation || status.record.scope !== 'share' ||
              !status.record.events.some(event => event.kind === 'work_ready') || !status.record.events.some(event => event.kind === 'asic_dispatch'))
            throw Error('heartbeat_dispatch_missing');
          run.dispatchStatus = { schema: 'str005-recovery-status-v1', scope: status.scope, state: status.state, record: status.record,
            observation: Object.fromEntries(['bootOrdinal', 'workerGeneration', 'serialTransportEpoch', 'observedAtUs', 'clockValid'].map(key => [key, status.observation[key]])) }; break;
        }
        await sleep(Math.min(50, Math.max(0, deadline - now())));
      }
      if (!run.dispatchStatus || now() >= deadline) throw Error('heartbeat_dispatch_timeout');
      phase = 'suppress'; run.suppressionRequestedAt = now();
      run.headroom = await bounded(() => gate.suppressHeartbeats(), deadline - now());
      run.suppressionConfirmedAt = now(); run.suppressedState = gate.state();
      if (run.suppressionConfirmedAt > deadline || !run.suppressedState.heartbeatSuppressed ||
          run.suppressedState.authorizationRecovery?.generation !== admission.generation || run.suppressedState.authorizationRecovery?.matched !== null ||
          run.headroom.leaseRemainingMs < bounds.headroomMs || run.headroom.workGateRemainingMs < bounds.headroomMs)
        throw Error('heartbeat_suppression_unproved');
      await bounded(() => recordSuppression({ clientConfirmedAtMs: run.suppressionConfirmedAt, headroom: run.headroom, state: run.suppressedState }), bounds.readMs);
      phase = 'observe';
      // No serial status/Stop poll here: allow native expiry, then five seconds of passive tail.
      if (observer.waitTail) await bounded(() => observer.waitTail(bounds.passiveWaitMs), bounds.passiveWaitMs + bounds.readMs);
      else await sleep(bounds.passiveWaitMs);
      await bounded(() => observer.requireAlive(), bounds.readMs);
    } catch { run.firstFailure = phase; timedOut = !settled && run.startInvokedAt !== null; }
    finally {
      // Cleanup precedes fallible evidence writes, on both the happy and failed paths.
      await stop();
      await collect('record', () => record(run), bounds.readMs);
      await collect('recovery', recover, bounds.stopMs);
      await collect('close', () => gate.close(), bounds.closeMs);
      await collect('closed-state', () => record({ closed: gate.state() }), bounds.readMs);
      await collect('observer-release', () => observer.stop(), bounds.readMs);
      await collect('release', release, 10000);
    }
    return { firstFailure: run.firstFailure, failures, suppressionConfirmed: run.suppressedState?.heartbeatSuppressed === true,
      complete: run.firstFailure === null && failures.length === 0, parityPromotion: false };
  };
}
