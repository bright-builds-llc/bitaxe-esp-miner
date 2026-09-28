import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { mkdtemp, rm, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { channelFixture } from '../str005-v2-serial/protocol-judge.test-helper.mjs';
import { ledger, original, state } from '../str005-noise-serial/test-fixture.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { renewScope, RENEW_TASK, retainedConclusion, verifyRetainedInputs } from './renew-successor.mjs';
import { currentProof, validatePart } from './model.mjs';
import { admitRecovery } from './install.mjs';
import { createBaselineCollector } from './client.mjs';
import { createProbeServer } from './server.mjs';
import { argumentsFor } from './main.mjs';
const attemptId = Buffer.alloc(16, 1).toString('base64url');
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) },
  scope: 'share', attemptId, renewSuccessor: true, ownerTask: RENEW_TASK,
  retainedBaseline: { attemptId, generation: 2, bootOrdinal: 9, ledger, originalBudget: original } };
function fixture() {
  const record = channelFixture().deviceRecords.at(-1);
  Object.assign(record, { scope: 'share', bootOrdinal: 9, authorityDeadlineDeviceUs: 180001000, observationDeadlineDeviceUs: null,
    poolSessionGeneration: 2, poolTransportEpoch: 3 });
  record.events = record.events.filter(row => row.kind !== 'connected').map((row, i) => ({ ...row, sequence: i + 1 }));
  const rawStatus = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'terminal', connection: null, record,
    observation: { bootOrdinal: 9, workerGeneration: 2, serialTransportEpoch: 3, observedAtUs: 15000, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
  const parts = { state: state(context, 'before'), closed: state(context, 'before', true), ledger: structuredClone(ledger),
    original_budget: structuredClone(original), finished: { failures: [] }, status: validatePart('status', rawStatus, context),
    diagnostics: { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 9, reset_reason: 'software_cpu', uptime_ms: 20 }] } };
  return { parts, rawStatus };
}
function tasks(lines = '') { return `## Active\n### ${RENEW_TASK} | fixture\nRenew image qualification: baseline enabled.\n${lines}`; }

test('successor admission is disabled by default and cannot enable original modes', () => {
  // Arrange / Act / Assert
  assert.throws(() => renewScope(tasks(), false));
  assert.throws(() => renewScope(tasks().replace('## Active', '## Future'), true));
  assert.throws(() => renewScope(tasks() + tasks(), true));
  assert.throws(() => renewScope(tasks().replace(RENEW_TASK, 'task-str005-start-panic-diagnosis'), true));
  assert.equal(renewScope(tasks(), true).installEnabled, false);
  assert.equal(argumentsFor(['renew-preflight', '--private-root', '/p', '--gate-root', '/g', '--manifest', '/m']).renewSuccessor, true);
  assert.equal(argumentsFor(['preflight', '--private-root', '/p', '--gate-root', '/g', '--manifest', '/m']).renewSuccessor, undefined);
});

test('installation and capture scopes require separate published stages', () => {
  // Arrange / Act / Assert
  const install = 'Renew image qualification: installation enabled.\n', capture = 'Renew image qualification: self-test enabled.\n';
  assert.equal(renewScope(tasks(install), true).selfTestEnabled, false);
  assert.equal(renewScope(tasks(capture), true).installEnabled, false);
  assert.throws(() => renewScope(tasks(install + capture), true));
});

test('released retained proof is v2 and never labels retained status idle', () => {
  // Arrange
  const { parts } = fixture();
  // Act
  const value = currentProof(context, parts, 1000);
  admitRecovery(value, context, 121000);
  // Assert
  assert.equal(retainedConclusion(parts, context).complete, true);
  assert.equal(value.schema, 'str005-current-recovery-proof-v2');
  assert.equal(value.current_v2_released, true);
  assert.equal(Object.hasOwn(value, 'current_v2_idle'), false);
  assert.equal(value.retained_attempt_id, attemptId);
  assert.throws(() => admitRecovery(value, context, 121001));
  assert.throws(() => admitRecovery(value, { ...context, ownerTask: 'task-str005-start-panic-diagnosis' }, 1000));
});

