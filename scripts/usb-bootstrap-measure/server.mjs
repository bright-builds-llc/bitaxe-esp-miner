import { readSourceSnapshot } from "./source-snapshot.mjs";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { BUNDLE, nonce } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { contextHash, load } from "./context.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { claim, inspectCapture } from "./install.mjs";
import { requireOperatorParent } from "./operator-parent.mjs";
import { fail } from "./failure.mjs";
import { check, object, schema, sha256, code } from "./values.mjs";
const ACCOUNTING_ROUTES = new Set(["/record", "/accounting-context", "/accounting"]);
/** The fixed accounting routes, shared with the simulated-device composition test. */
export async function serverAccountingRoute(root, context, journal, phase, path, value) {
  if (path === "/record") { object(value, ["state"]); return journal.record(phase, value.state); }
  if (path === "/accounting-context") { object(value, []); return { campaignId: context.originalCampaign.id }; }
  if (path === "/accounting") return saveAccounting(root, context, value, journal.last());
  check(false, "bootstrap_route_unavailable");
}
export async function createSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = await load(root, { operations, ancestry: true });
  const parent = await requireOperatorParent(root, context), hash = contextHash(context);
  await writeNew(resolve(root, "server.claim.json"), { schema: schema("server-claim"), contextSha256: hash });
  const journal = await createJournal(root, context), trust = (await proof(root, "snapshot/trust.json")).value;
  let phase = "before", stopping = false, queue = Promise.resolve();
  const configuration = () => {
    const source = phase === "before" ? context.beforeSource : context.package;
    return { expectedGateCommit: context.gate.commit, expectedFirmwareSourceCommit: source.firmware_commit, expectedAppElfSha256: source.app_elf_sha256, trust };
  };
  const server = createServer((request, response) => {
    const pending = queue.then(() => handle(request, response));
    queue = pending.catch(async error => { await fail(root, context, "supervisor", "request", error); if (!response.headersSent) send(response, 400, { error: code(error) }); else response.destroy(); });
  });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  async function handle(request, response) {
    parent.check(); check(!stopping, "bootstrap_terminal");
    const origin = `http://127.0.0.1:${server.address().port}`, path = new URL(request.url, origin).pathname;
    check(request.headers.host === `127.0.0.1:${server.address().port}`, "bootstrap_origin");
    if (request.method === "POST") check(request.headers.origin === origin || (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "bootstrap_origin");
    if (request.method === "GET") {
      if (path === "/context") return send(response, 200, configuration());
      if (path === "/supervisor-state") return send(response, 200, { phase, miningAuthorized: false });
      const source = path === "/" ? "snapshot/gate/page" : path === `/${BUNDLE}` ? "snapshot/gate/bundle" : path === "/bootstrap-client.mjs" ? "bootstrap-client" : path === "/accounting-baseline.mjs" ? "accounting-baseline" : null;
      if (source) {
        const maybeSourcePath = source === "bootstrap-client" ? "scripts/usb-bootstrap-measure/client.mjs" : source === "accounting-baseline" ? "scripts/usb-bootstrap-measure/accounting-baseline.mjs" : null;
        let bytes = maybeSourcePath ? await readSourceSnapshot(root, context, maybeSourcePath) : await readFile(resolve(root, source)); const expected = path === "/" ? context.gate.pageSha256 : path === `/${BUNDLE}` ? context.gate.bundleSha256 : context.sourceInventory.find(row => row.path === maybeSourcePath)?.sha256;
        check(sha256(bytes) === expected, "bootstrap_asset_changed");
        if (path === "/") bytes = Buffer.from(`${bytes.toString()}\n<script type="module" src="/bootstrap-client.mjs"></script>`);
        return send(response, 200, bytes, path === "/" ? "text/html" : "text/javascript");
      }
      return send(response, 404, { error: "bootstrap_route_unavailable" });
    }
    if (request.method !== "POST") return send(response, 404, { error: "bootstrap_route_unavailable" });
    const value = await body(request); parent.check();
    if (path === "/activate") { object(value, []); return send(response, 200, { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 }); }
    if (ACCOUNTING_ROUTES.has(path)) return send(response, 200, await serverAccountingRoute(root, context, journal, phase, path, value));
    if (path === "/client-failure") { object(value, ["code"]); check(value.code === "bootstrap_client_failed", "bootstrap_client_failure"); await fail(root, context, "browser", "browser", value); return send(response, 200, { recorded: true }); }
    if (path === "/install/claim") { await load(root, { operations }); parent.check(); return send(response, 200, await claim(root, context, value, operations)); }
    if (path === "/install/review") {
      object(value, []); const capture = await inspectCapture(root, context);
      await writeNew(resolve(root, "install-0.review.json"), { schema: schema("install-review"), contextSha256: hash, exitCode: capture.exitCode, flashVerdictSha256: capture.flashVerdictSha256, qualified: capture.qualified });
      phase = "candidate"; return send(response, 200, { install_measured: true, qualified: capture.qualified });
    }
    if (path === "/candidate-context") { object(value, []); check(phase === "candidate", "bootstrap_write_unproved"); await inspectCapture(root, context); return send(response, 200, configuration()); }
    return send(response, 404, { error: "bootstrap_route_unavailable" });
  }
  let maybeClose;
  server.closeQualificationResources = () => {
    if (maybeClose) return maybeClose; stopping = true;
    maybeClose = (async () => { server.closeAllConnections(); await queue; await new Promise(done => server.close(done)); parent.dispose(); })();
    return maybeClose;
  };
  parent.onLoss(() => { stopping = true; fail(root, context, "supervisor", "parent", { code: "bootstrap_parent_lost" }).finally(() => server.closeQualificationResources()).catch(() => { process.exitCode = 1; }); });
  server.qualificationReady = new Promise((done, reject) => server.once("listening", async () => {
    try { parent.check(); const owner = (await (operations.processSnapshot ?? processSnapshot)()).find(row => row.pid === process.pid);
      check(owner, "bootstrap_supervisor_owner"); await writeNew(resolve(root, "server-owner.json"), { schema: schema("server-owner"), contextSha256: hash, owner, port: server.address().port, origin: `http://127.0.0.1:${server.address().port}` }); done();
    } catch (e) { reject(e); }
  }));
  return server;
}
