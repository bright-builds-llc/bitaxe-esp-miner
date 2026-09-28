import test from 'node:test';
import assert from 'node:assert/strict';
import { dirname, resolve } from 'node:path';
import { verifyGateCompatibility } from './gate-compatibility.mjs';
const gateRoot = process.env.STARTUP_GATE_ROOT ?? (process.argv[2] ? dirname(resolve(process.argv[2])) : undefined);
test('production Gate clock renews twice and normal Stop prevents third renewal', { skip: !gateRoot }, async () => {
  const result = await verifyGateCompatibility(gateRoot);
  assert.equal(result.actualAutomaticRenewals, 2); assert.equal(result.renewalOrigin, 'completed-controller-start');
});
