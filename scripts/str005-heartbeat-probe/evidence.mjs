import { check, uint, object } from '../str005-v2-serial/values.mjs';
import { validateRecordProgress } from '../str005-v2-serial/device.mjs';
import { parseDeviceRecord } from '../str005-v2-serial/device-record.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { validateRecoveryParts, recoveryConclusion } from '../str005-v2-serial/recovery-evidence.mjs';
import { LIMITS } from './limits.mjs';
const BINDING = ['bootOrdinal', 'workerGeneration', 'poolSessionGeneration', 'serialTransportEpoch', 'poolTransportEpoch'];
/** Validate the partial producer packet before persistence, retaining failure-only observations. */
export function validateRun(run, context) {
  object(run, ['firstFailure', 'observedStart', 'startInvokedAt', 'startRepliedAt', 'suppressionRequestedAt', 'suppressionConfirmedAt',
    'stopRequestedAt', 'dispatchStatus', 'headroom', 'suppressedState']);
  check([null, 'prepare', 'start', 'dispatch', 'suppress', 'observe'].includes(run.firstFailure) && typeof run.observedStart === 'boolean', 'heartbeat_run_shape');
  for (const key of ['startInvokedAt', 'startRepliedAt', 'suppressionRequestedAt', 'suppressionConfirmedAt', 'stopRequestedAt'])
    check(run[key] === null || Number.isFinite(run[key]) && run[key] >= 0, 'heartbeat_host_clock');
  if (run.dispatchStatus !== null) validateRecoveryParts({ status: run.dispatchStatus }, { ...context, scope: 'share', attemptId: run.dispatchStatus.record?.attemptId });
  if (run.suppressedState !== null) validateState(run.suppressedState, context);
  if (run.headroom !== null) {
    object(run.headroom, ['schema', 'workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']);
    check(run.headroom.schema === 'worker-v2-fault-headroom-v1', 'heartbeat_headroom');
    for (const key of ['workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']) uint(run.headroom[key]);
  }
  return structuredClone(run);
}
/** Separate heartbeat completion from share acceptance; every native fact keeps its original outcome. */
export function judgeFault(run, record, qualification) {
  object(run, ['firstFailure', 'observedStart', 'startInvokedAt', 'startRepliedAt', 'suppressionRequestedAt', 'suppressionConfirmedAt',
    'stopRequestedAt', 'dispatchStatus', 'headroom', 'suppressedState']);
  check(run.firstFailure === null && run.observedStart === true, 'heartbeat_main_failed');
  for (const key of ['startInvokedAt', 'startRepliedAt', 'suppressionRequestedAt', 'suppressionConfirmedAt', 'stopRequestedAt'])
    check(Number.isFinite(run[key]) && run[key] >= 0, 'heartbeat_host_clock');
  check(run.startRepliedAt >= run.startInvokedAt && run.startRepliedAt - run.startInvokedAt <= LIMITS.replyMs &&
    run.suppressionRequestedAt >= run.startRepliedAt && run.suppressionConfirmedAt >= run.suppressionRequestedAt &&
    run.suppressionConfirmedAt - run.startRepliedAt <= LIMITS.suppressAfterReplyMs &&
    run.suppressionConfirmedAt - run.startInvokedAt <= LIMITS.suppressAfterInvocationMs &&
    run.stopRequestedAt - run.suppressionConfirmedAt >= LIMITS.passiveWaitMs, 'heartbeat_effect_timing');
  validateRecoveryParts({ status: run.dispatchStatus }, { scope: 'share', attemptId: record.attemptId });
  const initial = parseDeviceRecord(run.dispatchStatus.record), headroom = run.headroom, state = run.suppressedState;
  check(initial && initial.scope === 'share' && initial.state === 'running' && initial.firstFailure === null &&
    BINDING.every(key => initial[key] === record[key]) && initial.attemptId === record.attemptId, 'heartbeat_generation');
  validateRecordProgress(initial, record);
  const work = initial.events.find(event => event.kind === 'work_ready'), dispatched = initial.events.filter(event => event.kind === 'asic_dispatch');
  check(work && dispatched.length > 0 && dispatched.every(event => event.atDeviceUs >= work.atDeviceUs), 'heartbeat_dispatch_missing');
  object(headroom, ['schema', 'workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']);
  for (const key of ['workerGeneration', 'headroomObservedAtDeviceUs', 'leaseRemainingMs', 'workGateRemainingMs']) uint(headroom[key]);
  check(headroom.schema === 'worker-v2-fault-headroom-v1' && headroom.workerGeneration === record.workerGeneration &&
    headroom.leaseRemainingMs >= LIMITS.headroomMs && headroom.leaseRemainingMs <= 60000 &&
    headroom.workGateRemainingMs >= LIMITS.headroomMs && headroom.workGateRemainingMs <= 164450 &&
    initial.observedAtUs !== null && headroom.headroomObservedAtDeviceUs >= Math.floor(initial.observedAtUs / 1000) * 1000,
  'heartbeat_headroom');
  check(state.running && state.heartbeatSuppressed && state.renewalsConfirmed === 0 && !state.failure &&
    state.qualification?.generation === record.workerGeneration && state.qualification.work_dispatched > 0 &&
    state.authorizationRecovery?.generation === record.workerGeneration && state.authorizationRecovery.matched === null,
  'heartbeat_checkpoint');
  const q = qualification;
  check(q?.generation === record.workerGeneration && q.revocation_reason === 'heartbeat_timeout', 'heartbeat_native_reason');
  for (const key of ['last_valid_heartbeat_ms', 'gate_closed_ms', 'shutdown_started_ms']) uint(q[key]);
  const closed = q.gate_closed_ms * 1000, shutdown = q.shutdown_started_ms * 1000;
  check(Number.isSafeInteger(closed) && Number.isSafeInteger(shutdown) &&
    q.gate_closed_ms - q.last_valid_heartbeat_ms >= LIMITS.heartbeatExpiryMs &&
    q.gate_closed_ms - q.last_valid_heartbeat_ms <= LIMITS.shutdownInitiationMs &&
    q.shutdown_started_ms >= q.gate_closed_ms && q.shutdown_started_ms - q.last_valid_heartbeat_ms <= LIMITS.shutdownInitiationMs &&
    headroom.headroomObservedAtDeviceUs <= closed, 'heartbeat_native_deadline');
  check(record.authorityDeadlineDeviceUs !== null && headroom.headroomObservedAtDeviceUs + headroom.workGateRemainingMs * 1000 <=
    record.authorityDeadlineDeviceUs - LIMITS.shutdownReserveMs * 1000, 'heartbeat_shutdown_reserve');
  check(record.observedAtUs >= closed + LIMITS.postFaultTailMs * 1000, 'heartbeat_device_tail');
  for (const event of record.events) if (event.kind === 'asic_dispatch' || event.kind === 'submission')
    check(event.atDeviceUs <= closed, 'heartbeat_activity_after_revocation');
  for (const fact of record.shareFacts) check(fact.dispatchedAtDeviceUs <= closed &&
    (fact.writeStartedAtDeviceUs === null || fact.writeStartedAtDeviceUs <= closed), 'heartbeat_activity_after_revocation');
  for (const [kind, native] of [['revoked', closed], ['shutdown', shutdown]]) {
    const events = record.events.filter(event => event.kind === kind);
    check(events.length === 1 && events[0].atDeviceUs >= native && events[0].atDeviceUs <= record.observedAtUs, 'heartbeat_event_join');
  }
  // Firmware appends safety events when its poller observes them, which may follow the terminal record;
  // the qualification atoms (safe_stop_stage, safe_stop_complete) carry the native stop ordering.
  const cooled = record.events.filter(event => event.kind === 'cooled');
  check(cooled.length === 1 && cooled[0].atDeviceUs >= shutdown && cooled[0].atDeviceUs <= record.observedAtUs, 'heartbeat_cooling_event');
  check(record.secondaryFailures.length === 0, 'heartbeat_secondary_failure');
  if (record.firstFailure !== null) {
    const failure = record.firstFailure;
    check(record.outcome === 'rejected' && failure.stage === 'worker_quiescent' && failure.atDeviceUs >= closed + 1000 &&
      failure.atDeviceUs <= record.terminalAtDeviceUs && (failure.category === 'authority' ||
      failure.category === 'evidence' && failure.atDeviceUs === record.terminalAtDeviceUs), 'heartbeat_unrelated_failure');
  } else check(record.outcome === 'accepted', 'heartbeat_terminal_outcome');
  return { heartbeatToRevocationMs: q.gate_closed_ms - q.last_valid_heartbeat_ms,
    heartbeatToShutdownMs: q.shutdown_started_ms - q.last_valid_heartbeat_ms, acceptedShareRequired: false };
}
/** All recovery values must be authenticated producer projections, not synthetic success flags. */
export function judgeHeartbeat(parts, context) {
  validateLedger(parts.before.ledger); requireExhaustedOriginal(parts.before.original_budget);
  validateState(parts.before.state, context); validateState(parts.run.suppressedState, context);
  validateRecoveryParts(parts.recovery, { ...context, scope: 'share', attemptId: parts.before.attempt.id });
  const recovery = parts.recovery, blockers = recoveryConclusion(recovery).blockers;
  check(blockers.length === 0, 'heartbeat_recovery_incomplete');
  const record = recovery.status.record, q = recovery.state.qualification;
  const timing = judgeFault(parts.run, record, q), checkpoint = recovery.state.authorizationRecovery;
  check(checkpoint?.matched === true && checkpoint.generation === record.workerGeneration &&
    checkpoint.checkpointId === parts.run.suppressedState.authorizationRecovery.checkpointId &&
    recovery.state.renewalsConfirmed === 0 && recovery.closed.renewalsConfirmed === 0, 'heartbeat_checkpoint_recovery');
  check(recovery.status.observation.bootOrdinal === parts.run.dispatchStatus.observation.bootOrdinal &&
    recovery.status.observation.serialTransportEpoch !== parts.run.dispatchStatus.observation.serialTransportEpoch &&
    recovery.state.preservation.baseline_id === parts.before.state.preservation.baseline_id,
  'heartbeat_fresh_recovery');
  const before = parts.before.ledger, after = recovery.ledger;
  check(!before.pending && !after.pending && after.next_ordinal === before.next_ordinal + 1 &&
    after.last_completed_ordinal === before.next_ordinal && after.total_charged_ms === before.total_charged_ms + 180000,
  'heartbeat_accounting');
  check(q.safe_stop_complete && q.safe_stop_stage === 'fan_paused' && q.budget_complete && q.attempt?.complete &&
    q.attempt.ordinal === before.next_ordinal && q.attempt.purpose === 'normal' && q.attempt.reserved_ms === 180000 &&
    q.active_limit_ms === 180000 && q.shutdown_budget_ms === LIMITS.shutdownReserveMs && q.active_ms > 0 && q.active_ms <= 180000 &&
    q.owner_resources?.phase === 'shutdown_complete' && q.owner_resources.generation === record.workerGeneration &&
    q.owner_resources.stack_free_bytes >= 4096 && q.watchdog_alive && !q.mine_on_boot &&
    q.voltage_fresh && q.power_fresh && q.temperature_fresh && q.fan_fresh && q.voltage_volts >= 4.5 && q.voltage_volts <= 5.5 &&
    q.power_watts >= 0 && q.power_watts <= 15 && q.chip_temp_celsius <= 45 && q.fan_rpm > 0, 'heartbeat_cooling');
  check(parts.hostReleased === true, 'heartbeat_host_release');
  return { schema: 'str005-heartbeat-result-v1', complete: true, ...timing, parityPromotion: false };
}
