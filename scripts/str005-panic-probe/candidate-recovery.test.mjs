import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { createCandidateRecovery } from './client.mjs';
import { createProbeServer } from './server.mjs';
import { applyCandidateFailure } from './candidate-failure.mjs';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) }, scope: 'share', installEnabled: true, selfTestEnabled: false };
function coordinator(behavior) {
  const calls = [], failures = []; let closed = false;
  const gate = { stratumV2Possession: async () => ({}), stratumV2Status: behavior,
    stop: async () => calls.push('stop'), close: async () => { calls.push('close'); closed = true; } };
  const run = createCandidateRecovery({ gate, published: () => state(context, 'candidate', closed),
    begin: async () => { calls.push('http_begin'); return {}; }, collect: async () => ({ complete: true }),
    failure: async value => failures.push(value), timeoutMs: 5, cleanupMs: 5 });
  return { run, calls, failures, gate };
}

test('actual retained-history category before HTTP is durable and closes independently', async () => {
  // Arrange
  const f = coordinator(async () => { throw Object.assign(Error('v2_retained_evidence'), { code: 'v2_retained_evidence' }); });
  // Act
  const result = await f.run();
  // Assert
  assert.equal(result.complete, false); assert.deepEqual(f.calls, ['stop', 'close']);
  assert.deepEqual(f.failures[0].first_failure, { phase: 'status', category: 'v2_retained_evidence' });
  assert.equal(f.failures[0].close_complete, true);
  await assert.rejects(f.run(), /consumed/u); assert.equal(f.failures.length, 1);
});

test('stalled prelude and stalled Stop cannot prevent bounded Close', async () => {
  // Arrange
  const f = coordinator(async () => new Promise(() => {}));
  f.gate.stop = async () => { f.calls.push('stop'); await new Promise(() => {}); };
  // Act
  const result = await f.run();
  // Assert
  assert.equal(result.complete, false); assert.deepEqual(f.calls, ['stop', 'close']);
  assert.deepEqual(f.failures[0].first_failure, { phase: 'status', category: 'timeout' });
  assert.equal(f.failures[0].stop_complete, false); assert.equal(f.failures[0].close_complete, true);
});

test('candidate server saves failure before any round and permanently denies a retry', async t => {
  // Arrange
  const stored = [];
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    persist: async () => {}, persistProof: async () => {}, persistCandidate: async () => {}, validateDiagnostics: async value => value,
    inspectInstall: async () => ({ installation_verified: true }), persistCandidateFailure: async value => stored.push(value) });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  const idle = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
    observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
  await post('/diagnostic-export', { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 10 }] });
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle, closed: state(context, 'before', true), finished: { failures: [] } }))
    assert.equal((await post('/part', { stage, value })).status, 200);
  assert.equal((await post('/candidate', {})).status, 200);
  // Act
  const failure = { schema: 'str005-candidate-recovery-failure-v1', first_failure: { phase: 'status', category: 'v2_retained_evidence' },
    stop_complete: true, close_complete: true, closed: { ...state(context, 'candidate', true), deviceRestorationConfirmed: false } };
  assert.equal((await post('/candidate-recovery-failure', failure)).status, 200);
  const result = applyCandidateFailure({ complete: true, blockers: [], baseline_complete: true }, stored[0]);
  // Assert
  assert.equal(result.complete, false); assert.equal(result.candidate_close_confirmed, true);
  assert.equal(stored[0].stop_complete, false);
  assert.deepEqual(result.first_failure, failure.first_failure);
  assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: idle })).status, 400);
  assert.equal((await post('/candidate-recovery-failure', failure)).status, 400);
});

test('the production finalizer seals typed candidate failure without claiming qualification', async t => {
  // Arrange
  const { mkdtemp, realpath, rm } = await import('node:fs/promises');
  const { tmpdir } = await import('node:os');
  const { resolve } = await import('node:path');
  const { writeNew, proof, verifyInventory } = await import('../str005-noise-serial/files.mjs');
  const { sealProbeResult } = await import('./candidate-failure.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'candidate-partial-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const failure = { schema: 'str005-candidate-recovery-failure-v1', first_failure: { phase: 'status', category: 'v2_retained_evidence' },
    stop_complete: true, close_complete: true, closed_state_valid: true };
  await writeNew(resolve(root, 'candidate-recovery-failure.json'), failure);
  // Act
  const saved = (await proof(root, 'candidate-recovery-failure.json')).value;
  const outcome = await sealProbeResult(root, applyCandidateFailure({ complete: true, blockers: [], baseline_complete: true }, saved));
  // Assert
  assert.equal(outcome.complete, false); assert.equal(outcome.baseline_complete, true);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  assert.deepEqual((await proof(root, 'result.json')).value.first_failure, failure.first_failure);
  await assert.rejects(sealProbeResult(root, outcome));
});

test('an already closed port cannot substitute for confirmed device restoration', async () => {
  // Arrange
  const failures = [];
  const closed = { ...state(context, 'candidate', true), deviceRestorationConfirmed: false };
  const run = createCandidateRecovery({ gate: { stratumV2Possession: async () => { throw Error('disconnected'); } },
    published: () => closed, begin: async () => {}, collect: async () => {}, failure: async value => failures.push(value) });
  // Act
  const result = await run();
  // Assert
  assert.equal(result.complete, false);
  assert.equal(failures[0].stop_complete, false);
  assert.equal(failures[0].close_complete, true);
});
