import { readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileDigest, protectedPath, within } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { validateCompleteEvidence } from '../str005-panic-probe/self-test-evidence.mjs';
import { validateNativeAudit, validateStoreAudit } from '../str005-panic-probe/audit.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { check, object, sha256 } from '../str005-v2-serial/values.mjs';
import { sourceFingerprint } from '../str005-panic-probe/store-diagnostics.mjs';
import { validateBindings } from './contract.mjs';
export async function sealed(root) {
  await privateRoot(root); const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json'])); return seal.sha256;
}
export async function privateBytes(root, relative, size) {
  const path = within(root, resolve(root, relative)); await protectedPath(path);
  check((await stat(path)).size === size, 'startup_file_size'); return readFile(path);
}
export function validateInspection(value, inputs, identity, archiveSha, toolHashes) {
  check(value.schema === 'bitaxe-private-core-inspection/1' && value.chip === 'esp32s3' && value.decoder_version === '1.17.2' &&
    value.elf_sha256 === identity.app_elf_sha256 && value.dump_sha256 === archiveSha &&
    ['checksum_verified', 'full_elf_identity_verified', 'native_cutoff_verified', 'captured_memory_verified', 'asic_outputs_disabled',
      'generation_revoked', 'self_test_marked'].every(key => value[key] === true) && value.cause_proven === false,
  'startup_capture_unverified');
  check(inputs.schema === 'bitaxe-private-core-inputs/1' && inputs.elf_sha256 === value.elf_sha256 && inputs.dump_sha256 === archiveSha &&
    inputs.inspector_sha256 === toolHashes.decoder && inputs.cutoff_verifier_sha256 === toolHashes.cutoff &&
    value.cutoff_verifier_sha256 === toolHashes.cutoff, 'startup_decoder_binding');
}
export function validateRecovery(value, identity, physical, maybeNow) {
  check(value.schema === 'str005-current-recovery-proof-v1' && value.firmware_commit === identity.firmware_commit &&
    value.app_elf_sha256 === identity.app_elf_sha256 && value.gate_commit === identity.gate_commit &&
    value.physical_identity_sha256 === physical && /^[a-f0-9]{40}$/u.test(value.source_commit) &&
    ['safe_baseline', 'restoration_confirmed', 'device_lease_inactive', 'serial_ownership_released', 'preservation_matches', 'current_v2_idle'].every(key => value[key] === true) &&
    value.mine_on_boot === false, 'startup_recovery_unverified');
  validateLedger(value.ledger); requireExhaustedOriginal(value.original_budget);
  check(!value.ledger.pending && value.ledger.last_completed_ordinal + 1 === value.ledger.next_ordinal, 'startup_recovery_pending');
  if (maybeNow !== undefined) check(Number.isSafeInteger(value.observed_at_unix_ms) && maybeNow >= value.observed_at_unix_ms &&
    maybeNow - value.observed_at_unix_ms <= 120000, 'startup_recovery_stale');
}
/** Revalidates existing sealed producer artifacts and their actual bytes, never a supplied success flag. */
export async function captureEvidence(firmwareRoot, bindings, expectedCaptureSeal) {
  validateBindings(bindings);
  const seals = {};
  for (const key of ['captureRoot', 'archiveRoot', 'recoveryRoot']) seals[key] = await sealed(bindings[key]);
  check(seals.captureRoot === expectedCaptureSeal, 'startup_capture_not_published');
  within(bindings.captureRoot, bindings.decoderRoot);
  const context = (await proof(bindings.captureRoot, 'context.json')).value;
  check(context.schema === 'str005-panic-probe-v1' && /^[a-f0-9]{40}$/u.test(context.firmware_commit) &&
    /^[a-f0-9]{64}$/u.test(context.app_elf_sha256), 'startup_capture_context');
  const identity = { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, gate_commit: context.gate_commit };
  const evidence = (await proof(bindings.captureRoot, 'self-test-result.json')).value;
  const claim = (await proof(bindings.captureRoot, 'self-test-claim.json')).value;
  validateCompleteEvidence(evidence, claim.request, identity);
  validateNativeAudit((await proof(bindings.captureRoot, 'native-audit.json')).value, identity.app_elf_sha256, true);
  validateStoreAudit((await proof(bindings.captureRoot, 'store-audit.json')).value, identity.app_elf_sha256);
  const acquired = (await proof(bindings.captureRoot, 'self-test-core/result.private.json')).value;
  validateReadResult(acquired, context);
  validatePartition(await privateBytes(bindings.captureRoot, 'self-test-core/partition-table.private.bin', 4096));
  const archive = await privateBytes(bindings.archiveRoot, bindings.archiveRelative, 974848), archiveSha = sha256(archive);
  check(!archive.every(byte => byte === 255), 'startup_empty_archive');
  const store = (await proof(bindings.captureRoot, 'candidate-recovery-001/store-observation.json')).value;
  const storeStatus = (await proof(bindings.captureRoot, 'candidate-recovery-001/status.json')).value;
  validateStoreObservation(store, storeStatus, identity, claim.request.expectedBootOrdinal, archive.readUInt32LE(0));
  const inputs = (await proof(bindings.decoderRoot, 'inputs.json')).value;
  const inspection = (await proof(bindings.decoderRoot, 'inspection.json')).value;
  const toolHashes = { decoder: await fileDigest(resolve(firmwareRoot, 'scripts/core-dump/decode_core.py')),
    cutoff: await fileDigest(resolve(firmwareRoot, 'scripts/core-dump/cutoff.py')) };
  validateInspection(inspection, inputs, identity, archiveSha, toolHashes);
  check(await fileDigest(resolve(bindings.decoderRoot, 'dump.raw')) === archiveSha &&
    await fileDigest(resolve(bindings.decoderRoot, 'firmware.elf')) === identity.app_elf_sha256, 'startup_decode_input_changed');
  const recovery = (await proof(bindings.recoveryRoot, bindings.recoveryRelative)).value;
  const physical = context.detector.physical;
  validateRecovery(recovery, identity, physical);
  return { identity, seals, archiveSha, physical, recovery, originalCampaignId: context.original_campaign_id,
    selfTestBootOrdinal: evidence.summary.nextBootOrdinal };
}
export function validateClearResult(result, context) {
  object(result, ['schema_version', 'clearing_complete', 'terminal_category', 'first_failure_stage', 'source_commit', 'rom_admitted',
    'acquisition_complete', 'application_identity_restored', 'hardware_baseline_verified', 'cleanup_complete', 'expected_installed_source', 'expected_installed_elf']);
  check(result.schema_version === 'bitaxe-development-core-dump-clear-v1' && result.source_commit === context.source_commit &&
    result.expected_installed_source === context.firmware_commit && result.expected_installed_elf === context.app_elf_sha256 &&
    result.terminal_category === 'complete' && result.first_failure_stage === 'complete' && result.hardware_baseline_verified === false &&
    ['clearing_complete', 'rom_admitted', 'acquisition_complete', 'application_identity_restored', 'cleanup_complete'].every(key => result[key] === true), 'startup_clear_incomplete');
}

