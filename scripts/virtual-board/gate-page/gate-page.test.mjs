// The pinned Gate acceptance page, served by the real no-mining supervisor, drives the virtual device.
import { test, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { once } from 'node:events';
import { mkdir, mkdtemp, readdir, readFile, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { createNoMiningSupervisor } from '../../fixed-usb-qualification/no-mining-server.mjs';
import { authoritySigner, throwawayAuthority } from '../lease-authority.mjs';
import { buildFirmwareTree, buildGateTree, DEVICE_IDENTITY, pinnedGateCommit, virtualNoMiningContext } from './assets.mjs';
import { createLeaseSupervisor, LEASE_RECORD } from './lease-supervisor.mjs';

const run = promisify(execFile);
const options = Object.fromEntries(process.argv.slice(2).map(argument => {
  const match = /^--([a-z-]+)=(.*)$/u.exec(argument);
  if (!match) throw Error('test_arguments');
  return [match[1], resolve(match[2])];
}));
const fixture = resolve(dirname(fileURLToPath(import.meta.url)), 'page.fixture.mjs');
const TRANSPORT_LOSS = new Set(['credit_closed', 'disconnected', 'liveness_lost', 'read_failed', 'write_failed', 'closed']);
let root, gateRoot, commit, authority;
const servers = [];
const gateSource = () => dirname(options['gate-package']);

before(async () => {
  root = await mkdtemp(join(tmpdir(), 'virtual-board-gate-page-'));
  commit = await pinnedGateCommit(options['module-bazel']);
  gateRoot = join(root, 'gate');
  await buildGateTree({ gateSource: gateSource(), pagesSource: dirname(options.page), commit, gateRoot });
  // A freshly built executable can be held at first launch; never let that hit the Hello deadline.
  await run(options.device, ['list'], { timeout: 120000 });
  authority = await throwawayAuthority({ gateRoot: gateSource(), root, firmwareTrustPath: options['firmware-trust'] });
});
after(async () => {
  for (const server of servers) { server.closeAllConnections(); await new Promise(done => server.close(done)); }
});

async function listen(server) {
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  servers.push(server);
  return `http://127.0.0.1:${server.address().port}`;
}

/** A real supervisor on an ephemeral loopback port publishing `trust`. */
async function supervise(name, trust) {
  const firmwareRoot = join(root, `firmware-${name}`), privateRoot = join(root, `records-${name}`);
  await buildFirmwareTree({ trust, firmwareRoot });
  await mkdir(privateRoot, { mode: 0o700 });
  const context = await virtualNoMiningContext({ gateRoot, firmwareRoot, commit });
  // The virtual context has no git checkout or physical package to re-verify.
  const server = await createNoMiningSupervisor({ privateRoot, context }, { verifyFrozen: async () => {} });
  return { origin: await listen(server), privateRoot, deviceTrust: options['firmware-trust'],
    records: () => readRecords(privateRoot, /^no-mining-state-[0-9]{4}\.json$/u, record => record.state) };
}

/** The lease supervisor publishing the virtual deployment trust its throwaway authority signs under. */
async function superviseLeases(name) {
  const privateRoot = join(root, `records-${name}`);
  await mkdir(privateRoot, { mode: 0o700 });
  const context = { schema: 'virtual-board-lease-context-v1', gate_commit: commit, ...DEVICE_IDENTITY };
  const trust = JSON.parse(await readFile(authority.trustPath, 'utf8'));
  const server = await createLeaseSupervisor({ privateRoot, context, gateRoot, trust,
    sign: authoritySigner({ gateRoot: gateSource(), directory: authority.directory }) });
  return { origin: await listen(server), privateRoot, deviceTrust: authority.trustPath,
    records: () => readRecords(privateRoot, LEASE_RECORD, record => record.value) };
}

async function readRecords(privateRoot, pattern, select) {
  const files = (await readdir(privateRoot)).filter(file => pattern.test(file)).sort();
  return Promise.all(files.map(async file => select(JSON.parse(await readFile(join(privateRoot, file), 'utf8')))));
}
const records = supervisor => supervisor.records();

async function page(mode, trustOrSupervisor) {
  const supervisor = typeof trustOrSupervisor === 'string' ? await supervise(mode, trustOrSupervisor) : trustOrSupervisor;
  const work = join(root, `page-${mode}`);
  await mkdir(work, { mode: 0o700 });
  const { stdout } = await run('bun', [fixture, `--origin=${supervisor.origin}`, `--device=${options.device}`,
    `--capability=${options.capability}`, `--device-trust=${supervisor.deviceTrust}`, `--mode=${mode}`, `--work=${work}`], { timeout: 120000 });
  return { result: JSON.parse(stdout), recorded: await records(supervisor), supervisor, work };
}

test('the served page admits the virtual device, reports to the supervisor and closes cleanly', async () => {
  // Arrange / Act
  const { result, recorded } = await page('admit', options['firmware-trust']);
  // Assert
  assert.deepEqual(result.final, { status: 'closed', connected: false, serialOwnershipReleased: true, deviceLeaseInactive: true,
    failure: null, admissionFailureStage: null, serialFailureCategory: null });
  assert.ok(recorded.some(state => state.status === 'ready' && state.connected), 'a connected ready state was recorded');
  assert.equal(recorded.at(-1).status, 'closed');
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released, result.lock_held], [0, true, false]);
});

