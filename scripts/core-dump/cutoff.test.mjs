import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { chmod, mkdir, mkdtemp, readFile, realpath, rm, stat, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cutoffFixture } from './cutoff-fixture.mjs';
const source = dirname(fileURLToPath(import.meta.url));
const repo = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
async function run(t, kind) {
  await mkdir(join(repo, 'scratch'), { recursive: true });
  const root = await realpath(await mkdtemp(join(repo, 'scratch/cutoff-test-')));
  await chmod(root, 0o700); t.after(() => rm(root, { recursive: true, force: true }));
  const fixture = cutoffFixture(kind);
  await writeFile(join(root, 'dump'), fixture.dump, { mode: 0o600 });
  await writeFile(join(root, 'elf'), fixture.program, { mode: 0o600 });
  const result = spawnSync(process.execPath, [join(source, 'main.mjs'), 'verify-cutoff', '--dump', join(root, 'dump'), '--elf', join(root, 'elf'),
    '--elf-sha256', fixture.identity, '--private-root', join(root, 'result')], { cwd: repo, encoding: 'utf8' });
  return { root, result, fixture };
}
test('cutoff comes from a captured 28-byte user region without bulk heap after vendor checksum and ELF identity verification', async t => {
  // Arrange / Act
  const { root, result, fixture } = await run(t, 'valid');
  // Assert
  assert.equal(result.status, 0, result.stderr);
  const summary = JSON.parse(result.stdout);
  for (const field of ['full_elf_identity_verified', 'checksum_verified', 'native_cutoff_verified', 'captured_memory_verified', 'asic_outputs_disabled', 'generation_revoked', 'self_test_marked']) assert.equal(summary[field], true);
  assert.equal(summary.elf_sha256, fixture.identity);
  assert.match(summary.cutoff_verifier_sha256, /^[0-9a-f]{64}$/);
  assert.equal((await stat(join(root, 'result/inspection.json'))).mode & 0o777, 0o600);
  assert.doesNotMatch(result.stdout, /configuredMask|revokedState|50434f32|3fc90000/);
});
for (const kind of ['missing-symbol', 'missing-memory', 'program-receipt-only', 'truncated', 'overlap', 'unsafe', 'unconfigured', 'input-mode', 'bad-magic', 'unrevoked', 'generation-mismatch', 'marker-invalid', 'checksum']) test(`rejects ${kind} cutoff evidence`, async t => {
  // Arrange / Act
  const { root, result } = await run(t, kind);
  // Assert
  assert.equal(result.status, 1); assert.equal(JSON.parse(result.stderr).category, 'decoder_failed');
  assert.equal(result.stdout, '');
  const categories = { 'missing-symbol': 'cutoff_symbol', 'missing-memory': 'cutoff_mapping', 'program-receipt-only': 'cutoff_mapping', truncated: 'cutoff_truncated',
    overlap: 'cutoff_mapping', unsafe: 'cutoff_unsafe_outputs', unconfigured: 'cutoff_unsafe_outputs', 'input-mode': 'cutoff_unsafe_outputs',
    'bad-magic': 'cutoff_magic', unrevoked: 'cutoff_unrevoked', 'generation-mismatch': 'cutoff_unrevoked', 'marker-invalid': 'cutoff_self_test_marker' };
  if (categories[kind]) assert.match(await readFile(join(root, 'result/inspect.stderr'), 'utf8'), new RegExp(categories[kind]));
});
test('ordinary panic cutoff cannot masquerade as intentional self-test', async t => {
  // Arrange / Act
  const { result } = await run(t, 'ordinary');
  // Assert
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).native_cutoff_verified, true);
  assert.equal(JSON.parse(result.stdout).self_test_marked, false);
});
test('decoder constants agree with the actual native receipt ABI', async () => {
  // Arrange / Act
  const native = await readFile(join(repo, 'firmware/bitaxe/src/panic_cutoff.rs'), 'utf8');
  const model = await readFile(join(repo, 'firmware/bitaxe/src/panic_cutoff_model.rs'), 'utf8');
  const revocation = await readFile(join(repo, 'firmware/bitaxe/src/production_mining_session/revocation.rs'), 'utf8');
  // Assert
  assert.match(native, /link_section = "\.dram2\.coredump\.bitaxe_panic_cutoff"/);
  assert.match(native, /BITAXE_PANIC_CUTOFF_RECEIPT: \[u32; 7\]/);
  assert.match(native, /write_volatile\(receipt, 0x50434f32\)/);
  assert.match(native, /SELF_TEST_MARKER.store\(0x53544631/);
  assert.match(model, /ENABLE_MASK: u32 = 1 << 10/); assert.match(model, /RESET_MASK: u32 = 1 << 1/);
  assert.match(revocation, /REVOKED: u32 = 4/); assert.match(revocation, /FLAGS: u32 = 7/);
  assert.match(revocation, /GENERATION_SHIFT: u32 = 3/);
});
