import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { validateRecoveryParts, recoveryConclusion } from '../str005-v2-serial/recovery-evidence.mjs';
import { normalStopVerified } from '../str005-startup-probe/normal-stop.mjs';
import { check, object, digest } from '../str005-v2-serial/values.mjs';
export function validateRun(value) {
  object(value, ['firstFailure', 'observedStart', 'startInvokedAt', 'startRepliedAt', 'stopRequestedAt', 'proof']);
  check([null, 'prepare', 'start', 'share', 'renewal'].includes(value.firstFailure) && typeof value.observedStart === 'boolean', 'share_run_shape');
  for (const key of ['startInvokedAt', 'startRepliedAt', 'stopRequestedAt']) check(value[key] === null || Number.isFinite(value[key]) && value[key] >= 0, 'share_run_clock');
  if (value.proof) {
    object(value.proof, ['generation', 'selectionSha256', 'renewalsConfirmed']); digest(value.proof.selectionSha256);
    check(Number.isSafeInteger(value.proof.generation) && value.proof.generation > 0 &&
      Number.isInteger(value.proof.renewalsConfirmed) && value.proof.renewalsConfirmed >= 0 && value.proof.renewalsConfirmed <= 2, 'share_run_proof');
  }
  return structuredClone(value);
}
/** Live independent PoW/ACK verification and terminal normal-stop are separate proofs. */
export function judge(parts, context) {
  const blockers = [], run = parts.run ? validateRun(parts.run) : null, before = parts.before;
  if (!run?.observedStart || !run.proof || run.firstFailure !== null) blockers.push('share_start_or_ack_unproven');
  if (!run || run.startInvokedAt === null || run.startRepliedAt === null || run.stopRequestedAt === null ||
    run.startRepliedAt < run.startInvokedAt || run.startRepliedAt - run.startInvokedAt > 30000 ||
    run.stopRequestedAt < run.startRepliedAt || run.stopRequestedAt - run.startRepliedAt > (context.observe_window_ms ?? 45000))
    blockers.push('share_stop_request_bound_unproven');
  if (before) { validateLedger(before.ledger); requireExhaustedOriginal(before.original_budget); }
  else blockers.push('share_before_missing');
  const recovery = parts.recovery ?? {};
  validateRecoveryParts(recovery, { ...context, attemptId: before?.attempt?.id ?? context.attemptId });
  blockers.push(...recoveryConclusion(recovery).blockers);
  if (!before || !recovery.ledger || recovery.ledger.pending || recovery.ledger.next_ordinal !== before.ledger.next_ordinal + 1 ||
    recovery.ledger.last_completed_ordinal !== before.ledger.next_ordinal || recovery.ledger.total_charged_ms !== before.ledger.total_charged_ms + 180000)
    blockers.push('share_charge_completion_unproven');
  const actualRenewals = recovery.state?.renewalsConfirmed;
  if (!Number.isInteger(actualRenewals) || actualRenewals < (run?.proof?.renewalsConfirmed ?? 0) || actualRenewals > 2) blockers.push('share_renewal_count_unproven');
  const minimumRenewals = context.minimum_renewals ?? 0;
  if (minimumRenewals > 0 && (!Number.isInteger(actualRenewals) || actualRenewals < minimumRenewals ||
    (run?.proof?.renewalsConfirmed ?? 0) < minimumRenewals)) blockers.push('share_renewal_minimum_unproven');
  const generation = run?.proof?.generation;
  if (!generation || !normalStopVerified(recovery.status?.record, recovery.state, generation)) blockers.push('share_normal_stop_unproven');
  if (!generation || recovery.state?.authorizationRecovery?.matched !== true || recovery.state.authorizationRecovery.generation !== generation)
    blockers.push('share_checkpoint_unproven');
  if (parts.shareVerified !== true) blockers.push('share_independent_proof_unproven');
  if (parts.hostReleased !== true) blockers.push('share_host_release_unproven');
  return { schema: 'str005-share-result-v1', complete: blockers.length === 0, blockers, parity_promotion: false,
    renewals_confirmed: Number.isInteger(actualRenewals) ? actualRenewals : null, accepted_share_verified: parts.shareVerified === true };
}
