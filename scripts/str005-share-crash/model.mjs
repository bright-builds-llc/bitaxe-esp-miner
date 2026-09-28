import { createHash } from 'node:crypto';
import { object, check } from '../str005-v2-serial/values.mjs';
export const INSTALLED_SOURCE = 'f000872f2e436aa7cdaa8cbfa41eee965a27731e';
export const INSTALLED_ELF = 'a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2';
export const DUMP_BYTES = 974848;
export function producerResult(value) {
  object(value, ['schema_version', 'clearing_complete', 'terminal_category', 'first_failure_stage', 'source_commit',
    'rom_admitted', 'acquisition_complete', 'application_identity_restored', 'hardware_baseline_verified', 'cleanup_complete',
    'expected_installed_source', 'expected_installed_elf']);
  check(value.schema_version === 'bitaxe-development-core-dump-read-v1' &&
    ['clearing_complete', 'rom_admitted', 'acquisition_complete', 'application_identity_restored', 'hardware_baseline_verified', 'cleanup_complete']
      .every(key => typeof value[key] === 'boolean') && /^[a-f0-9]{40}$/u.test(value.source_commit) &&
    /^[a-f0-9]{40}$/u.test(value.expected_installed_source) && /^[a-f0-9]{64}$/u.test(value.expected_installed_elf) &&
    ['complete', 'acquisition_failed', 'application_return_failed', 'cleanup_failed'].includes(value.terminal_category) &&
    ['session_admission', 'physical_identity', 'rom_admission', 'partition_table_read', 'partition_table_validation', 'dump_read',
      'complete', 'acquisition_failed', 'application_return_failed', 'cleanup_failed'].includes(value.first_failure_stage),
  'acquisition_producer_shape');
  return value;
}
/** Verify the original table's checksum and current full-region geometry without rewriting bytes. */
export function fullRegion(table) {
  check(table.length === 4096, 'acquisition_table_length');
  let checksum = false, found = 0; const ranges = [];
  for (let at = 0; at < table.length; at += 32) {
    const magic = table.readUInt16LE(at);
    if (magic === 0xebeb) {
      const digest = createHash('md5').update(table.subarray(0, at)).digest();
      check(table.subarray(at + 2, at + 16).every(byte => byte === 255) && table.subarray(at + 16, at + 32).equals(digest) &&
        table.subarray(at + 32).every(byte => byte === 255), 'acquisition_table_checksum');
      checksum = true; break;
    }
    check(magic === 0x50aa, 'acquisition_table_entry');
    const offset = table.readUInt32LE(at + 4), size = table.readUInt32LE(at + 8), end = offset + size;
    check(size > 0 && offset >= 0x9000 && end <= 0x1000000 && !ranges.some(([start, stop]) => offset < stop && start < end),
      'acquisition_table_range'); ranges.push([offset, end]);
    if (table[at + 2] === 1 && table[at + 3] === 3) {
      found++;
      check(offset === 0xf12000 && size === DUMP_BYTES && table.readUInt32LE(at + 28) === 0, 'acquisition_full_region');
    }
  }
  check(checksum && found === 1, 'acquisition_table_missing');
}
export function acquisitionBlockers(result, source) {
  const blockers = [];
  if (result.source_commit !== source) blockers.push('acquisition_source_mismatch');
  if (result.expected_installed_source !== INSTALLED_SOURCE || result.expected_installed_elf !== INSTALLED_ELF)
    blockers.push('acquisition_installed_identity');
  if (result.clearing_complete || result.hardware_baseline_verified) blockers.push('acquisition_wrong_effect');
  for (const key of ['rom_admitted', 'acquisition_complete', 'application_identity_restored', 'cleanup_complete'])
    if (result[key] !== true) blockers.push(`acquisition_${key}_unproved`);
  if (result.terminal_category !== 'complete' || result.first_failure_stage !== 'complete') blockers.push('acquisition_producer_failed');
  return blockers;
}
