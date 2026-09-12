import test from "node:test";
import assert from "node:assert/strict";
import { judgeRecovery, RECOVERY_SCHEMA, validateLossReceipt } from "./recovery-judge.mjs";
import { validateState } from "./judge.mjs";
import { judgeIterative } from "./iterative-judge.mjs";

function fixture(phase = "loss") {
  const context = { schema: RECOVERY_SCHEMA, recovery_phase: phase, required_no_mining_cycles: 4, owner_stack_minimum_bytes: 4096,
    suggested_difficulty: 1000, firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
    qualification_attempt: { ordinal: 5, purpose: "diagnostic", maximumActiveMilliseconds: 30000 } };
  const qualification = { schema: "worker-qualification-v1", generation: 7, active_ms: 1000, generation_elapsed_ms: 2000,
    budget_reserved_ms: 240000, budget_complete: true, active_limit_ms: 30000, shutdown_budget_ms: 15550,
    work_gate_remaining_ms: 10000, submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 1,
    last_valid_heartbeat_ms: 1000, gate_closed_ms: null, shutdown_started_ms: null, safe_stop_complete: false,
    safe_stop_stage: "not_started", revocation_reason: "none", voltage_volts: 5, power_watts: 10, chip_temp_celsius: 40,
    fan_rpm: 2000, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false,
    owner_resources: { schema: "worker-owner-resources-v1", generation: 7, phase: "active", observed_at_ms: "2000",
      heap_free_bytes: 50000, heap_largest_bytes: 12000, stack_free_bytes: 8192 },
    attempt: { schema: "worker-qualification-observation-v1", ordinal: 5, purpose: "diagnostic", maximum_active_ms: 30000,
      reserved_ms: 30000, complete: false, active_ms: 1000 } };
  const before = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit,
    expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256,
    status: "running", connected: true, running: true, heartbeatSuppressed: false, renewalsConfirmed: 0,
    deviceRestorationConfirmed: false, deviceBaselineConfirmed: false, deviceLeaseInactive: false, serialOwnershipReleased: false,
    authorizationRecovery: { schema: "worker-authorization-recovery-v1", checkpointId: Buffer.alloc(16, 2).toString("base64url"), generation: 7, matched: null },
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: Buffer.alloc(16, 1).toString("base64url"),
      device_identity_match: true, settings_match: true, authorization_high_water_match: false, mine_on_boot: false }, qualification };
  const after = { ...before, status: "disconnected", connected: false, running: false, serialOwnershipReleased: true };
  const recovered = { ...before, status: "ready", running: false, deviceRestorationConfirmed: true, deviceBaselineConfirmed: true,
    deviceLeaseInactive: true, authorizationRecovery: { ...before.authorizationRecovery, matched: true }, helloRecovery: { discardedRecords: 1, discardedBytes: 2136, discardedReplies: 0 },
    qualification: { ...qualification, active_ms: 4000, generation_elapsed_ms: 17000, gate_closed_ms: 3800, shutdown_started_ms: 3900,
      safe_stop_complete: true, safe_stop_stage: "fan_paused", revocation_reason: "heartbeat_timeout",
      owner_resources: { ...qualification.owner_resources, phase: "shutdown_complete" },
      attempt: { ...qualification.attempt, active_ms: 4000, complete: true } } };
  const receipt = { schema: "worker-mining-interruption-v1", generation: 7, workDispatched: 1, workGateRemainingMs: 10000,
    ownershipReleased: true, controlRecordsSent: 0 };
  const records = [before, after, recovered, { ...recovered, status: "closed", connected: false, serialOwnershipReleased: true }]
    .map((state, index) => ({ sequence: index + 1, state }));
  const fault = validateLossReceipt({ receipt, before, after }, context, records.slice(0, 2));
  return { context, before, after, receipt, records, fault };
}

test("recovery loss accepts fresh admission with zero stale replies without claiming stale-reply coverage", () => {
  const f = fixture();
  const result = judgeIterative(f.context, f.records, f.fault);
  assert.equal(result.fresh_admission_verified, true);
  assert.equal(result.stale_reply_hardware_coverage, false);
  assert.equal(result.discarded_replies, 0);
});

test("a pending logical response is never accepted as a live loss receipt", () => {
  const f = fixture();
  assert.throws(() => validateLossReceipt({ receipt: { schema: "worker-read-interruption-v1", response_pending: true },
    before: f.before, after: f.after }, f.context, f.records.slice(0, 2)));
});

test("live loss requires work and remaining shutdown headroom at the actual checkpoint", () => {
  for (const changes of [{ work_dispatched: 0 }, { work_gate_remaining_ms: 3000 }, { gate_closed_ms: 1 }]) {
    const f = fixture();
    f.before.qualification = { ...f.before.qualification, ...changes };
    assert.throws(() => validateLossReceipt({ receipt: f.receipt, before: f.before, after: f.after }, f.context, f.records.slice(0, 2)));
  }
});

test("loss receipt cannot claim a control-free cut when a control record was sent", () => {
  const f = fixture();
  f.receipt.controlRecordsSent = 1;
  assert.throws(() => validateLossReceipt({ receipt: f.receipt, before: f.before, after: f.after }, f.context, f.records.slice(0, 2)));
});

