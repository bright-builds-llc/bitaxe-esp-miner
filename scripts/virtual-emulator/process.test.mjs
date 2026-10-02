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
  if (mode === 'projection-lines') {
    process.stdout.write('ordinary output\nVIRTUAL_U205 first\nnoise VIRTUAL_U205 embedded\nVIRTUAL_U205 partial');
    process.stderr.write('ordinary stderr\n');
    return;
  }
  if (mode === 'projection-split') {
    process.stdout.write('VIRT');
    setTimeout(() => process.stdout.write('UAL_U205 split\nordinary\nVIRTUAL_U205 second\r'), 10);
    setTimeout(() => process.stdout.write('\nVIRTUAL_U205 unfinished'), 20);
    return;
  }
  if (mode === 'projection-oversized') {
    process.stdout.write(`VIRTUAL_U205 ${'x'.repeat(65536)}\n`);
    return;
  }
  if (mode === 'projection-late-parent') {
    spawn(process.execPath, [self, '--child', 'projection-late-writer'], { stdio: ['ignore', 1, 2] }).unref();
    process.stdout.write('VIRTUAL_U205 parent\n');
    return;
  }
  if (mode === 'projection-late-writer') {
    setInterval(() => {
      process.stdout.write('ordinary late output\nVIRTUAL_U205 late\n');
      process.stderr.write('ordinary late stderr\n');
    }, 5);
    return;
  }
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

  test('projection persists only complete matching stdout lines and discards stderr', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-output-'));
    try {
      // Act
      const outcome = await runPrivate(process.execPath, [self, '--child', 'projection-lines'], root, 'projected', {
        timeoutMs: 1000, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      });
      // Assert
      assert.equal(await readFile(join(root, 'projected.stdout.log'), 'utf8'), 'VIRTUAL_U205 first\n');
      assert.equal((await stat(join(root, 'projected.stderr.log'))).size, 0);
      assert.ok(outcome.outputBytes > outcome.persistedOutputBytes);
      assert.equal(outcome.persistedOutputBytes, Buffer.byteLength('VIRTUAL_U205 first\n'));
      await assertReleased(root, 'projected');
    } finally { await rm(root, { recursive: true }); }
  });

  test('projection reconstructs matching lines split across chunks without retaining an unfinished line', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-split-'));
    try {
      // Act
      await runPrivate(process.execPath, [self, '--child', 'projection-split'], root, 'split', {
        timeoutMs: 1000, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      });
      // Assert
      assert.equal(await readFile(join(root, 'split.stdout.log'), 'utf8'), 'VIRTUAL_U205 split\nVIRTUAL_U205 second\r\n');
      await assertReleased(root, 'split');
    } finally { await rm(root, { recursive: true }); }
  });

  test('projection stops visibly when a partial stdout line exceeds sixty-four kibibytes', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-line-bound-'));
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'projection-oversized'], root, 'large', {
        timeoutMs: 1000, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      }), /emulator_line_bound/);
      // Assert
      const outcome = JSON.parse(await readFile(join(root, 'large.process.json'), 'utf8'));
      assert.equal(outcome.outputLineLimitExceeded, true);
      assert.equal(outcome.stdoutPartialLineLimitBytes, 65536);
      assert.equal((await stat(join(root, 'large.stdout.log'))).size, 0);
      await assertReleased(root, 'large');
    } finally { await rm(root, { recursive: true }); }
  });

  test('discarded output still consumes the shared raw output budget', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-raw-bound-'));
    try {
      // Act
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'spam'], root, 'spam', {
        timeoutMs: 1000, maxOutputBytes: 8192, outputLinePrefix: 'VIRTUAL_U205 ',
      }), /emulator_output_bound/);
      // Assert
      const outcome = JSON.parse(await readFile(join(root, 'spam.process.json'), 'utf8'));
      assert.equal(outcome.outputBytes, 8192);
      assert.equal(outcome.persistedOutputBytes, 0);
      assert.equal(outcome.outputLimitExceeded, true);
      await assertReleased(root, 'spam');
    } finally { await rm(root, { recursive: true }); }
  });

  test('projected late writers stop before descriptors and the lease release', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-late-writer-'));
    try {
      // Act
      const outcome = await runPrivate(process.execPath, [self, '--child', 'projection-late-parent'], root, 'late', {
        timeoutMs: 200, allowTimeout: true, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      });
      const before = await readFile(join(root, 'late.stdout.log'), 'utf8');
      await wait(40);
      // Assert
      assert.equal(outcome.timedOut, true);
      assert.ok(before.includes('VIRTUAL_U205 late\n'));
      assert.ok(before.split('\n').filter(Boolean).every(line => line.startsWith('VIRTUAL_U205 ')));
      assert.equal(await readFile(join(root, 'late.stdout.log'), 'utf8'), before);
      assert.equal((await stat(join(root, 'late.stderr.log'))).size, 0);
      await assertReleased(root, 'late');
    } finally { await rm(root, { recursive: true }); }
  });

  test('projection requires a valid literal prefix and a raw output bound before effects', async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'projected-invalid-'));
    try {
      // Act
      for (const outputLinePrefix of ['', null, 42, 'line\n', 'line\r', 'line\0', 'x'.repeat(65537)]) {
        await assert.rejects(runPrivate(process.execPath, [self, '--child', 'small'], root, 'invalid', {
          outputLinePrefix, maxOutputBytes: 2097152,
        }), /emulator_output_projection_invalid/);
      }
      await assert.rejects(runPrivate(process.execPath, [self, '--child', 'small'], root, 'invalid', {
        outputLinePrefix: 'VIRTUAL_U205 ',
      }), /emulator_output_projection_invalid/);
      // Assert
      assert.deepEqual(await readdir(root), []);
    } finally { await rm(root, { recursive: true }); }
  });
}
