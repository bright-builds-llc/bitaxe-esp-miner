import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { capturePackage } from './prepare.mjs';
import { argumentsFor } from './contract.mjs';
import { sha256 } from '../str005-v2-serial/values.mjs';
async function fixture(t) {
  const repo = await realpath(await mkdtemp(resolve(tmpdir(), 'retained-capture-'))); t.after(() => rm(repo, { recursive: true, force: true }));
  const archive = resolve(repo, 'archive'); await mkdir(archive, { mode: 0o700 });
  execFileSync('git', ['init', '--quiet'], { cwd: repo }); await writeFile(resolve(repo, '.git/info/exclude'), 'archive/\n');
  const kinds = ['firmware_elf', 'firmware_ota_image', 'www_spiffs_image', 'factory_merged_image', 'partition_table', 'otadata_initial', 'bootloader', 'partition_table_binary'];
  const artifacts = [];
  for (const kind of kinds) {
    const path = kind === 'firmware_elf' ? 'retained.elf' : `${kind}.bin`, contents = Buffer.from(`synthetic-${kind}`);
    await writeFile(resolve(kind === 'partition_table' ? repo : archive, path), contents, { mode: 0o600 });
    artifacts.push({ kind, path, sha256: sha256(contents) });
  }
  const manifest = { schema_version: 4, build_identity: { source_dirty: false }, source_commit: 'a'.repeat(40), reference_commit: 'b'.repeat(40),
    app_elf_sha256: artifacts[0].sha256, artifacts, update_segments: [['bootloader', 0], ['partition_table_binary', 0x8000], ['firmware_ota_image', 0x10000], ['www_spiffs_image', 0x410000], ['otadata_initial', 0xf10000]]
      .map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`synthetic-${artifact_kind}`) })) };
  const bytes = JSON.stringify(manifest), path = resolve(archive, 'manifest.json'); await writeFile(path, bytes, { mode: 0o600 });
  const installed = { context: { commit: 'a'.repeat(40), firmware_commit: manifest.source_commit, reference_commit: manifest.reference_commit,
    app_elf_sha256: manifest.app_elf_sha256, manifest_sha256: sha256(bytes), candidateElf: '/missing/mutable/bazel-bin.elf' } };
  return { repo, archive, path, installed };
}
test('capture-only retained manifest is required and cannot replace canonical installation input', () => {
  const base = ['preflight', '--stage', 'capture', '--private-root', '/p', '--gate-root', '/g', '--installation-root', '/i', '--recovery-root', '/r'];
  assert.throws(() => argumentsFor(base));
  assert.equal(argumentsFor([...base, '--retained-manifest', '/archive/manifest']).options['--retained-manifest'], '/archive/manifest');
  assert.throws(() => argumentsFor(['preflight', '--stage', 'installation', '--private-root', '/p', '--gate-root', '/g', '--manifest', '/canonical', '--clear-root', '/c', '--retained-manifest', '/archive']));
});
test('capture resolves the archived exact ELF without opening overwritten or missing canonical outputs', async t => {
  const f = await fixture(t);
  const result = await capturePackage(f.path, f.installed, f.repo);
  assert.equal(result.candidateElf, resolve(f.archive, 'retained.elf'));
  assert.equal(result.retainedManifestSha256, f.installed.context.manifest_sha256);
  assert.equal(f.installed.context.candidateElf, '/missing/mutable/bazel-bin.elf');
});
test('wrong manifest identity or any corrupted retained artifact blocks capture', async t => {
  const f = await fixture(t);
  await assert.rejects(capturePackage(f.path, { context: { ...f.installed.context, firmware_commit: 'c'.repeat(40) } }, f.repo));
  await assert.rejects(capturePackage(f.path, { context: { ...f.installed.context, manifest_sha256: 'f'.repeat(64) } }, f.repo));
  await writeFile(resolve(f.archive, 'www_spiffs_image.bin'), 'corrupt fixture', { mode: 0o600 });
  await assert.rejects(capturePackage(f.path, f.installed, f.repo));
});
