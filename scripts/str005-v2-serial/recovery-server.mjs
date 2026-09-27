import { createServer } from "node:http";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { BUNDLE, nonce } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { validateDiagnosticExport } from "../fixed-usb-qualification/diagnostic-export.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { configuration } from "./server-assets.mjs";
import { projectRecoveryPart } from "./recovery-evidence.mjs";
import { check, object } from "./values.mjs";

/** No signer, fixture, flash adapter or qualification supervisor exists in this server. */
export function createRecoveryServer({ root, context, page, bundle, client, trust }, operations = {}) {
  let queue = Promise.resolve(), finished = false;
  // Both sessions observe the installed pair; this is not an installation cycle.
  const recoveryContext = { ...context, before_source: context };
  const saved = new Set();
  const scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 };
  const persist = operations.persist ?? ((stage, value) => writeNew(resolve(root, `${stage}.json`), value));
  const server = createServer((request, response) => {
    const pending = queue.then(() => handle(request, response));
    queue = pending.catch(() => {
      if (!response.headersSent && !response.destroyed) send(response, 400, { error: "recovery_request_rejected" });
      else response.destroy();
    });
  });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  async function save(stage, input) {
    check(!finished && !saved.has(stage), "recovery_part_consumed");
    const value = projectRecoveryPart(stage, input, context);
    await persist(stage, value); saved.add(stage);
  }
  async function handle(request, response) {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, "recovery_host");
    const path = new URL(request.url, origin).pathname;
    if (request.method === "GET") {
      if (path === "/context") return send(response, 200, configuration(recoveryContext, "before", trust));
      if (path === "/") return send(response, 200, Buffer.from(`${page}\n<script type="module" src="/recovery-client.mjs"></script>`), "text/html");
      if (path === `/${BUNDLE}`) return send(response, 200, bundle, "text/javascript");
      if (path === "/recovery-client.mjs") return send(response, 200, client, "text/javascript");
      return send(response, 404, { error: "recovery_route_unavailable" });
    }
    check(request.method === "POST" && (request.headers.origin === origin ||
      (!request.headers.origin && request.headers["sec-fetch-site"] === "same-origin")), "recovery_origin");
    const input = await body(request);
    if (path === "/activate") { object(input, []); return send(response, 200, scope); }
    if (path === "/recovery-context") {
      object(input, []); return send(response, 200, { campaignId: context.original_campaign_id, attemptId: context.attemptId,
        candidateConfiguration: configuration(recoveryContext, "candidate", trust) });
    }
    if (path === "/diagnostic-export") {
      const value = await (operations.validateDiagnostics ?? validateDiagnosticExport)(input, context.gate_root);
      await save("diagnostics", value);
      return send(response, 200, { diagnostic_export_saved: true, review_file: "diagnostic-export-recovery.json" });
    }
    check(path === "/part", "recovery_route_unavailable");
    object(input, ["stage", "value"]);
    if (input.stage === "finished") {
      object(input.value, ["failures"]);
      check(!finished && Array.isArray(input.value.failures) && input.value.failures.length <= 6 &&
        input.value.failures.every(stage => ["ledger", "original_budget", "diagnostics", "status", "state", "closed"].includes(stage)), "recovery_failure_shape");
      await persist("finished", input.value); finished = true;
    } else await save(input.stage, input.value);
    send(response, 200, { recorded: true });
  }
  server.release = async () => {
    const closed = server.listening ? once(server, "close") : Promise.resolve();
    server.close(); server.closeAllConnections(); await queue; await closed;
  };
  return server;
}

export async function recoveryAssets(firmwareRoot, oldRoot, context) {
  return {
    page: await readFile(resolve(oldRoot, "qualified-artifacts/gate", context.gate_page_relative_path)),
    bundle: await readFile(resolve(oldRoot, "qualified-artifacts/gate", BUNDLE)),
    client: await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/recovery-client.mjs")),
    trust: JSON.parse(await readFile(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), "utf8")),
  };
}
