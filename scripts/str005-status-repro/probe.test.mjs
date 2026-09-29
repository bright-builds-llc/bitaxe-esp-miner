import test from 'node:test';
import assert from 'node:assert/strict';
import { createCoordinator } from './client-core.mjs';
import { argumentsFor, taskEnabled } from './contract.mjs';
import { judge } from './finish.mjs';
import { dirname, resolve } from 'node:path';
import { verifyGateCompatibility } from '../str005-startup-probe/gate-compatibility.mjs';

const tasks = text => `## Active\n### task-str005-v2-accepted-share-probe | fixture\n${text}\n`;
test('effect admission requires the compiled flag and one active task marker', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => taskEnabled(tasks('Status reproduction hardware: enabled.')));
  assert.throws(() => taskEnabled(tasks('Status reproduction hardware: enabled.'), false));
  assert.throws(() => taskEnabled(tasks('Status reproduction hardware: disabled.'), true));
  assert.throws(() => taskEnabled(tasks('Status reproduction hardware: enabled.') + tasks('Status reproduction hardware: enabled.'), true));
});
test('command arguments have no renewal, flash, clear or replay override', () => {
  // Arrange / Act / Assert
  assert.equal(argumentsFor(['finish', '--private-root', '/p']).action, 'finish');
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/p', '--gate-root', '/g', '--fixture-binary', '/f']));
  for (const option of ['--renewals', '--flash', '--clear', '--attempt-id', '--ordinal'])
    assert.throws(() => argumentsFor(['finish', '--private-root', '/p', option, '1']));
});
test('blocked status cannot delay Stop or suppress cleanup and first failure', async () => {
  // Arrange
  const calls = [], values = []; let started = 0;
  const gate = { startWindow: async () => { started++; calls.push('start'); },
    state: () => ({ running: true, failure: null, heartbeatSuppressed: false, renewalsConfirmed: 0, qualification: { generation: 7 } }),
    stratumV2Status: () => new Promise(() => {}), stop: async () => { calls.push('stop'); }, close: async () => { calls.push('close'); } };
  const run = createCoordinator({ gate, prepare: async () => ({ generation: 7, attemptId: 'attempt', binding: 'binding' }),
    record: async value => values.push(value), recover: async () => calls.push('recover'), release: async () => calls.push('release'),
    post: async () => { throw Error('unexpected'); }, limits: { replyMs: 30, statusMs: 10, stopFromReplyMs: 15,
      stopFromInvocationMs: 45, cleanupMs: 30, releaseMs: 30 } });
  // Act
  const result = await run();
  // Assert
  assert.equal(started, 1); assert.equal(result.firstFailure, 'status'); assert.equal(result.complete, false);
  assert.ok(calls.indexOf('stop') < calls.indexOf('recover')); assert.ok(calls.includes('close')); assert.ok(calls.includes('release'));
  assert.equal(values[0].firstFailure, 'status'); assert.equal(values[0].proof, null);
  await assert.rejects(run(), /status_repro_consumed/u);
});
test('late Start reply requests another Stop and Close without a second Start', async () => {
  // Arrange
  const calls = []; let resolveStart;
  const gate = { startWindow: () => { calls.push('start'); return new Promise(resolve => { resolveStart = resolve; }); },
    state: () => ({ running: true, qualification: { generation: 7 } }),
    stop: async () => calls.push('stop'), close: async () => calls.push('close') };
  const run = createCoordinator({ gate, prepare: async () => ({ generation: 7 }), record: async () => {},
    recover: async () => {}, release: async () => {}, post: async path => { calls.push(path); },
    limits: { replyMs: 5, statusMs: 5, stopFromReplyMs: 5, stopFromInvocationMs: 10, cleanupMs: 20, releaseMs: 20 } });
  // Act
  const result = await run(); resolveStart(); await new Promise(resolve => setTimeout(resolve, 10));
  // Assert
  assert.equal(result.firstFailure, 'start'); assert.equal(calls.filter(value => value === 'start').length, 1);
  assert.ok(calls.filter(value => value === 'stop').length >= 2); assert.ok(calls.filter(value => value === 'close').length >= 2);
  assert.ok(calls.includes('/status-repro/late-completion'));
});
test('safe current recovery does not manufacture missing historical retained proof or qualification', () => {
  // Arrange
  const before = { attempt: { id: 'a' }, ledger: { next_ordinal: 21, total_charged_ms: 2100000, pending: false }, original_budget: { pending: false } };
  // Act
  const result = judge({ before, recovery: {}, hostReleased: true, fixtureStarted: false, fixtureCompletion: false, fixtureReleased: true }, {});
  // Assert
  assert.equal(result.complete, false); assert.equal(result.historical_retained_proof, false);
  assert.equal(result.qualification_success, false); assert.ok(result.blockers.includes('status_repro_retained_attempt_unavailable'));
});
test('a failed result write still runs recovery, Close and fixture release', async () => {
  // Arrange
  const calls = [];
  const gate = { startWindow: async () => {},
    state: () => ({ running: true, failure: null, heartbeatSuppressed: false, renewalsConfirmed: 0, qualification: { generation: 4 } }),
    stratumV2Status: async () => ({ status: 'fixture' }), stop: async () => calls.push('stop'), close: async () => calls.push('close') };
  const run = createCoordinator({ gate, prepare: async () => ({ generation: 4, attemptId: 'a', binding: 'b' }),
    post: async () => ({ observed: true, generation: 4 }), record: async () => { throw Error('disk_failure'); },
    recover: async () => calls.push('recover'), release: async () => calls.push('release'),
    limits: { replyMs: 30, statusMs: 30, stopFromReplyMs: 15, stopFromInvocationMs: 45, cleanupMs: 30, releaseMs: 30 } });
  // Act
  const result = await run();
  // Assert
  assert.equal(result.complete, false); assert.equal(result.firstFailure, null); assert.ok(result.failures.includes('result'));
  assert.deepEqual(calls, ['stop', 'recover', 'close', 'release']);
});
test('fixture natural completion remains a qualification failure after ownership release', () => {
  // Arrange / Act
  const result = judge({ recovery: {}, hostReleased: true, fixtureStarted: true,
    fixtureCompletion: false, fixtureReleased: true }, {});
  // Assert
  assert.equal(result.fixture_resources_released, true); assert.equal(result.fixture_natural_completion, false);
  assert.ok(result.blockers.includes('status_repro_fixture_natural_completion_failed'));
  assert.equal(result.complete, false);
});