test('wrong attempt active work retained fence changed ledger and failed close cannot authorize reset', () => {
  // Arrange / Act / Assert
  for (const mutate of [p => { p.status.record.attemptId = Buffer.alloc(16, 2).toString('base64url'); },
    p => { p.status.state = p.status.record.state = 'running'; }, p => { p.status.record.resources.fenceRetained = true; },
    p => { p.status.record.resources.workerQuiescent = false; }, p => { p.status.observation.bootOrdinal++; },
    p => { p.closed.serialOwnershipReleased = false; }, p => { p.closed.preservation.authorization_high_water_match = false; },
    p => { p.ledger.total_charged_ms++; }]) {
    const { parts } = fixture(); mutate(parts); assert.throws(() => currentProof(context, parts, 1000));
  }
});

test('collector begins before observing and uses retained id without an idle query', async () => {
  // Arrange
  const calls = [], { rawStatus } = fixture();
  const gate = { refresh: async () => calls.push('refresh'), reviewQualificationAttempts: async () => ledger,
    reviewBudget: async () => original, stratumV2Possession: async () => ({}),
    stratumV2Status: async (_scope, id) => { calls.push(id); return rawStatus; }, exportDiagnostics: async () => {},
    stop: async () => calls.push('stop'), close: async () => calls.push('close') };
  // Act
  await createBaselineCollector({ gate, published: () => state(context, 'before'), save: async () => {}, attemptId,
    begin: async () => calls.push('begin') })();
  // Assert
  assert.deepEqual(calls, ['begin', 'refresh', attemptId, 'stop', 'close']);
});

test('successor server uses one-use actual collection time for v2 proof', async t => {
  // Arrange
  const { parts, rawStatus } = fixture(), proofs = [], beginnings = [];
  let now = 1000;
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    now: () => now, persist: async () => {}, persistBegin: async v => beginnings.push(v), persistProof: async v => proofs.push(v), validateDiagnostics: async v => v });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  // Act / Assert
  assert.equal((await post('/part', { stage: 'ledger', value: ledger })).status, 400);
  assert.equal((await post('/baseline-begin', {})).status, 200);
  assert.equal((await post('/baseline-begin', {})).status, 400);
  now = 2000;
  assert.equal((await post('/diagnostic-export', parts.diagnostics)).status, 200);
  for (const [stage, value] of Object.entries({ ...parts, status: rawStatus }).filter(([key]) => key !== 'diagnostics' && key !== 'finished'))
    assert.equal((await post('/part', { stage, value })).status, 200, stage);
  assert.equal((await post('/part', { stage: 'finished', value: parts.finished })).status, 200);
  assert.equal(proofs.length, 1); assert.equal(proofs[0].observed_at_unix_ms, beginnings[0].startedAtUnixMs);
});

test('persisted retained digest and collection time are reverified before installation', async t => {
  // Arrange
  const root = await mkdtemp(resolve(tmpdir(), 'renew-proof-')); t.after(() => rm(root, { recursive: true, force: true }));
  const { parts } = fixture(), value = currentProof(context, parts, 1000);
  await writeNew(resolve(root, 'baseline-begin.json'), { schema: 'str005-renew-baseline-begin-v1', startedAtUnixMs: 1000 });
  for (const [key, part] of Object.entries(parts)) await writeNew(resolve(root, `baseline-${key}.json`), part);
  // Act / Assert
  await verifyRetainedInputs(root, context, value);
  await assert.rejects(verifyRetainedInputs(root, context, { ...value, retained_status_sha256: '0'.repeat(64) }));
  await assert.rejects(verifyRetainedInputs(root, context, { ...value, observed_at_unix_ms: 999 }));
});

test('rejected or stalled begin still independently closes and saves the first failure', async () => {
  // Arrange
  const { createRenewBaselineCollector } = await import('./client.mjs');
  for (const stalled of [false, true]) {
    const calls = [], saved = [];
    const gate = { stop: async () => calls.push('stop'), close: async () => calls.push('close') };
    // Act
    const outcome = await createRenewBaselineCollector({ gate, published: () => state(context, 'before', true),
      save: async (stage, value) => saved.push({ stage, value }), timeoutMs: 5, cleanupMs: 5,
      begin: async () => { if (stalled) await new Promise(() => {}); else throw Error('synthetic'); } })();
    // Assert
    assert.deepEqual(calls, ['stop', 'close']);
    assert.deepEqual(outcome.first_failure, { phase: 'begin', category: stalled ? 'timeout' : 'operation_failed' });
    assert.deepEqual(saved.map(row => row.stage), ['closed', 'finished']);
    assert.deepEqual(outcome.failures, ['state']);
  }
});