/** Exact official read schema: acquisition is not hardware-baseline proof. */
export function validateReadResult(result, context) {
  object(result, ['schema_version', 'clearing_complete', 'terminal_category', 'first_failure_stage', 'source_commit', 'rom_admitted',
    'acquisition_complete', 'application_identity_restored', 'hardware_baseline_verified', 'cleanup_complete', 'expected_installed_source', 'expected_installed_elf']);
  check(result.schema_version === 'bitaxe-development-core-dump-read-v1' && result.clearing_complete === false &&
    result.source_commit === context.commit && result.expected_installed_source === context.firmware_commit &&
    result.expected_installed_elf === context.app_elf_sha256 && result.terminal_category === 'complete' && result.first_failure_stage === 'complete' &&
    result.hardware_baseline_verified === false && ['rom_admitted', 'acquisition_complete', 'application_identity_restored', 'cleanup_complete'].every(key => result[key] === true),
  'startup_archive_acquisition');
}
export function validatePartition(table) {
  check(table.length === 4096, 'startup_partition_table_size'); let found = 0;
  for (let offset = 0; offset + 32 <= table.length; offset += 32) {
    if (table.readUInt16LE(offset) !== 0x50aa) break;
    if (table[offset + 2] !== 1 || table[offset + 3] !== 3) continue;
    found++;
    check(table.readUInt32LE(offset + 4) === 0xf12000 && table.readUInt32LE(offset + 8) === 974848 && table.readUInt32LE(offset + 28) === 0,
      'startup_partition_geometry');
  }
  check(found === 1, 'startup_partition_count');
}

export function validateStoreObservation(store, status, identity, boot, dumpLength) {
  check(store.schema === 'str005-core-store-observation-v1' && store.outcome === 'store_reported_success' &&
    store.origin === 'previous_boot' && store.status === 'valid' && store.source_hash === sourceFingerprint(identity.firmware_commit) &&
    store.boot_ordinal === String(boot) && store.stage === 'store_returned' && store.self_test_marked === true &&
    store.capacity_bytes === 974848 && Number.isInteger(store.requested_bytes) && store.requested_bytes > 0 &&
    Number.isInteger(store.prepared_bytes) && store.prepared_bytes === dumpLength && dumpLength <= 974848 &&
    ['init_result', 'prepare_result', 'start_result', 'end_result', 'store_result'].every(key => store[key] === 0) &&
    status.observation?.bootOrdinal === boot + 1, 'startup_current_store_unproven');
}
