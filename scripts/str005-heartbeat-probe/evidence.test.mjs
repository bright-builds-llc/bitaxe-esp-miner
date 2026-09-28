import test from 'node:test';
import assert from 'node:assert/strict';
import { judgeFault } from './evidence.mjs';
import { fixture } from './fault.fixture.mjs';
import { argumentsFor } from './contract.mjs';
test('heartbeat proof accepts zero shares and retains rejected cancellation outcome', () => {
  const { run, record, q } = fixture();
  assert.equal(judgeFault(run, record, q).heartbeatToRevocationMs, 2800);
  assert.equal(record.outcome, 'rejected'); assert.equal(record.shareFacts.length, 0);
});
for (const [name, change] of [
  ['early revocation', f => f.q.gate_closed_ms--],
  ['late revocation', f => f.q.gate_closed_ms = 13001],
  ['late shutdown', f => f.q.shutdown_started_ms = 13001],
  ['shutdown before revocation', f => f.q.shutdown_started_ms = 12799],
  ['lease expiry instead', f => f.q.revocation_reason = 'lease_or_budget_expired'],
  ['wrong generation', f => f.q.generation++],
  ['missing dispatch', f => f.run.dispatchStatus.record.events.pop()],
  ['no lease headroom', f => f.run.headroom.leaseRemainingMs = 4999],
  ['no work headroom', f => f.run.headroom.workGateRemainingMs = 4999],
  ['short shutdown reserve', f => f.record.authorityDeadlineDeviceUs = 125549999],
  ['late suppression', f => f.run.suppressionConfirmedAt = 15001],
  ['missing fault checkpoint', f => f.run.suppressedState.authorizationRecovery = undefined],
  ['renewal issued', f => f.run.suppressedState.renewalsConfirmed++],
  ['short passive tail', f => f.run.stopRequestedAt--],
  ['short device tail', f => f.record.observedAtUs = 17799999],
  ['dispatch after cutoff', f => f.record.events.push({ ...f.record.events[0], kind: 'asic_dispatch', atDeviceUs: 12800001 })],
  ['earlier authority failure', f => f.record.firstFailure.atDeviceUs = 12799000],
  ['unrelated protocol failure', f => f.record.firstFailure.category = 'protocol'],
]) test(`heartbeat rejects ${name}`, () => {
  const value = fixture(); change(value);
  assert.throws(() => judgeFault(value.run, value.record, value.q));
});
test('effect commands remain unavailable', () => {
  for (const command of ['serve', 'start', 'preflight', 'flash', 'clear'])
    assert.throws(() => argumentsFor([command, '--private-root', '/private']));
});
