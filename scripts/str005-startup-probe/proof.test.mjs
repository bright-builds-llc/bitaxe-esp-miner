import test from 'node:test';
import assert from 'node:assert/strict';
import { validateInspection, validateClearResult, validateRecovery } from './capture.mjs';
import { admitArguments, requireEnabled } from './contract.mjs';
const identity = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40) };
const archive = 'd'.repeat(64), toolHashes = { decoder: 'e'.repeat(64), cutoff: 'f'.repeat(64) };
function inspection() { return { schema: 'bitaxe-private-core-inspection/1', chip: 'esp32s3', decoder_version: '1.17.2',
  elf_sha256: identity.app_elf_sha256, dump_sha256: archive, checksum_verified: true, full_elf_identity_verified: true,
  native_cutoff_verified: true, captured_memory_verified: true, asic_outputs_disabled: true, generation_revoked: true,
  self_test_marked: true, cause_proven: false, cutoff_verifier_sha256: toolHashes.cutoff }; }
const inputs = { schema: 'bitaxe-private-core-inputs/1', elf_sha256: identity.app_elf_sha256, dump_sha256: archive,
  inspector_sha256: toolHashes.decoder, cutoff_verifier_sha256: toolHashes.cutoff };
test('actual inspection fields require checksum, full ELF and captured cutoff evidence', () => {
  assert.doesNotThrow(() => validateInspection(inspection(), inputs, identity, archive, toolHashes));
  for (const flag of ['checksum_verified', 'full_elf_identity_verified', 'native_cutoff_verified', 'captured_memory_verified',
    'asic_outputs_disabled', 'generation_revoked', 'self_test_marked']) {
    const value = inspection(); value[flag] = false; assert.throws(() => validateInspection(value, inputs, identity, archive, toolHashes));
  }
  assert.throws(() => validateInspection(inspection(), inputs, identity, '0'.repeat(64), toolHashes));
  assert.throws(() => validateInspection(inspection(), { ...inputs, inspector_sha256: '0'.repeat(64) }, identity, archive, toolHashes));
});
test('official clear result cannot claim completion on acquisition, return or release failure', () => {
  const context = { ...identity, source_commit: 'd'.repeat(40) };
  const result = { schema_version: 'bitaxe-development-core-dump-clear-v1', source_commit: context.source_commit,
    expected_installed_source: identity.firmware_commit, expected_installed_elf: identity.app_elf_sha256, clearing_complete: true,
    terminal_category: 'complete', first_failure_stage: 'complete', rom_admitted: true, acquisition_complete: true,
    application_identity_restored: true, hardware_baseline_verified: false, cleanup_complete: true };
  validateClearResult(result, context);
  for (const field of ['clearing_complete', 'rom_admitted', 'acquisition_complete', 'application_identity_restored', 'cleanup_complete'])
    assert.throws(() => validateClearResult({ ...result, [field]: false }, context));
  assert.throws(() => validateClearResult({ ...result, expected_installed_elf: '0'.repeat(64) }, context));
});
test('disabled admission rejects before private paths and stays tested after activation', () => {
  const tasks = '## Active\n### task-str005-mining-startup-probe | fixture\nStartup probe hardware: disabled.\n';
  assert.throws(() => requireEnabled(tasks, true), { code: 'startup_hardware_disabled' });
  assert.throws(() => admitArguments(['serve', '--private-root', '/unreadable/attempt', '--authority-directory', '/unreadable/authority'], false),
    { code: 'startup_hardware_disabled' });
  assert.throws(() => admitArguments(['clear', '--private-root', '/unreadable/attempt'], false), { code: 'startup_hardware_disabled' });
});

test('same-root capture lineage and current SDK store receipt exclude unrelated or old dumps', async () => {
  const { validateBindings } = await import('./contract.mjs');
  const { validateStoreObservation } = await import('./capture.mjs');
  const { sourceFingerprint } = await import('../str005-panic-probe/store-diagnostics.mjs');
  const bindings = { schema: 'str005-startup-proof-inputs-v1', captureRoot: '/private/capture', archiveRoot: '/private/capture',
    recoveryRoot: '/private/capture', decoderRoot: '/private/capture/cutoff-review', archiveRelative: 'self-test-core/core-dump.private.bin',
    recoveryRelative: 'candidate-recovery-002/current-recovery.json' };
  validateBindings(bindings); assert.throws(() => validateBindings({ ...bindings, archiveRoot: '/private/other' }));
  const store = { schema: 'str005-core-store-observation-v1', outcome: 'store_reported_success', origin: 'previous_boot', status: 'valid',
    source_hash: sourceFingerprint(identity.firmware_commit), boot_ordinal: '5', stage: 'store_returned', self_test_marked: true,
    capacity_bytes: 974848, requested_bytes: 88668, prepared_bytes: 88704, init_result: 0, prepare_result: 0, start_result: 0, end_result: 0, store_result: 0 };
  const status = { observation: { bootOrdinal: 6 } };
  validateStoreObservation(store, status, identity, 5, 88704);
  for (const change of [{ store_result: 257 }, { init_result: 1 }, { boot_ordinal: '4' }, { prepared_bytes: 90000 }, { self_test_marked: false }])
    assert.throws(() => validateStoreObservation({ ...store, ...change }, status, identity, 5, 88704));
});
