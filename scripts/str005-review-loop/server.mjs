import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { CATEGORIES, LIMITS, OPERATIONS } from './loop.mjs';

/** Validates one closed row from the page; device payloads never reach the owner. */
export function validateRow(value, iterations = LIMITS.iterations, batches = LIMITS.batches) {
  check(value && typeof value === 'object' && ['progress', 'complete', 'failure'].includes(value.kind), 'review_loop_row');
  object(value, value.kind === 'failure' ? ['batch', 'kind', 'completed', 'iteration', 'operation', 'category'] : ['batch', 'kind', 'completed']);
  check(Number.isSafeInteger(value.batch) && value.batch >= 1 && value.batch <= batches, 'review_loop_row');
  check(Number.isSafeInteger(value.completed) && value.completed >= 0 && value.completed <= iterations, 'review_loop_row');
  if (value.kind === 'complete') check(value.completed === iterations, 'review_loop_row');
  if (value.kind === 'failure') check(value.iteration === value.completed + 1 && OPERATIONS.includes(value.operation) &&
    CATEGORIES.includes(value.category), 'review_loop_row');
  return value;
}

/** Batch admission: strictly sequential, one at a time, and none after a failure or the last batch. */
export function createBatches(batches = LIMITS.batches) {
  let next = 1, active = null, finished = false;
  return {
    begin(batch) { check(!finished && active === null && batch === next, 'review_loop_batch'); active = batch; },
    row(value) {
      check(active !== null && value.batch === active, 'review_loop_batch');
      if (value.kind === 'failure') { finished = true; active = null; }
      else if (value.kind === 'complete') { active = null; next += 1; finished = next > batches; }
    },
    get finished() { return finished; },
  };
}

/**
 * The Gate refuses Connect without a V2 scope. It admits possession and V2 status only in
 * the candidate phase, and only after a before-phase baseline in the same page, so the page
 * starts on `before` and reconfigures to `candidate` once that baseline exists.
 */
export function gateConfiguration(context, trust, phase) {
  check(context.scope === 'share' && ['before', 'candidate'].includes(phase), 'review_loop_scope');
  return configuration({ ...context, before_source: context }, phase, trust);
}

/** One loop per served root; batches begin in order and every row is persisted in order. */
export function createLoopServer({ root, context, assets, verify }, operations = {}) {
  const now = operations.now ?? Date.now;
  const challenge = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
  const batches = createBatches(); let rows = 0, queue = Promise.resolve();
  const server = createServer((request, response) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'review_loop_host'); const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/loop-page.mjs"></script>`), 'text/html');
      if (path === '/context') return send(response, 200, gateConfiguration(context, assets.trust, 'before'));
      const asset = path === `/${BUNDLE}` ? assets.bundle : assets.modules?.[path];
      if (asset) return send(response, 200, asset, 'text/javascript');
      return send(response, 404, { error: 'review_loop_route' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin || (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'review_loop_origin');
    const input = await body(request);
    if (path === '/activate') { object(input, []); await verify(); return send(response, 200, challenge); }
    if (path === '/loop/candidate') { object(input, []); await verify(); return send(response, 200, gateConfiguration(context, assets.trust, 'candidate')); }
    if (path === '/loop/begin') {
      object(input, ['state', 'batch']); await verify();
      check(input.state?.status === 'ready' && input.state.connected === true && input.state.running === false &&
        input.state.deviceLeaseInactive === true, 'review_loop_not_idle');
      batches.begin(input.batch);
      await writeNew(resolve(root, `loop-begin-${input.batch}.json`), { schema: 'str005-review-loop-begin-v2', batch: input.batch,
        startedAtUnixMs: now(), iterations: LIMITS.iterations, batches: LIMITS.batches, operations: OPERATIONS });
      return send(response, 200, { campaignId: context.original_campaign_id, iterations: LIMITS.iterations, batch: input.batch });
    }
    if (path === '/loop/row') {
      const row = validateRow(input); batches.row(row); rows += 1;
      await writeNew(resolve(root, `loop-row-${String(rows).padStart(3, '0')}.json`), { ...row, atUnixMs: now() });
      return send(response, 200, { recorded: true, finished: batches.finished });
    }
    return send(response, 404, { error: 'review_loop_route' });
  }).catch(() => { if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'review_loop_rejected' }); else response.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const closed = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await queue; await closed; };
  return server;
}
