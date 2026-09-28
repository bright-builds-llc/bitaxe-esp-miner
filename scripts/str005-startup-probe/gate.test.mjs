import test from 'node:test';
import assert from 'node:assert/strict';
import { dirname, resolve } from 'node:path';
import { verifyGateCompatibility } from './gate-compatibility.mjs';
const maybeGateRoot = process.env.STARTUP_GATE_ROOT ?? (process.argv[2] ? dirname(resolve(process.argv[2])) : undefined);
test('actual Gate functions accept zero renewals and begin the 20-second timer after Start reply', { skip: !maybeGateRoot }, async () => {
  const result = await verifyGateCompatibility(maybeGateRoot);
  assert.equal(result.zeroRenewalsAccepted, true); assert.equal(result.renewalOrigin, 'completed-controller-start');
  assert.equal(result.renewAfterMilliseconds, 20000); assert.equal(result.hardwareExercised, false);
});
