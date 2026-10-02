import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, readdir, readlink, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as wait } from 'node:timers/promises';
import { runPrivate } from './process.mjs';

const self = fileURLToPath(import.meta.url);

function runChild(mode) {
  if (mode === 'spam') {
    const block = Buffer.alloc(32768, 0x61);
    setInterval(() => { process.stdout.write(block); process.stderr.write(block); }, 1);
    return;
  }
  if (mode === 'late-parent' || mode === 'late-failure') {
    spawn(process.execPath, [self, '--child', 'late-writer'], { stdio: ['ignore', 1, 2] }).unref();
    process.stdout.write('parent completed\n');
    if (mode === 'late-failure') process.exitCode = 7;
    return;
  }
  if (mode === 'late-writer') {
    setInterval(() => { process.stdout.write('late stdout\n'); process.stderr.write('late stderr\n'); }, 5);
    return;
  }
  if (mode === 'failure') { process.exitCode = 7; return; }
  if (mode === 'small') { process.stdout.write('stdout'); process.stderr.write('stderr'); return; }
  throw Error('unknown_test_child');
}

async function assertReleased(root, label) {
  await assert.rejects(stat(join(root, `${label}.writer.json`)), { code: 'ENOENT' });
  const owner = JSON.parse(await readFile(join(root, `${label}.owner.json`), 'utf8'));
  assert.throws(() => process.kill(-owner.pgid, 0), { code: 'ESRCH' });
  const links = await Promise.allSettled((await readdir('/dev/fd')).map(fd => readlink(`/dev/fd/${fd}`)));
  assert.equal(links.filter(result => result.status === 'fulfilled' && result.value.startsWith(root)).length, 0);
}

if (process.argv[2] === '--child') {
  runChild(process.argv[3]);
} else {
  const { test } = await import('node:test');

  test('combined stdout and stderr cannot exceed the private output budget', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-output-'));
    const budget = 70000;
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'spam'], root, 'spam', {
        timeoutMs: 1000, allowTimeout: true, maxOutputBytes: budget,
      }), /emulator_output_bound/);
      // Assert
      const outcome = JSON.parse(await readFile(join(root, 'spam.process.json'), 'utf8'));
      const bytes = (await stat(join(root, 'spam.stdout.log'))).size + (await stat(join(root, 'spam.stderr.log'))).size;
      assert.equal(bytes, budget);
      assert.equal(outcome.outputBytes, bytes);
      assert.equal(outcome.outputLimitExceeded, true);
      assert.equal(outcome.timedOut, false);
      await assertReleased(root, 'spam');
    } finally { await rm(root, { recursive: true }); }
  });

  test('late descendant writers are terminated before the durable lease is released', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-late-writer-'));
    try {
      // Act
      const outcome = await runPrivate(process.execPath, [self, '--child', 'late-parent'], root, 'late', {
        timeoutMs: 200, allowTimeout: true, maxOutputBytes: 70000,
      });
      const before = await readFile(join(root, 'late.stdout.log'));
      await wait(40);
      // Assert
      assert.equal(outcome.timedOut, true);
      assert.equal(outcome.released, true);
      assert.ok(before.toString().includes('late stdout'));
      assert.deepEqual(await readFile(join(root, 'late.stdout.log')), before);
      await assertReleased(root, 'late');
    } finally { await rm(root, { recursive: true }); }
  });

  test('failed natural completion remains failed after bounded capture and release', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-failed-child-'));
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'failure'], root, 'failure', {
        timeoutMs: 1000, maxOutputBytes: 70000,
      }), /emulator_command_failed/);
      // Assert
      const outcome = JSON.parse(await readFile(join(root, 'failure.process.json'), 'utf8'));
      assert.equal(outcome.code, 7);
      assert.equal(outcome.released, true);
      assert.equal(outcome.outputLimitExceeded, false);
      await assertReleased(root, 'failure');
    } finally { await rm(root, { recursive: true }); }
  });

  test('a later allowed cleanup timeout cannot replace the direct child failure', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-late-failure-'));
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'late-failure'], root, 'failure', {
        timeoutMs: 200, allowTimeout: true, maxOutputBytes: 70000,
      }), /emulator_command_failed/);
      // Assert
      const outcome = JSON.parse(await readFile(join(root, 'failure.process.json'), 'utf8'));
      assert.equal(outcome.code, 7);
      assert.equal(outcome.timedOut, true);
      await assertReleased(root, 'failure');
    } finally { await rm(root, { recursive: true }); }
  });

  test('successful bounded capture persists both streams in private files', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-small-child-'));
    try {
      // Act
      const outcome = await runPrivate(process.execPath, [self, '--child', 'small'], root, 'small', {
        timeoutMs: 1000, maxOutputBytes: 12,
      });
      // Assert
      assert.equal(outcome.outputBytes, 12);
      assert.equal(outcome.outputLimitExceeded, false);
      assert.equal(await readFile(join(root, 'small.stdout.log'), 'utf8'), 'stdout');
      assert.equal(await readFile(join(root, 'small.stderr.log'), 'utf8'), 'stderr');
      for (const suffix of ['stdout.log', 'stderr.log', 'process.json', 'owner.json']) {
        assert.equal((await stat(join(root, `small.${suffix}`))).mode & 0o777, 0o600);
      }
      await assertReleased(root, 'small');
    } finally { await rm(root, { recursive: true }); }
  });

  test('spawn failure releases logs and the lease without an owner or late writer', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-spawn-failure-'));
    try {
      // Act
      await assert.rejects(runPrivate(join(root, 'absent-command'), [], root, 'absent', {
        timeoutMs: 1000, maxOutputBytes: 12,
      }), /emulator_spawn/);
      // Assert
      await assert.rejects(stat(join(root, 'absent.writer.json')), { code: 'ENOENT' });
      await assert.rejects(stat(join(root, 'absent.owner.json')), { code: 'ENOENT' });
      assert.equal((await stat(join(root, 'absent.stdout.log'))).size, 0);
      assert.equal((await stat(join(root, 'absent.stderr.log'))).size, 0);
    } finally { await rm(root, { recursive: true }); }
  });

  test('failed completion is preserved when result persistence also fails', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-result-failure-'));
    await writeFile(join(root, 'failure.process.json'), 'sealed earlier result', { flag: 'wx', mode: 0o600 });
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'failure'], root, 'failure', {
        timeoutMs: 1000, maxOutputBytes: 70000,
      }), error => error.message === 'emulator_command_failed' && error.cleanupFailures?.[0].includes('EEXIST'));
      // Assert
      assert.equal(await readFile(join(root, 'failure.process.json'), 'utf8'), 'sealed earlier result');
      await assertReleased(root, 'failure');
    } finally { await rm(root, { recursive: true }); }
  });

  test('invalid output bounds are rejected before a writer lease or process exists', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'bounded-invalid-budget-'));
    try {
      // Act
      for (const maxOutputBytes of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
        await assert.rejects(runPrivate(process.execPath, [self, '--child', 'small'], root, 'invalid', {
          maxOutputBytes,
        }), /emulator_output_bound_invalid/);
      }
      // Assert
      assert.deepEqual(await readdir(root), []);
    } finally { await rm(root, { recursive: true }); }
  });
}
