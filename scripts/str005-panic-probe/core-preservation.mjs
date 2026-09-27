/** An immutable empty core partition admits a bounded diagnostic, never a reset-cause claim. */
import { readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import { protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';

const CORE_BYTES = 974848;
export function validateCorePreservation(result, context, status, dump, table) {
  const source = context.before_source;
  check(result.schema_version === 'bitaxe-development-core-dump-read-v1' && result.terminal_category === 'complete' &&
    ['rom_admitted', 'acquisition_complete', 'application_identity_restored', 'cleanup_complete'].every(key => result[key] === true) &&
    result.source_commit === context.commit && result.expected_installed_source === source.firmware_commit &&
    result.expected_installed_elf === source.app_elf_sha256, 'panic_core_preservation_result');
  check(status.scope === 'share' && status.state === 'idle' && status.record === null &&
    Number.isSafeInteger(status.observation?.bootOrdinal) && status.observation.bootOrdinal >= 1 &&
    status.observation.bootOrdinal < Number.MAX_SAFE_INTEGER, 'panic_core_preservation_prior_boot');
  check(dump.length === CORE_BYTES && dump.every(byte => byte === 255), 'panic_core_preservation_not_empty');
  check(table.length === 4096, 'panic_core_preservation_table');
  const ranges = [];
  for (let offset = 0; offset < table.length; offset += 32) {
    if (table[offset] === 0xeb && table[offset + 1] === 0xeb) break;
    if (table[offset] !== 0xaa || table[offset + 1] !== 0x50) break;
    if (table[offset + 2] === 1 && table[offset + 3] === 3) ranges.push([table.readUInt32LE(offset + 4), table.readUInt32LE(offset + 8)]);
  }
  check(ranges.length === 1 && ranges[0][0] === 0xf12000 && ranges[0][1] === CORE_BYTES, 'panic_core_preservation_geometry');
  return { expected_boot_ordinal: status.observation.bootOrdinal + 1, empty_core_dump: true,
    dump_sha256: sha256(dump), partition_table_sha256: sha256(table), bytes: CORE_BYTES };
}

export async function verifyCorePreservation(root, expectedContext) {
  await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const saved = (await proof(root, 'context.json')).value;
  check(JSON.stringify(saved) === JSON.stringify(expectedContext) && saved.recoveryOnly === true, 'panic_core_preservation_context');
  const result = await proof(root, 'installed-core/result.private.json');
  const status = await proof(root, 'baseline-status.json');
  const dumpPath = resolve(root, 'installed-core/core-dump.private.bin'), tablePath = resolve(root, 'installed-core/partition-table.private.bin');
  await protectedPath(resolve(root, 'installed-core'), true); await protectedPath(dumpPath); await protectedPath(tablePath);
  check((await stat(dumpPath)).size === CORE_BYTES && (await stat(tablePath)).size === 4096, 'panic_core_preservation_size');
  const value = validateCorePreservation(result.value, saved, status.value, await readFile(dumpPath), await readFile(tablePath));
  return { schema: 'str005-existing-core-preservation-v1', recovery_seal_sha256: seal.sha256, result_sha256: result.sha256,
    firmware_commit: saved.before_source.firmware_commit, app_elf_sha256: saved.before_source.app_elf_sha256, ...value };
}
