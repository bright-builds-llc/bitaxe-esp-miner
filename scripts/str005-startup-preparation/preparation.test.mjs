import test from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
import { restartPacket as packetFixture } from '../fixed-usb-qualification/reset-origin-restart-fixtures.mjs';
import { argumentsFor } from './main.mjs';
import { conclusion, restartEvidence, restartPacket } from './model.mjs';
import { createPreparationClient } from './restart-client.mjs';
import { createRestartServer, restartConfiguration } from './server.mjs';
const context = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40), historical_gate_commit: 'd'.repeat(40),
  expectedLedger: ledger, originalBudget: original, before_boot_ordinal: 7, request_nonce: Buffer.alloc(16, 1).toString('base64url') };
const request = { requestNonce: context.request_nonce, expectedBootOrdinal: 7 };
const validation = { validateDiagnostics: async value => value };
function successfulParts() {
  const before = state(context), after = state(context), closed = state(context, 'candidate', true);
  return { before_state: before, before_ledger: ledger, before_budget: original, after_state: after, after_ledger: ledger, after_budget: original,
    closed, finished: { failures: [] }, evidence: packetFixture(context), evidenceVerified: true };
}
test('restart proof requires matched nonce, software reset, successor boot, identity and bounded completion', async () => {
  const value = packetFixture(context); await restartEvidence(value, context, request, validation);
  for (const mutate of [v => { v.ack.requestNonceSha256 = '0'.repeat(64); }, v => { v.summary.durationMs = 30001; },
    v => { v.summary.portReopens = 2; }, v => { v.observations[0].diagnostic.boot_ordinal = 9; },
    v => { v.observations[0].diagnostic.reset_reason = 'panic'; }, v => { v.observations[1].diagnostic.app_elf_sha256 = '0'.repeat(64); },
    v => { v.observations.pop(); }]) { const changed = structuredClone(value); mutate(changed); await assert.rejects(restartEvidence(changed, context, request, validation)); }
});
test('failed typed observer evidence is retained without a complete restart claim', async () => {
  const value = packetFixture(context); value.summary.stage = 'failed'; value.summary.durationMs = 30100;
  value.lifecycle.at(-1).event = 'failed'; value.lifecycle.at(-1).atMs = 30100;
  const retained = await restartPacket(value, context, request, validation); assert.equal(retained.summary.stage, 'failed');
  await assert.rejects(restartEvidence(retained, context, request, validation));
});
test('same-page preservation and both unchanged ledgers gate preparation completion', () => {
  const parts = successfulParts(); assert.equal(conclusion(parts, context, true).complete, true);
  for (const mutate of [p => { p.after_ledger = { ...ledger, total_charged_ms: ledger.total_charged_ms + 30000 }; },
    p => { p.after_state.preservation.settings_match = false; }, p => { p.after_state.preservation.baseline_id = Buffer.alloc(16, 2).toString('base64url'); },
    p => { p.closed.serialOwnershipReleased = false; }, p => { p.evidenceVerified = false; }]) {
    const changed = structuredClone(parts); mutate(changed); assert.equal(conclusion(changed, context, true).complete, false);
  }
});
test('restart client consumes once and releases after ambiguous restart and evidence failure', async () => {
  const calls = [], gate = { state: () => ({}), refresh: async () => {}, reviewQualificationAttempts: async () => ledger, reviewBudget: async () => original,
    exportDiagnostics: async () => {}, qualificationRestart: async () => { calls.push('restart'); throw Error('ambiguous'); },
    exportRestartEvidence: () => undefined, stop: async () => { calls.push('stop'); }, close: async () => { calls.push('close'); } };
  const run = createPreparationClient({ gate, post: async path => path === '/preparation-context' ? { campaignId: 'fixture' } : path === '/restart-claim' ? request : {} });
  const result = await run(); assert.equal(result.complete, false); assert.deepEqual(calls, ['restart', 'stop', 'close']);
  await assert.rejects(run(), /preparation_consumed/u);
});
test('restart-only configuration has no V2 mode or effectful service routes', async () => {
  const config = restartConfiguration(context, {}); assert.equal(config.restartQualification, true); assert.equal(config.stratumV2Qualification, undefined);
  const server = createRestartServer({ root: '/unused', context, assets: {}, verify: async () => {} }, { persist: async () => {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); const origin = `http://127.0.0.1:${server.address().port}`;
  try { for (const path of ['/authorization-context', '/window-artifacts', '/startup/fixture', '/self-test-claim', '/clear']) {
    const response = await fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'Content-Type': 'application/json' }, body: '{}' }); assert.equal(response.status, 400);
  } } finally { await server.release(); }
});
test('disabled admission and closed command arguments cannot accept authority or flash inputs', () => {
  assert.throws(() => argumentsFor(['serve', '--private-root', '/unused', '--stage', 'restart'], false));
  assert.throws(() => argumentsFor(['serve', '--private-root', '/unused', '--stage', 'restart', '--authority-directory', '/unused'], true));
});
const maybeGateRoot = process.env.STARTUP_GATE_ROOT ?? (process.argv[2] ? dirname(resolve(process.argv[2])) : undefined);
test('actual Gate restart adapter/page/observer preserves its baseline and allows at most one port reopen', { skip: !maybeGateRoot }, async () => {
  const result = await promisify(execFile)('bun', [resolve(dirname(fileURLToPath(import.meta.url)), 'production.fixture.mjs'), maybeGateRoot], { timeout: 30000 });
  assert.equal(result.stdout.trim(), 'preparation_production_boundary_passed'); assert.equal(result.stderr, '');
});

