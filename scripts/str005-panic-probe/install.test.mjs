import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { admitRecovery, runChild, validateFlashReceipt, flashArguments, INSTALL_TIMEOUT_MS } from './install.mjs';
import { ledger, original } from '../str005-noise-serial/test-fixture.mjs';
const context = { commit: 'a'.repeat(40), firmware_commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), reference_commit: 'c'.repeat(40),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64), port: '/dev/synthetic' }, manifest: '/repo/package.json' };
function recovery() { return { schema: 'str005-current-recovery-proof-v1', source_commit: context.commit, gate_commit: context.gate_commit,
  firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
  physical_identity_sha256: context.detector.physical, observed_at_unix_ms: 1000, ledger, original_budget: original,
  safe_baseline: true, restoration_confirmed: true, device_lease_inactive: true, serial_ownership_released: true,
  preservation_matches: true, mine_on_boot: false, current_v2_idle: true }; }

test('installation requires fresh complete same-source same-device current recovery', () => {
  assert.doesNotThrow(() => admitRecovery(recovery(), context, 121000));
  for (const mutate of [v => { v.observed_at_unix_ms = 999; }, v => { v.physical_identity_sha256 = '0'.repeat(64); },
    v => { v.ledger.pending = true; }, v => { v.mine_on_boot = true; }, v => { v.current_v2_idle = false; }]) {
    const value = structuredClone(recovery()); mutate(value); assert.throws(() => admitRecovery(value, context, 121000));
  }
});
test('installation argv cannot seed NVS, erase flash or invoke nested build', () => {
  const args = flashArguments('/private/root', context);
  assert.equal(args[args.indexOf('--expected-physical-sha256') + 1], context.detector.physical);
  assert.equal(args[0], 'flash-monitor'); assert.equal(args.includes('--wifi-credentials'), false);
  assert.equal(args.includes('--evidence-mode'), true); assert.equal(args.includes('--redact-evidence'), false); assert.equal(args.at(-1), '360');
});
test('real child timeout terminates process group and keeps output private', async t => {
  const root = await mkdtemp(join(tmpdir(), 'panic-install-')); t.after(() => rm(root, { recursive: true, force: true }));
  const result = await runChild(process.execPath, ['-e', 'console.log("synthetic-private");setInterval(()=>{},1000)'], root, 200);
  assert.equal(result.timed_out, true); assert.notEqual(result.code, 0); assert.throws(() => process.kill(result.pid, 0));
  assert.match(await readFile(join(root, 'install.stdout.log'), 'utf8'), /synthetic-private/);
  assert.equal((await stat(join(root, 'install.stdout.log'))).mode & 0o777, 0o600);
});
test('flash receipt binds private log, firmware, successful monitoring and observed time', () => {
  const runner = { started_at_unix_ms: 1000, finished_at_unix_ms: 2000 }, digest = 'a'.repeat(64);
  const f = { command_kind: 'flash-monitor', board: '205', flash_status: 'completed', capture_mode: 'noninteractive', capture_status: 'completed',
    monitor_evidence_status: 'trusted', trusted_output: true, firmware_commit: context.firmware_commit, observed_firmware_commit: context.firmware_commit,
    reference_commit: context.reference_commit, observed_reference_commit: context.reference_commit, trust_basis: 'fixed_serial', nvs_seed_status: 'not_provided',
    redaction_mode: 'dual', capture_timeout_seconds: 360, manifest_path: context.manifest, private_log_role: 'classifier-input-private', private_monitor_log_sha256: digest,
    timestamp: '1', fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true, startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } };
  assert.doesNotThrow(() => validateFlashReceipt(f, context, runner, digest));
  for (const field of ['flash_status', 'observed_firmware_commit', 'private_monitor_log_sha256', 'nvs_seed_status']) {
    assert.throws(() => validateFlashReceipt({ ...f, [field]: 'invalid' }, context, runner, digest));
  }
});

import "./recovery-predecessor.test.mjs";

test('future install supervisor budgets twenty minutes while capture stays six minutes', () => {
  assert.equal(INSTALL_TIMEOUT_MS, 1200000);
  assert.equal(flashArguments('/private/root', context).at(-1), '360');
});

import "./core-preservation.test.mjs";
