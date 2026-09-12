import { isDeepStrictEqual } from "node:util";
import { exactObject, requireCondition } from "./contract.mjs";
import { validateState } from "./judge.mjs";

export const RECOVERY_SCHEMA = "fixed-usb-iterative-context-v5";
export function validateRecoveryPhase(context) {
  requireCondition(context.schema === RECOVERY_SCHEMA && ["loss", "resume"].includes(context.recovery_phase) &&
    context.qualification_attempt.purpose === "diagnostic" && context.qualification_attempt.maximumActiveMilliseconds === 30000,
  "recovery_policy");
}

export function validateLossReceipt(input, context, records) {
  validateRecoveryPhase(context);
  requireCondition(context.recovery_phase === "loss", "recovery_loss_scope");
  exactObject(input, ["receipt", "before", "after"]);
  const { receipt, before, after } = input;
  validateState(before, context); validateState(after, context);
  exactObject(receipt, ["schema", "generation", "workDispatched", "workGateRemainingMs", "ownershipReleased", "controlRecordsSent"]);
  const q = before.qualification;
  requireCondition(receipt.schema === "worker-mining-interruption-v1" && receipt.ownershipReleased === true &&
    receipt.controlRecordsSent === 0 && before.running && before.connected && !before.failure &&
    q?.attempt?.ordinal === context.qualification_attempt.ordinal && q.work_dispatched > 0 &&
    q.work_gate_remaining_ms > 3000 && q.gate_closed_ms === null && !q.safe_stop_complete &&
    receipt.generation === q.generation && receipt.workDispatched === q.work_dispatched &&
    receipt.workGateRemainingMs === q.work_gate_remaining_ms, "recovery_work_checkpoint");
  requireCondition(!after.connected && !after.running && after.serialOwnershipReleased && !after.failure &&
    after.qualification?.generation === q.generation &&
    before.preservation?.baseline_id === after.preservation?.baseline_id, "recovery_loss_release");
  const checkpoint = before.authorizationRecovery;
  requireCondition(checkpoint?.generation === q.generation && checkpoint.matched === null &&
    isDeepStrictEqual(after.authorizationRecovery, checkpoint), "recovery_authorization_checkpoint");
  const maybeBefore = records.findLast((record) => isDeepStrictEqual(record.state, before));
  const maybeAfter = records.findLast((record) => isDeepStrictEqual(record.state, after));
  requireCondition(maybeBefore && maybeAfter && maybeAfter.sequence > maybeBefore.sequence &&
    maybeAfter.sequence === records.at(-1)?.sequence, "recovery_loss_journal_binding");
  return { kind: "transport_interrupted", generation: q.generation, after_sequence: maybeBefore.sequence,
    released_sequence: maybeAfter.sequence, receipt };
}

export function judgeRecovery(context, records, fault, windowResult, traceEvidence = []) {
  validateRecoveryPhase(context);
  requireCondition(records.every((record) => !record.state.failure), "recovery_browser_failure");
  const observed = records.filter((record) => record.state.qualification?.attempt?.ordinal === context.qualification_attempt.ordinal);
  const maybeTerminal = observed.at(-1)?.state.qualification;
  requireCondition(maybeTerminal?.safe_stop_complete, "recovery_terminal_missing");
  requireCondition(observed.every(record => record.state.qualification.generation === maybeTerminal.generation), "recovery_generation_changed");
  if (context.recovery_phase === "resume") {
    requireCondition(maybeTerminal.generation !== context.recovery_loss_generation, "recovery_generation_reused");
    requireCondition(fault === undefined && observed.some((record) => record.state.running &&
      record.state.qualification.work_dispatched > 0), "recovery_resumed_work_missing");
    return { ...windowResult, recovery_phase: "resume", reauthorized_work_verified: true };
  }
  requireCondition(fault, "recovery_fault_missing");
  exactObject(fault, ["kind", "generation", "after_sequence", "released_sequence", "receipt"]);
  requireCondition(fault.kind === "transport_interrupted", "recovery_fault_missing");
  const maybeBefore = records.find((record) => record.sequence === fault.after_sequence)?.state;
  const maybeAfter = records.find((record) => record.sequence === fault.released_sequence)?.state;
  const expected = validateLossReceipt({ receipt: fault.receipt, before: maybeBefore, after: maybeAfter }, context,
    records.filter((record) => record.sequence <= fault.released_sequence));
  const linkClosedSupported = traceEvidence.some(entry => entry.stage === "recovered" && entry.source === "device" && entry.link_closed_verified === true);
  requireCondition(isDeepStrictEqual(expected, fault) && maybeTerminal.generation === fault.generation &&
    (maybeTerminal.revocation_reason === "heartbeat_timeout" ||
      (maybeTerminal.revocation_reason === "link_closed" && linkClosedSupported)), "recovery_stop_binding");
  const checkpoint = maybeBefore.authorizationRecovery;
  requireCondition(!records.some(record => record.sequence > fault.released_sequence && record.state.authorizationRecovery?.matched === false),
    "recovery_authorization_changed");
  const maybeRecovered = records.find((record) => record.sequence > fault.released_sequence && record.state.status === "ready" &&
    record.state.connected && record.state.deviceBaselineConfirmed === true && record.state.deviceLeaseInactive &&
    record.state.helloRecovery !== undefined && record.state.preservation?.device_identity_match &&
    record.state.preservation.settings_match && record.state.preservation.mine_on_boot === false &&
    record.state.preservation.baseline_id === maybeBefore.preservation?.baseline_id &&
    record.state.qualification?.generation === fault.generation && record.state.qualification.safe_stop_complete &&
    record.state.authorizationRecovery?.checkpointId === checkpoint.checkpointId &&
    record.state.authorizationRecovery.generation === checkpoint.generation && record.state.authorizationRecovery.matched === true);
  requireCondition(maybeRecovered, "recovery_fresh_admission_missing");
  return { ...windowResult, recovery_phase: "loss", fresh_admission_verified: true,
    revocation_reason: maybeTerminal.revocation_reason,
    discarded_records: maybeRecovered.state.helloRecovery.discardedRecords,
    discarded_replies: maybeRecovered.state.helloRecovery.discardedReplies ?? 0,
    stale_reply_hardware_coverage: (maybeRecovered.state.helloRecovery.discardedReplies ?? 0) > 0 };
}
