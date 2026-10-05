import { CONTROL_REJECTIONS } from '../str005-v2-serial/safety-diagnostics.mjs';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
export const RECOVERY_STAGES = ['ledger', 'original_budget', 'diagnostics', 'status', 'stop', 'state', 'closed'];
export function validateFinished(value) {
  object(value, ['failures']); check(Array.isArray(value.failures) && value.failures.length <= RECOVERY_STAGES.length + 1, 'recovery_failure_bound');
  const seen = new Set();
  // Rows may carry the Worker's closed rejection; earlier rows have none.
  for (const row of value.failures) { object(row, 'rejection' in row ? ['stage', 'category', 'rejection'] : ['stage', 'category']);
    check([...RECOVERY_STAGES, 'begin'].includes(row.stage) && !seen.has(row.stage) && ['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed', 'operation_failed'].includes(row.category) &&
      (row.rejection === undefined || row.rejection === null || CONTROL_REJECTIONS.includes(row.rejection)), 'recovery_failure_shape'); seen.add(row.stage); }
  return value;
}
/** No signer, fixture, clear adapter or funded Start route exists in this owner. */
export function createCurrentRecoveryServer({ root, context, assets }, operations = {}) {
  let queue = Promise.resolve(), finished = false; const saved = new Set();
  const now = operations.now ?? Date.now;
  const persist = operations.persist ?? ((stage, value) => writeNew(resolve(root, `${stage}.json`), value));
  const scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 };
  const server = createServer((request, response) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'recovery_host'); const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/recovery-client.mjs"></script>`), 'text/html');
      if (path === '/context') return send(response, 200, { ...configuration({ ...context, before_source: context }, 'before', assets.trust), coreDumpSelfTestQualification: true });
      if (path === `/${BUNDLE}`) return send(response, 200, assets.bundle, 'text/javascript');
      if (path === '/recovery-client.mjs') return send(response, 200, assets.client, 'text/javascript');
      if (path === '/retained-status.mjs') return send(response, 200, assets.retainedStatus, 'text/javascript');
      return send(response, 404, { error: 'recovery_route_unavailable' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin || (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'recovery_origin');
    const input = await body(request);
    if (path === '/activate') { object(input, []); return send(response, 200, scope); }
    if (path === '/recovery-context') { object(input, []); return send(response, 200, { attemptId: context.attemptId, campaignId: context.original_campaign_id }); }
    if (path === '/recovery-begin') {
      object(input, []); check(!finished && saved.size === 0, 'recovery_collection_consumed');
      await persist('collection-begin', { schema: 'str005-recovery-collection-v1', startedAtUnixMs: now() });
      saved.add('collection-begin'); return send(response, 200, { begun: true });
    }
    let stage, value;
    if (path === '/diagnostic-export') { stage = 'diagnostics'; value = projectRecoveryPart(stage,
      await (operations.validateDiagnostics ?? validateDiagnosticExport)(input, context.gate_root), context); }
    else {
      check(path === '/recovery-part', 'recovery_route_unavailable'); object(input, ['stage', 'value']); stage = input.stage;
      if (stage === 'finished') value = validateFinished(input.value);
      else if (stage === 'stop') { object(input.value, ['requested']); check(input.value.requested === true, 'recovery_stop_shape'); value = input.value; }
      else { check(RECOVERY_STAGES.includes(stage) && stage !== 'diagnostics', 'recovery_stage'); value = projectRecoveryPart(stage, input.value, context); }
    }
    check(!finished && !saved.has(stage), 'recovery_stage_consumed'); await persist(stage, value); saved.add(stage); finished = stage === 'finished';
    return send(response, 200, stage === 'diagnostics' ? { diagnostic_export_saved: true, review_file: 'diagnostic-export-current-recovery.json' } : { recorded: true });
  }).catch(() => { if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'recovery_request_rejected' }); else response.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const closed = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await queue; await closed; };
  return server;
}
