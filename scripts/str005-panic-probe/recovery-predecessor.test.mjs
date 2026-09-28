import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, realpath, chmod, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { validateFailedInstallation, validateTimedOutInstallation, validateUnobservedReferenceInstallation, recoveryPredecessor } from './recovery-predecessor.mjs';
import { flashArguments } from './install.mjs';
import { sha256 } from '../str005-v2-serial/values.mjs';
import { writeNew, retain, proof, inventory } from '../str005-noise-serial/files.mjs';

function input(root = '/private/failed') {
  const context = { schema: 'str005-panic-probe-v1', commit: 'a'.repeat(40), gate_commit: '4'.repeat(40), before_source: { firmware_commit: '5'.repeat(40), app_elf_sha256: '6'.repeat(64) }, firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), reference_commit: 'c'.repeat(40),
    detector: { physical: 'd'.repeat(64), port: '/dev/old' }, manifest: '/package/package.json', flashBinary: '/host/flash', flashBinarySha256: 'e'.repeat(64) };
  const args = flashArguments(root, context), command = sha256(JSON.stringify(args));
  return { context, contextDigest: 'f'.repeat(64), ownerDigest: '1'.repeat(64), owner: { physicalIdentitySha256: context.detector.physical }, recoveryDigest: '2'.repeat(64),
    claim: { root, schema: 'str005-panic-install-claim-v1', context_sha256: 'f'.repeat(64), recovery_sha256: '2'.repeat(64), server_owner_sha256: '1'.repeat(64),
      physical_identity_sha256: context.detector.physical, port: '/dev/old', started_at_unix_ms: 1000, binary_sha256: context.flashBinarySha256, program: context.flashBinary, argv: args, command_sha256: command },
    runner: { schema: 'str005-panic-install-runner-v1', code: 1, spawn_failed: false, timed_out: false, interrupted: false, serial_holders_absent: true,
      started_at_unix_ms: 1000, finished_at_unix_ms: 2000, binary_sha256: context.flashBinarySha256, command_sha256: command },
    receipt: { command_kind: 'flash-monitor', board: '205', flash_status: 'completed', firmware_commit: context.firmware_commit, observed_firmware_commit: context.firmware_commit,
      reference_commit: context.reference_commit, observed_reference_commit: 'Unavailable', nvs_seed_status: 'not_provided', redaction_mode: 'dual', capture_mode: 'noninteractive',
      capture_timeout_seconds: 360, manifest_path: context.manifest, private_log_role: 'classifier-input-private', private_monitor_log_sha256: '3'.repeat(64), timestamp: '1', trusted_output: false,
      fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true, startup_complete: false, startup_failed: true, stable_boot: true, retained_failure_history: false, issues: ['error_diagnostic', 'startup_failed', 'startup_incomplete'] } },
    logDigest: '3'.repeat(64), log: 'storage_http_failure schema=v1 phase=http_server error=http_task redacted=true\nusb_startup schema=v1 stage=runtime_ready state=failed first_failure=storage_http uptime_ms=500 redacted=true\n' };
}

