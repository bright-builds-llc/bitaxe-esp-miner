import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { proof } from '../str005-noise-serial/files.mjs';
import { sourceFingerprint } from '../str005-panic-probe/store-diagnostics.mjs';
import { validatePanicJoin, validateProvenanceSummary } from './capture-proof.mjs';
import { finalizeDiagnostic } from './finalize.mjs';

const context = { stage: 'capture', firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64),
  gate_root: '/unused', firmware_root: '/unused', detector: { physical: 'c'.repeat(64) } };
const fields = ['checksum_verified', 'full_elf_identity_verified', 'native_cutoff_verified', 'captured_memory_verified',
  'asic_outputs_disabled', 'generation_revoked', 'self_test_marked', 'fault_provenance_verified',
  'original_frame_meaningful', 'self_test_provenance_qualified'];

test('qualified provenance must join the exact dump and installed ELF', () => {
  // Arrange
  const dumpSha = 'd'.repeat(64), report = { schema: 'bitaxe-private-core-inspection/1', chip: 'esp32s3',
    decoder_version: '1.17.2', elf_sha256: context.app_elf_sha256, dump_sha256: dumpSha, cause_proven: false };
  for (const field of fields) report[field] = true;
  // Act / Assert
  assert.doesNotThrow(() => validateProvenanceSummary(report, context, dumpSha));
  assert.throws(() => validateProvenanceSummary({ ...report, original_frame_meaningful: false }, context, dumpSha));
  assert.throws(() => validateProvenanceSummary({ ...report, dump_sha256: 'e'.repeat(64) }, context, dumpSha));
});

test('captured original frame joins the installed source and self-test boot', () => {
  // Arrange
  const words = Array(48).fill(0), fingerprint = BigInt(`0x${sourceFingerprint(context.firmware_commit)}`);
  words[4] = Number(fingerprint & 0xffffffffn); words[5] = Number(fingerprint >> 32n);
  words[6] = 20;
  const record = { schema: 'bitaxe-private-fault-provenance/1', elf_sha256: context.app_elf_sha256,
    cause_proven: false, panic_words: words };
  // Act / Assert
  assert.doesNotThrow(() => validatePanicJoin(record, context, { expectedBootOrdinal: 20 }));
  assert.throws(() => validatePanicJoin(record, context, { expectedBootOrdinal: 21 }));
  assert.throws(() => validatePanicJoin({ ...record, panic_words: [...words.slice(0, 4), 0, ...words.slice(5)] }, context,
    { expectedBootOrdinal: 20 }));
});

test('missing acquisition seals incomplete capture while preserving an earlier failure', async t => {
  // Arrange
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'diagnostic-capture-finalize-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const firstFailure = { phase: 'status', category: 'timeout' };
  // Act
  const result = await finalizeDiagnostic(root, context, { complete: false, blockers: [], first_failure: firstFailure });
  // Assert
  assert.equal(result.complete, false); assert.equal(result.core_capture_verified, false);
  assert.deepEqual(result.first_failure, firstFailure);
  assert.ok(result.blockers.includes('diagnostic_capture_acquisition_unavailable'));
  assert.ok((await proof(root, 'sealed-inventory.json')).value.files);
});

test('unreleased offline decoder leaves a private unsealed result and fails finalization', async t => {
  // Arrange
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'diagnostic-decoder-release-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const review = { complete: false, failures: [{ phase: 'provenance', category: 'release_unproven' }],
    first_failure: { phase: 'provenance', category: 'release_unproven' }, decoder_released: false };
  // Act / Assert
  await assert.rejects(finalizeDiagnostic(root, context, { complete: true, blockers: [] },
    { assessCapture: async () => review }), { code: 'diagnostic_decoder_release_unproven' });
  assert.equal((await proof(root, 'capture-unsealed-result.json')).value.complete, false);
  await assert.rejects(proof(root, 'sealed-inventory.json'), { code: 'ENOENT' });
});
