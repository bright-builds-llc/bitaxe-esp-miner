import assert from 'node:assert/strict';
import test from 'node:test';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, realpath, rm, stat, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { checkCommand, verifyCommandCheck } from './command-check.mjs';
import { flashArguments, runChild } from './install.mjs';

const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
test('actual prebuilt flash CLI rejects the historical conflicting flags and accepts generated dry-run against canonical package', async t => {
  // Arrange: an intentionally nonexistent node makes accidental hardware reliance fail visibly.
  await mkdir(resolve(repo, 'scratch'), { recursive: true });
  const root = await mkdtemp(resolve(repo, 'scratch/panic-command-check-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const flashBinary = await realpath(resolve(repo, 'bazel-bin/tools/flash/flash'));
  const manifest = resolve(repo, 'bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json');
  const context = { flashBinary, manifest, flashBinarySha256: await fileDigest(flashBinary), manifest_sha256: await fileDigest(manifest),
    detector: { port: '/dev/bitaxe-probe-dry-run-never-open', physical: 'a'.repeat(64) } };
  const bad = resolve(root, 'historical-invalid'); await mkdir(bad, { mode: 0o700 });
  // Act
  const old = await runChild(flashBinary, [...flashArguments(bad, context), '--redact-evidence', '--dry-run'], bad, 60000);
  const checked = await checkCommand(root, context);
  // Assert
  assert.notEqual(old.code, 0); assert.equal(checked.device_effects, false);
  assert.match(checked.runner_sha256, /^[a-f0-9]{64}$/u);
  assert.equal((await stat(resolve(root, 'command-check'))).mode & 0o777, 0o700);
  await verifyCommandCheck(root, { ...context, commandCheckSha256: checked.runner_sha256 });
  await writeFile(resolve(root, 'command-check/runner.json'), '{}');
  await assert.rejects(verifyCommandCheck(root, { ...context, commandCheckSha256: checked.runner_sha256 }), { code: 'panic_command_check_changed' });
});
