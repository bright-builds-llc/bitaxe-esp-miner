import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, readFile, realpath, stat, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { diagnosticProcesses, parseProcessRows, processSnapshot, sameProcess, ownedProcesses, processOwnership, runDiagnostic } from './capture.mjs';
import { runRecordedCommand } from './runner.mjs';

async function options(source, extra = {}) {
  const parent = await realpath(await mkdtemp(path.join(tmpdir(), 'host-stalls-test-')));
  return { command: [process.execPath, '-e', source], cwd: parent,
    evidenceRoot: path.join(parent, 'run'), label: 'test', maxCaptures: 0, processPollMs: 50, ...extra };
}

test('quiet command can finish successfully', async () => {
  // Arrange
  const input = await options('setTimeout(() => {}, 100)', { quietMs: 20 });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  assert.equal(result.outcome, 'success');
  assert.equal(result.stdout.totalBytes, 0);
  assert.equal(result.cleanup.complete, true);
});

test('first output records the delayed marker separately from spawn', async () => {
  // Arrange
  const input = await options('setTimeout(() => process.stdout.write("entered_main\\n"), 100)');
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  const spawn = result.events.find((event) => event.type === 'spawned');
  const marker = result.events.find((event) => event.type === 'first_stdout');
  assert(marker.offsetMs > spawn.offsetMs + 50);
  assert.equal(await readFile(path.join(input.evidenceRoot, 'stdout.log'), 'utf8'), 'entered_main\n');
});

test('nonzero exit is preserved', async () => {
  const result = await runRecordedCommand(await options('process.exit(7)'));
  assert.equal(result.outcome, 'nonzero');
  assert.equal(result.exitCode, 7);
});

test('spawn error becomes recorded evidence', async () => {
  const input = await options('', { command: ['/no/such/host-stall-program'] });
  const result = await runRecordedCommand(input);
  assert.equal(result.outcome, 'spawn_error');
  assert.equal(result.spawnError.code, 'ENOENT');
});

test('output cap truncates storage while draining the entire stream', async () => {
  // Arrange
  const input = await options('process.stdout.write("a".repeat(2000000))', { maxOutputBytes: 512 });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  assert.equal(result.outcome, 'success');
  assert.equal(result.stdout.totalBytes, 2000000);
  assert.equal(result.stdout.storedBytes, 512);
  assert.equal((await stat(path.join(input.evidenceRoot, 'stdout.log'))).size, 512);
});

test('timeout cleans the owned descendant after root exit', async () => {
  // Arrange
  const input = await options('require("node:child_process").spawn(process.execPath,["-e","setInterval(()=>{},1000)"],{stdio:"inherit"});setTimeout(()=>process.exit(0),100)', { timeoutMs: 300 });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  assert.equal(result.outcome, 'timeout');
  assert.equal(result.cleanup.complete, true);
});