test('stalled or rejected Stop cannot prevent Close or erase the primary observation failure', async () => {
  // Arrange
  const { createRenewBaselineCollector } = await import('./client.mjs');
  for (const stalled of [false, true]) {
    const calls = [], saved = [];
    const gate = { refresh: async () => { throw Error('first-observation'); }, reviewQualificationAttempts: async () => ledger,
      reviewBudget: async () => original, stratumV2Possession: async () => ({}), stratumV2Status: async () => fixture().rawStatus,
      exportDiagnostics: async () => {}, stop: async () => { calls.push('stop'); if (stalled) await new Promise(() => {}); else throw Error('stop'); },
      close: async () => calls.push('close') };
    // Act
    const outcome = await createRenewBaselineCollector({ gate, published: () => state(context, 'before', true),
      save: async (stage, value) => saved.push({ stage, value }), attemptId, begin: async () => {}, timeoutMs: 5, cleanupMs: 5 })();
    // Assert
    assert.deepEqual(calls, ['stop', 'close']);
    assert.deepEqual(outcome.first_failure, { phase: 'state', category: 'operation_failed' });
    assert.ok(outcome.failures.includes('closed'));
    assert.equal(saved.find(row => row.stage === 'closed').value.serialOwnershipReleased, true);
  }
});

test('only closed diagnostic categories leave the successor CLI', async () => {
  // Arrange
  const { failureCategory } = await import('./main.mjs');
  // Act / Assert
  assert.equal(failureCategory({ code: 'renew_retained_proof' }), 'renew_retained_proof');
  assert.equal(failureCategory({ code: 'panic_recovery_prerequisite' }), 'panic_recovery_prerequisite');
  assert.equal(failureCategory({ code: 'renew_error secret' }), 'panic_operation_rejected');
  assert.equal(failureCategory(Error('private path or credential')), 'panic_operation_rejected');
});

test('clear scope is independent and parser has no command or partition override', async () => {
  // Arrange
  const { clearArguments } = await import('./renew-clear.mjs');
  const enabled = 'Renew image qualification: core clear enabled.\n';
  // Act / Assert
  assert.equal(renewScope(tasks(enabled), true).clearEnabled, true);
  assert.throws(() => renewScope(tasks(enabled + 'Renew image qualification: self-test enabled.\n'), true));
  assert.throws(() => renewScope(tasks(enabled + 'Renew image qualification: installation enabled.\n'), true));
  assert.equal(clearArguments(['renew-clear-preflight', '--private-root', '/p', '--capture-root', '/c', '--recovery-root', '/r']).action, 'renew-clear-preflight');
  assert.throws(() => clearArguments(['renew-clear', '--private-root', '/p', '--force', '/yes']));
});

test('clear requires fresh qualified successor recovery and unchanged accounting', async () => {
  // Arrange
  const { validateClearRecovery } = await import('./renew-clear.mjs');
  const identity = context.before_source, capture = { identity: { ...identity, gate_commit: context.gate_commit }, physical: context.detector.physical,
    recovery: { ledger, original_budget: original } };
  const recovered = { schema: 'str005-current-recovery-proof-v1', ...capture.identity, physical_identity_sha256: capture.physical,
    source_commit: context.commit, observed_at_unix_ms: 1000, safe_baseline: true, restoration_confirmed: true, device_lease_inactive: true,
    serial_ownership_released: true, preservation_matches: true, current_v2_idle: true, mine_on_boot: false, ledger, original_budget: original };
  const recoveryContext = { ...context, recoveryOnly: true }, result = { complete: true, host_resources_released: true, baseline_complete: true, blockers: [] };
  // Act / Assert
  validateClearRecovery(recovered, recoveryContext, result, capture, context.commit, 121000);
  assert.throws(() => validateClearRecovery(recovered, recoveryContext, result, capture, context.commit, 121001));
  assert.throws(() => validateClearRecovery(recovered, { ...recoveryContext, ownerTask: 'old' }, result, capture, context.commit, 1000));
  assert.throws(() => validateClearRecovery(recovered, recoveryContext, { ...result, host_resources_released: false }, capture, context.commit, 1000));
  assert.throws(() => validateClearRecovery({ ...recovered, source_commit: '0'.repeat(40) }, recoveryContext, result, capture, context.commit, 1000));
  assert.throws(() => validateClearRecovery({ ...recovered, ledger: { ...ledger, next_ordinal: 19, last_completed_ordinal: 18 } }, recoveryContext, result, capture, context.commit, 1000));
});

