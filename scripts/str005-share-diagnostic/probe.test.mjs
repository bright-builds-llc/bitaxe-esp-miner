import test from 'node:test';
import assert from 'node:assert/strict';
import { FLAGS, TASK, sourceScope, argumentsFor } from './contract.mjs';
import { clearArguments } from './clear.mjs';
import { createProbeServer } from '../str005-panic-probe/server.mjs';
import { once } from 'node:events';
function tasks(stage) { return `## Active\n### ${TASK} | fixture\nShare diagnostic ${stage} hardware: enabled.\n`; }
test('each diagnostic stage needs its compiled and published admission', () => {
  const disabled = Object.fromEntries(Object.keys(FLAGS).map(stage => [stage, false]));
  for (const stage of Object.keys(FLAGS)) {
    assert.equal(typeof FLAGS[stage], 'boolean'); assert.throws(() => sourceScope(tasks(stage), stage, disabled));
    const enabled = { ...FLAGS, [stage]: true }; assert.equal(sourceScope(tasks(stage), stage, enabled).stage, stage);
    assert.throws(() => sourceScope(tasks(stage).replace('## Active', '## Future'), stage, enabled));
    assert.throws(() => sourceScope(tasks(stage) + tasks(stage), stage, enabled));
    assert.throws(() => sourceScope(tasks(stage).replace('Share diagnostic', 'Development panic probe:'), stage, enabled));
  }
});
test('each stage has a closed argument set and cannot accept authority or write overrides', () => {
  assert.equal(argumentsFor(['preflight', '--stage', 'recovery', '--private-root', '/p', '--gate-root', '/g']).options['--stage'], 'recovery');
  assert.throws(() => argumentsFor(['preflight', '--stage', 'installation', '--private-root', '/p', '--gate-root', '/g']));
  assert.throws(() => argumentsFor(['preflight', '--stage', 'clear', '--private-root', '/p', '--gate-root', '/g', '--recovery-root', '/r', '--manifest', '/m']));
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p', '--authority-directory', '/a']));
  assert.throws(() => argumentsFor(['start', '--private-root', '/p']));
});
test('old dump clear has no caller-selected partition or factory reset route', () => {
  const context = { detector: { physical: 'f'.repeat(64) }, firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64),
    archivePath: '/private/original/core-dump.private.bin', archiveSha: 'c'.repeat(64) };
  const args = clearArguments('/private/clear', context, '/dev/cu.fixture');
  assert.equal(args[0], 'core-dump-clear'); assert.ok(args.includes('--recovery-proof')); assert.ok(args.includes('--preserved-dump'));
  for (const forbidden of ['--wifi-credentials', 'erase-flash', 'write-bin', '--offset', '--force']) assert.equal(args.includes(forbidden), false);
});
test('recovery owner exposes no installation candidate or fault claim', async t => {
  const server = createProbeServer({ root: '/unused', context: { diagnosticSuccessor: true, recoveryOnly: true, selfTestEnabled: false, installEnabled: false },
    page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  for (const path of ['/candidate', '/self-test-claim', '/install', '/start', '/grant', '/clear']) {
    const response = await fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: '{}' });
    assert.equal(response.status, 400, path);
  }
});

import './clear-supervisor.test.mjs';

test('historical capture lineage does not renew the clear effect proof clock', async () => {
  // Arrange
  const { requireFreshRecovery } = await import('./clear.mjs');
  const historical = { seal: 'f'.repeat(64), context: { commit: 'a'.repeat(40) }, current: { value: { observed_at_unix_ms: 1000 } } };
  // Act / Assert
  assert.equal(requireFreshRecovery(historical, 'a'.repeat(40), 121000), historical);
  assert.throws(() => requireFreshRecovery(historical, 'a'.repeat(40), 121001), { code: 'diagnostic_recovery_freshness' });
  assert.throws(() => requireFreshRecovery(historical, 'b'.repeat(40), 1001), { code: 'diagnostic_recovery_freshness' });
  // The historical object is unchanged and may be rejudged as lineage, never retimestamped.
  assert.equal(historical.current.value.observed_at_unix_ms, 1000);
});

test('capture preflight accepts rejudged historical empty-core lineage without treating it as fresh authority', async () => {
  // Arrange
  const { capturePrerequisite } = await import('./prepare.mjs');
  const { requireFreshRecovery } = await import('./clear.mjs');
  const recovered = { seal: 'f'.repeat(64), context: { commit: 'a'.repeat(40) }, current: { value: { observed_at_unix_ms: 1000 } } };
  const empty = { empty_core_dump: true, expected_boot_ordinal: 22 };
  // Act
  const result = await capturePrerequisite('/history', '/repo', {}, 'physical', {
    recoveryAnchor: async () => recovered,
    verifyCorePreservation: async (_root, context) => { assert.equal(context, recovered.context); return empty; },
  });
  // Assert
  assert.equal(result.corePreservation, empty); assert.equal(result.recovered, recovered);
  assert.throws(() => requireFreshRecovery(result.recovered, 'a'.repeat(40), 999999), { code: 'diagnostic_recovery_freshness' });
  assert.equal(result.recovered.current.value.observed_at_unix_ms, 1000);
});
