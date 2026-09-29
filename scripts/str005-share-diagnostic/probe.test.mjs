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
  assert.equal(argumentsFor(['preflight', '--stage', 'archive-clear', '--private-root', '/p', '--gate-root', '/g',
    '--recovery-root', '/r', '--capture-root', '/c']).options['--stage'], 'archive-clear');
  assert.throws(() => argumentsFor(['preflight', '--stage', 'archive-clear', '--private-root', '/p', '--gate-root', '/g', '--recovery-root', '/r']));
  assert.throws(() => argumentsFor(['preflight', '--stage', 'clear', '--private-root', '/p', '--gate-root', '/g',
    '--recovery-root', '/r', '--capture-root', '/c']));
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

test('production Gate pre-Stop ready/not_required yields proof only after actual Stop and Close confirmation', async () => {
  const { execFile } = await import('node:child_process'), { promisify } = await import('node:util');
  const { dirname, resolve } = await import('node:path'), { fileURLToPath } = await import('node:url');
  const here = dirname(fileURLToPath(import.meta.url));
  const gate = process.argv[2] ? dirname(resolve(process.argv[2])) : resolve(here, '../../../bitaxe-turnstile-system');
  const child = await promisify(execFile)('bun', [resolve(here, 'baseline-boundary.fixture.mjs'), gate], { timeout: 15000 });
  assert.equal(child.stdout.trim(), 'diagnostic_pre_stop_boundary_passed'); assert.equal(child.stderr, '');
});

test('production diagnostic finalizer seals a successful baseline with no install claim as incomplete', async t => {
  // Arrange
  const { mkdtemp, realpath, rm } = await import('node:fs/promises'), { tmpdir } = await import('node:os'), { resolve } = await import('node:path');
  const { proof, verifyInventory } = await import('../str005-noise-serial/files.mjs');
  const { baselineConclusion, validatePart } = await import('../str005-panic-probe/model.mjs');
  const { state, ledger, original } = await import('../str005-noise-serial/test-fixture.mjs');
  const { finalizeDiagnostic } = await import('./finalize.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'no-install-claim-'))); t.after(() => rm(root, { recursive: true, force: true }));
  const context = { stage: 'installation', firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40) };
  context.before_source = { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 };
  const before = { ...state(context), deviceRestorationConfirmed: false };
  const observed = { state: before, closed: state(context, 'candidate', true), ledger, original_budget: original,
    diagnostics: { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 1 }] },
    status: validatePart('status', { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
      observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }, { ...context, scope: 'share' }),
    finished: { failures: [] } };
  const derived = { ...baselineConclusion(observed, context), baseline_complete: true, host_resources_released: true };
  assert.equal(derived.complete, true);
  // Act
  const result = await finalizeDiagnostic(root, context, derived);
  // Assert
  assert.equal(result.complete, false); assert.equal(result.baseline_complete, true); assert.equal(result.installation_complete, false);
  assert.ok(result.blockers.includes('diagnostic_installation_not_attempted')); assert.equal(derived.complete, true);
  const saved = (await proof(root, 'result.json')).value; assert.equal(saved.complete, false);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
});

import './finalizer.test.mjs';
import './capture-proof.test.mjs';

import './retained-package.test.mjs';
