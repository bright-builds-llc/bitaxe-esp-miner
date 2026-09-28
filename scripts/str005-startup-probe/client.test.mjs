import test from 'node:test';
import assert from 'node:assert/strict';
import { createCoordinator, collectRecovery } from './client.mjs';
function fixture() {
  let time = 0, running = false, counter = 0; const calls = [];
  const gate = { state: () => ({ running, heartbeatSuppressed: false, renewalsConfirmed: 0, qualification: { generation: 7, work_dispatched: counter } }),
    async startWindow() { calls.push('start'); running = true; }, async refresh() { calls.push('refresh'); time += 100; counter++; },
    async stop() { calls.push('stop'); running = false; }, async close() { calls.push('close'); } };
  return { calls, gate, options: { gate, now: () => time, sleep: async ms => { time += ms; },
    prepare: async () => ({ generation: 7 }), record: async () => { calls.push('record'); }, recover: async () => { calls.push('recover'); },
    release: async () => { calls.push('release'); } } };
}
test('one completed Start and a same-generation dispatch increase request Stop before persistence', async () => {
  const f = fixture(), run = createCoordinator(f.options);
  const result = await run();
  assert.equal(result.complete, true); assert.equal(result.dispatchIncreased, true);
  assert.deepEqual(f.calls.slice(0, 4), ['start', 'refresh', 'stop', 'record']);
  await assert.rejects(run(), /startup_consumed/u);
});
test('late reply after timeout performs another Stop and Close without another Start', async () => {
  const f = fixture(); let resolveStart;
  f.gate.startWindow = () => { f.calls.push('start'); return new Promise(resolve => { resolveStart = resolve; }); };
  const result = await createCoordinator({ ...f.options, limits: { replyMs: 5 } })();
  resolveStart(); await new Promise(resolve => setTimeout(resolve, 10));
  assert.equal(result.complete, false); assert.equal(result.lateReplyPending, true);
  assert.equal(f.calls.filter(value => value === 'start').length, 1); assert.deepEqual(f.calls.slice(-2), ['stop', 'close']);
});
test('dispatch wait is bounded from invocation, not a fresh unbounded wait', async () => {
  const f = fixture(); let time = 0;
  f.options.now = () => time; f.options.sleep = async ms => { time += ms; };
  const original = f.gate.startWindow; f.gate.startWindow = async () => { await original(); time = 30000; };
  f.gate.refresh = async () => { f.calls.push('refresh'); time = Math.min(time + 1000, 35000); };
  let saved;
  const result = await createCoordinator({ ...f.options, record: async value => { if (value.proof !== undefined) saved = value; } })();
  assert.equal(result.complete, false); assert.equal(saved.stopRequestedAt, 35000);
});
for (const failure of ['stop', 'record', 'recover']) test(`${failure} failure does not bypass Close or fixture release`, async () => {
  const f = fixture(); const reject = async () => { f.calls.push(failure); throw Error('synthetic'); };
  if (failure === 'stop') f.gate.stop = reject; else f.options[failure] = reject;
  const result = await createCoordinator(f.options)();
  assert.equal(result.complete, false); assert.ok(f.calls.includes('close')); assert.equal(f.calls.at(-1), 'release');
});
test('failed recovery ledger read cannot suppress later independent status and diagnostics', async () => {
  const calls = [], gate = { async refresh() { calls.push('state'); }, state: () => ({}),
    async reviewQualificationAttempts() { calls.push('ledger'); throw Error('read failed'); }, async reviewBudget() { calls.push('budget'); return {}; },
    async stratumV2Possession() { return 'binding'; }, async stratumV2Status() { calls.push('status'); return {}; },
    async exportDiagnostics() { calls.push('diagnostics'); } };
  await assert.rejects(collectRecovery({ gate, campaignId: 'test', attemptId: 'test', statusMode: 'not_invoked', save: async () => {} }), /recovery_incomplete/u);
  assert.deepEqual(calls, ['ledger', 'budget', 'diagnostics', 'state', 'status']);
});

test('timed-out read cannot submit late evidence after its collection phase closed', async () => {
  let resolveLedger; const saved = [];
  const gate = { async refresh() {}, state: () => ({}), reviewQualificationAttempts: () => new Promise(resolve => { resolveLedger = resolve; }),
    async reviewBudget() { return {}; }, async stratumV2Possession() { return 'binding'; }, async stratumV2Status() { return {}; }, async exportDiagnostics() {} };
  await assert.rejects(collectRecovery({ gate, campaignId: 'test', attemptId: 'test', statusMode: 'not_invoked', limitMs: 5, save: async stage => { saved.push(stage); } }));
  resolveLedger({}); await new Promise(resolve => setTimeout(resolve, 10));
  assert.equal(saved.includes('ledger'), false); assert.ok(saved.includes('status'));
});

test('startup preserves typed initial failure separately from cleanup outcomes', async () => {
  const f = fixture(); let saved;
  f.gate.startWindow = async () => { throw Object.assign(Error('private'), { category: 'timeout' }); };
  await createCoordinator({ ...f.options, recordFailure: async value => { saved = value; } })();
  assert.equal(saved.phase, 'start'); assert.equal(saved.category, 'timeout'); assert.ok(f.calls.includes('close'));
});

test('same-session recovery retains accounting and diagnostics before a failing known status query', async () => {
  const calls = [], saved = [];
  const gate = { reviewQualificationAttempts: async () => { calls.push('ledger'); return {}; }, reviewBudget: async () => { calls.push('budget'); return {}; },
    exportDiagnostics: async () => { calls.push('diagnostics'); }, refresh: async () => { calls.push('state'); }, state: () => ({}),
    stratumV2Possession: async () => 'binding', stratumV2Status: async (_scope, id) => { assert.equal(id, 'known'); calls.push('status'); throw Error('lost'); } };
  await assert.rejects(collectRecovery({ gate, campaignId: 'test', attemptId: 'known', statusMode: 'confirmed', save: async stage => { saved.push(stage); } }));
  assert.deepEqual(calls, ['ledger', 'budget', 'diagnostics', 'state', 'status']); assert.deepEqual(saved, ['ledger', 'original_budget', 'state']);
});