test('failed HTTP startup admits current observation without claiming install or historical preservation', () => {
  const result = validateFailedInstallation(input());
  assert.equal(result.recovery_only, true); assert.equal(result.installation_complete, false);
  assert.equal(result.continuity_basis, 'current-session-only'); assert.equal(result.historical_resource_proof, false);
});
test('failed-install recovery rejects missing bindings, incomplete writes, unsafe state and other failures', () => {
  for (const mutate of [v => { v.claim.context_sha256 = '0'; }, v => { v.claim.physical_identity_sha256 = '0'; }, v => { v.claim.argv.push('--factory-reset'); },
    v => { v.runner.serial_holders_absent = false; }, v => { v.runner.timed_out = true; }, v => { v.runner.code = 0; }, v => { v.receipt.flash_status = 'failed'; },
    v => { v.receipt.observed_firmware_commit = '0'; }, v => { v.receipt.reference_commit = '0'; }, v => { v.receipt.private_monitor_log_sha256 = '0'; },
    v => { v.receipt.fixed_serial_assessment.safe_baseline_confirmed = false; }, v => { v.receipt.fixed_serial_assessment.stable_boot = false; },
    v => { v.receipt.fixed_serial_assessment.issues.push('identity_mismatch'); }, v => { v.log = v.log.replace('http_task', 'network'); }, v => { v.log += 'wifi_startup_failure schema=v1 phase=driver error=no_memory redacted=true\n'; }, v => { v.log += 'rust_panic_receipt schema=v1 file_hash=01234567 line=1 redacted=true\n'; }, v => { v.receipt.fixed_serial_assessment.retained_failure_history = true; }]) {
    const value = input(); mutate(value); assert.throws(() => validateFailedInstallation(value));
  }
});
test('sealed predecessor is read-only and any changed retained byte rejects recovery admission', async t => {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'panic-predecessor-'))); t.after(() => rm(root, { recursive: true, force: true }));
  await chmod(root, 0o700); execFileSync('git', ['init', '--quiet'], { cwd: root });
  await writeFile(resolve(root, '.git/info/exclude'), '*\n');
  // Place the protected attempt below the isolated repository; its seal excludes Git metadata.
  const attempt = resolve(root, 'attempt'); const { mkdir } = await import('node:fs/promises'); await mkdir(attempt, { mode: 0o700 });
  const value = input(attempt);
  const audit = { schema: 'str005-native-panic-cutoff-audit-v1', elf_sha256: value.context.app_elf_sha256, wrapper_iram: true, literals_iram: true,
    state_internal_dram: true, safe_latches_before_delegate: true, generation_revoked_before_delegate: true, no_calls_or_branches_before_cutoff: true,
    port_routes_wrapper: true, wrapper_instructions: 10, hardware_verified: false };
  await writeNew(resolve(attempt, 'native-audit.json'), audit); value.context.nativeAuditSha256 = (await proof(attempt, 'native-audit.json')).sha256;
  await writeNew(resolve(attempt, 'context.json'), value.context); value.claim.context_sha256 = (await proof(attempt, 'context.json')).sha256;
  await writeNew(resolve(attempt, 'server-owner.json'), value.owner); value.claim.server_owner_sha256 = (await proof(attempt, 'server-owner.json')).sha256;
  await writeNew(resolve(attempt, 'current-recovery.json'), { schema: 'str005-current-recovery-proof-v1', source_commit: value.context.commit, gate_commit: value.context.gate_commit, firmware_commit: value.context.before_source.firmware_commit, app_elf_sha256: value.context.before_source.app_elf_sha256, physical_identity_sha256: value.context.detector.physical, observed_at_unix_ms: 1000, ledger: { schema: 'worker-qualification-ledger-v1', next_ordinal: 18, last_completed_ordinal: 17, total_charged_ms: 1560000, pending: false }, original_budget: { schema: 'worker-budget-review-v1', campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false }, safe_baseline: true, restoration_confirmed: true, device_lease_inactive: true, serial_ownership_released: true, preservation_matches: true, mine_on_boot: false, current_v2_idle: true }); value.claim.recovery_sha256 = (await proof(attempt, 'current-recovery.json')).sha256;
  await writeNew(resolve(attempt, 'install-claim.json'), value.claim); await writeNew(resolve(attempt, 'install-runner.json'), value.runner);
  await retain(resolve(attempt, 'install/flash-monitor.classifier-input.log'), Buffer.from(value.log)); value.receipt.private_monitor_log_sha256 = sha256(value.log);
  await writeNew(resolve(attempt, 'install/flash-command-evidence.private.json'), value.receipt);
  await writeNew(resolve(attempt, 'sealed-inventory.json'), { files: await inventory(attempt) });
  const before = (await proof(attempt, 'sealed-inventory.json')).sha256;
  assert.equal((await recoveryPredecessor(attempt, root)).review.installation_complete, false);
  assert.equal((await proof(attempt, 'sealed-inventory.json')).sha256, before);
  await writeFile(resolve(attempt, 'install/flash-monitor.classifier-input.log'), 'changed');
  await assert.rejects(recoveryPredecessor(attempt, root));
});


test('bounded timeout after reported application return permits only fresh recovery', () => {
  const value = input(); value.runner.code = null; value.runner.timed_out = true;
  value.stdout = 'manifest: protected-operational\nflash_image: protected-operational\nflash_command: protected-operational\napplication_exit_transport: serial_jtag_runtime\n';
  const result = validateTimedOutInstallation(value);
  assert.equal(result.application_return_reported, true); assert.equal(result.installed_identity_authenticated, false);
  assert.equal(result.installation_complete, false); assert.equal(result.recovery_only, true);
  for (const change of [v => { v.stdout = ''; }, v => { v.stdout += 'application_exit_transport: serial_jtag_runtime\n'; },
    v => { v.stdout = v.stdout.replace('flash_command: protected-operational', 'unrelated'); }, v => { v.stdout += 'nvs_seed_status: provided\n'; },
    v => { v.runner.interrupted = true; }, v => { v.runner.serial_holders_absent = false; }, v => { v.claim.timeout_ms = 900000; }]) {
    const invalid = structuredClone(value); change(invalid); assert.throws(() => validateTimedOutInstallation(invalid));
  }
});

test('healthy capture with unavailable reference permits only current-state recovery', () => {
  // Arrange
  const value = input(); value.runner.code = 0;
  value.finalization = { complete: false, baseline_complete: true, installation_attempted: true, installation_complete: false, host_resources_released: true, blockers: ["installation_review_missing"] };
  Object.assign(value.receipt, { capture_status: 'timed_out_after_trusted_output', monitor_evidence_status: 'trusted', trusted_output: true, trust_basis: 'fixed_serial' });
  Object.assign(value.receipt.fixed_serial_assessment, { startup_complete: true, startup_failed: false, issues: [] });
  // Act
  const result = validateUnobservedReferenceInstallation(value);
  // Assert
  assert.equal(result.runtime_capture_healthy, true); assert.equal(result.installation_complete, false);
  assert.equal(result.recovery_only, true); assert.equal(result.continuity_basis, 'current-session-only');
  for (const change of [v => { v.finalization.complete = true; }, v => { v.finalization.blockers.push('unrelated'); }, v => { v.runner.code = 1; }, v => { v.runner.serial_holders_absent = false; },
    v => { v.receipt.observed_reference_commit = v.context.reference_commit; }, v => { v.receipt.reference_commit = '0'; },
    v => { v.receipt.trusted_output = false; }, v => { v.receipt.fixed_serial_assessment.startup_failed = true; },
    v => { v.receipt.fixed_serial_assessment.issues.push('identity_mismatch'); }, v => { v.receipt.private_monitor_log_sha256 = '0'; }]) {
    const invalid = structuredClone(value); change(invalid); assert.throws(() => validateUnobservedReferenceInstallation(invalid));
  }
});
