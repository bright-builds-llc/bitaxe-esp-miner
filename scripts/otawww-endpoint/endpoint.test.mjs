import assert from 'node:assert/strict';
import { once } from 'node:events';
import test from 'node:test';
import { MAX_ATTEMPTS, admitEndpoint, attemptRecord, createEndpointServer, handoffConfiguration } from './server.mjs';
import { handoffResult, workspacePath } from './main.mjs';

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
  assert.equal(written.get('closed.json').failed_attempts, 0);
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

const failedAttempt = { step: 'observe', gate: { status: 'ready', connected: true, serialOwnershipReleased: false },
  error: { category: 'command_rejected', rejection: 'invalid_request' }, ports: { granted: 1, attached: 1 }, foreground: { visible: true, focused: true } };

async function startServer() {
  const written = new Map(), logs = [];
  const server = createEndpointServer({ root: '/unused', context, assets: { page: '', bundle: '', pageModule: '', trust: {} } },
    { now: () => NOW, persist: async (file, value) => { written.set(file, value); }, log: line => logs.push(line) });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'content-type': 'application/json' }, body: JSON.stringify(value) });
  return { server, origin, post, written, logs };
}

test('each activation issues a fresh challenge, as a fresh handoff would', async () => {
  // Arrange
  const { server, post } = await startServer();

  try {
    // Act
    const first = await (await post('/activate', {})).json();
    const second = await (await post('/activate', {})).json();

    // Assert
    assert.notEqual(first.challengeId, second.challengeId);
  } finally { await server.release(); }
});

test('a failed attempt is recorded with its rejection and the next attempt gets a fresh nonce', async () => {
  // Arrange
  const { server, post, written, logs } = await startServer();

  try {
    const { nonce: stale } = await (await post('/endpoint-context', {})).json();
    const refused = await post('/endpoint', { nonce: stale, endpoint, requestedAtUnixMs: NOW - 9_000, receivedAtUnixMs: NOW - 100 });

    // Act
    const attempt = await (await post('/attempt', failedAttempt)).json();
    const { nonce: fresh } = await (await post('/endpoint-context', {})).json();
    const replayed = await post('/endpoint', { nonce: stale, endpoint, requestedAtUnixMs: NOW - 200, receivedAtUnixMs: NOW - 100 });
    const saved = await post('/endpoint', { nonce: fresh, endpoint, requestedAtUnixMs: NOW - 200, receivedAtUnixMs: NOW - 100 });

    // Assert
    assert.equal(refused.status, 400);
    assert.deepEqual(attempt, { retry: true, failed: 1, max: MAX_ATTEMPTS });
    assert.equal(written.get('attempt-001.json').endpoint_rejection, 'endpoint_freshness');
    assert.deepEqual(written.get('attempt-001.json').error, { category: 'command_rejected', rejection: 'invalid_request' });
    assert.match(logs[0], /^endpoint_attempt_failed=1 max=5 step=observe stage=none serial=none rejection=invalid_request$/u);
    assert.notEqual(fresh, stale);
    assert.equal(replayed.status, 400);
    assert.equal(saved.status, 200);
  } finally { await server.release(); }
});

test('attempts stop at the bound and no further endpoint context is issued', async () => {
  // Arrange
  const { server, post, written } = await startServer();

  try {
    // Act
    const replies = [];
    for (let index = 0; index < MAX_ATTEMPTS; index += 1) replies.push(await (await post('/attempt', failedAttempt)).json());
    const extra = await post('/attempt', failedAttempt);
    const context = await post('/endpoint-context', {});

    // Assert
    assert.deepEqual(replies.map(reply => reply.retry), [true, true, true, true, false]);
    assert.equal(written.has(`attempt-00${MAX_ATTEMPTS}.json`), true);
    assert.equal(extra.status, 400);
    assert.equal(context.status, 400);
  } finally { await server.release(); }
});

test('an attempt record keeps closed categories and counts only', () => {
  // Act
  const record = attemptRecord({ ...failedAttempt, step: 'connect', gate: { status: 'failed', failure: 'connect_failed',
    serialFailureCategory: 'operation_failed', admissionFailureStage: 'permission', connected: false } }, 2, undefined, NOW);

  // Assert
  assert.deepEqual(record, { schema: 'otawww-endpoint-attempt-v1', ordinal: 2, step: 'connect', gate: { status: 'failed', failure: 'connect_failed',
    serialFailureCategory: 'operation_failed', admissionFailureStage: 'permission', connected: false },
  error: { category: 'command_rejected', rejection: 'invalid_request' }, ports: { granted: 1, attached: 1 }, foreground: { visible: true, focused: true },
  recordedAtUnixMs: NOW });
});

test('an attempt record refuses free text, values and unknown fields', () => {
  // Act / Assert
  assert.throws(() => attemptRecord({ ...failedAttempt, gate: { status: '192.168.1.20' } }, 1, undefined, NOW), { code: 'attempt_gate' });
  assert.throws(() => attemptRecord({ ...failedAttempt, error: { message: 'Worker Serial timeout' } }, 1, undefined, NOW), { code: 'attempt_error' });
  assert.throws(() => attemptRecord({ ...failedAttempt, gate: { expectedAppElfSha256: 'c' } }, 1, undefined, NOW), { code: 'attempt_gate' });
  assert.throws(() => attemptRecord({ ...failedAttempt, ports: { granted: 1, attached: 2 } }, 1, undefined, NOW), { code: 'attempt_ports' });
  assert.throws(() => attemptRecord({ ...failedAttempt, step: 'other' }, 1, undefined, NOW), { code: 'attempt_step' });
  assert.throws(() => attemptRecord({ ...failedAttempt, extra: 1 }, 1, undefined, NOW), { code: 'attempt_shape' });
});

test('finish succeeds only for a closed page that saved an endpoint', () => {
  // Act
  const saved = handoffResult({ endpoint_saved: true }, 1);
  const unsaved = handoffResult({ endpoint_saved: false }, 5);
  const open = handoffResult(undefined, 2);

  // Assert
  assert.deepEqual(saved, { result: { schema: 'otawww-endpoint-result-v1', endpoint_recorded: true, page_closed: true, host_released: true,
    failed_attempts: 1 }, maybeFailure: undefined });
  assert.equal(unsaved.maybeFailure, 'endpoint_not_recorded');
  assert.equal(open.maybeFailure, 'endpoint_page_not_closed');
  assert.equal(open.result.page_closed, false);
});
