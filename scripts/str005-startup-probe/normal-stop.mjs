/** Join native Stop timing to retained V2 cleanup without relabeling its outcome. */
export function normalStopVerified(record, state, generation) {
  const maybeQualification = state?.qualification;
  if (!record || record.state !== 'terminal' || record.scope !== 'share' || record.workerGeneration !== generation ||
      maybeQualification?.generation !== generation || maybeQualification.revocation_reason !== 'restoration_requested' ||
      maybeQualification.safe_stop_complete !== true || maybeQualification.budget_complete !== true ||
      !Number.isSafeInteger(maybeQualification.gate_closed_ms) || !Number.isSafeInteger(maybeQualification.shutdown_started_ms) ||
      maybeQualification.shutdown_started_ms < maybeQualification.gate_closed_ms ||
      maybeQualification.shutdown_started_ms - maybeQualification.gate_closed_ms > 3000 || record.secondaryFailures.length !== 0) return false;
  // The native millisecond field is floored. Same-millisecond ordering is unproved.
  const afterGateClosedUs = (maybeQualification.gate_closed_ms + 1) * 1000;
  if (!Number.isSafeInteger(afterGateClosedUs) || record.resources.socketClosed !== true ||
      record.resources.workerQuiescent !== true || record.resources.fenceRetained !== false ||
      !Number.isSafeInteger(record.resources.socketClosedAtUs) || record.resources.socketClosedAtUs < afterGateClosedUs ||
      !Number.isSafeInteger(record.resources.workerQuiescentAtUs) || record.resources.workerQuiescentAtUs < afterGateClosedUs ||
      !Number.isSafeInteger(record.terminalAtDeviceUs) || record.terminalAtDeviceUs < record.resources.workerQuiescentAtUs ||
      record.resources.socketClosedAtUs > record.resources.workerQuiescentAtUs) return false;
  const maybeFailure = record.firstFailure;
  if (maybeFailure === null) return record.outcome === 'accepted';
  if (record.outcome !== 'rejected' || maybeFailure.stage !== 'worker_quiescent' ||
      !Number.isSafeInteger(maybeFailure.atDeviceUs) || maybeFailure.atDeviceUs < afterGateClosedUs || maybeFailure.atDeviceUs > record.terminalAtDeviceUs) return false;
  if (maybeFailure.category === 'authority') return true;
  // A clean pre-share finish emits Evidence at terminalization, not at an earlier operation.
  return maybeFailure.category === 'evidence' && maybeFailure.atDeviceUs === record.terminalAtDeviceUs &&
    !record.shareFacts.some(fact => fact.writeCompletedAtDeviceUs !== null && fact.ackAtDeviceUs !== null && fact.ackAcceptedCount === 1);
}
