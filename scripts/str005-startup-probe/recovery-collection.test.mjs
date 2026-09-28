import test from 'node:test';
import assert from 'node:assert/strict';
import { createRecoveryCollection } from './recovery-collection.mjs';
import { discoverCurrentStatus } from './retained-status.mjs';
function fixture() {
  const calls = [], saved = new Map();
  const gate = { stratumV2Possession: async () => 'fresh-status', reviewQualificationAttempts: async () => { calls.push('ledger'); return {}; }, reviewBudget: async () => { calls.push('budget'); return {}; },
    exportDiagnostics: async () => { calls.push('diagnostics'); }, stop: async () => { calls.push('stop'); },
    refresh: async () => { calls.push('state'); }, state: () => ({}), close: async () => { calls.push('close'); },
    stratumV2Status: async (_scope, id) => { calls.push(id === null ? 'idle' : 'known'); return {}; } };
  return { gate, calls, saved, options: { gate, begin: async () => ({ binding: 'fresh', campaignId: 'test', attemptId: 'known', statusMode: 'confirmed' }),
    save: async (stage, value) => { saved.set(stage, value); } } };
}
for (const [phase, method] of [['ledger', 'reviewQualificationAttempts'], ['original_budget', 'reviewBudget'], ['diagnostics', 'exportDiagnostics'],
  ['stop', 'stop'], ['state', 'refresh'], ['status', 'stratumV2Status'], ['closed', 'close']]) test(`${phase} failure never skips other collection or cleanup stages`, async () => {
  const f = fixture(); f.gate[method] = async () => { f.calls.push(method); throw Object.assign(Error('private payload'), { category: 'timeout' }); };
  const result = await createRecoveryCollection(f.options)();
  assert.equal(result.firstFailure.phase, phase); assert.equal(result.firstFailure.category, 'timeout');
  assert.ok(f.calls.includes('close')); assert.ok(f.saved.has('closed')); assert.ok(f.saved.has('finished'));
  assert.equal(JSON.stringify(result).includes('private payload'), false);
});
test('begin and persistence failure still execute Stop and Close and preserve earliest error', async () => {
  const f = fixture(); f.options.begin = async () => { throw Object.assign(Error('secret'), { category: 'command_rejected' }); };
  f.options.save = async () => { throw Error('disk failed'); };
  const result = await createRecoveryCollection(f.options)();
  assert.deepEqual(f.calls, ['stop', 'close']); assert.deepEqual(result.firstFailure, { phase: 'begin', category: 'command_rejected' });
});
test('late read result cannot write after its bounded stage ends', async () => {
  const f = fixture(); let resolveLedger;
  f.gate.reviewQualificationAttempts = () => new Promise(resolve => { resolveLedger = resolve; });
  const result = await createRecoveryCollection({ ...f.options, readMs: 5 })();
  resolveLedger({}); await new Promise(resolve => setTimeout(resolve, 5));
  assert.equal(f.saved.has('ledger'), false); assert.equal(result.firstFailure.category, 'timeout'); assert.ok(f.saved.has('closed'));
});
for (const category of ['command_rejected', 'timeout', 'closed']) test(`current discovery never falls back on ${category}`, async () => {
  const calls = [], gate = { stratumV2Status: async (_scope, id) => { calls.push(id); throw Object.assign(Error('private'), { category }); } };
  await assert.rejects(discoverCurrentStatus(gate, 'known', 'binding'));
  assert.deepEqual(calls, [null]);
});
test('only typed local idle correlation permits exact-known fallback', async () => {
  const calls = [], gate = { stratumV2Status: async (_scope, id) => { calls.push(id);
    if (id === null) throw Object.assign(Error('private'), { category: 'v2_idle_correlation' }); return { state: 'terminal' }; } };
  assert.equal((await discoverCurrentStatus(gate, 'known', 'binding')).state, 'terminal'); assert.deepEqual(calls, [null, 'known']);
});

test('stage timeout closes its server ticket before proceeding and rejects late persistence', async () => {
  // Arrange
  const f = fixture(); let resolveLedger; const active = new Set(), events = [];
  f.gate.reviewQualificationAttempts = () => new Promise(resolve => { resolveLedger = resolve; });
  // Act
  const result = await createRecoveryCollection({ ...f.options, readMs: 5,
    beforeStage: async phase => { active.add(phase); events.push(`open:${phase}`); return phase; },
    afterStage: async (phase, ticket) => { assert.equal(ticket, phase); active.delete(phase); events.push(`close:${phase}`); },
    save: async (phase, value, ticket) => { if (ticket !== undefined) assert.ok(active.has(ticket)); f.saved.set(phase, value); },
  })();
  resolveLedger({}); await new Promise(resolve => setTimeout(resolve, 5));
  // Assert
  assert.equal(result.firstFailure.phase, 'ledger'); assert.equal(f.saved.has('ledger'), false);
  assert.ok(events.indexOf('close:ledger') < events.indexOf('open:original_budget')); assert.equal(active.size, 0);
});

test('all operation and stage-close errors retain an admissible earliest receipt', async () => {
  // Arrange
  const f = fixture(); const reject = async () => { throw Object.assign(Error('private'), { category: 'timeout' }); };
  for (const method of ['reviewQualificationAttempts', 'reviewBudget', 'exportDiagnostics', 'stop', 'refresh', 'stratumV2Status', 'close']) f.gate[method] = reject;
  const { validateRecoveryErrors } = await import('./recovery-errors.mjs');
  // Act
  const result = await createRecoveryCollection({ ...f.options, beforeStage: async phase => phase, afterStage: reject })();
  // Assert
  const errors = f.saved.get('errors'); validateRecoveryErrors(errors);
  assert.ok(errors.errors.length > 10); assert.deepEqual(errors.firstFailure, { phase: 'ledger', category: 'timeout' });
  assert.deepEqual(result.firstFailure, errors.firstFailure); assert.ok(f.saved.has('finished'));
});

test('post-Stop possession failure stays in status phase and cannot skip Close', async () => {
  const f = fixture();
  f.gate.stratumV2Possession = async () => { throw Object.assign(Error('private'), { category: 'timeout' }); };
  const result = await createRecoveryCollection(f.options)();
  assert.deepEqual(result.firstFailure, { phase: 'status', category: 'timeout' });
  assert.ok(f.calls.includes('stop')); assert.ok(f.calls.includes('close')); assert.equal(f.calls.includes('known'), false);
  assert.ok(f.saved.has('ledger')); assert.ok(f.saved.has('closed'));
});

test('late post-Stop possession cannot launch a status read after its stage deadline', async () => {
  const f = fixture(); let resolvePossession;
  f.gate.stratumV2Possession = () => new Promise(resolve => { resolvePossession = resolve; });
  const result = await createRecoveryCollection({ ...f.options, readMs: 5 })();
  resolvePossession('late-binding'); await new Promise(resolve => setTimeout(resolve, 5));
  assert.equal(result.firstFailure.category, 'timeout'); assert.equal(f.calls.includes('known'), false); assert.ok(f.saved.has('closed'));
});
