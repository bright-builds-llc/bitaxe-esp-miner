import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createCurrentRecovery } from './recovery-client.mjs';
import { createCurrentRecoveryServer } from './recovery-server.mjs';
import { currentConclusion, recoveryArguments } from './recovery-main.mjs';
import { collectRecovery } from './client.mjs';
const maybeGateRoot = process.env.STARTUP_GATE_ROOT ?? (process.argv[2] ? dirname(resolve(process.argv[2])) : undefined);
test('actual Gate decoding demonstrates why idle probing a retained attempt destroys recovery', { skip: !maybeGateRoot }, async () => {
  const output = await promisify(execFile)('bun', [resolve(dirname(fileURLToPath(import.meta.url)), 'retained-recovery.fixture.mjs'), maybeGateRoot], { timeout: 30000 });
  assert.equal(output.stdout.trim(), 'retained_recovery_boundary_passed'); assert.equal(output.stderr, '');
});
test('recovery collects all other stages and releases after typed status rejection', async () => {
  const calls = [], parts = {}, gate = {
    async reviewQualificationAttempts() { calls.push('ledger'); return {}; }, async reviewBudget() { calls.push('budget'); return {}; },
    async exportDiagnostics() { calls.push('diagnostics'); }, async stratumV2Possession() { return 'fresh'; },
    async stratumV2Status(_scope, id) { assert.equal(id, 'known'); calls.push('status'); throw Object.assign(Error('private'), { category: 'command_rejected' }); },
    async stop() { calls.push('stop'); }, async refresh() { calls.push('state'); }, state: () => ({}), async close() { calls.push('close'); },
  };
  const result = await createCurrentRecovery({ gate, attemptId: 'known', campaignId: 'fixture', save: async (stage, value) => { parts[stage] = value; } })();
  assert.deepEqual(calls, ['ledger', 'budget', 'diagnostics', 'status', 'stop', 'state', 'close']);
  assert.deepEqual(result.failures, [{ stage: 'status', category: 'command_rejected' }]); assert.ok(parts.closed); assert.ok(parts.ledger);
  assert.equal(result.qualification_complete, false); assert.equal(JSON.stringify(result).includes('private'), false);
});
test('an unconfirmed Start defaults to unknown and never probes either attempt mode', async () => {
  let statuses = 0;
  const gate = { refresh: async () => {}, state: () => ({}), reviewQualificationAttempts: async () => ({}), reviewBudget: async () => ({}),
    stratumV2Possession: async () => 'binding', stratumV2Status: async () => { statuses++; return {}; }, exportDiagnostics: async () => {} };
  await assert.rejects(collectRecovery({ gate, campaignId: 'test', attemptId: 'host-created-id', save: async () => {} }));
  assert.equal(statuses, 0);
});
test('recovery-only server rejects effect routes without preventing partial sealing data', async () => {
  const persisted = [], server = createCurrentRecoveryServer({ root: '/unused', context: {}, assets: {} }, { persist: async (...value) => { persisted.push(value); } });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); const origin = `http://127.0.0.1:${server.address().port}`;
  try {
    for (const path of ['/authorization-context', '/startup/fixture', '/window-artifacts', '/self-test-claim', '/clear']) {
      const response = await fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'Content-Type': 'application/json' }, body: '{}' });
      assert.equal(response.status, 400);
    }
    const response = await fetch(`${origin}/recovery-part`, { method: 'POST', headers: { origin, 'Content-Type': 'application/json' },
      body: JSON.stringify({ stage: 'finished', value: { failures: [{ stage: 'status', category: 'command_rejected' }] } }) });
    assert.equal(response.status, 200); assert.equal(persisted[0][0], 'finished');
  } finally { await server.release(); }
});
test('current-only evaluator preserves historical gaps and ledger comparison ignores key order', () => {
  const ledger = { schema: 'worker-qualification-ledger-v1', next_ordinal: 19, last_completed_ordinal: 18, total_charged_ms: 1740000, pending: false };
  const reordered = Object.fromEntries(Object.entries(ledger).reverse());
  const result = currentConclusion({ ledger: reordered }, { expectedLedger: ledger }, false);
  assert.equal(result.blockers.includes('accounting_changed_since_startup_seal'), false);
  assert.equal(result.historical_authorization_checkpoint_verified, false); assert.equal(result.historical_preservation_verified, false);
  assert.equal(result.qualification_complete, false); assert.equal(result.parity_promotion, false);
});
test('recovery arguments exclude all authority and effect inputs before activation', () => {
  assert.throws(() => recoveryArguments(['recover-serve', '--private-root', '/unused'], false), { code: 'startup_recovery_disabled' });
  for (const key of ['--authority-directory', '--flash-binary', '--fixture-binary'])
    assert.throws(() => recoveryArguments(['recover-serve', '--private-root', '/unused', key, '/never-read'], true));
});
