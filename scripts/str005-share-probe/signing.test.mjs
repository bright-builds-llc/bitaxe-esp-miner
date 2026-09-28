import test from 'node:test';
import assert from 'node:assert/strict';
import { signWindow } from './signing.mjs';
import { argumentsFor, requireEnabled, requireGatePin, requireFrozenSource, TASK } from './contract.mjs';
const stratum = { profile: 'bwg-worker-stratum-v2-standard/0.1', endpoint: 'stratum+tcp://192.168.1.2:3333/',
  authorityPublicKey: Buffer.alloc(32, 1).toString('base64url'), userIdentity: 'synthetic' };
test('disabled hardware rejects before any private path is opened', () => {
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/nonexistent'], false), /share_hardware_disabled/u);
  assert.throws(() => requireEnabled('', false), /share_hardware_disabled/u);
});
test('bounded window signs one fresh normal allowance and exactly two same-lease renewals', async () => {
  const calls = [], request = { attempt: { schema: 'worker-qualification-attempt-v1', id: Buffer.alloc(16, 1).toString('base64url'), ordinal: 20, purpose: 'normal', maximumActiveMilliseconds: 180000 },
    challengeId: 'challenge_test', binding: Buffer.alloc(32, 2).toString('base64url'), stratum, verify: async () => {},
    sign: async (operation, value) => { calls.push(value); return { profile: 'bwg-worker-lease-authorization-artifact/0.1', operation, authorization: 'opaque' }; } };
  const window = await signWindow(request);
  assert.deepEqual(calls.map(x => x.operation), ['start', 'renew', 'renew']); assert.equal(window.renewals.length, 2);
  assert.ok(window.renewals.every(x => x.leaseId === window.grant.leaseId && !Object.hasOwn(x, 'qualificationAttempt')));
});

for (const [label, tasks] of [
  ['future', `## Future\n### ${TASK} | test\nShare probe hardware: enabled.\nShare admission sha256: ${'a'.repeat(64)}.`],
  ['duplicate', `## Active\n### ${TASK} | test\nShare probe hardware: enabled.\nShare admission sha256: ${'a'.repeat(64)}.\n### ${TASK} | again`],
  ['embedded', `## Active\n### ${TASK} | test\nDo not set Share probe hardware: enabled.\nShare admission sha256: ${'a'.repeat(64)}.`],
]) test(`task admission rejects ${label} authority`, () => { assert.throws(() => requireEnabled(tasks, true)); });
test('Gate identity must equal the unique published MODULE pin', () => {
  const pin = 'a'.repeat(40), moduleText = `strip_prefix = "bitaxe-turnstile-system-${pin}"`;
  requireGatePin(moduleText, pin);
  assert.throws(() => requireGatePin(moduleText, 'b'.repeat(40)));
  assert.throws(() => requireGatePin(`${moduleText}\n${moduleText}`, pin));
});

for (const key of ['source_commit', 'admissionSha256', 'contractSha256']) test(`live owner rejects changed ${key}`, () => {
  const context = { source_commit: 'a'.repeat(40), admissionSha256: 'b'.repeat(64), contractSha256: 'c'.repeat(64) };
  assert.throws(() => requireFrozenSource({ ...context, [key]: 'd'.repeat(64) }, context), { code: 'share_source_changed' });
});
