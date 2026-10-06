import test from 'node:test';
import assert from 'node:assert/strict';
import { createShareCoordinator } from './client.mjs';
function fixture(renewalsConfirmed = 0) {
  const calls = []; let running = false;
  const gate = { state: () => ({ running, heartbeatSuppressed: false, renewalsConfirmed, qualification: { generation: 3 } }),
    async startWindow() { calls.push('start'); running = true; }, async stop() { calls.push('stop'); running = false; },
    async close() { calls.push('close'); }, async stratumV2Status(scope, id) { assert.equal(id, 'known'); calls.push('status'); return {}; } };
  return { calls, gate, options: { gate, prepare: async () => ({ generation: 3, attemptId: 'known', binding: 'bound' }),
    post: async () => ({ selected: true, selectionSha256: 'a'.repeat(64) }), record: async () => { calls.push('record'); },
    recover: async () => { calls.push('recover'); }, release: async () => { calls.push('release'); } } };
}
for (const renewals of [0, 1, 2]) test(`verified ACK stops immediately with ${renewals} actual renewals`, async () => {
  const f = fixture(renewals), run = createShareCoordinator(f.options);
  const result = await run();
  assert.equal(result.complete, true); assert.deepEqual(f.calls.slice(0, 4), ['start', 'status', 'stop', 'record']);
  await assert.rejects(run(), /share_consumed/u);
});
test('independent deadline requests Stop while status read remains blocked', async () => {
  // Arrange
  const f = fixture(); let resolveRead;
  f.gate.stratumV2Status = () => new Promise(resolve => { resolveRead = resolve; });
  let posts = 0; f.options.post = async () => { posts++; return { selected: true }; };
  // Act
  const result = await createShareCoordinator({ ...f.options, limits: { observeMs: 15 } })();
  resolveRead({}); await new Promise(resolve => setTimeout(resolve, 5));
  // Assert
  assert.equal(result.complete, false); assert.ok(f.calls.includes('stop')); assert.ok(f.calls.includes('release')); assert.equal(posts, 0);
});
test('late Start reply receives another Stop and Close without repeated Start', async () => {
  const f = fixture(); let resolveStart;
  f.gate.startWindow = () => { f.calls.push('start'); return new Promise(resolve => { resolveStart = resolve; }); };
  const result = await createShareCoordinator({ ...f.options, limits: { replyMs: 5 } })();
  resolveStart(); await new Promise(resolve => setTimeout(resolve, 10));
  assert.equal(result.complete, false); assert.equal(f.calls.filter(x => x === 'start').length, 1);
  assert.deepEqual(f.calls.slice(-2), ['stop', 'close']);
});
for (const phase of ['post', 'recover', 'record']) test(`${phase} failure preserves independent cleanup`, async () => {
  const f = fixture(); f.options[phase] = async () => { throw Error('failure'); };
  const result = await createShareCoordinator(f.options)();
  assert.equal(result.complete, false); assert.ok(f.calls.includes('stop')); assert.ok(f.calls.includes('close')); assert.ok(f.calls.includes('release'));
});
test('third renewal fails closed before requesting share evidence', async () => {
  const f = fixture(3); const result = await createShareCoordinator(f.options)();
  assert.equal(result.complete, false); assert.equal(f.calls.includes('status'), false); assert.ok(f.calls.includes('stop'));
});

test('first typed share timeout is persisted after Stop request separately from cleanup failures', async () => {
  // Arrange
  const f = fixture(); let saved;
  f.gate.stratumV2Status = async () => { throw Object.assign(Error('sensitive detail'), { category: 'timeout' }); };
  f.gate.stop = async () => { f.calls.push('stop'); throw Error('cleanup failed'); };
  // Act
  const result = await createShareCoordinator({ ...f.options, recordFailure: async value => { f.calls.push('failure'); saved = value; } })();
  // Assert
  assert.equal(saved.phase, 'share'); assert.equal(saved.category, 'timeout'); assert.ok(Number.isFinite(saved.observedAtMs));
  assert.ok(f.calls.indexOf('stop') < f.calls.indexOf('failure')); assert.equal(result.firstFailure, 'share');
  assert.equal(JSON.stringify(saved).includes('sensitive'), false); assert.ok(f.calls.includes('close'));
});

test('internal Start deadline records timeout category without an underlying Gate exception', async () => {
  const f = fixture(); let saved;
  f.gate.startWindow = () => new Promise(() => {});
  await createShareCoordinator({ ...f.options, limits: { replyMs: 5 }, recordFailure: async value => { saved = value; } })();
  assert.equal(saved.phase, 'start'); assert.equal(saved.category, 'timeout');
});

test('a renewal probe keeps running after the ACK until the Gate confirms one renewal', async () => {
  // Arrange
  let renewals = 0, polls = 0; const f = fixture(); const records = [];
  f.gate.state = () => { if (++polls > 6) renewals = 1;
    return { running: !f.calls.includes('stop'), heartbeatSuppressed: false, renewalsConfirmed: renewals, qualification: { generation: 3 } }; };
  f.options.record = async value => { records.push(value); };
  // Act
  const result = await createShareCoordinator({ ...f.options, limits: { minRenewals: 1, pollMs: 1 } })();
  // Assert
  assert.equal(result.complete, true);
  assert.equal(records[0].proof.renewalsConfirmed, 1);
  assert.equal(f.calls.filter(call => call === 'stop').length, 1);
});

test('a renewal probe without a renewal before the deadline fails in the renewal phase and still stops', async () => {
  // Arrange
  const f = fixture(0); const records = [];
  f.options.record = async value => { records.push(value); };
  // Act
  const result = await createShareCoordinator({ ...f.options, limits: { minRenewals: 1, observeMs: 30, pollMs: 1 } })();
  // Assert
  assert.equal(result.complete, false);
  assert.equal(records[0].firstFailure, 'renewal');
  assert.ok(f.calls.includes('stop')); assert.ok(f.calls.includes('release'));
});