for (const stale of [false, true]) test(`restart claim ${stale ? 'rejects stale observations' : 'is durable and one-use'}`, async () => {
  let now = 1000; const saved = new Map();
  const server = createRestartServer({ root: '/unused', context, assets: {}, verify: async () => {} },
    { now: () => now, persist: async (name, value) => { assert.ok(!saved.has(name)); saved.set(name, value); }, validateDiagnostics: async value => value });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  try {
    assert.equal((await post('/begin', {})).status, 200);
    for (const [stage, value] of Object.entries({ before_state: state(context), before_ledger: ledger, before_budget: original }))
      assert.equal((await post('/part', { stage, value })).status, 200);
    assert.equal((await post('/diagnostic-export', { schema: 'worker-diagnostic-export-v1', observations: [
      { category: 'boot', boot_ordinal: 7 }, { category: 'runtime_identity', firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 },
      { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none' },
    ] })).status, 200);
    if (stale) now += 120001;
    const claim = await post('/restart-claim', {}); assert.equal(claim.status, stale ? 400 : 200);
    assert.equal(saved.has('restart-claim.json'), !stale);
    if (!stale) { assert.equal((await claim.json()).expectedBootOrdinal, 7); assert.equal((await post('/restart-claim', {})).status, 400); }
  } finally { await server.release(); }
});

test('rejected begin preserves its first cause and still records successful Close', async () => {
  // Arrange
  const stored = new Map(), closed = state(context, 'candidate', true);
  const server = createRestartServer({ root: '/unused', context, assets: {},
    verify: async () => { throw Object.assign(Error('stale'), { code: 'preparation_recovery_stale' }); } },
  { persist: async (name, value) => { stored.set(name, value); } });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  try {
    // Act
    assert.equal((await post('/begin', {})).status, 400);
    const close = await post('/part', { stage: 'closed', value: closed });
    // Assert
    assert.equal(close.status, 200);
    assert.equal(stored.get('closed.json').serialOwnershipReleased, true);
    assert.deepEqual(stored.get('first-failure.json'), { schema: 'str005-preparation-failure-v1', phase: 'begin', category: 'preparation_recovery_stale' });
  } finally { await server.release(); }
});
