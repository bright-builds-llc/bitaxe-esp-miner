import test from 'node:test';
import assert from 'node:assert/strict';
import { validateCorePreservation } from './core-preservation.mjs';

function fixture() {
  const context = { commit: 'a'.repeat(40), before_source: { firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64) } };
  const result = { schema_version: 'bitaxe-development-core-dump-read-v1', terminal_category: 'complete', rom_admitted: true,
    acquisition_complete: true, application_identity_restored: true, cleanup_complete: true, source_commit: context.commit,
    expected_installed_source: context.before_source.firmware_commit, expected_installed_elf: context.before_source.app_elf_sha256 };
  const status = { scope: 'share', state: 'idle', record: null, observation: { bootOrdinal: 50 } };
  const dump = Buffer.alloc(974848, 255), table = Buffer.alloc(4096, 255);
  table.set([0xaa, 0x50, 1, 3]); table.writeUInt32LE(0xf12000, 4); table.writeUInt32LE(974848, 8);
  return { context, result, status, dump, table };
}
test('full empty partition and completed matching read bind the next managed boot', () => {
  const f = fixture(), value = validateCorePreservation(f.result, f.context, f.status, f.dump, f.table);
  assert.equal(value.expected_boot_ordinal, 51); assert.equal(value.empty_core_dump, true); assert.equal(value.bytes, 974848);
  assert.match(value.dump_sha256, /^[a-f0-9]{64}$/u); assert.match(value.partition_table_sha256, /^[a-f0-9]{64}$/u);
});
test('nonempty or shortened partition and wrong table geometry cannot authorize capture', () => {
  for (const mutate of [f => { f.dump[0] = 0; }, f => { f.dump = f.dump.subarray(1); }, f => { f.table.writeUInt32LE(0xf00000, 4); },
    f => { f.table.writeUInt32LE(65536, 8); }, f => { f.table = f.table.subarray(1); }]) {
    const f = fixture(); mutate(f); assert.throws(() => validateCorePreservation(f.result, f.context, f.status, f.dump, f.table));
  }
});
test('read failure, unproved cleanup, wrong image and invalid predecessor boot cannot authorize capture', () => {
  for (const mutate of [f => { f.result.terminal_category = 'failed'; }, f => { f.result.schema_version = 'bitaxe-development-core-dump-clear-v1'; },
    f => { f.result.rom_admitted = false; }, f => { f.result.acquisition_complete = false; }, f => { f.result.application_identity_restored = false; },
    f => { f.result.cleanup_complete = false; }, f => { f.result.source_commit = 'd'.repeat(40); }, f => { f.result.expected_installed_source = 'd'.repeat(40); },
    f => { f.result.expected_installed_elf = 'd'.repeat(64); }, f => { f.status.observation.bootOrdinal = 0; }]) {
    const f = fixture(); mutate(f); assert.throws(() => validateCorePreservation(f.result, f.context, f.status, f.dump, f.table));
  }
});
