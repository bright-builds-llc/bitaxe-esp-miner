// The pinned Gate serial controller (real code, run by Bun) drives the virtual device child.
import { test, before } from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const run = promisify(execFile);
const options = Object.fromEntries(process.argv.slice(2).map(argument => {
  const match = /^--([a-z-]+)=(.*)$/u.exec(argument);
  if (!match) throw Error('test_arguments');
  return [match[1], resolve(match[2])];
}));
const fixture = resolve(dirname(fileURLToPath(import.meta.url)), 'admission.fixture.mjs');
const TRANSPORT_LOSS = new Set(['credit_closed', 'disconnected', 'liveness_lost', 'read_failed', 'write_failed', 'closed']);
let root;

before(async () => {
  root = await mkdtemp(join(tmpdir(), 'virtual-board-gate-serial-'));
  // A freshly built executable can be held at first launch; never let that hit the Hello deadline.
  await run(options.device, ['list'], { timeout: 120000 });
});

async function gate(mode, trust) {
  const work = join(root, mode);
  await mkdir(work, { mode: 0o700 });
  const { stdout } = await run('bun', [fixture, `--gate-root=${dirname(options['gate-package'])}`, `--device=${options.device}`,
    `--capability=${options.capability}`, `--trust=${trust}`, `--mode=${mode}`, `--work=${work}`], { timeout: 60000 });
  return JSON.parse(stdout);
}

test('the real Gate admits the virtual device and closes it cleanly', async () => {
  // Arrange / Act
  const result = await gate('admit', options['firmware-trust']);
  // Assert
  assert.deepEqual([result.permission, result.state], ['ready', 'baseline']);
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released], [0, true]);
});

test('the real Gate refuses a capability its trust did not sign', async () => {
  // Arrange / Act
  const result = await gate('untrusted', options['gate-trust']);
  // Assert
  assert.equal(result.permission, 'rejected');
  assert.equal(result.outcome.outcome.authenticated, false);
  assert.equal(result.outcome.outcome.frames_answered, 1, 'only discover is answered');
  assert.equal(result.device.released, true);
});

test('the real Gate reports a lost device and releases it', async () => {
  // Arrange / Act
  const result = await gate('loss', options['firmware-trust']);
  // Assert
  assert.equal(result.permission, 'ready');
  assert.ok(TRANSPORT_LOSS.has(result.category), result.category);
  assert.deepEqual([result.device.signal, result.device.released], ['SIGKILL', true]);
});
