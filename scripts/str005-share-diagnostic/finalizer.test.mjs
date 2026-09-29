import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { writeNew, proof } from '../str005-noise-serial/files.mjs';
import { sha256 } from '../str005-v2-serial/values.mjs';
import { flashArguments, INSTALL_TIMEOUT_MS } from '../str005-panic-probe/install.mjs';
import { finalizeDiagnostic } from './finalize.mjs';
async function fixture(t, trusted) {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'install-finalize-'))); t.after(() => rm(root, { recursive: true, force: true }));
  const context = { stage: 'installation', firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), reference_commit: 'c'.repeat(40),
    flashBinarySha256: 'd'.repeat(64), flashBinary: '/protected/flash', manifest: '/protected/manifest', detector: { port: '/dev/cu.synthetic', physical: 'f'.repeat(64) }, installTimeoutMs: INSTALL_TIMEOUT_MS };
  await writeNew(resolve(root, 'context.json'), context);
  await writeNew(resolve(root, 'current-recovery.json'), { fixture: 'baseline proof already validated by shared finalizer' });
  await writeNew(resolve(root, 'server-owner.json'), { physicalIdentitySha256: context.detector.physical });
  const args = flashArguments(root, context), commandSha256 = sha256(JSON.stringify(args));
  await writeNew(resolve(root, 'install-claim.json'), { schema: 'str005-panic-install-claim-v1', context_sha256: (await proof(root, 'context.json')).sha256,
    recovery_sha256: (await proof(root, 'current-recovery.json')).sha256, server_owner_sha256: (await proof(root, 'server-owner.json')).sha256,
    physical_identity_sha256: context.detector.physical, port: context.detector.port, timeout_ms: INSTALL_TIMEOUT_MS,
    started_at_unix_ms: 1000, command_sha256: commandSha256 });
  await writeNew(resolve(root, 'install-runner.json'), { schema: 'str005-panic-install-runner-v1', timeout_ms: INSTALL_TIMEOUT_MS,
    code: 0, spawn_failed: false, timed_out: false, interrupted: false, serial_holders_absent: true,
    started_at_unix_ms: 1000, finished_at_unix_ms: 2000, binary_sha256: context.flashBinarySha256, command_sha256: commandSha256 });
  await mkdir(resolve(root, 'install'), { mode: 0o700 });
  const log = 'synthetic private classifier output\n';
  await writeFile(resolve(root, 'install/flash-monitor.classifier-input.log'), log, { mode: 0o600 });
  await writeNew(resolve(root, 'install/flash-command-evidence.private.json'), { command_kind: 'flash-monitor', board: '205', flash_status: 'completed',
    capture_mode: 'noninteractive', capture_status: 'completed', monitor_evidence_status: trusted ? 'trusted' : 'untrusted', trusted_output: trusted,
    firmware_commit: context.firmware_commit, observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit,
    observed_reference_commit: 'Unavailable', trust_basis: 'fixed_serial', nvs_seed_status: 'not_provided', redaction_mode: 'dual',
    capture_timeout_seconds: 360, manifest_path: context.manifest, private_log_role: 'classifier-input-private', private_monitor_log_sha256: sha256(log),
    timestamp: '1', fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true, startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } });
  return { root, context };
}
const completed = () => ({ complete: true, blockers: [], baseline_complete: true, installation_complete: true,
  candidate_install_reviewed: true, candidate_recoveries: [{ complete: true, blockers: [] }], host_resources_released: true });
test('invalid actual installation receipt cannot inherit a successful baseline and preserves earliest failure', async t => {
  // Arrange
  const f = await fixture(t, false), derived = { ...completed(), complete: false, first_failure: { phase: 'status', category: 'timeout' } };
  // Act
  const result = await finalizeDiagnostic(f.root, f.context, derived);
  // Assert
  assert.equal(result.complete, false); assert.equal(result.installation_complete, false);
  assert.equal(result.installation_blocker, 'diagnostic_installation_unverified'); assert.deepEqual(result.first_failure, derived.first_failure);
  assert.equal((await proof(f.root, 'result.json')).value.complete, false);
});
test('verified installation receipts and shared candidate recovery remain admissible', async t => {
  // Arrange
  const f = await fixture(t, true);
  // Act
  const result = await finalizeDiagnostic(f.root, f.context, completed());
  // Assert
  assert.equal(result.complete, true); assert.equal(result.installation_complete, true); assert.deepEqual(result.blockers, []);
});
