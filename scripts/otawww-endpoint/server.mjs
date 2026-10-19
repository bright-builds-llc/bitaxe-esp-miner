import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';

export const ENDPOINT_FILE = 'endpoint.private.json';
export const CLOSED_FILE = 'closed.json';
/** Fresh handoffs succeeded by their second or third Connect; five bounds one server's retries. */
export const MAX_ATTEMPTS = 5;
export const ATTEMPT_FILE = /^attempt-\d{3}\.json$/u;
const IPV4 = /^(?:\d{1,3}\.){3}\d{1,3}$/u;
const CATEGORY = /^[a-z][a-z0-9_]{0,63}$/u;
const STEPS = ['connect', 'endpoint_context', 'observe', 'submit'];
const GATE_CATEGORIES = ['status', 'failure', 'serialFailureCategory', 'admissionFailureStage'];
const GATE_FLAGS = ['connected', 'serialOwnershipReleased'];
const ERROR_CATEGORIES = ['category', 'rejection', 'message'];

/** The Gate configuration for a one-use station endpoint handoff of the exact installed pair. */
export function handoffConfiguration(context, trust) {
  return { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, trust, stationEndpointHandoff: true };
}

/** Accepts only a fresh, possession-bound endpoint for a private LAN address. */
export function admitEndpoint(input, expectedNonce, now) {
  check(input && typeof input === 'object' && Object.keys(input).sort().join() === 'endpoint,nonce,receivedAtUnixMs,requestedAtUnixMs', 'endpoint_shape');
  check(typeof expectedNonce === 'string' && input.nonce === expectedNonce, 'endpoint_nonce');
  const { endpoint, requestedAtUnixMs, receivedAtUnixMs } = input;
  check([requestedAtUnixMs, receivedAtUnixMs].every(Number.isSafeInteger) && requestedAtUnixMs <= receivedAtUnixMs &&
    receivedAtUnixMs - requestedAtUnixMs <= 5000 && now >= receivedAtUnixMs && now - receivedAtUnixMs <= 15000, 'endpoint_freshness');
  check(endpoint && endpoint.schema === 'worker-telemetry-endpoint-v1' && typeof endpoint.ipv4 === 'string' && IPV4.test(endpoint.ipv4), 'endpoint_schema');
  const [a, b, ...rest] = endpoint.ipv4.split('.').map(Number);
  check([a, b, ...rest].every(octet => octet <= 255) && (a === 10 || (a === 172 && b >= 16 && b <= 31) || (a === 192 && b === 168)), 'endpoint_private_address');
  check(Number.isSafeInteger(endpoint.httpPort) && endpoint.httpPort > 0 && endpoint.httpPort <= 65535 &&
    Number.isSafeInteger(endpoint.bootOrdinal) && endpoint.bootOrdinal > 0 && Number.isSafeInteger(endpoint.generation) && endpoint.generation > 0 &&
    Number.isSafeInteger(endpoint.observedAtUs) && endpoint.observedAtUs >= 0 && typeof endpoint.controlSessionBindingSha256 === 'string', 'endpoint_values');
  return { schema: 'otawww-station-endpoint-v1', ipv4: endpoint.ipv4, httpPort: endpoint.httpPort, bootOrdinal: endpoint.bootOrdinal,
    generation: endpoint.generation, observedAtUs: endpoint.observedAtUs, hostReceivedAtMs: now };
}

function exactKeys(value, allowed) {
  return value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).every(key => allowed.includes(key));
}
function categories(value, keys, code) {
  check(exactKeys(value, keys) && Object.values(value).every(item => typeof item === 'string' && CATEGORY.test(item)), code);
  return { ...value };
}

/**
 * One failed attempt as closed vocabulary only: the step, the Gate's public failure categories, the thrown error's
 * categories, the granted and attached Worker port counts and foreground flags. Free text and values never pass.
 */