test('failed launched clear finalization seals real partial output and retains child cause', async t => {
  // Arrange
  const { finalizeClear } = await import('./renew-clear.mjs');
  const { mkdir, readFile } = await import('node:fs/promises');
  const { verifyInventory, proof } = await import('../str005-noise-serial/files.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'renew-clear-failed-'))); t.after(() => rm(root, { recursive: true, force: true }));
  const c = { source_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64) };
  await mkdir(resolve(root, 'clear'), { mode: 0o700 });
  await writeNew(resolve(root, 'clear-launch.json'), { schema: 'str005-startup-clear-launch-v1' });
  await writeNew(resolve(root, 'clear-failure.json'), { schema: 'str005-startup-clear-failure-v1', firstFailure: 'decoder_failed', serialOwnershipReleased: true });
  await writeNew(resolve(root, 'clear/result.private.json'), { schema_version: 'bitaxe-development-core-dump-clear-v1', source_commit: c.source_commit,
    expected_installed_source: c.firmware_commit, expected_installed_elf: c.app_elf_sha256,
    terminal_category: 'acquisition_failed', first_failure_stage: 'dump_erase_readback', cleanup_complete: false });
  // Act
  const result = await finalizeClear(root, c);
  // Assert
  assert.equal(result.complete, false);
  assert.equal(result.first_failure, 'renew_clear_child_dump_erase_readback');
  assert.equal(result.cleanup_receipt_verified, false);
  assert.equal(result.host_serial_release_verified, true);
  assert.equal(result.hardware_baseline_verified, false);
  assert.equal(result.fresh_post_clear_baseline_required, true);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  assert.equal(JSON.parse(await readFile(resolve(root, 'result.json'), 'utf8')).complete, false);
  await assert.rejects(finalizeClear(root, c));
});

test('changed runtime inputs are retained as failed finalization without calling a clear effect', async t => {
  // Arrange
  const { finalizeClear } = await import('./renew-clear.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'renew-clear-changed-'))); t.after(() => rm(root, { recursive: true, force: true }));
  // Act
  const result = await finalizeClear(root, {}, async () => { throw Object.assign(Error('synthetic'), { code: 'renew_clear_input_changed' }); });
  // Assert
  assert.equal(result.complete, false);
  assert.equal(result.first_failure, 'renew_clear_input_changed');
  assert.equal(result.host_serial_release_verified, false);
  assert.equal(result.cleanup_receipt_verified, false);
});

test('malformed partial clear receipts are preserved and sealed without release claims', async t => {
  // Arrange
  const { finalizeClear } = await import('./renew-clear.mjs');
  const { writeFile } = await import('node:fs/promises');
  const { proof, verifyInventory } = await import('../str005-noise-serial/files.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'renew-clear-malformed-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(resolve(root, 'clear-failure.json'), '{partial', { mode: 0o600 });
  // Act
  const result = await finalizeClear(root, {});
  // Assert
  assert.equal(result.complete, false);
  assert.equal(result.first_failure, 'renew_clear_receipt_invalid');
  assert.equal(result.cleanup_receipt_verified, false);
  assert.equal(result.host_serial_release_verified, false);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
});

test('successor effect rejection preserves independent read and close collection', async t => {
  // Arrange
  let checks = 0;
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    persist: async () => {}, verifyEffect: async () => { checks++; throw Object.assign(Error('changed'), { code: 'renew_source_changed' }); },
  });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  // Act / Assert
  assert.equal((await post('/self-test-claim', {})).status, 400);
  assert.equal(checks, 1);
  assert.equal((await post('/probe-context', {})).status, 200);
  assert.equal((await post('/part', { stage: 'closed', value: state(context, 'before', true) })).status, 200);
});
