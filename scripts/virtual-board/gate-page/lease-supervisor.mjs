// A loopback lease supervisor for the virtual device. It is test-only: it serves the pinned
// page with the production supervisor client, judges every published state with the
// production `validateState`, and signs one qualification attempt per control session with
// the production `signAttempt` over a throwaway authority. It has no campaign lineage,
// preflight, pool input or cooling and budget reviews, which the virtual device lacks.
import { createServer } from 'node:http';
import { readdir, readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { BUNDLE, canonicalBase64, digest, exactObject, nonce, PAGE, QualificationError, requireCondition, writeNew }
  from '../../fixed-usb-qualification/contract.mjs';
import { body, send } from '../../fixed-usb-qualification/http.mjs';
import { validateState } from '../../fixed-usb-qualification/judge.mjs';
import { signVirtualAttempt } from '../lease-authority.mjs';

const CLIENT = resolve(dirname(fileURLToPath(import.meta.url)), '../../fixed-usb-qualification/client.mjs');
export const LEASE_RECORD = /^lease-state-[0-9]{4}\.json$/u;

/** `context` names the identities `validateState` checks; `trust` is what the page verifies with. */
export async function createLeaseSupervisor({ privateRoot, context, gateRoot, trust, sign }) {
  const root = resolve(privateRoot);
  const assets = new Map([
    ['/', { path: resolve(gateRoot, PAGE), type: 'text/html' }],
    [`/${BUNDLE}`, { path: resolve(gateRoot, BUNDLE), type: 'text/javascript' }],
    ['/client.mjs', { path: CLIENT, type: 'text/javascript' }],
  ]);
  let maybeChallengeId, maybeWindow, recordQueue = Promise.resolve();
  const server = createServer((request, response) => {
    handle(request, response).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : 'local_operation_failed' });
    });
  });
  server.requestTimeout = 5000;
  server.headersTimeout = 10000;
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, 'host_rejected');
    const url = new URL(request.url, origin);
    if (request.method === 'POST') requireCondition(request.headers.origin === origin, 'origin_rejected');
    if (request.method === 'GET' && url.pathname === '/context') {
      return send(response, 200, { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
        expectedAppElfSha256: context.app_elf_sha256, trust });
    }
    if (request.method === 'POST' && url.pathname === '/activate') {
      exactObject(await body(request), []);
      maybeChallengeId = `challenge_${nonce()}`;
      maybeWindow = undefined;
      return send(response, 200, { challengeId: maybeChallengeId, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 });
    }
    if (request.method === 'POST' && url.pathname === '/authorization-context') {
      const input = await body(request);
      exactObject(input, ['controlSessionBindingSha256']);
      requireCondition(maybeChallengeId !== undefined && canonicalBase64(input.controlSessionBindingSha256, 32), 'authorization_context');
      maybeWindow = await signVirtualAttempt({ challengeId: maybeChallengeId, binding: input.controlSessionBindingSha256, sign });
      return send(response, 200, { ready: true });
    }
    if (request.method === 'GET' && url.pathname === '/window-artifacts') {
      requireCondition(maybeWindow !== undefined, 'window_unavailable');
      const window = maybeWindow;
      maybeWindow = undefined;
      return send(response, 200, window);
    }
    if (request.method === 'POST' && url.pathname === '/record') {
      const input = await body(request);
      exactObject(input, ['state']);
      const state = validateState(input.state, context);
      return send(response, 200, await record('lease-state', state));
    }
    if (request.method === 'POST' && url.pathname === '/fault') {
      return send(response, 200, await record('lease-fault', await body(request)));
    }
    const asset = request.method === 'GET' ? assets.get(url.pathname) : undefined;
    if (asset) {
      let bytes = await readFile(asset.path);
      if (url.pathname === '/') bytes = Buffer.from(`${bytes.toString('utf8')}\n<script type="module" src="/client.mjs"></script>`);
      return send(response, 200, bytes, asset.type);
    }
    send(response, 404, { error: 'route_unavailable' });
  }
  /** Records in arrival order, each in a new private file. */
  function record(prefix, value) {
    const operation = recordQueue.then(async () => {
      const count = (await readdir(root)).filter((file) => file.startsWith(`${prefix}-`)).length;
      requireCondition(count < 512, 'sample_bound');
      const sequence = count + 1;
      await writeNew(resolve(root, `${prefix}-${String(sequence).padStart(4, '0')}.json`),
        { schema: 'virtual-board-lease-record-v1', context_sha256: digest(JSON.stringify(context)), sequence, value });
      return { recorded: true, sequence };
    });
    recordQueue = operation.then(() => undefined, () => undefined);
    return operation;
  }
  return server;
}
