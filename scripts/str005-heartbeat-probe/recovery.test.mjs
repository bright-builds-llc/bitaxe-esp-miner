import test from 'node:test';
import assert from 'node:assert/strict';
import { recoveryFixture } from './recovery.fixture.mjs';
import { judgeHeartbeat } from './evidence.mjs';
test('fresh recovery completes heartbeat proof without an accepted share', () => {
  const f = recoveryFixture();
  assert.equal(judgeHeartbeat(f.parts, f.context).complete, true);
});
for (const [name, mutate] of [
  ['missing checkpoint', p => { delete p.recovery.state.authorizationRecovery; }],
  ['changed checkpoint', p => { p.recovery.state.authorizationRecovery = { ...p.recovery.state.authorizationRecovery, checkpointId: Buffer.alloc(16, 5).toString('base64url') }; }],
  ['pending charge', p => { p.recovery.ledger.pending = true; }],
  ['missing reservation charge', p => { p.recovery.ledger.total_charged_ms -= 180000; }],
  ['same session recovery', p => { p.recovery.status.observation.serialTransportEpoch = 3; }],
  ['reset before recovery', p => { p.recovery.status.observation.bootOrdinal++; p.recovery.status.record.bootOrdinal++; }],
  ['hot final temperature', p => { p.recovery.state.qualification.chip_temp_celsius = 46; }],
  ['unreleased host', p => { p.hostReleased = false; }],
  ['unreleased device', p => { p.recovery.status.record.resources.fenceRetained = true; }],
  ['unreleased serial', p => { p.recovery.closed.serialOwnershipReleased = false; }],
]) test(`heartbeat cannot complete with ${name}`, () => {
  const f = recoveryFixture(); mutate(f.parts);
  assert.throws(() => judgeHeartbeat(f.parts, f.context));
});