test('signal cancellation restores process listener counts', async () => {
  // Arrange
  const count = process.listenerCount('SIGTERM');
  const input = await options('process.stderr.write("ready\\n");setInterval(()=>{},1000)');
  const pending = runRecordedCommand(input);
  const until = Date.now() + 5000;
  for (;;) {
    try {
      if ((await readFile(path.join(input.evidenceRoot, 'stderr.log'), 'utf8')).includes('ready')) break;
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    if (Date.now() > until) throw new Error('test_child_ready_marker_missing');
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  // Act
  process.emit('SIGTERM');
  const result = await pending;
  // Assert
  assert.equal(result.outcome, 'cancelled');
  assert.equal(result.cleanup.complete, true);
  assert.equal(process.listenerCount('SIGTERM'), count);
});

test('existing evidence directory cannot be overwritten', async () => {
  const input = await options('');
  await runRecordedCommand(input);
  await assert.rejects(runRecordedCommand(input), /EEXIST/);
});

test('symlinked evidence parent is rejected', async () => {
  // Arrange
  const input = await options('');
  const alias = path.join(input.cwd, 'alias');
  await symlink(input.cwd, alias);
  // Act / Assert
  await assert.rejects(runRecordedCommand({ ...input, evidenceRoot: path.join(alias, 'run') }), /symlink/);
});

test('evidence files are owner-only', async () => {
  const input = await options('');
  await runRecordedCommand(input);
  assert.equal((await stat(input.evidenceRoot)).mode & 0o777, 0o700);
  assert.equal((await stat(path.join(input.evidenceRoot, 'summary.json'))).mode & 0o777, 0o600);
});


test('missing diagnostic tool is an explicit capture error', async () => {
  const result = await runDiagnostic(['/missing/diagnostic-tool']);
  assert.equal(result.error.code, 'ENOENT');
});

test('diagnostic timeout is explicit and bounded', async () => {
  const result = await runDiagnostic([process.execPath, '-e', 'setInterval(()=>{},1000)'], 50);
  assert.equal(result.timedOut, true);
  assert.equal(result.signal, 'SIGKILL');
});

test('quiet capture records diagnostics without failing a quiet command', async () => {
  // Arrange
  const input = await options('setTimeout(()=>{}, 400)', { quietMs: 20, maxCaptures: 1 });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  assert.equal(result.outcome, 'success');
  assert.equal(result.captures.length, 1);
  assert.equal(result.captures[0].status, 'captured');
  const capture = JSON.parse(await readFile(path.join(input.evidenceRoot, 'capture-001.json'), 'utf8'));
  assert(capture.processes.some((entry) => entry.pid === result.pid));
});

test('abort signal cancels only its own recording', async () => {
  // Arrange
  const controller = new AbortController();
  const own = await options('setInterval(()=>{},1000)', { signal: controller.signal });
  const peer = await options('setTimeout(()=>{},200)');
  setTimeout(() => controller.abort(), 100);
  // Act
  const [cancelled, completed] = await Promise.all([runRecordedCommand(own), runRecordedCommand(peer)]);
  // Assert
  assert.equal(cancelled.outcome, 'cancelled');
  assert.equal(completed.outcome, 'success');
});

test('environment overrides reject unrelated keys before spawning', async () => {
  const input = await options('', { env: { PRIVATE_CREDENTIAL: 'never-record' } });
  await assert.rejects(runRecordedCommand(input), /invalid_environment_override/);
});


test('undefined optional settings retain defaults', async () => {
  const input = await options('', { quietMs: undefined, maxOutputBytes: undefined });
  const result = await runRecordedCommand(input);
  assert.equal(result.outcome, 'success');
});

test('already aborted recording does not spawn a command', async () => {
  // Arrange
  const controller = new AbortController();
  controller.abort();
  const input = await options('process.stdout.write("unexpected")', { signal: controller.signal });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  assert.equal(result.outcome, 'cancelled');
  assert.equal(result.pid, null);
  assert.equal(result.stdout.totalBytes, 0);
});


test('stderr marker matches across chunks after earlier stderr and only once', async () => {
  // Arrange
  const input = await options('process.stderr.write("loader diagnostic\\n");setTimeout(()=>process.stderr.write("entered_"),50);setTimeout(()=>process.stderr.write("main entered_main"),100)', { expectedStderrMarker: 'entered_main' });
  // Act
  const result = await runRecordedCommand(input);
  // Assert
  const first = result.events.find((event) => event.type === 'first_stderr');
  const markers = result.events.filter((event) => event.type === 'stderr_marker_observed');
  assert.equal(markers.length, 1);
  assert(markers[0].offsetMs >= first.offsetMs + 50);
});

test('stderr without marker does not claim main entry', async () => {
  const input = await options('process.stderr.write("loader failed")', { expectedStderrMarker: 'entered_main' });
  const result = await runRecordedCommand(input);
  assert.equal(result.events.some((event) => event.type === 'stderr_marker_observed'), false);
});


test('escaped descendant cannot admit a recycled original process group', () => {
  // Arrange
  const escaped = { pid: 12, ppid: 1, pgid: 12, startedAt: 'original-child' };
  const recycled = { pid: 10, ppid: 1, pgid: 10, startedAt: 'unrelated-new-root' };
  const escapedChild = { pid: 13, ppid: 12, pgid: 12, startedAt: 'owned-grandchild' };
  // Act
  const selected = ownedProcesses([escaped, recycled, escapedChild], 10, [escaped]);
  // Assert
  assert.deepEqual(selected.map((entry) => entry.pid).sort(), [12, 13]);
});


test('rejected process group stays unanchored on the next cleanup pass', () => {
  // Arrange
  const escaped = { pid: 12, ppid: 1, pgid: 12, startedAt: 'owned' };
  const unrelated = { pid: 10, ppid: 1, pgid: 10, startedAt: 'reused' };
  const rows = [escaped, unrelated];
  // Act
  const first = processOwnership(rows, 10, [escaped]);
  const second = processOwnership(rows, 10, first.owned);
  // Assert
  assert.deepEqual(first.unanchored, [unrelated]);
  assert.deepEqual(second.unanchored, [unrelated]);
  assert.deepEqual(second.owned, [escaped]);
});

test('live process selection prunes dead identities instead of retaining history', () => {
  // Arrange
  const root = { pid: 10, ppid: 1, pgid: 10, startedAt: 'root' };
  const exited = { pid: 11, ppid: 10, pgid: 10, startedAt: 'old-child' };
  const child = { pid: 12, ppid: 10, pgid: 10, startedAt: 'new-child' };
  // Act
  const current = processOwnership([root, child], 10, [root, exited]);
  // Assert
  assert.deepEqual(current.owned, [root, child]);
});

test('empty ownership history requires explicit fresh root admission', () => {
  // Arrange
  const root = { pid: 10, ppid: 1, pgid: 10, startedAt: 'root' };
  // Act / Assert
  assert.deepEqual(ownedProcesses([root], 10, []), []);
  assert.deepEqual(ownedProcesses([root], 10, [], true), [root]);
});


test('bounded diagnostic selection prefers compiler leaves before wrappers', () => {
  // Arrange
  const root = { pid: 10, ppid: 1, state: 'S' };
  const wrapper = { pid: 11, ppid: 10, state: 'S' };
  const compiler = { pid: 12, ppid: 11, state: 'S' };
  const zombie = { pid: 13, ppid: 11, state: 'Z' };
  // Act
  const selected = diagnosticProcesses([root, wrapper, compiler, zombie]);
  // Assert
  assert.deepEqual(selected, [compiler, root, wrapper]);
});


test('compact process metadata supports large parallel process populations', () => {
  // Arrange
  const output = Array.from({ length: 4000 }, (_, index) => `${index + 1} 1 1 Fri Sep 11 02:21:34 2026 S 0.0`).join('\n');
  assert(Buffer.byteLength(output) > 131072);
  // Act
  const rows = parseProcessRows(output);
  // Assert
  assert.equal(rows.length, 4000);
  assert.equal(rows.some((row) => 'executable' in row), false);
});

test('global process metadata rejects executable names and arguments', () => {
  const output = '10 1 10 Fri Sep 11 02:21:34 2026 S 0.0 /private/unrelated-long-path';
  assert.throws(() => parseProcessRows(output), /process_snapshot_parse_failed/);
});


test('cleanup failure releases recorder handles so the standalone process exits', async () => {
  // Arrange
  const input = await options('setInterval(()=>{},1000)', { timeoutMs: 100 });
  const runner = new URL('./runner.mjs', import.meta.url).href;
  const source = `import { runRecordedCommand } from ${JSON.stringify(runner)};
const result = await runRecordedCommand(${JSON.stringify(input)}, { snapshot: async () => { throw new Error('injected_snapshot_failure'); } });
process.stdout.write(JSON.stringify(result));`;
  let maybeResult;
  try {
    // Act
    const output = await promisify(execFile)(process.execPath, ['--input-type=module', '-e', source], { timeout: 10000 });
    maybeResult = JSON.parse(output.stdout);
    // Assert
    assert.equal(maybeResult.outcome, 'cleanup_failed');
    assert.equal(maybeResult.cleanup.complete, false);
    assert.equal(maybeResult.cleanup.signals.length, 0);
    assert(maybeResult.cleanup.failures.some((entry) => entry.error === 'injected_snapshot_failure'));
  } finally {
    if (!maybeResult) {
      try { maybeResult = JSON.parse(await readFile(path.join(input.evidenceRoot, 'summary.json'), 'utf8')); }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
    }
    if (maybeResult?.pid) {
      const maybeOwned = (await processSnapshot()).find((row) => row.pid === maybeResult.pid && row.pgid === maybeResult.pid);
      if (maybeOwned && (await processSnapshot()).some((row) => sameProcess(row, maybeOwned))) process.kill(maybeOwned.pid, 'SIGTERM');
    }
  }
});
