import assert from 'node:assert/strict';
import { once } from 'node:events';
import test from 'node:test';
import { connectHold, errorFields, portCounts, startHandoffPage } from './page.mjs';
import { MAX_ATTEMPTS, createEndpointServer } from './server.mjs';

const context = { gate_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64) };
const endpoint = { schema: 'worker-telemetry-endpoint-v1', ipv4: '192.168.1.20', httpPort: 80, observedAtUs: 1, bootOrdinal: 7,
  generation: 3, controlSessionBindingSha256: 'fresh' };
const worker = attached => ({ connected: attached, getInfo: () => ({ usbVendorId: 0x303a, usbProductId: 0x1001 }) });
const adapter = { connected: true, getInfo: () => ({ usbVendorId: 0x10c4, usbProductId: 0xea60 }) };

function fakeGate(maybeObserve) {
  return {
    current: { status: 'configured', connected: false, running: false, serialOwnershipReleased: true, deviceLeaseInactive: false },
    observed: 0,
    state() { return { ...this.current }; },
    async stop() { throw Error('idle'); },
    async close() { this.current = { ...this.current, status: 'closed', connected: false }; },
    async observeStationEndpoint() { this.observed += 1; return maybeObserve ? maybeObserve() : endpoint; },
    admit() { this.current = { ...this.current, status: 'ready', connected: true, deviceLeaseInactive: true }; },
  };
}

