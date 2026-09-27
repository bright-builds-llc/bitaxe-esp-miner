// Explicit local integration: the sibling pinned Gate checkout is a developer prerequisite,
// matching the production parser adapter. Bazel unit suites cover the same closed snapshot shape.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync, execFileSync } from 'node:child_process';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { validateCaptureDiagnosticPair } from './capture-diagnostics.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';

test('actual Gate latest-value producer supplies separate advancing snapshots, never two ready rows', async t => {
  const directory = dirname(fileURLToPath(import.meta.url)), repo = resolve(directory, '../..');
  const gate = process.env.BITAXE_GATE_TEST_ROOT ?? resolve(repo, '../bitaxe-turnstile-system');
  const pin = (await readFile(resolve(repo, 'MODULE.bazel'), 'utf8')).match(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/u)?.[1];
  assert.equal(execFileSync('git', ['-C', gate, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), pin);
  execFileSync('git', ['-C', gate, 'diff', '--quiet', 'HEAD', '--', 'web/worker-serial-diagnostics.ts', 'web/worker-diagnostic-export.ts']);
  const produced = spawnSync('bun', [resolve(directory, 'gate-history.fixture.mjs'), gate], { encoding: 'utf8', timeout: 15000, maxBuffer: 1048576 });
  assert.equal(produced.status, 0); assert.equal(produced.stderr, '');
  const samples = JSON.parse(produced.stdout);
  for (const sample of Object.values(samples)) assert.equal(sample.observations.filter(row => row.category === 'startup').length, 1);
  assert.equal(samples.second.observations[0].category, 'startup');
  const state = { status: 'ready', connected: true, running: false, deviceBaselineConfirmed: true, deviceLeaseInactive: true,
    serialOwnershipReleased: false, heartbeatSuppressed: false, preservation: { baseline_id: 'synthetic', settings_match: true,
      device_identity_match: true, authorization_high_water_match: true, mine_on_boot: false } };
  const status = { state: 'idle', record: null, observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 7, clockValid: true, observedAtUs: 3000000 } };
  const context = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64) };
  const parts = { state, status, diagnostics: samples.first, diagnostics_confirmation: samples.second, diagnostics_confirmation_status: { state, status } };
  const review = validateCaptureDiagnosticPair(parts, context);
  assert.deepEqual(review.readiness_uptime_ms, [1000, 2000]);
  const root = await mkdtemp(resolve(tmpdir(), 'gate-snapshots-')); t.after(() => rm(root, { recursive: true, force: true }));
  await writeNew(resolve(root, 'first.json'), samples.first); await writeNew(resolve(root, 'second.json'), samples.second);
  assert.equal(review.first_diagnostics_sha256, await fileDigest(resolve(root, 'first.json')));
  assert.equal(review.confirmation_diagnostics_sha256, await fileDigest(resolve(root, 'second.json')));
  for (const invalid of [samples.same, samples.mixedBoot, samples.wrongIdentity, samples.failure, undefined])
    assert.throws(() => validateCaptureDiagnosticPair({ ...parts, diagnostics_confirmation: invalid }, context));
});