export function attemptRecord(input, ordinal, maybeRejection, now) {
  check(exactKeys(input, ['step', 'gate', 'error', 'ports', 'foreground']) && Object.keys(input).length >= 4, 'attempt_shape');
  const { step, gate, error = {}, ports, foreground } = input;
  check(STEPS.includes(step), 'attempt_step');
  check(exactKeys(gate, [...GATE_CATEGORIES, ...GATE_FLAGS]) && GATE_FLAGS.every(key => gate[key] === undefined || typeof gate[key] === 'boolean'), 'attempt_gate');
  const gateCategories = categories(Object.fromEntries(GATE_CATEGORIES.filter(key => gate[key] !== undefined).map(key => [key, gate[key]])), GATE_CATEGORIES, 'attempt_gate');
  const count = value => Number.isSafeInteger(value) && value >= 0 && value <= 16;
  check(exactKeys(ports, ['granted', 'attached']) && count(ports.granted) && count(ports.attached) && ports.attached <= ports.granted, 'attempt_ports');
  check(exactKeys(foreground, ['visible', 'focused']) && typeof foreground.visible === 'boolean' && typeof foreground.focused === 'boolean', 'attempt_foreground');
  const flags = Object.fromEntries(GATE_FLAGS.filter(key => gate[key] !== undefined).map(key => [key, gate[key]]));
  return { schema: 'otawww-endpoint-attempt-v1', ordinal, step, gate: { ...gateCategories, ...flags }, error: categories(error, ERROR_CATEGORIES, 'attempt_error'),
    ports: { granted: ports.granted, attached: ports.attached }, foreground: { visible: foreground.visible, focused: foreground.focused },
    ...(maybeRejection ? { endpoint_rejection: maybeRejection } : {}), recordedAtUnixMs: now };
}

/**
 * Serves the pinned Gate in endpoint-handoff mode and records one endpoint plus the closed Worker state. A failed
 * Connect or endpoint read is recorded and the page reloads into a fresh Gate document: each activation gets a fresh
 * challenge and each attempt a fresh nonce, as a fresh handoff would, up to MAX_ATTEMPTS.
 */
export function createEndpointServer({ root, context, assets }, operations = {}) {
  const now = operations.now ?? Date.now;
  const persist = operations.persist ?? ((file, value) => writeNew(resolve(root, file), value));
  const log = operations.log ?? (line => process.stdout.write(`${line}\n`));
  let pending, maybeRejection, attempts = 0, saved = false, closed = false, queue = Promise.resolve();
  const open = () => !saved && !closed && attempts < MAX_ATTEMPTS;
  const server = createServer((request, response) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'endpoint_host'); const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/endpoint-page.mjs"></script>`), 'text/html');
      if (path === '/context') return send(response, 200, handoffConfiguration(context, assets.trust));
      if (path === '/attempts') return send(response, 200, { failed: attempts, max: MAX_ATTEMPTS });
      if (path === `/${BUNDLE}`) return send(response, 200, assets.bundle, 'text/javascript');
      if (path === '/endpoint-page.mjs') return send(response, 200, assets.pageModule, 'text/javascript');
      return send(response, 404, { error: 'endpoint_route' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin || (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'endpoint_origin');
    const input = await body(request);
    if (path === '/activate') {
      return send(response, 200, { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 });
    }
    if (path === '/endpoint-context') {
      check(open(), 'endpoint_context_consumed'); pending = nonce(); maybeRejection = undefined; return send(response, 200, { nonce: pending });
    }
    if (path === '/endpoint') {
      check(open(), 'endpoint_consumed');
      let record;
      try { record = admitEndpoint(input, pending, now()); } catch (error) {
        maybeRejection = typeof error.code === 'string' && CATEGORY.test(error.code) ? error.code : 'endpoint_rejected'; throw error;
      }
      pending = undefined; await persist(ENDPOINT_FILE, record); saved = true; return send(response, 200, { endpoint_saved: true });
    }
    if (path === '/attempt') {
      check(open(), 'endpoint_attempts_exhausted');
      const record = attemptRecord(input, attempts + 1, maybeRejection, now());
      await persist(`attempt-${String(record.ordinal).padStart(3, '0')}.json`, record);
      attempts = record.ordinal; pending = undefined; maybeRejection = undefined;
      log(`endpoint_attempt_failed=${attempts} max=${MAX_ATTEMPTS} step=${record.step} stage=${record.gate.admissionFailureStage ?? 'none'} ` +
        `serial=${record.gate.serialFailureCategory ?? 'none'} rejection=${record.error.rejection ?? record.endpoint_rejection ?? 'none'}`);
      return send(response, 200, { retry: attempts < MAX_ATTEMPTS, failed: attempts, max: MAX_ATTEMPTS });
    }
    check(path === '/closed' && !closed, 'endpoint_route');
    const state = input?.state;
    check(state && state.status === 'closed' && state.connected === false && state.running === false && state.serialOwnershipReleased === true &&
      state.deviceLeaseInactive === true, 'endpoint_closed_state');
    // An inactive lease proves no Gate mining. Local mining (ADR-0038) stays off while the link is
    // active, and a handoff runs on a paused device.
    await persist(CLOSED_FILE, { schema: 'otawww-endpoint-closed-v1', endpoint_saved: saved, worker_lease_inactive: true, failed_attempts: attempts,
      closedAtUnixMs: now() }); closed = true;
    send(response, 200, { closed_recorded: true });
    setImmediate(() => server.release());
  }).catch(() => { if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'endpoint_rejected' }); else response.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const done = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await done; };
  return server;
}