import './http.test.mjs';

const maybeGateRoot = process.argv[2] ? dirname(resolve(process.argv[2])) : undefined;
test('actual Gate accepts zero renewals and starts its renewal clock after the Start reply',
  { skip: !maybeGateRoot }, async () => {
    const result = await verifyGateCompatibility(maybeGateRoot);
    assert.equal(result.zeroRenewalsAccepted, true);
    assert.equal(result.renewalOrigin, 'completed-controller-start');
    assert.equal(result.renewAfterMilliseconds, 20000);
  });
test('rebooted idle recovery saves current safety without inventing retained resources', async () => {
  // Arrange
  const { state, ledger, original } = await import('../str005-noise-serial/test-fixture.mjs');
  const { projectRecoveryPart } = await import('../str005-v2-serial/recovery-evidence.mjs');
  const context = { scope: 'share', gate_commit: 'c'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64),
    before_source: { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64) }, attemptId: 'a' };
  const idle = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
    observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100,
      clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
  const recovery = { state: state(context), closed: state(context, 'candidate', true), ledger, original_budget: original,
    status: projectRecoveryPart('status', idle, context), diagnostics: projectRecoveryPart('diagnostics',
      { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false,
        boot_ordinal: 3, reset_reason: 'panic', uptime_ms: 1 }] }, context), finished: { failures: [] } };
  // Act
  const result = judge({ before: { attempt: { id: 'a' }, ledger, original_budget: original }, recovery,
    hostReleased: true, fixtureStarted: false, fixtureReleased: true, fixtureCompletion: false }, context);
  // Assert
  assert.equal(result.current_safe_recovery, true); assert.equal(result.historical_retained_proof, false);
  assert.equal(result.qualification_success, false); assert.ok(result.blockers.includes('status_repro_retained_attempt_unavailable'));
});
