import assert from 'node:assert/strict';
import { once } from 'node:events';
import test from 'node:test';
import { admitEndpoint, createEndpointServer, handoffConfiguration } from './server.mjs';
import { workspacePath } from './main.mjs';

const context = { gate_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64) };
const endpoint = { schema: 'worker-telemetry-endpoint-v1', ipv4: '192.168.1.20', httpPort: 80, observedAtUs: 1, bootOrdinal: 7,
  generation: 3, controlSessionBindingSha256: 'fresh' };
const NOW = 1_000_000;

function submission(overrides = {}) {
  return { nonce: 'n', endpoint, requestedAtUnixMs: NOW - 200, receivedAtUnixMs: NOW - 100, ...overrides };
}

test('a fresh private endpoint is admitted without its session binding', () => {
  // Act
  const record = admitEndpoint(submission(), 'n', NOW);

  // Assert
  assert.deepEqual(record, { schema: 'otawww-station-endpoint-v1', ipv4: '192.168.1.20', httpPort: 80, bootOrdinal: 7,
    generation: 3, observedAtUs: 1, hostReceivedAtMs: NOW });
});

test('public, stale or unbound endpoint submissions are refused', () => {
  // Act / Assert
  assert.throws(() => admitEndpoint(submission({ endpoint: { ...endpoint, ipv4: '8.8.8.8' } }), 'n', NOW), { code: 'endpoint_private_address' });
  assert.throws(() => admitEndpoint(submission({ receivedAtUnixMs: NOW - 20_000, requestedAtUnixMs: NOW - 20_001 }), 'n', NOW), { code: 'endpoint_freshness' });
  assert.throws(() => admitEndpoint(submission(), 'other', NOW), { code: 'endpoint_nonce' });
  assert.throws(() => admitEndpoint(submission({ endpoint: { ...endpoint, bootOrdinal: 0 } }), 'n', NOW), { code: 'endpoint_values' });
});

test('the Gate is configured only for the endpoint handoff of the exact pair', () => {
  // Act
  const configuration = handoffConfiguration(context, { keys: [] });

  // Assert
  assert.deepEqual(configuration, { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, trust: { keys: [] }, stationEndpointHandoff: true });
});

test('one endpoint and the closed state are recorded, then the server releases', async () => {
  // Arrange
  const written = new Map();
  let clock = NOW;
  const server = createEndpointServer({ root: '/unused', context, assets: { page: '<html>', bundle: 'js', pageModule: 'page', trust: {} } },
    { now: () => clock, persist: async (file, value) => { written.set(file, value); } });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'content-type': 'application/json' }, body: JSON.stringify(value) });

  // Act
  const { nonce } = await (await post('/endpoint-context', {})).json();
  const saved = await post('/endpoint', { nonce, endpoint, requestedAtUnixMs: clock - 200, receivedAtUnixMs: clock - 100 });
  const replay = await post('/endpoint', { nonce, endpoint, requestedAtUnixMs: clock - 200, receivedAtUnixMs: clock - 100 });
  clock += 1_000;
  const released = once(server, 'close');
  const leased = await post('/closed', { state: { status: 'closed', connected: false, running: false, serialOwnershipReleased: true, deviceLeaseInactive: false } });
  const closed = await post('/closed', { state: { status: 'closed', connected: false, running: false, serialOwnershipReleased: true, deviceLeaseInactive: true } });
  await released;

  // Assert
  assert.equal(saved.status, 200);
  assert.equal(replay.status, 400);
  assert.equal(leased.status, 400);
  assert.equal(closed.status, 200);
  assert.equal(written.get('closed.json').worker_lease_inactive, true);
  assert.equal(written.get('endpoint.private.json').bootOrdinal, 7);
  assert.equal(written.get('closed.json').endpoint_saved, true);
});

test('a cross-origin post is refused', async () => {
  // Arrange
  const server = createEndpointServer({ root: '/unused', context, assets: { page: '', bundle: '', pageModule: '', trust: {} } },
    { persist: async () => {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;

  try {
    // Act
    const response = await fetch(`${origin}/endpoint-context`, { method: 'POST', headers: { origin: 'http://evil.example' }, body: '{}' });

    // Assert
    assert.equal(response.status, 400);
  } finally { await server.release(); }
});

test('operator paths resolve against the workspace, not the runfiles directory', () => {
  // Act
  const gate = workspacePath('/work/bitaxe-esp-miner', '../bitaxe-turnstile-system');

  // Assert
  assert.equal(gate, '/work/bitaxe-turnstile-system');
});
