import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { chmod, mkdtemp, readFile, realpath, rm, symlink, writeFile, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { argumentsFor, verifyAcquisition } from './main.mjs';
import { DUMP_BYTES, INSTALLED_SOURCE, INSTALLED_ELF } from './model.mjs';
import { inventory, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
const SOURCE = 'a'.repeat(40);
function table(size = DUMP_BYTES) {
  const data = Buffer.alloc(4096, 255); data.writeUInt16LE(0x50aa); data[2] = 1; data[3] = 3;
  data.writeUInt32LE(0xf12000, 4); data.writeUInt32LE(size, 8); data.fill(0, 12, 32); data.write('coredump', 12);
  data.writeUInt16LE(0xebeb, 32); createHash('md5').update(data.subarray(0, 32)).digest().copy(data, 48);
  return data;
}
async function fixture(t, changes = {}) {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'share-crash-offline-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const result = { schema_version: 'bitaxe-development-core-dump-read-v1', clearing_complete: false, terminal_category: 'complete',
    first_failure_stage: 'complete', source_commit: SOURCE, rom_admitted: true, acquisition_complete: true, application_identity_restored: true,
    hardware_baseline_verified: false, cleanup_complete: true, expected_installed_source: INSTALLED_SOURCE, expected_installed_elf: INSTALLED_ELF,
    ...changes };
  await writeFile(resolve(root, 'result.private.json'), JSON.stringify(result), { mode: 0o600 });
  await writeFile(resolve(root, 'partition-table.private.bin'), table(), { mode: 0o600 });
  await writeFile(resolve(root, 'core-dump.private.bin'), Buffer.alloc(DUMP_BYTES, 42), { mode: 0o600 });
  return root;
}
test('offline full-region verifier preserves original bytes and seals digest-only evidence once', async t => {
  // Arrange
  const root = await fixture(t), before = await inventory(root);
  // Act
  const result = await verifyAcquisition(root, { source: SOURCE });
  // Assert
  assert.equal(result.complete, true); assert.equal(result.sealed, true); assert.equal(result.decoder_verified, false);
  assert.equal(result.full_region_bytes, DUMP_BYTES);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  assert.deepEqual(seal.value.files.filter(row => before.some(old => old.path === row.path)), before);
  await assert.rejects(verifyAcquisition(root, { source: SOURCE }));
});
for (const [name, mutate, blocker] of [
  ['legacy 64 KiB partition', async root => writeFile(resolve(root, 'partition-table.private.bin'), table(65536)), 'acquisition_full_region'],
  ['truncated dump', async root => writeFile(resolve(root, 'core-dump.private.bin'), Buffer.alloc(100)), 'acquisition_dump_length'],
  ['altered table checksum', async root => { const data = table(); data[48] ^= 1; await writeFile(resolve(root, 'partition-table.private.bin'), data); }, 'acquisition_table_checksum'],
]) test(`offline verifier rejects ${name} while sealing released partial acquisition`, async t => {
  const root = await fixture(t); await mutate(root);
  const result = await verifyAcquisition(root, { source: SOURCE });
  assert.equal(result.complete, false); assert.equal(result.sealed, true); assert.ok(result.blockers.includes(blocker));
});
for (const [key, value] of [
  ['expected_installed_source', 'b'.repeat(40)], ['expected_installed_elf', 'b'.repeat(64)], ['source_commit', 'b'.repeat(40)],
  ['rom_admitted', false], ['acquisition_complete', false], ['application_identity_restored', false], ['cleanup_complete', false],
  ['clearing_complete', true], ['hardware_baseline_verified', true],
]) test(`offline verifier cannot pass conflicting producer ${key}`, async t => {
  const root = await fixture(t, { [key]: value });
  const result = await verifyAcquisition(root, { source: SOURCE });
  assert.equal(result.complete, false); assert.ok(result.blockers.length > 0);
  if (key === 'cleanup_complete') { assert.equal(result.sealed, false); await assert.rejects(access(resolve(root, 'sealed-inventory.json'))); }
});
test('an unknown producer field cannot be treated as proof of release', async t => {
  const root = await fixture(t, { fabricated: true });
  const result = await verifyAcquisition(root, { source: SOURCE });
  assert.equal(result.complete, false); assert.equal(result.release_proven, false); assert.equal(result.sealed, false);
});
test('unsafe file modes fail before writing a receipt', async t => {
  const root = await fixture(t); await chmod(resolve(root, 'core-dump.private.bin'), 0o644);
  await assert.rejects(verifyAcquisition(root, { source: SOURCE }));
  await assert.rejects(access(resolve(root, 'acquisition-verification.json')));
});
test('an aliased root is rejected without modifying the original', async t => {
  const root = await fixture(t), alias = `${root}-alias`; await symlink(root, alias); t.after(() => rm(alias));
  await assert.rejects(verifyAcquisition(alias, { source: SOURCE }));
  await assert.rejects(access(resolve(root, 'acquisition-verification.json')));
});
test('concurrent producer-proof tampering is caught before receipt or seal', async t => {
  // Arrange
  const root = await fixture(t); let calls = 0;
  // Act
  await assert.rejects(verifyAcquisition(root, { source: SOURCE, recheckSource: async () => {
    if (++calls === 2) {
      const path = resolve(root, 'result.private.json'), result = JSON.parse(await readFile(path));
      result.expected_installed_elf = 'e'.repeat(64); await writeFile(path, JSON.stringify(result));
    }
  } }), /noise_inventory_changed/);
  // Assert
  await assert.rejects(access(resolve(root, 'acquisition-verification.json')));
  await assert.rejects(access(resolve(root, 'sealed-inventory.json')));
});
test('command exposes only the offline verifier and an absolute root', () => {
  assert.equal(argumentsFor(['verify-acquisition', '--private-root', '/private/root']), '/private/root');
  for (const command of ['read', 'flash', 'clear']) assert.throws(() => argumentsFor([command, '--private-root', '/private/root']));
  assert.throws(() => argumentsFor(['verify-acquisition', '--private-root', 'relative']));
});
