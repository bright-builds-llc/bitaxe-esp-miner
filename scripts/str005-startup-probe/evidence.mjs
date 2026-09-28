import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { recoveryConclusion, validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { LIMITS } from './contract.mjs';
import { normalStopVerified } from './normal-stop.mjs';
export function baseline(state, context) {
  validateState(state, context); const p = state.preservation;
  check(state.connected && !state.running && !state.heartbeatSuppressed && state.renewalsConfirmed === 0 &&
    state.deviceLeaseInactive && state.deviceBaselineConfirmed && p?.settings_match && p.device_identity_match &&
    p.authorization_high_water_match && !p.mine_on_boot && !state.failure, 'startup_baseline');
}
export function validateRun(value, context) {
  object(value, ['firstFailure', 'observedStart', 'startInvokedAt', 'startRepliedAt', 'stopRequestedAt', 'proof']);
  check([null, 'prepare', 'start', 'dispatch'].includes(value.firstFailure) && typeof value.observedStart === 'boolean', 'startup_run_shape');
  for (const key of ['startInvokedAt', 'startRepliedAt', 'stopRequestedAt']) check(value[key] === null || (Number.isFinite(value[key]) && value[key] >= 0), 'startup_run_clock');
  if (value.proof !== null) {
    object(value.proof, ['generation', 'initialWorkDispatched', 'workDispatched', 'state']);
    const p = value.proof, q = p.state.qualification; validateState(p.state, context);
    check(Number.isInteger(p.generation) && p.generation > 0 && Number.isSafeInteger(p.initialWorkDispatched) && p.initialWorkDispatched >= 0 &&
      Number.isSafeInteger(p.workDispatched) && p.workDispatched > p.initialWorkDispatched && q?.generation === p.generation &&
      q.work_dispatched === p.workDispatched && p.state.running && !p.state.failure && !p.state.heartbeatSuppressed && p.state.renewalsConfirmed === 0,
    'startup_dispatch_proof');
  }
  return structuredClone(value);
}
export function judge(parts, context) {
  const blockers = [];
  const run = parts.run ? validateRun(parts.run, context) : null;
  if (!run?.observedStart || !run.proof || run.firstFailure !== null) blockers.push('startup_not_proven');
  if (run && !(run.startInvokedAt !== null && run.startRepliedAt !== null && run.stopRequestedAt !== null &&
      run.startRepliedAt >= run.startInvokedAt && run.startRepliedAt - run.startInvokedAt <= LIMITS.replyMs &&
      run.stopRequestedAt >= run.startRepliedAt && run.stopRequestedAt - run.startInvokedAt <= LIMITS.stopRequestMs &&
      run.stopRequestedAt - run.startRepliedAt <= LIMITS.observeMs)) blockers.push('startup_request_bound_unproven');
  const before = parts.before;
  if (before) { validateLedger(before.ledger); requireExhaustedOriginal(before.original_budget); }
  else blockers.push('before_accounting_missing');
  const recovery = parts.recovery ?? {}, recoveryContext = { ...context, attemptId: before?.attempt?.id ?? context.attemptId };
  validateRecoveryParts(recovery, recoveryContext);
  const restored = recoveryConclusion(recovery); blockers.push(...restored.blockers);
  if (before && recovery.ledger && !(recovery.ledger.next_ordinal === before.ledger.next_ordinal + 1 &&
    recovery.ledger.last_completed_ordinal === before.ledger.next_ordinal && recovery.ledger.total_charged_ms === before.ledger.total_charged_ms + 180000 &&
    recovery.ledger.pending === false)) blockers.push('startup_charge_completion_unproven');
  const record = recovery.status?.record;
  if (run?.proof && (!record || record.workerGeneration !== run.proof.generation || !record.events.some(event => event.kind === 'asic_dispatch')))
    blockers.push('startup_device_dispatch_unproven');
  if (run?.proof && recovery.state && (recovery.state.authorizationRecovery?.matched !== true ||
      recovery.state.authorizationRecovery.generation !== run.proof.generation)) blockers.push('startup_authorization_checkpoint_unproven');
  if (run?.proof && !normalStopVerified(record, recovery.state, run.proof.generation)) blockers.push('startup_normal_stop_unproven');
  if (parts.hostReleased !== true) blockers.push('startup_host_release_unproven');
  return { schema: 'str005-startup-result-v1', complete: blockers.length === 0, blockers, parity_promotion: false,
    historical_resource_proof: false, start_proven: Boolean(run?.observedStart), dispatch_increased: Boolean(run?.proof),
    core_capture_verified: context.captureVerified === true };
}
