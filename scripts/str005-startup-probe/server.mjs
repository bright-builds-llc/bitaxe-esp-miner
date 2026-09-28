import { createServer } from 'node:http';
import { once } from 'node:events';
import { readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { BUNDLE, admitTrust, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { createSigner } from '../str005-v2-serial/signing.mjs';
import { startFixture } from '../str005-v2-serial/fixture-owner.mjs';
import { requireGone, requirePoolListenerAbsent } from '../str005-v2-serial/host-resources.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { proof, writeNew } from '../str005-noise-serial/files.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { verifyClear } from './clear.mjs';
import { createRoutes } from './routes.mjs';

export async function createServerOwner({ root, context, assets, authorityDirectory, verify }, operations = {}) {
  await verify(); let fixture, maybeSigner, maybeDetectorSha, released = false, queue = Promise.resolve();
  const persist = (name, value) => writeNew(resolve(root, name), value);
  async function release() {
    if (released) return; released = true;
    if (fixture) {
      await fixture.close();
      const owner = (await proof(root, 'fixture-owner.json')).value.owner;
      await requireGone([owner]); requirePoolListenerAbsent(fixture.ready.listenPort);
    }
    await persist('fixture-release.json', { schema: 'str005-startup-fixture-release-v1', complete: true });
  }
  const verifyEffect = async () => {
    await verify(); await verifyClear(root, context);
    const path = resolve(dirname(root), 'startup-detector.stdout.log'); await protectedPath(path);
    const bytes = await readFile(path), age = Date.now() - (await stat(path)).mtimeMs;
    const detector = parseDetector(bytes.toString('utf8'), context.physical, age);
    const digest = sha256(bytes); check(!maybeDetectorSha || digest === maybeDetectorSha, 'startup_physical_changed');
    if (!maybeDetectorSha) { await persist('start-detector.json', { ...detector, sha256: digest, observedAtUnixMs: Date.now() }); maybeDetectorSha = digest; }
  };
  const prewarm = async () => {
    await verifyEffect();
    if (!maybeSigner) {
      maybeSigner = await (operations.createSigner ?? createSigner)(root, context, authorityDirectory, routes.failed);
      admitTrust(assets.trust, await maybeSigner('public-trust'));
    }
  };
  const routes = createRoutes(context, { verify, verifyEffect, persist, release, prewarm,
    sign: async (...args) => {
      check(maybeSigner, 'startup_signer_not_prewarmed'); await verifyEffect();
      return maybeSigner(...args);
    }, launch: async (attemptContext, station) => {
      check(!released, 'startup_fixture_released');
      fixture = await (operations.startFixture ?? startFixture)(root, attemptContext, station, () => routes.fail()); return fixture;
    } });
  const server = createServer((request, response) => {
    queue = queue.then(async () => {
      const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
      check(request.headers.host === host, 'startup_host'); const path = new URL(request.url, origin).pathname;
      if (request.method === 'GET') {
        if (path === '/context') return send(response, 200, { ...configuration(context, 'before', assets.trust), coreDumpSelfTestQualification: true });
        if (path === '/') return send(response, 200, Buffer.from(`${assets.page}\n<script type="module" src="/startup-page.mjs"></script>`), 'text/html');
        if (path === `/${BUNDLE}`) return send(response, 200, assets.bundle, 'text/javascript');
        if (path === '/startup-page.mjs') return send(response, 200, assets.pageClient, 'text/javascript');
        if (path === '/client.mjs') return send(response, 200, assets.coordinator, 'text/javascript');
        if (path === '/retained-status.mjs') return send(response, 200, await readFile(resolve(context.firmware_root, 'scripts/str005-startup-probe/retained-status.mjs')), 'text/javascript');
        if (path === '/window-artifacts') return send(response, 200, await routes.handle(path, undefined, 'GET'));
        return send(response, 404, { error: 'startup_route_unavailable' });
      }
      check(request.method === 'POST' && (request.headers.origin === origin ||
        (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'startup_origin');
      const input = await body(request);
      if (path === '/diagnostic-export') {
        const exported = await validateDiagnosticExport(input, context.gate_root);
        await routes.diagnostics(exported);
        return send(response, 200, { diagnostic_export_saved: true, review_file: 'diagnostic-export-startup.json' });
      }
      const result = await routes.handle(path, input);
      return send(response, 200, path === '/startup/candidate' ? { ...configuration(context, 'candidate', assets.trust), coreDumpSelfTestQualification: true } : result);
    }).catch(() => { routes.fail(); if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'startup_request_rejected' }); else response.destroy(); });
  });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  server.release = async () => { routes.fail(); const closed = server.listening ? once(server, 'close') : Promise.resolve();
    server.close(); server.closeAllConnections(); try { await queue; await release(); } finally { await closed; } };
  return server;
}
