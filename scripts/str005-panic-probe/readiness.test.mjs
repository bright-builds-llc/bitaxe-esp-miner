import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { evaluateReadiness } from './readiness.mjs';
import { awaitSelfTestReady, runReadySelfTest } from './self-test-readiness.mjs';
import { sourceFingerprint } from './store-diagnostics.mjs';
import { createProbeServer } from './server.mjs';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) }, scope: 'share',
  selfTestEnabled: true, installEnabled: true, storeAuditRequired: true };
function diagnostics(receipt = true) {
  return { schema: 'worker-diagnostic-export-v1', observations: [
    { category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 10 },
    { category: 'runtime_identity', firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 },
    ...(receipt ? [{ category: 'core_dump_store_receipt', authoritative: false, origin: 'current_boot', status: 'valid',
      source_hash: sourceFingerprint(context.firmware_commit), boot_ordinal: '3', stage: 'ready', capacity_bytes: 974848,
      requested_bytes: 'unavailable', prepared_bytes: 'unavailable', init_result: 'unavailable', prepare_result: 'unavailable',
      start_result: 'unavailable', end_result: 'unavailable', store_result: 'unavailable', self_test_marked: false }] : []) ] };
}
function idle() { return { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }; }

test('only absent current receipt is pending; contradictory rows and stale captures reject', () => {
  // Arrange / Act / Assert
  assert.deepEqual(evaluateReadiness(diagnostics(false), context, idle(), null, 1000, 1000), { state: 'pending' });
  assert.deepEqual(evaluateReadiness(diagnostics(), context, idle(), null, 1000, 1000), { state: 'ready' });
  for (const mutate of [d => { d.observations[2].boot_ordinal = '2'; }, d => { d.observations[2].source_hash = 'f'.repeat(16); },
    d => { d.observations[2].stage = 'store_entered'; }, d => { d.observations[2].capacity_bytes = 0; },
    d => { d.observations[2].status = 'corrupt'; }, d => { d.observations.push({ ...d.observations[2], status: 'corrupt' }); },
    d => { d.observations[1].app_elf_sha256 = '0'.repeat(64); }, d => { d.observations[0].boot_ordinal = 2; }]) {
    const value = diagnostics(); mutate(value); assert.throws(() => evaluateReadiness(value, context, idle(), null, 1000, 1000));
  }
  const capture = { ...context, captureExisting: true };
  const review = { schema: 'str005-existing-capture-review-v1', capture_admitted: true, boot_ordinal: 3, observed_at_unix_ms: 1000 };
  assert.throws(() => evaluateReadiness(diagnostics(false), capture, idle(), review, 121001, 121001));
  assert.throws(() => evaluateReadiness(diagnostics(), context, idle(), null, 1000, 31001));
});

test('late receipt requires fresh exports and bounded cadence, never sleep-only readiness', async () => {
  // Arrange
  let time = 0, exports = 0, queries = 0;
  // Act
  const result = await awaitSelfTestReady({ now: () => time, wait: async ms => { time += ms; },
    exportDiagnostics: async () => { exports++; return { diagnostic_export_saved: true, review_file: 'fixed' }; },
    readiness: async () => { queries++; return { state: exports < 3 ? 'pending' : 'ready' }; } });
  // Assert
  assert.deepEqual(result, { state: 'ready' }); assert.equal(exports, 3); assert.equal(queries, 3); assert.equal(time, 500);
});

test('deadline or late ready response cannot become readiness success', async () => {
  // Arrange / Act / Assert
  let time = 0;
  await assert.rejects(awaitSelfTestReady({ now: () => time, wait: async ms => { time += ms; },
    exportDiagnostics: async () => {}, readiness: async () => ({ state: 'pending' }) }), /readiness_timeout/u);
  assert.equal(time, 30000);
  time = 0;
  await assert.rejects(awaitSelfTestReady({ now: () => time, exportDiagnostics: async () => {},
    readiness: async () => { time = 30000; return { state: 'ready' }; } }), /readiness_timeout/u);
});

test('preclaim timeout never calls claim or fault and stalled Stop cannot prevent Close', async () => {
  // Arrange
  const calls = [], failures = [];
  const gate = { exportDiagnostics: async () => new Promise(() => {}), stop: async () => { calls.push('stop'); await new Promise(() => {}); },
    close: async () => calls.push('close') };
  // Act
  const result = await runReadySelfTest({ gate, published: () => state(context, 'candidate', true), readiness: async () => ({ state: 'ready' }),
    claim: async () => { calls.push('claim'); }, fault: async () => calls.push('fault'), exportEvidence: () => {}, saveEvidence: async () => {},
    saveFailure: async value => failures.push(value), timeoutMs: 5, cleanupMs: 5 });
  // Assert
  assert.equal(result.complete, false); assert.deepEqual(calls, ['stop', 'close']);
  assert.equal(failures[0].claim_created, false); assert.deepEqual(failures[0].first_failure, { phase: 'readiness', category: 'timeout' });
});

