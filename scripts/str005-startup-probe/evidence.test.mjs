import test from 'node:test';
import assert from 'node:assert/strict';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
import { channelFixture } from '../str005-v2-serial/protocol-judge.test-helper.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { judge } from './evidence.mjs';
function fixture() {
  const record = channelFixture().deviceRecords.at(-1), generation = record.workerGeneration;
  Object.assign(record, { scope: 'share', outcome: 'rejected', firstFailure: { stage: 'worker_quiescent', category: 'evidence', atDeviceUs: 13000 }, authorityDeadlineDeviceUs: 180001000,
    observationDeadlineDeviceUs: null, poolSessionGeneration: 2, poolTransportEpoch: 3 });
  record.events = record.events.filter(row => row.kind !== 'connected');
  record.events.splice(1, 0, { ...record.events[0], kind: 'asic_dispatch' });
  record.events = record.events.map((row, index) => ({ ...row, sequence: index + 1 }));
  const context = { scope: 'share', attemptId: record.attemptId, firmware_commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), captureVerified: true };
  const qualification = { schema: 'worker-qualification-v1', generation, active_ms: 100, generation_elapsed_ms: 100,
    budget_reserved_ms: 180000, submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 2,
    last_valid_heartbeat_ms: 100, budget_complete: true, safe_stop_complete: true, voltage_fresh: true, power_fresh: true,
    temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false, voltage_volts: 5, power_watts: 3,
    chip_temp_celsius: 35, fan_rpm: 3000, gate_closed_ms: 9, shutdown_started_ms: 9, safe_stop_stage: 'fan_paused',
    revocation_reason: 'restoration_requested', active_limit_ms: 180000, shutdown_budget_ms: 15550, work_gate_remaining_ms: 0 };
  const checkpoint = { schema: 'worker-authorization-recovery-v1', checkpointId: Buffer.alloc(16, 3).toString('base64url'), generation, matched: true };
  const restored = { ...state(context), qualification, authorizationRecovery: checkpoint };
  const closed = { ...state(context, 'candidate', true), qualification, authorizationRecovery: checkpoint };
  const status = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'terminal', connection: null, record,
    observation: { bootOrdinal: 1, workerGeneration: generation + 1, serialTransportEpoch: 10, observedAtUs: 15000,
      clockValid: true, stationIpv4: '192.168.1.2', wifiConnected: true, socket: null } };
  const recovery = { finished: { failures: [] }, ...Object.fromEntries(Object.entries({
    ledger: { ...ledger, next_ordinal: ledger.next_ordinal + 1, last_completed_ordinal: ledger.next_ordinal, total_charged_ms: ledger.total_charged_ms + 180000 },
    original_budget: original, state: restored, closed, status, diagnostics: { schema: 'worker-diagnostic-export-v1', observations: [] },
  }).map(([stage, value]) => [stage, projectRecoveryPart(stage, value, context)])) };
  const running = { ...state(context), status: 'running', running: true, qualification };
  return { context, parts: { before: { ledger, original_budget: original, attempt: { id: record.attemptId } }, recovery, hostReleased: true,
    run: { firstFailure: null, observedStart: true, startInvokedAt: 1000, startRepliedAt: 2000, stopRequestedAt: 2500,
      proof: { generation, initialWorkDispatched: 1, workDispatched: 2, state: running } } } };
}
test('complete startup evidence passes without accepted-share or heartbeat-fault requirements', () => {
  const { parts, context } = fixture(); const result = judge(parts, context);
  assert.equal(result.complete, true); assert.deepEqual(result.blockers, []); assert.equal(result.parity_promotion, false);
});
for (const [label, mutate, blocker] of [
  ['charged completion', p => { p.recovery.ledger.total_charged_ms -= 180000; }, 'startup_charge_completion_unproven'],
  ['retained dispatch', p => { p.recovery.status.record.events = p.recovery.status.record.events.filter(row => row.kind !== 'asic_dispatch').map((row, index) => ({ ...row, sequence: index + 1 })); }, 'startup_device_dispatch_unproven'],
  ['authorization checkpoint', p => { p.recovery.state.authorizationRecovery.matched = false; }, 'startup_authorization_checkpoint_unproven'],
  ['device resources', p => { p.recovery.status.record.resources.fenceRetained = true; }, 'recovery_device_resources_unreleased'],
  ['host resources', p => { p.hostReleased = false; }, 'startup_host_release_unproven'],
]) test(`startup evaluator rejects missing ${label}`, () => {
  const { parts, context } = fixture(); mutate(parts); const result = judge(parts, context);
  assert.equal(result.complete, false); assert.ok(result.blockers.includes(blocker));
});

test('startup cannot reinterpret an authority failure before requested Stop as cleanup', () => {
  // Arrange
  const { parts, context } = fixture();
  parts.recovery.status.record.firstFailure = { stage: 'worker_quiescent', category: 'authority', atDeviceUs: 6000 };
  // Act
  const result = judge(parts, context);
  // Assert
  assert.equal(result.complete, false);
  assert.ok(result.blockers.includes('startup_normal_stop_unproven'));
});

for (const [label, mutate] of [
  ['different revocation reason', p => { p.recovery.state.qualification.revocation_reason = 'heartbeat_timeout'; }],
  ['same-millisecond ambiguous failure', p => { p.recovery.status.record.firstFailure.atDeviceUs = 9500; }],
  ['different native generation', p => { p.recovery.state.qualification.generation += 1; }],
  ['earlier socket release', p => { p.recovery.status.record.resources.socketClosedAtUs = 8500; }],
  ['nonterminal evidence failure after Stop', p => { p.recovery.status.record.firstFailure.atDeviceUs = 10001; }],
  ['protocol failure after Stop', p => { p.recovery.status.record.firstFailure.category = 'protocol'; }],
]) test(`normal Stop classification rejects ${label}`, () => {
  // Arrange
  const { parts, context } = fixture(); mutate(parts);
  // Act
  const result = judge(parts, context);
  // Assert
  assert.equal(result.complete, false); assert.ok(result.blockers.includes('startup_normal_stop_unproven'));
});
test('post-Stop authority cancellation may prove startup without changing rejected outcome', () => {
  // Arrange
  const { parts, context } = fixture();
  parts.recovery.status.record.firstFailure = { stage: 'worker_quiescent', category: 'authority', atDeviceUs: 10000 };
  // Act
  const result = judge(parts, context);
  // Assert
  assert.equal(result.complete, true); assert.equal(parts.recovery.status.record.outcome, 'rejected');
});
