import assert from 'node:assert/strict';
import test from 'node:test';
import { once } from 'node:events';
import { createBaselineCollector } from './client.mjs';
import { createProbeServer } from './server.mjs';
import { baselineConclusion, currentProof, validatePart } from './model.mjs';
import { ledger, original, state } from '../str005-noise-serial/test-fixture.mjs';
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) }, scope: 'share' };
const diagnostics = { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 10 }] };
function idle() { return { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }; }
function parts() { return { state: state(context, 'before'), closed: state(context, 'before', true), ledger, original_budget: original,
  diagnostics, status: validatePart('status', idle(), context), finished: { failures: [] } }; }

test('current proof measures an arbitrary fresh ordinal without claiming historical resources', () => {
  // Arrange
  const observed = parts(); observed.ledger = { ...ledger, next_ordinal: 41, last_completed_ordinal: 40, total_charged_ms: 6600000 };
  // Act
  const result = baselineConclusion(observed), proof = currentProof(context, observed, 1000);
  // Assert
  assert.equal(result.complete, true); assert.equal(result.historical_resource_proof, false);
  assert.equal(proof.ledger.next_ordinal, 41); assert.equal(proof.current_v2_idle, true);
});

test('pending accounting, unclosed serial or mismatched boot cannot authorize a current proof', () => {
  for (const mutate of [p => { p.ledger = { ...ledger, pending: true }; }, p => { p.closed.serialOwnershipReleased = false; },
    p => { p.diagnostics = { ...diagnostics, observations: [{ ...diagnostics.observations[0], boot_ordinal: 4 }] }; }]) {
    const value = structuredClone(parts()); mutate(value); assert.throws(() => currentProof(context, value));
  }
});

test('collector always closes after failed accounting and does not issue any effects', async () => {
  // Arrange
  const steps = [], saved = [];
  const gate = { refresh: async () => {}, reviewQualificationAttempts: async () => { throw Error('failed'); },
    reviewBudget: async () => original, exportDiagnostics: async () => {}, stratumV2Possession: async () => ({}),
    stratumV2Status: async () => idle(), stop: async () => steps.push('stop'), close: async () => steps.push('close') };
  const collect = createBaselineCollector({ gate, published: () => state(context, 'before'), campaignId: 'original', save: async (stage, value) => saved.push({ stage, value }) });
  // Act
  const result = await collect();
  // Assert
  assert.deepEqual(steps, ['stop', 'close']); assert.deepEqual(result.failures, ['ledger']);
  assert.equal(saved.at(-1).stage, 'finished'); await assert.rejects(collect(), /consumed/);
});

test('real loopback server denies Start, candidate, grant and self-test routes while retaining full validated diagnostics', async t => {
  // Arrange
  const saved = [], proofs = [];
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} },
    { persist: async (stage, value) => saved.push({ stage, value }), persistProof: async value => proofs.push(value), validateDiagnostics: async value => value });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = async (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  // Act / Assert
  for (const path of ['/grant', '/start', '/candidate', '/core-dump-self-test', '/flash']) assert.equal((await post(path, {})).status, 400);
  assert.equal((await post('/diagnostic-export', diagnostics)).status, 200);
  assert.deepEqual(saved[0].value, diagnostics);
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle(), closed: state(context, 'before', true), finished: { failures: [] } })) {
    assert.equal((await post('/part', { stage, value })).status, 200, stage);
  }
  assert.equal(proofs.length, 1); assert.equal(proofs[0].schema, 'str005-current-recovery-proof-v1');
  assert.equal((await post('/part', { stage: 'ledger', value: ledger })).status, 400);
});

test('stop failure still persists observed closure and marks the collection incomplete', async () => {
  // Arrange
  const saved = [];
  const gate = { refresh: async () => {}, reviewQualificationAttempts: async () => ledger, reviewBudget: async () => original,
    exportDiagnostics: async () => {}, stratumV2Possession: async () => ({}), stratumV2Status: async () => idle(),
    stop: async () => { throw Error('stop'); }, close: async () => {} };
  // Act
  const result = await createBaselineCollector({ gate, published: () => state(context, 'before', true), campaignId: 'original',
    save: async (stage, value) => saved.push({ stage, value }) })();
  // Assert
  assert.deepEqual(result.failures, ['closed']); assert.equal(saved.find(row => row.stage === 'closed').value.serialOwnershipReleased, true);
});

test('candidate transition consumes actual install review and self-test claim is one-use', async t => {
  // Arrange
  const c = { ...context, installEnabled: true, selfTestEnabled: true }, stored = [];
  let installChecks = 0;
  const server = createProbeServer({ root: '/unused', context: c, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    persist: async () => {}, persistProof: async () => {}, validateDiagnostics: async value => value,
    verifyNativeAudit: async () => {},
    inspectInstall: async () => { installChecks += 1; return { installation_verified: true }; },
    persistCandidate: async value => stored.push(value), persistClaim: async value => stored.push(value),
    createRecoveryRound: async path => stored.push(path), persistCandidatePart: async (path, value) => stored.push({ path, value }),
    persistCandidateProof: async (path, value) => stored.push({ path, value }), persistCandidateDiagnostics: async () => {},
  });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = async (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  // Act / Assert
  assert.equal((await post('/candidate', {})).status, 400); assert.equal(installChecks, 0);
  await post('/diagnostic-export', diagnostics);
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle(), closed: state(context, 'before', true), finished: { failures: [] } })) await post('/part', { stage, value });
  const response = await post('/candidate', {}); assert.equal(response.status, 200);
  const config = await response.json(); assert.equal(config.expectedAppElfSha256, context.app_elf_sha256); assert.equal(installChecks, 1);
  assert.equal((await post('/candidate', {})).status, 400);
  const input = { state: state(context), ledger, status: idle() };
  const claim = await post('/self-test-claim', input); assert.equal(claim.status, 200);
  assert.equal((await claim.json()).expectedBootOrdinal, 3);
  assert.equal((await post('/self-test-claim', input)).status, 400);
  for (let sequence = 1; sequence <= 8; sequence += 1) {
    const observed = idle(); observed.observation.bootOrdinal = sequence + 2;
    const beginning = await post('/candidate-recovery-begin', { state: state(context), status: observed });
    assert.equal(beginning.status, 200); const round = await beginning.json(); assert.equal(round.sequence, sequence);
    assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: observed })).status, 400);
    await post('/diagnostic-export', { ...diagnostics, observations: [{ ...diagnostics.observations[0], boot_ordinal: sequence + 2 }] });
    for (const [stage, value] of Object.entries({ state: state(context), ledger, original_budget: original, status: observed,
      closed: state(context, 'candidate', true), finished: { failures: [] } })) {
      assert.equal((await post('/candidate-part', { sequence, stage, value })).status, 200, `${sequence}:${stage}`);
    }
    assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: observed })).status, 400);
    assert.ok(stored.some(row => row.path === `candidate-recovery-${String(sequence).padStart(3, '0')}/current-recovery.json`));
  }
  const ninth = idle(); ninth.observation.bootOrdinal = 11;
  assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: ninth })).status, 400);
});

test('current proof independently rejects mismatched published runtime identity despite positive booleans', () => {
  // Arrange / Act / Assert
  for (const key of ['state', 'closed']) {
    const value = parts(); value[key].expectedAppElfSha256 = '0'.repeat(64);
    assert.throws(() => currentProof(context, value), { code: 'panic_current_proof_identity' });
  }
});
