import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { parseStatus } from '../str005-v2-serial/device.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';

export const LIMITS = Object.freeze({ replyMs: 30000, statusMs: 10000, stopFromReplyMs: 15000, stopFromInvocationMs: 45000 });
export function validateRun(value, context) {
  object(value, ['firstFailure', 'observedStart', 'startInvokedAt', 'startRepliedAt', 'stopRequestedAt', 'proof']);
  check([null, 'prepare', 'start', 'status'].includes(value.firstFailure) && typeof value.observedStart === 'boolean', 'status_repro_run_shape');
  for (const key of ['startInvokedAt', 'startRepliedAt', 'stopRequestedAt'])
    check(value[key] === null || Number.isFinite(value[key]) && value[key] >= 0, 'status_repro_run_clock');
  if (value.proof !== null) {
    object(value.proof, ['generation', 'observedAtMs']);
    check(Number.isSafeInteger(value.proof.generation) && value.proof.generation >= 0 &&
      Number.isFinite(value.proof.observedAtMs) && value.proof.observedAtMs >= 0 && value.observedStart === true,
    'status_repro_status_proof');
  }
  return structuredClone(value);
}
/** One projected known-attempt status; later failures remain independently collectible. */
export function routePolicy(now = Date.now) {
  let observed = false;
  return { issuanceCounts: { authorizationCount: 1, renewalCount: 0 }, validateRun,
    async extraRoute({ path, input, save, before, observeReady, context }) {
      if (path === '/status-repro/late-completion') {
        object(input, ['stopComplete', 'closeComplete']);
        check(before && typeof input.stopComplete === 'boolean' && typeof input.closeComplete === 'boolean',
          'status_repro_late_completion');
        await save('late-completion.json', input);
        return { handled: true, value: { recorded: true } };
      }
      if (path !== '/status-repro/observe') return;
      await observeReady(); object(input, ['status', 'state']); check(!observed && before, 'status_repro_observation_consumed'); observed = true;
      const status = parseStatus(input.status), state = input.state;
      validateState(state, context);
      const projected = projectRecoveryPart('status', status, { ...context, attemptId: before.attempt.id });
      await save('status-observation.json', { status: projected, running: state.running, renewalsConfirmed: state.renewalsConfirmed,
        generation: state.qualification?.generation ?? null, capturedAtUnixMs: now() });
      const record = status.record;
      check(status.scope === 'share' && record && record.attemptId === before.attempt.id &&
        record.bootOrdinal === context.expectedBootOrdinal && record.workerGeneration === state.qualification?.generation &&
        state.running === true && state.renewalsConfirmed === 0 && !state.failure && !state.heartbeatSuppressed,
      'status_repro_record_unverified');
      return { handled: true, value: { observed: true, generation: record.workerGeneration } };
    } };
}