test('actual server readiness is read-only pending then ready without a claim', async t => {
  // Arrange
  const claims = [], failures = []; let sourceCurrent = true;
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    verifyEffect: async () => { if (!sourceCurrent) throw Object.assign(Error('changed'), { code: 'panic_source_changed' }); },
    now: () => 1000, persist: async () => {}, persistProof: async () => {}, persistCandidate: async () => {}, persistCandidateDiagnostics: async () => {},
    validateDiagnostics: async v => v, inspectInstall: async () => ({ installation_verified: true }), verifyNativeAudit: async () => {},
    persistClaim: async v => claims.push(v), persistReadinessFailure: async v => failures.push(v) });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  await post('/diagnostic-export', diagnostics());
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle(), closed: state(context, 'before', true), finished: { failures: [] } }))
    assert.equal((await post('/part', { stage, value })).status, 200);
  assert.equal((await post('/candidate', {})).status, 200);
  // Act / Assert
  const query = { state: state(context), status: idle() };
  await post('/diagnostic-export', diagnostics(false));
  assert.deepEqual(await (await post('/self-test-readiness', query)).json(), { state: 'pending' });
  assert.equal(claims.length, 0);
  await post('/diagnostic-export', diagnostics());
  assert.deepEqual(await (await post('/self-test-readiness', query)).json(), { state: 'ready' });
  assert.equal(claims.length, 0);
  sourceCurrent = false;
  assert.equal((await post('/self-test-readiness', query)).status, 400);
  assert.equal((await post('/self-test-claim', { ...query, ledger })).status, 400);
  sourceCurrent = true;
  const wrong = diagnostics(); wrong.observations[2].capacity_bytes = 0;
  await post('/diagnostic-export', wrong);
  assert.equal((await post('/self-test-readiness', query)).status, 400);
  assert.equal((await post('/self-test-claim', { ...query, ledger })).status, 400);
  assert.equal(claims.length, 0);
  assert.equal((await post('/self-test-readiness-failure', { schema: 'str005-self-test-readiness-failure-v1', first_failure: { phase: 'readiness', category: 'rejected' },
    claim_created: false, stop_complete: true, close_complete: true, closed: { ...state(context, 'candidate', true), deviceRestorationConfirmed: false } })).status, 200);
  assert.equal(failures.length, 1); assert.equal(failures[0].stop_complete, false);
  await post('/diagnostic-export', diagnostics());
  assert.equal((await post('/self-test-claim', { ...query, ledger })).status, 400);
});

test('claim timeout is bounded independently from cleanup and never invokes fault', async () => {
  // Arrange
  const calls = [], failures = [];
  const gate = { exportDiagnostics: async () => {}, stop: async () => calls.push('stop'), close: async () => calls.push('close') };
  // Act
  await runReadySelfTest({ gate, published: () => state(context, 'candidate', true), readiness: async () => ({ state: 'ready' }),
    claim: async () => { calls.push('claim'); await new Promise(() => {}); }, fault: async () => calls.push('fault'),
    exportEvidence: () => {}, saveEvidence: async () => {}, saveFailure: async v => failures.push(v), operationMs: 5, cleanupMs: 1000 });
  // Assert
  assert.deepEqual(calls, ['claim', 'stop', 'close']);
  assert.deepEqual(failures[0].first_failure, { phase: 'claim', category: 'timeout' });
  assert.equal(failures[0].claim_created, false);
});

test('a returned fault call without observer evidence stays incomplete and releases', async () => {
  // Arrange
  const calls = [], failures = [];
  const gate = { exportDiagnostics: async () => {}, stop: async () => calls.push('stop'), close: async () => calls.push('close') };
  // Act
  const result = await runReadySelfTest({ gate, published: () => state(context, 'candidate', true),
    readiness: async () => ({ state: 'ready' }), claim: async () => ({}), fault: async () => calls.push('fault'),
    exportEvidence: () => undefined, saveEvidence: async () => calls.push('evidence'), saveFailure: async value => failures.push(value) });
  // Assert
  assert.equal(result.complete, false);
  assert.deepEqual(calls, ['fault', 'stop', 'close']);
  assert.deepEqual(failures[0].first_failure, { phase: 'evidence', category: 'rejected' });
});
