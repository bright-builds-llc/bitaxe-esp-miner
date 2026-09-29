import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, chmod, writeFile, readFile, rm, stat } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync, spawnSync } from 'node:child_process';
import { provenanceFixture } from './provenance-fixture.mjs';
const repo = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim(), here = dirname(fileURLToPath(import.meta.url));
async function run(t, kind) {
  const root = await realpath(await mkdtemp(resolve(repo, 'scratch/provenance-test-'))); await chmod(root, 0o700); t.after(() => rm(root, { recursive: true }));
  const f = provenanceFixture(kind); await writeFile(resolve(root, 'dump'), f.dump, { mode: 0o600 }); await writeFile(resolve(root, 'elf'), f.program, { mode: 0o600 });
  return { root, result: spawnSync(process.execPath, [resolve(here, 'main.mjs'), 'verify-provenance', '--dump', resolve(root, 'dump'), '--elf', resolve(root, 'elf'), '--elf-sha256', f.identity, '--private-root', resolve(root, 'result')], { cwd: repo, encoding: 'utf8' }) };
}
for (const kind of ['real', 'fake']) test(`real vendor decoder verifies copied original frame with ${kind} SDK stack`, async t => {
  const { root, result } = await run(t, kind);
  assert.equal(result.status, 0, result.stderr); const summary = JSON.parse(result.stdout);
  assert.equal(summary.original_frame_meaningful, true); assert.equal(summary.sdk_fake_crashed_frame, kind === 'fake');
  assert.equal(summary.self_test_provenance_qualified, true); assert.equal(summary.cause_proven, false);
  assert.doesNotMatch(result.stdout, /panic_words|allocation_records":|framePtr|3fc90000|1070/);
  assert.equal((await stat(resolve(root, 'result/provenance.private.json'))).mode & 0o777, 0o600);
  assert.equal(JSON.parse(await readFile(resolve(root, 'result/provenance.private.json'))).panic_words.length, 48);
});
for (const kind of ['bad-pointer', 'bad-pc', 'wrong-source', 'wrong-boot', 'torn', 'bad-note', 'missing-memory', 'overlap', 'allocation-torn', 'allocation-checksum']) test(`provenance rejects ${kind}`, async t => {
  const { result } = await run(t, kind); assert.equal(result.status, 1); assert.equal(result.stdout, '');
});

for (const kind of ['allocation-history', 'allocation-early']) test(`bounded ${kind} preserves first failure and ring metadata privately`, async t => {
  const { result } = await run(t, kind); assert.equal(result.status, 0, result.stderr);
  const summary = JSON.parse(result.stdout); assert.equal(summary.allocation_records_verified, 9);
  assert.equal(summary.allocation_history_overwritten, true); assert.equal(summary.allocation_early_unbound_records, kind === 'allocation-early' ? 1 : 0);
});

test('torn allocation history preserves independent raw captured panic bytes without qualification', async t => {
  const { root, result } = await run(t, 'allocation-torn');
  assert.equal(result.status, 1); assert.equal(result.stdout, '');
  const raw = JSON.parse(await readFile(resolve(root, 'result/provenance-raw.private.json')));
  assert.equal(raw.verified, false); assert.equal(raw.cause_proven, false); assert.equal(raw.panic_words.length, 48);
  assert.equal(raw.allocation_words.length, 384); assert.equal(raw.panic_words[0], 0x50465231);
  assert.equal((await stat(resolve(root, 'result/provenance-raw.private.json'))).mode & 0o777, 0o600);
  await assert.rejects(stat(resolve(root, 'result/provenance.private.json')), { code: 'ENOENT' });
});

test('last full internal-DRAM frame span is readable at the inclusive SDK boundary', async t => {
  const { result } = await run(t, 'last-span'); assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).original_frame_meaningful, true);
});
