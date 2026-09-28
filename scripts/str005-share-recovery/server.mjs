import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { check, object, bytes, sha256 } from '../str005-v2-serial/values.mjs';
import { STAGES, finished as validateFinished, validateErrors } from './model.mjs';
export function createRecoveryServer({ root, context, assets, verify }, operations = {}) {
  const now = operations.now ?? Date.now, saved = new Set(); let begun, preparation, recoveryChallenge, complete = false, queue = Promise.resolve();
  const stages = new Map(); let activeStage;
  const persist = operations.persist ?? ((stage, value) => writeNew(resolve(root, `${stage}.json`), value));
  const challenge = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
  const server = createServer((request, response) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'share_recovery_host'); const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/recovery-page.mjs"></script>`), 'text/html');
      if (path === '/context') return send(response, 200, configuration({ ...context, before_source: context }, 'before', assets.trust));
      const asset = path === `/${BUNDLE}` ? assets.bundle : assets.modules?.[path];
      if (asset) return send(response, 200, asset, 'text/javascript');
      return send(response, 404, { error: 'share_recovery_route' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin || (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'share_recovery_origin');
    const input = await body(request);
    if (path === '/activate') { object(input, []); await verify(); return send(response, 200, challenge); }
    if (path === '/recovery-context') { object(input, []); return send(response, 200, { candidateConfiguration: configuration({ ...context, before_source: context }, 'candidate', assets.trust) }); }
    if (path === '/recovery-prepare') {
      object(input, ['before', 'closed']); await verify(); check(!preparation && !begun, 'share_recovery_prepare_consumed');
      validateState(input.before, context); validateState(input.closed, context);
      check(input.before.status === 'ready' && input.before.connected && !input.before.running && input.before.deviceLeaseInactive &&
        input.before.deviceBaselineConfirmed && ['settings_match', 'device_identity_match', 'authorization_high_water_match'].every(key => input.before.preservation?.[key] === true && input.closed.preservation?.[key] === true) &&
        input.before.preservation.mine_on_boot === false && input.closed.preservation.mine_on_boot === false && input.closed.status === 'closed' && !input.closed.connected && input.closed.serialOwnershipReleased &&
        input.closed.deviceRestorationConfirmed && input.closed.deviceLeaseInactive &&
        input.closed.preservation?.baseline_id === input.before.preservation?.baseline_id, 'share_recovery_prepare_baseline');
      preparation = { baselineId: input.before.preservation.baseline_id };
      await persist('preparation', { schema: 'str005-share-recovery-preparation-v1', before: input.before, closed: input.closed });
      return send(response, 200, { prepared: true });
    }
    if (path === '/recovery-challenge') {
      object(input, []); await verify(); check(preparation && !recoveryChallenge && !begun && !complete, 'share_recovery_challenge_consumed');
      recoveryChallenge = { nonce: nonce(), at: now() };
      await persist('challenge', { schema: 'str005-share-recovery-challenge-v1', ...recoveryChallenge });
      return send(response, 200, recoveryChallenge);
    }
    if (path === '/recovery-begin') {
      object(input, ['state', 'nonce', 'binding']); await verify(); check(preparation && recoveryChallenge && !begun && !complete && saved.size === 0 &&
        input.nonce === recoveryChallenge.nonce && now() >= recoveryChallenge.at && now() - recoveryChallenge.at <= 30000, 'share_recovery_consumed');
      bytes(input.binding, 32);
      validateState(input.state, context);
      check(input.state.status === 'ready' && input.state.connected && !input.state.running && input.state.deviceLeaseInactive && input.state.deviceBaselineConfirmed && input.state.preservation?.baseline_id === preparation.baselineId, 'share_recovery_baseline');
      begun = { schema: 'str005-share-recovery-begin-v1', startedAtUnixMs: now(), collectionId: nonce(), bindingSha256: sha256(input.binding), preservationBaselineId: preparation.baselineId };
      await persist('collection-begin', begun);
      return send(response, 200, { ...begun, attemptId: context.attemptId, campaignId: context.original_campaign_id, statusMode: 'discover_current' });
    }
    if (path === '/stage-begin') {
      object(input, ['collectionId', 'phase']); await verify();
      check(begun && input.collectionId === begun.collectionId && !complete && now() >= begun.startedAtUnixMs && now() - begun.startedAtUnixMs <= 600000 &&
        ['ledger', 'original_budget', 'diagnostics', 'state', 'status'].includes(input.phase) && !stages.has(input.phase) && !activeStage, 'share_recovery_stage_admission');
      const ticket = { phase: input.phase, token: nonce(), startedAtUnixMs: now(), deadlineUnixMs: now() + 30000 };
      stages.set(input.phase, ticket); activeStage = ticket;
      await persist(`stage-${input.phase}-begin`, ticket); return send(response, 200, ticket);
    }
    if (path === '/stage-end') {
      object(input, ['collectionId', 'phase', 'token']);
      check(begun && input.collectionId === begun.collectionId && activeStage?.phase === input.phase && activeStage.token === input.token, 'share_recovery_stage_admission');
      await persist(`stage-${input.phase}-end`, { endedAtUnixMs: now(), token: input.token }); activeStage = undefined;
      return send(response, 200, { ended: true });
    }
    let stage, value;
    if (path === '/diagnostic-export') {
      stage = 'diagnostics'; value = projectRecoveryPart(stage, await (operations.validateDiagnostics ?? validateDiagnosticExport)(input, context.gate_root), context);
    } else {
      check(path === '/recovery-part', 'share_recovery_route'); object(input, ['collectionId', 'stage', 'ticket', 'value']); stage = input.stage;
      check(STAGES.includes(stage) && stage !== 'diagnostics', 'share_recovery_stage');
      check(input.collectionId === begun?.collectionId || (!begun && input.collectionId === null && ['closed', 'errors', 'finished'].includes(stage)), 'share_recovery_collection');
      value = stage === 'finished' ? validateFinished(input.value) : stage === 'errors' ? validateErrors(input.value) : projectRecoveryPart(stage, input.value, context);
    }
    if (begun && ['state', 'closed'].includes(stage)) check(value.preservation?.baseline_id === begun.preservationBaselineId, 'share_recovery_baseline_changed');
    check(!complete && !saved.has(stage), 'share_recovery_stage_consumed');
    check(['closed', 'errors', 'finished'].includes(stage) || (begun && activeStage?.phase === stage &&
      (stage === 'diagnostics' || input.ticket === activeStage.token) && now() >= activeStage.startedAtUnixMs && now() <= activeStage.deadlineUnixMs), 'share_recovery_collection_expired');
    await persist(stage, value); saved.add(stage); complete = stage === 'finished';
    return send(response, 200, stage === 'diagnostics' ? { diagnostic_export_saved: true, review_file: 'diagnostics.json' } : { recorded: true });
  }).catch(() => { if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'share_recovery_rejected' }); else response.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const closed = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await queue; await closed; };
  return server;
}