test('the served page refuses a capability the published trust did not sign', async () => {
  // Arrange / Act
  const { result, recorded } = await page('untrusted', options['gate-trust']);
  // Assert
  assert.equal(result.final.failure, 'connect_failed');
  assert.equal(result.final.admissionFailureStage, 'capability');
  assert.ok(recorded.some(state => state.failure === 'connect_failed'), 'the failure was recorded');
  assert.equal(result.outcome.outcome.authenticated, false);
  assert.deepEqual([result.device.released, result.lock_held], [true, false]);
});

test('the served page reports a lost device and releases it', async () => {
  // Arrange / Act
  const { result, recorded } = await page('loss', options['firmware-trust']);
  // Assert
  assert.equal(result.final.connected, false);
  assert.ok(recorded.some(state => state.status === 'disconnected' || TRANSPORT_LOSS.has(state.serialFailureCategory)),
    JSON.stringify(recorded.map(state => [state.status, state.serialFailureCategory])));
  assert.deepEqual([result.device.signal, result.device.released, result.lock_held], ['SIGKILL', true, false]);
});

test('the supervisor refuses a record from another origin', async () => {
  // Arrange
  const supervisor = await supervise('origin', options['firmware-trust']);
  // Act
  const response = await fetch(`${supervisor.origin}/record`, { method: 'POST',
    headers: { origin: 'http://127.0.0.1:1', 'content-type': 'application/json' }, body: '{"state":{}}' });
  // Assert
  assert.equal(response.status, 400);
  assert.deepEqual(await records(supervisor), []);
});

test('page results are category-only and every written file is private', async () => {
  // Arrange / Act
  const { result, supervisor, work } = await page('privacy', options['firmware-trust']);
  // Assert
  const text = JSON.stringify(result);
  for (const secret of [supervisor.origin, root, '127.0.0.1']) assert.ok(!text.includes(secret), secret);
  const check = async directory => {
    assert.equal((await stat(directory)).mode & 0o777, 0o700, directory);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) await check(path);
      else assert.equal((await stat(path)).mode & 0o777, 0o600, entry.name);
    }
  };
  for (const directory of [work, supervisor.privateRoot]) await check(directory);
});

test('the served page starts, renews and stops a signed lease through the lease supervisor', async () => {
  // Arrange
  const supervisor = await superviseLeases('lease');
  // Act
  const { result, recorded } = await page('lease', supervisor);
  // Assert
  const statuses = recorded.map(state => state.status);
  for (const status of ['window_loaded', 'running', 'stopping', 'baseline_confirmed', 'closed']) assert.ok(statuses.includes(status), status);
  assert.ok(recorded.some(state => state.running && state.qualification?.owner_resources?.phase === 'active'), 'running evidence recorded');
  assert.ok(result.renewals >= 1, JSON.stringify(result));
  assert.equal(result.final.failure, null);
  const outcome = result.outcome.outcome;
  assert.ok(outcome.leases_started === 1 && outcome.renewals >= 1 && outcome.restorations === 1, JSON.stringify(outcome));
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released, result.lock_held], [0, true, false]);
});

test('closing the served page with an active lease restores the virtual device', async () => {
  // Arrange
  const supervisor = await superviseLeases('lease-close');
  // Act
  const { result, recorded } = await page('lease-close', supervisor);
  // Assert
  assert.ok(recorded.some(state => state.status === 'running'), 'the lease ran');
  assert.equal(result.final.status, 'closed');
  assert.equal(recorded.at(-1).status, 'closed');
  const outcome = result.outcome.outcome;
  assert.deepEqual([outcome.leases_started, outcome.restorations, outcome.lease_pending], [1, 1, false]);
  assert.equal(result.outcome.clean, true, JSON.stringify(result.outcome));
  assert.deepEqual([result.device.code, result.device.released, result.lock_held], [0, true, false]);
});

test('the lease supervisor delivers a signed window once', async () => {
  // Arrange
  const supervisor = await superviseLeases('one-shot');
  const post = (path, value) => fetch(`${supervisor.origin}${path}`, { method: 'POST',
    headers: { origin: supervisor.origin, 'content-type': 'application/json' }, body: JSON.stringify(value) });
  await post('/activate', {});
  await post('/authorization-context', { controlSessionBindingSha256: 'A'.repeat(43) });
  // Act
  const first = await fetch(`${supervisor.origin}/window-artifacts`);
  const second = await fetch(`${supervisor.origin}/window-artifacts`);
  // Assert
  assert.equal(first.status, 200);
  assert.ok((await first.json()).grant.qualificationAttempt);
  assert.equal(second.status, 400);
});
