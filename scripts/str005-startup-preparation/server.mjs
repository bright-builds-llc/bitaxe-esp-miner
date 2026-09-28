import { createServer } from 'node:http';
import { once } from 'node:events';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { baseline, part, readyDiagnostics, restartEvidence, restartPacket, FRESH_MS } from './model.mjs';
import { isDeepStrictEqual as equal } from 'node:util';
export function restartConfiguration(context, trust) {
  return { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, trust, restartQualification: true };
}
/** Separate owner/page: the restart-only contract never shares or downgrades a V2 configuration. */
export function createRestartServer({ root, context, assets, verify }, operations = {}) {
  const now = operations.now ?? Date.now, persist = operations.persist ?? ((name, value) => writeNew(resolve(root, name), value));
  let queue = Promise.resolve(), startedAt, request, claimed = false, finished = false;
  const saved = new Set(), parts = {};
  const scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
  async function save(name, value) { check(!saved.has(name) && !finished, 'preparation_part_consumed'); await persist(name, value); saved.add(name); }
  const server = createServer((req, res) => { queue = queue.then(async () => {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`, path = new URL(req.url, origin).pathname;
    check(req.headers.host === host, 'preparation_host');
    if (req.method === 'GET') {
      if (path === '/') return send(res, 200, Buffer.from(`${assets.page}\n<script type="module" src="/restart-client.mjs"></script>`), 'text/html');
      if (path === '/context') return send(res, 200, restartConfiguration(context, assets.trust));
      if (path === `/${BUNDLE}`) return send(res, 200, assets.bundle, 'text/javascript');
      if (path === '/restart-client.mjs') return send(res, 200, assets.client, 'text/javascript');
      return send(res, 404, { error: 'preparation_route_unavailable' });
    }
    check(req.method === 'POST' && (req.headers.origin === origin || (!req.headers.origin && req.headers['sec-fetch-site'] === 'same-origin')), 'preparation_origin');
    const input = await body(req);
    if (path === '/activate') { object(input, []); return send(res, 200, scope); }
    if (path === '/preparation-context') { object(input, []); return send(res, 200, { campaignId: context.original_campaign_id }); }
    if (path === '/begin') { object(input, []); check(startedAt === undefined, 'preparation_begin_consumed'); await verify(); startedAt = now();
      await save('begin.json', { startedAtUnixMs: startedAt }); return send(res, 200, { begun: true }); }
    if (path === '/part') {
      object(input, ['stage', 'value']); const value = part(input.stage, input.value, context);
      check(startedAt !== undefined, 'preparation_not_begun'); await save(`${input.stage}.json`, value); parts[input.stage] = value;
      return send(res, 200, { recorded: true });
    }
    if (path === '/diagnostic-export') {
      check(!claimed && startedAt !== undefined, 'preparation_diagnostics_phase');
      const value = await (operations.validateDiagnostics ?? validateDiagnosticExport)(input, context.gate_root);
      readyDiagnostics(value, context, context.before_boot_ordinal); await save('before_diagnostics.json', value); parts.diagnostics = value;
      return send(res, 200, { diagnostic_export_saved: true, review_file: 'diagnostic-export-preparation.json' });
    }
    if (path === '/restart-claim') {
      object(input, []); check(!claimed && startedAt !== undefined && now() >= startedAt && now() - startedAt <= FRESH_MS, 'preparation_stale');
      await verify(); baseline(parts.before_state, context); readyDiagnostics(parts.diagnostics, context, context.before_boot_ordinal);
      check(equal(parts.before_ledger, context.expectedLedger) && equal(parts.before_budget, context.originalBudget), 'preparation_before_accounting');
      request = { requestNonce: nonce(), expectedBootOrdinal: context.before_boot_ordinal }; claimed = true;
      await save('restart-claim.json', { request, claimedAtUnixMs: now(), beforeStartedAtUnixMs: startedAt });
      return send(res, 200, request);
    }
    if (path === '/restart-evidence') {
      object(input, ['evidence']); check(claimed && request, 'preparation_restart_unclaimed');
      const evidence = await restartPacket(input.evidence, context, request, operations);
      await save('evidence.json', evidence); let verified = false;
      try { await restartEvidence(evidence, context, request, operations); verified = true; } catch { verified = false; }
      await save('evidence-verification.json', { verified }); return send(res, 200, { recorded: true, verified });
    }
    if (path === '/finished') {
      object(input, ['failures']); check(Array.isArray(input.failures) && input.failures.length <= 12 && new Set(input.failures).size === input.failures.length &&
        input.failures.every(stage => ['before_state', 'before_ledger', 'before_budget', 'after_state', 'after_ledger', 'after_budget', 'restart', 'evidence', 'preparation', 'stop', 'closed'].includes(stage)), 'preparation_failures');
      await save('finished.json', input); finished = true; return send(res, 200, { recorded: true });
    }
    check(false, 'preparation_route_unavailable');
  }).catch(() => { if (!res.headersSent && !res.destroyed) send(res, 400, { error: 'preparation_rejected' }); else res.destroy(); }); });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { const closed = server.listening ? once(server, 'close') : Promise.resolve(); server.close(); server.closeAllConnections(); await queue; await closed; };
  return server;
}
