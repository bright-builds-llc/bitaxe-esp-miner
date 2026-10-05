import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, chmod, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { inventory } from '../str005-noise-serial/files.mjs';
import { HEAD, advanceInstall, recordStart, validateHead, verifyHead } from './head.mjs';

const identity = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40) };

async function sealedRoot(firmwareRoot, path, files) {
  const root = resolve(firmwareRoot, path);
  await mkdir(root, { recursive: true, mode: 0o700 }); await chmod(root, 0o700);
  for (const [name, value] of Object.entries(files)) await writeFile(resolve(root, name), JSON.stringify(value), { mode: 0o600 });
  await writeFile(resolve(root, 'sealed-inventory.json'), JSON.stringify({ files: await inventory(root) }), { mode: 0o600 });
  return root;
}
async function lineage(startImage = identity) {
  const firmwareRoot = await realpath(await mkdtemp(resolve(tmpdir(), 'lineage-')));
  await mkdir(resolve(firmwareRoot, 'scratch'), { mode: 0o700 });
  await sealedRoot(firmwareRoot, 'scratch/install', {
    'context.json': { context: { profile: 'fixture-install', ...identity } },
    'final-result.json': { schema: 'noise-serial-result-v2', status: 'passed', outcome: 'complete' } });
  const record = { attemptId: 'device-attempt', scope: 'share' };
  await sealedRoot(firmwareRoot, 'scratch/start', {
    'context.json': { ...startImage, attemptId: 'owner-nonce' }, 'before.json': { attempt: { id: 'device-attempt' } },
    'run.json': { dispatchStatus: { record } }, 'recovery-1-status.json': { state: 'terminal', record },
    'recovery-1-ledger.json': { next_ordinal: 27 }, 'result.json': { complete: true } });
  return firmwareRoot;
}

test('the committed head is well formed', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => validateHead(HEAD));
});

test('advancing the install derives its pins and clears the latest Start', async () => {
  // Arrange
  const firmwareRoot = await lineage();
  // Act
  const head = await advanceInstall(firmwareRoot, 'scratch/install');
  // Assert
  assert.deepEqual([head.install.profile, head.install.identity, head.latestStart], ['fixture-install', identity, null]);
  assert.deepEqual(await verifyHead(firmwareRoot, head), head);
});

test('a Start on the installed image is recorded and verifies', async () => {
  // Arrange
  const firmwareRoot = await lineage(), installed = await advanceInstall(firmwareRoot, 'scratch/install');
  // Act
  const head = await recordStart(firmwareRoot, installed, 'scratch/start');
  // Assert
  assert.equal(head.latestStart.path, 'scratch/start');
  assert.deepEqual(await verifyHead(firmwareRoot, head), head);
});

test('a Start on another image is refused', async () => {
  // Arrange
  const firmwareRoot = await lineage({ ...identity, firmware_commit: 'd'.repeat(40) });
  const installed = await advanceInstall(firmwareRoot, 'scratch/install');
  // Act / Assert
  await assert.rejects(recordStart(firmwareRoot, installed, 'scratch/start'), /lineage_start_other_image/u);
});

test('a head whose pins no longer match the sealed root is refused', async () => {
  // Arrange
  const firmwareRoot = await lineage(), head = await advanceInstall(firmwareRoot, 'scratch/install');
  const typed = { ...head, install: { ...head.install, seal: 'e'.repeat(64) } };
  // Act / Assert
  await assert.rejects(verifyHead(firmwareRoot, typed), /lineage_head_install_changed/u);
});

test('paths outside the scratch evidence tree are refused', () => {
  // Arrange
  const escaped = { ...HEAD, install: { ...HEAD.install, path: 'scratch/../TASKS.md' } };
  // Act / Assert
  assert.throws(() => validateHead(escaped), /lineage_head_shape/u);
  assert.throws(() => validateHead({ ...HEAD, install: { ...HEAD.install, path: '/tmp/install' } }), /lineage_head_shape/u);
});