test("recovered safe state must follow release and bind the interrupted generation", () => {
  for (const mutate of [
    (f) => { f.records[2].state.status = "configured"; },
    (f) => { f.records[2].state.qualification.generation = 8; },
    (f) => { f.records[2].state.helloRecovery = undefined; },
    (f) => { f.records[2].state.qualification.revocation_reason = "restoration_requested"; },
  ]) {
    const f = fixture(); mutate(f);
    assert.throws(() => judgeRecovery(f.context, f.records, f.fault, {}));
  }
});

test("recovery preserves existing safety, stack and local shutdown timing gates", () => {
  for (const mutate of [
    (f) => { f.before.qualification.power_watts = 16; },
    (f) => { f.before.qualification.owner_resources.stack_free_bytes = 4095; },
    (f) => { f.records[2].state.qualification.shutdown_started_ms = 4001; },
  ]) {
    const f = fixture(); mutate(f);
    assert.throws(() => judgeIterative(f.context, f.records, f.fault));
  }
});

test("resume requires new observed work and rejects another fault", () => {
  const f = fixture(); f.context.recovery_phase = "resume"; f.context.recovery_loss_generation = 6;
  assert.equal(judgeIterative(f.context, f.records).reauthorized_work_verified, true);
  assert.throws(() => judgeIterative(f.context, f.records, f.fault));
  f.records[0].state.running = false;
  assert.throws(() => judgeRecovery(f.context, f.records, undefined, {}));
});


test("resumed work cannot reuse the interrupted Worker generation", () => {
  const f = fixture();
  f.context.recovery_phase = "resume"; f.context.recovery_loss_generation = 7;
  assert.throws(() => judgeIterative(f.context, f.records), { code: "recovery_generation_reused" });
});


test("link-closed recovery requires retained same-epoch TX abandonment and revocation evidence", () => {
  const f = fixture(); f.records[2].state.qualification.revocation_reason = "link_closed";
  assert.throws(() => judgeIterative(f.context, f.records, f.fault), { code: "recovery_stop_binding" });
  const result = judgeIterative(f.context, f.records, f.fault, [{ stage: "recovered", source: "device", link_closed_verified: true }]);
  assert.equal(result.revocation_reason, "link_closed");
});

test("post-work authorization checkpoint preserves legitimately advanced high-water state through recovery", () => {
  const f = fixture();
  assert.equal(f.before.preservation.authorization_high_water_match, false);
  assert.equal(f.records[2].state.preservation.authorization_high_water_match, false);
  assert.equal(judgeIterative(f.context, f.records, f.fault).fresh_admission_verified, true);
});
test("loss requires matching pending authorization checkpoints before and after the cut", () => {
  for (const mutate of [
    f => { delete f.before.authorizationRecovery; },
    f => { f.before.authorizationRecovery = { ...f.before.authorizationRecovery, matched: true }; },
    f => { f.after.authorizationRecovery = { ...f.after.authorizationRecovery, matched: true }; },
    f => { f.after.authorizationRecovery = { ...f.after.authorizationRecovery, checkpointId: Buffer.alloc(16, 3).toString("base64url") }; },
    f => { f.before.authorizationRecovery = { ...f.before.authorizationRecovery, generation: 8 }; },
  ]) {
    const f = fixture(); mutate(f);
    assert.throws(() => validateLossReceipt({ receipt: f.receipt, before: f.before, after: f.after }, f.context, f.records.slice(0, 2)),
      { code: "recovery_authorization_checkpoint" });
  }
});
test("fresh recovery must match the same post-work authorization checkpoint and generation", () => {
  for (const change of [{ matched: null }, { matched: false }, { generation: 8 }, { checkpointId: Buffer.alloc(16, 4).toString("base64url") }]) {
    const f = fixture();
    f.records[2].state.authorizationRecovery = { ...f.records[2].state.authorizationRecovery, ...change };
    assert.throws(() => judgeIterative(f.context, f.records, f.fault));
  }
});
test("a later matching snapshot cannot erase an observed authorization mismatch", () => {
  const f = fixture();
  f.records.splice(2, 0, { sequence: 3, state: { ...f.records[2].state, authorizationRecovery: { ...f.records[2].state.authorizationRecovery, matched: false } } });
  f.records.forEach((record, index) => { record.sequence = index + 1; });
  assert.throws(() => judgeIterative(f.context, f.records, f.fault), { code: "recovery_authorization_changed" });
});
test("authorization recovery metadata rejects raw digests and malformed closed fields", () => {
  const f = fixture();
  for (const change of [{ checkpointId: "not-canonical" }, { generation: 0 }, { matched: "true" }, { highWaterDigest: "must-not-persist" }]) {
    assert.throws(() => validateState({ ...f.before, authorizationRecovery: { ...f.before.authorizationRecovery, ...change } }, f.context));
  }
  const legacy = { ...f.before }; delete legacy.authorizationRecovery;
  assert.equal(validateState(legacy, f.context), legacy);
});
test("resumed diagnostics do not require the prior authorization checkpoint field", () => {
  const f = fixture(); f.context.recovery_phase = "resume"; f.context.recovery_loss_generation = 6;
  for (const record of f.records) delete record.state.authorizationRecovery;
  assert.equal(judgeIterative(f.context, f.records).reauthorized_work_verified, true);
});
