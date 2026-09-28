import test from 'node:test';
import assert from 'node:assert/strict';
import { createHeartbeatCoordinator } from './client.mjs';
import { fixture } from './fault.fixture.mjs';
function setup(change = {}) {
  let clock = 0, suppressed = false; const calls = [], records = [], f = fixture();
  const gate = { state: () => ({ running: true, heartbeatSuppressed: suppressed, renewalsConfirmed: 0,
    qualification: { generation: 2, work_dispatched: 1 }, authorizationRecovery: suppressed ? { generation: 2, matched: null } : undefined }),
    startWindow: async () => { calls.push('start'); clock += 10000; }, refresh: async () => { calls.push('refresh'); clock += 100; },
    suppressHeartbeats: async () => { calls.push('suppress'); suppressed = true; return f.run.headroom; },
    stop: async () => { calls.push('stop'); }, close: async () => { calls.push('close'); }, ...change };
  const coordinator = createHeartbeatCoordinator({ gate, prepare: async () => { calls.push('prepare'); return { generation: 2 }; },
    readStatus: async () => f.run.dispatchStatus, record: async run => { calls.push('record'); records.push(structuredClone(run)); },
    recover: async () => { calls.push('recover'); }, release: async () => { calls.push('release'); }, now: () => clock,
    sleep: async ms => { calls.push(`sleep:${ms}`); clock += ms; },
    observer: { requireArmed: async () => { calls.push('armed'); }, requireAlive: async () => { calls.push('alive'); }, stop: async () => { calls.push('observer-stop'); } } });
  return { coordinator, calls, records };
}
test('one-shot heartbeat flow prearms, suppresses before renewal, waits passively and releases', async () => {
  // Arrange
  const f = setup();
  // Act
  const result = await f.coordinator();
  // Assert
  assert.equal(result.complete, true);
  assert.deepEqual(f.calls, ['prepare', 'armed', 'start', 'refresh', 'suppress', 'sleep:8000', 'alive', 'stop', 'record', 'recover', 'close', 'record', 'observer-stop', 'release']);
  assert.ok(f.records[0].suppressionConfirmedAt - f.records[0].startRepliedAt < 5000);
  assert.equal(f.records[0].dispatchStatus.schema, 'str005-recovery-status-v1');
  assert.equal(Object.hasOwn(f.records[0].dispatchStatus.observation, 'stationIpv4'), false);
  await assert.rejects(f.coordinator(), /heartbeat_consumed/);
});
test('failed suppression promptly stops and still collects independent recovery', async () => {
  // Arrange
  const f = setup({ suppressHeartbeats: async () => { throw Error('lost'); } });
  // Act
  const result = await f.coordinator();
  // Assert
  assert.equal(result.firstFailure, 'suppress'); assert.equal(f.calls.some(call => call.startsWith('sleep:')), false);
  assert.ok(f.calls.indexOf('stop') < f.calls.indexOf('record'));
  assert.deepEqual(f.calls.slice(-6), ['record', 'recover', 'close', 'record', 'observer-stop', 'release']);
});
test('failed Stop cannot skip Close or observer and fixture release', async () => {
  const f = setup({ stop: async () => { throw Error('stop'); } });
  const result = await f.coordinator();
  assert.deepEqual(result.failures, ['stop']); assert.ok(f.calls.includes('close')); assert.equal(f.calls.at(-1), 'release');
});
test('a late Start resolution repeats Stop and Close without repeating Start', async () => {
  // Arrange
  let resolveStart, starts = 0, stops = 0, closes = 0;
  const run = createHeartbeatCoordinator({ gate: { startWindow: () => { starts++; return new Promise(resolve => { resolveStart = resolve; }); },
    stop: async () => { stops++; }, close: async () => { closes++; } }, prepare: async () => ({ generation: 2 }),
    readStatus: async () => {}, record: async () => {}, recover: async () => {}, release: async () => {},
    observer: { requireArmed: async () => {}, stop: async () => {} }, limits: { replyMs: 5 } });
  // Act
  const result = await run(); resolveStart({}); await new Promise(resolve => setTimeout(resolve, 10));
  // Assert
  assert.equal(result.firstFailure, 'start'); assert.equal(starts, 1); assert.equal(stops, 2); assert.equal(closes, 2);
});