/** A real endpoint server and one page load driven through its injected browser surface. */
async function harness({ gate = fakeGate(), ports = [worker(true), adapter], published = () => gate.state(), priorAttempts = 0 } = {}) {
  const written = new Map();
  const server = createEndpointServer({ root: '/unused', context, assets: { page: '', bundle: '', pageModule: '', trust: {} } },
    { persist: async (file, value) => { written.set(file, value); }, log: () => {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = async (path, value) => {
    const response = await fetch(`${origin}${path}`, { method: 'POST', headers: { origin, 'content-type': 'application/json' }, body: JSON.stringify(value) });
    if (!response.ok) throw Error('endpoint_handoff_rejected');
    return response.json();
  };
  for (let index = 0; index < priorAttempts; index += 1) await post('/attempt', { step: 'connect', gate: {}, ports: { granted: 1, attached: 1 }, foreground: { visible: true, focused: true } });
  const shown = [], navigation = [];
  let clicked;
  const page = startHandoffPage({
    gate, serial: { getPorts: async () => ports }, show: text => shown.push(text), visible: () => true, focused: () => true,
    publishedState: published, onConnectClick: listener => { clicked = listener; }, onPortsChanged: () => {}, every: () => {},
    after: (_milliseconds, listener) => listener(), now: () => Date.now(), get: async path => (await fetch(`${origin}${path}`)).json(), post,
    reload: () => navigation.push('reload'), leave: () => navigation.push('about:blank'),
  });
  const click = () => {
    const event = { held: false, preventDefault() {}, stopImmediatePropagation() { this.held = true; } };
    clicked(event); return event;
  };
  return { page, gate, click, written, shown, navigation, release: () => server.release() };
}

test('only an attached grant for the Worker filter counts as attached', () => {
  // Act
  const counts = portCounts([worker(true), worker(false), adapter]);

  // Assert
  assert.deepEqual(counts, { granted: 2, attached: 1 });
});

test('Connect is held while the Gate loads or the granted Worker port is not attached', () => {
  // Act / Assert
  assert.equal(connectHold({ status: 'unconfigured' }, { granted: 1, attached: 1 }), 'gate_loading');
  assert.equal(connectHold({ status: 'configured' }, { granted: 1, attached: 0 }), 'worker_port_not_attached');
  assert.equal(connectHold({ status: 'configured' }, { granted: 2, attached: 2 }), 'worker_port_not_attached');
  assert.equal(connectHold({ status: 'configured' }, { granted: 1, attached: 1 }), undefined);
  assert.equal(connectHold({ status: 'configured' }, { granted: 0, attached: 0 }), undefined);
});

test('error categories pass and free text is dropped', () => {
  // Act
  const fields = errorFields(Object.assign(Error('Worker Serial command_rejected'), { category: 'command_rejected', rejection: 'invalid_request' }));

  // Assert
  assert.deepEqual(fields, { category: 'command_rejected', rejection: 'invalid_request' });
});

test('a click while the Worker port re-enumerates never reaches the Gate', async () => {
  // Arrange
  const h = await harness({ ports: [worker(false)] });

  try {
    await h.page.tick();

    // Act
    const event = h.click();

    // Assert
    assert.equal(event.held, true);
    assert.match(h.shown.at(-1), /worker_port_not_attached/u);
  } finally { await h.release(); }
});

test('a failed Connect is recorded with its click-time ports and reloads into a fresh Gate document', async () => {
  // Arrange
  const h = await harness();
  await h.page.tick(); h.click();
  h.gate.current = { ...h.gate.current, status: 'failed', failure: 'connect_failed', serialFailureCategory: 'operation_failed', admissionFailureStage: 'permission' };

  try {
    // Act
    await h.page.tick();

    // Assert
    const attempt = h.written.get('attempt-001.json');
    assert.equal(attempt.step, 'connect');
    assert.deepEqual(attempt.gate, { status: 'failed', failure: 'connect_failed', serialFailureCategory: 'operation_failed',
      admissionFailureStage: 'permission', connected: false, serialOwnershipReleased: true });
    assert.deepEqual(attempt.ports, { granted: 1, attached: 1 });
    assert.deepEqual(h.navigation, ['reload']);
  } finally { await h.release(); }
});

test('the endpoint is read only after the Gate publishes its ready state', async () => {
  // Arrange
  let published = { status: 'configured' };
  const h = await harness({ published: () => published });
  await h.page.tick(); h.click(); h.gate.admit();

  try {
    // Act
    await h.page.tick();
    const readBeforePublication = h.gate.observed;
    published = { status: 'ready' };
    await h.page.tick();

    // Assert
    assert.equal(readBeforePublication, 0);
    assert.equal(h.gate.observed, 1);
    assert.equal(h.written.get('endpoint.private.json').bootOrdinal, 7);
    assert.equal(h.written.get('closed.json').endpoint_saved, true);
    assert.deepEqual(h.navigation, ['about:blank']);
  } finally { await h.release(); }
});

test('an endpoint miss is recorded with its rejection and the Worker is released before the reload', async () => {
  // Arrange
  const rejection = Object.assign(Error('Worker Serial command_rejected'), { category: 'command_rejected', rejection: 'invalid_request' });
  const h = await harness({ gate: fakeGate(() => { throw rejection; }) });
  await h.page.tick(); h.click(); h.gate.admit();

  try {
    // Act
    await h.page.tick();

    // Assert
    const attempt = h.written.get('attempt-001.json');
    assert.equal(attempt.step, 'observe');
    assert.deepEqual(attempt.error, { category: 'command_rejected', rejection: 'invalid_request' });
    assert.equal(h.gate.current.status, 'closed');
    assert.equal(h.written.has('closed.json'), false);
    assert.deepEqual(h.navigation, ['reload']);
  } finally { await h.release(); }
});

test('after the last attempt the page records the closed Worker and leaves', async () => {
  // Arrange
  const h = await harness({ gate: fakeGate(() => { throw Error('station_endpoint_binding'); }), priorAttempts: MAX_ATTEMPTS - 1 });
  await h.page.tick(); h.click(); h.gate.admit();

  try {
    // Act
    await h.page.tick();

    // Assert
    assert.deepEqual(h.written.get(`attempt-00${MAX_ATTEMPTS}.json`).error, { message: 'station_endpoint_binding' });
    assert.equal(h.written.get('closed.json').endpoint_saved, false);
    assert.equal(h.written.get('closed.json').failed_attempts, MAX_ATTEMPTS);
    assert.deepEqual(h.navigation, ['about:blank']);
  } finally { await h.release(); }
});
