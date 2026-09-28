import { resolve } from 'node:path';
import { createObserverRoutes } from '../str005-v2-serial/observer.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { check, object, sha256, uint } from '../str005-v2-serial/values.mjs';
/** Additional routes use the same physical owner and queue as the one-shot startup supervisor. */
export function createHeartbeatOwner(root, context, { verify, now = Date.now, operations = {} }) {
  const rows = [], devices = [];
  let failed = false, requested = false, confirmed, startClaim, before;
  const save = (name, value) => writeNew(resolve(root, name), value);
  const journal = {
    lastState: () => rows.at(-1),
    async state(phase, state, atHostMs) { validateState(state, context); const row = { sequence: rows.length + 1, phase, state, atHostMs }; rows.push(row); return row; },
  };
  const observer = createObserverRoutes(root, context, { now, ready: () => check(!failed, 'heartbeat_owner_failed'), failed: () => failed,
    fail: () => { failed = true; }, journal, operations });
  async function extraRoute({ path, input, ready, observeReady, before: baseline }) {
    before ??= baseline;
    if (path.startsWith('/observer/')) {
      await verify(); await ready();
      if (!rows.length && before) await journal.state('candidate', before.state, now());
      if (path === '/observer/start') await journal.state('candidate', input.state, now());
      return { handled: true, value: await observer.handle(path, input) };
    }
    if (!path.startsWith('/heartbeat/')) return undefined;
    if (path === '/heartbeat/observations') {
      object(input, ['status']); check(input.status?.record && before && devices.length === 0, 'heartbeat_observation');
      validateRecoveryParts({ status: input.status }, { ...context, attemptId: before.attempt.id });
      devices.push({ sequence: devices.length + 1, atHostMs: now(), record: input.status.record });
      await save('observer-state-rows.json', rows); await save('observer-device-rows.json', devices);
      return { handled: true, value: { recorded: true } };
    }
    if (path === '/heartbeat/release') { object(input, []); return { handled: true, value: await observer.finish() }; }
    if (path === '/heartbeat/alive') { object(input, []); observer.alive(); return { handled: true, value: { alive: true } }; }
    await observeReady(); observer.alive();
    if (path === '/heartbeat/start-claim') {
      object(input, []); check(!requested && before && observer.binding(), 'heartbeat_start_claim'); requested = true;
      startClaim = { contextSha256: sha256(JSON.stringify(context)), atHostMs: now() }; await save('heartbeat-start-claim.json', startClaim);
      return { handled: true, value: { claimed: true } };
    }
    if (path === '/heartbeat/suppressed') {
      object(input, ['clientConfirmedAtMs', 'headroom', 'state']);
      check(requested && !confirmed, 'heartbeat_suppression_once');
      check(Number.isFinite(input.clientConfirmedAtMs) && input.clientConfirmedAtMs >= 0, 'heartbeat_client_clock');
      validateState(input.state, context);
      const h = input.headroom;
      object(h, ['schema', 'workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']);
      for (const key of ['workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']) uint(h[key]);
      check(h.schema === 'worker-v2-fault-headroom-v1' && h.leaseRemainingMs >= 5000 && h.workGateRemainingMs >= 5000 &&
        input.state.heartbeatSuppressed && input.state.renewalsConfirmed === 0 && input.state.qualification?.generation === h.workerGeneration &&
        input.state.authorizationRecovery?.generation === h.workerGeneration && input.state.authorizationRecovery.matched === null &&
        observer.binding().workerGeneration === h.workerGeneration, 'heartbeat_suppression');
      confirmed = { confirmedAtHostMs: now(), ...input }; observer.markFault(confirmed.confirmedAtHostMs);
      await journal.state('candidate', input.state, confirmed.confirmedAtHostMs); await save('suppression-confirmed.json', confirmed);
      return { handled: true, value: { recorded: true } };
    }
    if (path === '/heartbeat/tail') {
      object(input, []); check(confirmed, 'heartbeat_not_suppressed');
      while (now() - confirmed.confirmedAtHostMs < 8000) {
        observer.alive(); check(!failed, 'heartbeat_observer_failed');
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      observer.alive(); return { handled: true, value: { tailObserved: true } };
    }
    check(false, 'heartbeat_route');
  }
  return { extraRoute,
    async release() { await observer.finish(); },
  };
}
