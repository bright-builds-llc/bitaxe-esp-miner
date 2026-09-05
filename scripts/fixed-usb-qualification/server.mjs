import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall, readPoolForSigning, signWindow } from "./authority.mjs";
import { BUNDLE, canonicalBase64, exactObject, missing, nonce, PAGE, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { loadContext, verifyFrozen } from "./preflight.mjs";
import { finishWindow, recordFault, recordState, requireCompleteCycles, selectedWindow } from "./store.mjs";
import { validateState } from "./judge.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function createSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot);
  const context = options.context ?? await loadContext(root);
  const verify = operations.verifyFrozen ?? (() => verifyFrozen(context, options.authorityDirectory, options.bun));
  await verify();
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  let scope, pendingWindow, lastBrowserState, signing = false, recordQueue = Promise.resolve();
  const sign = operations.sign ?? ((operation, input) => authorityCall(context.gate_root, options.authorityDirectory, `sign-${operation}`, input, options.bun));
  const readPool = operations.readPool ?? (() => readPoolForSigning(context.firmware_root, options.poolCredentials));
  const now = operations.now ?? Date.now;
  const serializeRecord = (operation) => {
    const result = recordQueue.then(operation);
    recordQueue = result.then(() => undefined, () => undefined);
    return result;
  };
  const server = createServer((request, response) => {
    handle(request, response).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    });
  });
  server.requestTimeout = 5000;
  server.headersTimeout = 10000;
  server.on("close", () => { pendingWindow = undefined; scope = undefined; });

  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    const url = new URL(request.url, origin);
    const allowedOrigin = request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin");
    if (request.method === "POST" || url.pathname === "/window-artifacts") requireCondition(allowedOrigin, "origin_rejected");
    if (request.method === "GET" && url.pathname === "/context") {
      return send(response, 200, { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
        expectedAppElfSha256: context.app_elf_sha256, trust });
    }
    if (request.method === "POST" && url.pathname === "/activate") {
      exactObject(await body(request), []);
      requireCondition(!signing && pendingWindow === undefined, "context_busy");
      scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
      return send(response, 200, scope);
    }
    if (request.method === "POST" && url.pathname === "/authorization-context") {
      const input = await body(request);
      exactObject(input, ["controlSessionBindingSha256"]);
      requireCondition(canonicalBase64(input.controlSessionBindingSha256, 32) && scope &&
        scope.retentionExpiryUnixSeconds > Math.floor(now() / 1000), "authorization_context");
      requireCondition(!signing && pendingWindow === undefined, "authorization_pending");
      signing = true;
      try {
        const index = await selectedWindow(root);
        requireCondition(index < 3, "campaign_complete");
        await missing(resolve(root, `window-${index}.issued.json`));
        await requireCompleteCycles(root, context, lastBrowserState, index);
        await verify();
        const artifacts = await signWindow({ campaignId: context.campaign_id, index, challengeId: scope.challengeId,
          binding: input.controlSessionBindingSha256, stratum: await readPool(), sign });
        requireCondition(Buffer.byteLength(JSON.stringify(artifacts)) <= 65536, "window_artifact_bound");
        await writeNew(resolve(root, `window-${index}.issued.json`), { schema: "fixed-usb-window-issuance-v1", window: index,
          private_payload_persisted: false, maximum_active_ms: context.window_limits_ms[index] });
        pendingWindow = { index, artifacts };
      } finally { signing = false; }
      return send(response, 200, { ready: true });
    }
    if (request.method === "GET" && url.pathname === "/window-artifacts") {
      requireCondition(pendingWindow !== undefined, "window_not_available");
      const value = pendingWindow;
      await writeNew(resolve(root, `window-${value.index}.consumed.json`), { window: value.index, delivery_attempted: true });
      pendingWindow = undefined;
      return send(response, 200, value.artifacts);
    }
    if (request.method === "POST" && url.pathname === "/record") {
      const value = await body(request);
      exactObject(value, ["state"]);
      lastBrowserState = validateState(value.state, context);
      return send(response, 200, await serializeRecord(() => recordState(root, context, value.state)));
    }
    if (request.method === "POST" && url.pathname === "/fault") {
      const value = await body(request);
      return send(response, 200, await serializeRecord(() => recordFault(root, value)));
    }
    if (request.method === "POST" && url.pathname === "/advance") {
      exactObject(await body(request), []);
      const result = await serializeRecord(async () => finishWindow(root, context, await selectedWindow(root)));
      return send(response, 200, { ...result, next_window: await selectedWindow(root) });
    }
    if (request.method === "GET" && url.pathname === "/supervisor-state") {
      return send(response, 200, { window: await selectedWindow(root), waiting_for_human_has_no_deadline: true,
        authority_context_active: scope !== undefined, private_payload_pending_in_memory: pendingWindow !== undefined });
    }
    if (request.method === "GET" && url.pathname === "/supervisor-client.mjs") {
      return send(response, 200, await readFile(resolve(SCRIPT_ROOT, "client.mjs")), "text/javascript");
    }
    if (request.method === "GET" && ["/", `/${PAGE}`, `/${BUNDLE}`].includes(url.pathname)) {
      const isPage = url.pathname !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(context.gate_root, isPage ? PAGE : BUNDLE));
      if (isPage) bytes = Buffer.from(`${bytes.toString("utf8")}\n<script type="module" src="/supervisor-client.mjs"></script>`);
      return send(response, 200, bytes, isPage ? "text/html" : "text/javascript");
    }
    send(response, 404, { error: "route_unavailable" });
  }
  return server;
}

async function body(request) {
  const chunks = [];
  let size = 0;
  for await (const chunk of request) {
    size += chunk.length;
    requireCondition(size <= 65536, "request_body_bound");
    chunks.push(chunk);
  }
  try { return JSON.parse(Buffer.concat(chunks).toString("utf8")); }
  catch { throw new QualificationError("request_json"); }
}
function send(response, status, value, contentType = "application/json") {
  response.writeHead(status, { "Content-Type": contentType, "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff",
    "Referrer-Policy": "no-referrer", "Cross-Origin-Resource-Policy": "same-origin" });
  response.end(Buffer.isBuffer(value) ? value : JSON.stringify(value));
}
