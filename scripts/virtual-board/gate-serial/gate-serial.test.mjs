// The pinned Gate serial controller (real code, run by Bun) drives the virtual device child.
import { test, before } from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { throwawayAuthority } from '../lease-authority.mjs';

const run = promisify(execFile);
const options = Object.fromEntries(process.argv.slice(2).map(argument => {
  const match = /^--([a-z-]+)=(.*)$/u.exec(argument);
  if (!match) throw Error('test_arguments');
  return [match[1], resolve(match[2])];
}));
const fixture = resolve(dirname(fileURLToPath(import.meta.url)), 'admission.fixture.mjs');
const TRANSPORT_LOSS = new Set(['credit_closed', 'disconnected', 'liveness_lost', 'read_failed', 'write_failed', 'closed']);
let root, authority;
const gateRoot = () => dirname(options['gate-package']);

before(async () => {
  root = await mkdtemp(join(tmpdir(), 'virtual-board-gate-serial-'));
  // A freshly built executable can be held at first launch; never let that hit the Hello deadline.
  await run(options.device, ['list'], { timeout: 120000 });
  authority = await throwawayAuthority({ gateRoot: gateRoot(), root, firmwareTrustPath: options['firmware-trust'] });
});

/** The controller verifies the capability with `trust`; the device verifies leases with `deviceTrust`. */
async function gate(mode, trust, deviceTrust = options['firmware-trust']) {
  const work = join(root, mode);
  await mkdir(work, { mode: 0o700 });
  const { stdout } = await run('bun', [fixture, `--gate-root=${gateRoot()}`, `--device=${options.device}`,
    `--capability=${options.capability}`, `--trust=${trust}`, `--device-trust=${deviceTrust}`, `--authority=${authority.directory}`,
    `--mode=${mode}`, `--work=${work}`], { timeout: 90000 });
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

test('the real Gate starts, renews and restores a signed lease on the virtual device', async () => {
  // Arrange / Act
  const result = await gate('lease', options['firmware-trust'], authority.trustPath);
  // Assert
  assert.equal(result.category, undefined);
  assert.deepEqual([result.started, result.renewed, result.restored], ['mining', 'mining', 'baseline']);
  const outcome = result.outcome.outcome;
  assert.deepEqual([outcome.leases_started, outcome.renewals, outcome.restorations], [1, 1, 1]);
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released], [0, true]);
});

test('closing the real Gate with an active lease restores the virtual device', async () => {
  // Arrange / Act
  const result = await gate('lease-close', options['firmware-trust'], authority.trustPath);
  // Assert
  assert.equal(result.started, 'mining');
  const outcome = result.outcome.outcome;
  assert.deepEqual([outcome.leases_started, outcome.restorations, outcome.lease_pending], [1, 1, false]);
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released], [0, true]);
});

test('the virtual device refuses a lease its deployment trust did not authorize', async () => {
  // Arrange / Act
  const result = await gate('lease-untrusted', options['firmware-trust']);
  // Assert
  assert.equal(result.started, undefined);
  assert.equal(result.outcome.outcome.leases_started, 0);
  assert.equal(result.outcome.outcome.lease_pending, false);
  assert.equal(result.device.released, true);
});
