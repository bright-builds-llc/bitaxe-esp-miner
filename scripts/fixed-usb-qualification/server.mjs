import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall, readPoolForSigning, signWindow } from "./authority.mjs";
import { BUNDLE, canonicalBase64, contextPage, digest, exactObject, missing, nonce, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { loadContext, verifyFrozen } from "./preflight.mjs";
import { finishWindow, recordFault, recordState, requireCompleteCycles, selectedWindow } from "./store.mjs";
import { loadSuccessor, requireSuccessorBaseline, validateBudgetReview, validateCoolingReview } from "./successor.mjs";
import { validateState } from "./judge.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function createSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot);
  const context = options.context ?? await loadContext(root);
  const page = contextPage(context);
  const verify = operations.verifyFrozen ?? (() => verifyFrozen(context, options.authorityDirectory, options.bun, {}, root));
  const frozen = await verify();
  const gateAssetRoot = frozen?.gate_root ?? context.gate_root;
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  let scope, pendingWindow, lastBrowserState, reviewChallenge, coolingChallenge, reviewedBudget, signing = false, recordQueue = Promise.resolve();
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
      reviewChallenge = undefined;
      coolingChallenge = undefined;
      reviewedBudget = undefined;
      scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
      return send(response, 200, scope);
    }
    if (request.method === "POST" && url.pathname === "/cooling-review-context") {
      exactObject(await body(request), []);
      requireCondition(scope && !signing && pendingWindow === undefined, "cooling_review_scope");
      reviewedBudget = undefined;
      reviewChallenge = undefined;
      coolingChallenge = { nonce: nonce(), challengeId: scope.challengeId, expires: now() + 45000 };
      return send(response, 200, { campaignId: context.campaign_id, nonce: coolingChallenge.nonce });
    }
    if (request.method === "POST" && url.pathname === "/cooling-review") {
      const input = await body(request);
      exactObject(input, ["nonce", "proof", "restoration", "budget_before", "budget_after", "state"]);
      const challenge = coolingChallenge;
      coolingChallenge = undefined;
      requireCondition(challenge && challenge.nonce === input.nonce && challenge.challengeId === scope?.challengeId &&
        challenge.expires > now() && !signing && pendingWindow === undefined, "cooling_review_challenge");
      const { nonce: requestNonce, ...review } = input;
      validateCoolingReview(review);
      await requireSuccessorBaseline(root, context, input.state);
      const file = `cooling-review-${challenge.nonce}.json`;
      await writeNew(resolve(root, file), { schema: "fixed-usb-cooling-review-v1", context_sha256: digest(JSON.stringify(context)), ...review });
      lastBrowserState = input.state;
      return send(response, 200, { cooling_review_saved: true, review_file: file });
    }
    if (request.method === "POST" && url.pathname === "/budget-review-context") {
      exactObject(await body(request), []);
      requireCondition(scope && !signing && pendingWindow === undefined, "budget_review_scope");
      reviewedBudget = undefined;
      reviewChallenge = { nonce: nonce(), challengeId: scope.challengeId, expires: now() + 45000 };
      return send(response, 200, { campaignId: context.campaign_id, nonce: reviewChallenge.nonce });
    }
    if (request.method === "POST" && url.pathname === "/budget-review") {
      const input = await body(request);
      exactObject(input, ["nonce", "report", "controlSessionBindingSha256", "state"]);
      const challenge = reviewChallenge;
      reviewChallenge = undefined;
      requireCondition(challenge && challenge.nonce === input.nonce && challenge.challengeId === scope?.challengeId &&
        challenge.expires > now() && canonicalBase64(input.controlSessionBindingSha256, 32), "budget_review_challenge");
      const successor = await loadSuccessor(root, context);
      const index = successor ? await selectedWindow(root) : input.report?.charged_ms === 210000 ? 2 : 1;
      validateBudgetReview(input.report, index);
      await requireSuccessorBaseline(root, context, input.state);
      const file = `budget-review-${challenge.nonce}.json`;
      await writeNew(resolve(root, file), { schema: "fixed-usb-budget-review-v1", context_sha256: digest(JSON.stringify(context)),
        report: input.report, state: input.state });
      reviewedBudget = { binding: input.controlSessionBindingSha256, challengeId: challenge.challengeId,
        expires: challenge.expires, report: input.report, index };
      lastBrowserState = input.state;
      return send(response, 200, { budget_review_saved: true, review_file: file });
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
        const successor = await loadSuccessor(root, context);
        requireCondition(!reviewedBudget || successor, "successor_creation_required");
        let reviewExpires;
        if (successor) {
          const reviewed = reviewedBudget;
          reviewedBudget = undefined;
          requireCondition(reviewed && reviewed.index === index && reviewed.binding === input.controlSessionBindingSha256 &&
            reviewed.challengeId === scope.challengeId && reviewed.expires > now(), "successor_fresh_review_required");
          validateBudgetReview(reviewed.report, index);
          reviewExpires = reviewed.expires;
        }
        requireCondition(index < 3, "campaign_complete");
        await missing(resolve(root, `window-${index}.issued.json`));
        await requireCompleteCycles(root, context, lastBrowserState, index);
        requireCondition(JSON.stringify(await verify()) === JSON.stringify(frozen), "supervisor_policy_changed");
        const artifacts = await signWindow({ campaignId: context.campaign_id, index, challengeId: scope.challengeId,
          binding: input.controlSessionBindingSha256, stratum: await readPool(), sign, successor: successor !== undefined });
        requireCondition(Buffer.byteLength(JSON.stringify(artifacts)) <= 65536, "window_artifact_bound");
        if (successor) requireCondition(now() < reviewExpires && lastBrowserState?.connected &&
          !lastBrowserState.running && !lastBrowserState.failure, "successor_review_expired_during_signing");
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
      if (!lastBrowserState.connected || lastBrowserState.failure || lastBrowserState.running) {
        reviewedBudget = undefined;
        reviewChallenge = undefined;
        coolingChallenge = undefined;
      }
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
    if (request.method === "GET" && ["/", `/${page}`, `/${BUNDLE}`].includes(url.pathname)) {
      const isPage = url.pathname !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(gateAssetRoot, isPage ? page : BUNDLE));
      requireCondition(digest(bytes) === (isPage ? context.gate_page_sha256 : context.gate_bundle_sha256), "served_asset_drift");
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
