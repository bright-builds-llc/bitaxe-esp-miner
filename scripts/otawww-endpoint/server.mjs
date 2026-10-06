import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';

export const ENDPOINT_FILE = 'endpoint.private.json';
export const CLOSED_FILE = 'closed.json';
const IPV4 = /^(?:\d{1,3}\.){3}\d{1,3}$/u;

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

/** Serves the pinned Gate in endpoint-handoff mode and records one endpoint plus the closed Worker state. */
export function createEndpointServer({ root, context, assets }, operations = {}) {
  const now = operations.now ?? Date.now;
  const persist = operations.persist ?? ((file, value) => writeNew(resolve(root, file), value));
  const challenge = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
  let pending, saved = false, closed = false, queue = Promise.resolve();
  const server = createServer((request, response) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'endpoint_host'); const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/endpoint-page.mjs"></script>`), 'text/html');
      if (path === '/context') return send(response, 200, handoffConfiguration(context, assets.trust));
      if (path === `/${BUNDLE}`) return send(response, 200, assets.bundle, 'text/javascript');
      if (path === '/endpoint-page.mjs') return send(response, 200, assets.pageModule, 'text/javascript');
      return send(response, 404, { error: 'endpoint_route' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin || (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'endpoint_origin');
    const input = await body(request);
    if (path === '/activate') return send(response, 200, challenge);
    if (path === '/endpoint-context') {
      check(!pending && !saved, 'endpoint_context_consumed'); pending = nonce(); return send(response, 200, { nonce: pending });
    }
    if (path === '/endpoint') {
      check(!saved, 'endpoint_consumed'); const record = admitEndpoint(input, pending, now()); pending = undefined;
      await persist(ENDPOINT_FILE, record); saved = true; return send(response, 200, { endpoint_saved: true });
    }
    check(path === '/closed' && !closed, 'endpoint_route');
    const state = input?.state;
    check(state && state.status === 'closed' && state.connected === false && state.running === false && state.serialOwnershipReleased === true &&
      state.deviceLeaseInactive === true, 'endpoint_closed_state');
    // Mining needs an active Worker lease, so an inactive lease proves the handed-off boot is not mining.
    await persist(CLOSED_FILE, { schema: 'otawww-endpoint-closed-v1', endpoint_saved: saved, worker_lease_inactive: true, closedAtUnixMs: now() }); closed = true;
    send(response, 200, { closed_recorded: true });
    setImmediate(() => server.release());
  }).catch(() => { if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'endpoint_rejected' }); else response.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const done = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await done; };
  return server;
}
