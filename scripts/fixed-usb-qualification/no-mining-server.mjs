import { readFrozenCampaign, saveNoMiningAccounting } from "./no-mining-accounting.mjs";
import { createServer } from "node:http";
import { readFile, readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, digest, exactObject, nonce, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { saveDiagnosticExport, validateDiagnosticExport } from "./diagnostic-export.mjs";
import { body, send } from "./http.mjs";
import { validateState } from "./judge.mjs";
import { validateNoMiningContext, verifyNoMiningFrozen } from "./no-mining-context.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function createNoMiningSupervisor(options, operations = {}) {
  requireCondition(options.authorityDirectory === undefined && options.poolCredentials === undefined, "no_mining_credentials_forbidden");
  const root = resolve(options.privateRoot), context = options.context;
  validateNoMiningContext(context);
  const verify = operations.verifyFrozen ?? (() => verifyNoMiningFrozen(context));
  await verify();
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  let recordQueue = Promise.resolve();
  const server = createServer((request, response) => {
    handle(request, response).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    });
  });
  server.requestTimeout = 5000;
  server.headersTimeout = 10000;
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    const url = new URL(request.url, origin);
    if (request.method === "POST") requireCondition(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "origin_rejected");
    if (request.method === "GET" && url.pathname === "/context") {
      await verify();
      return send(response, 200, { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
        expectedAppElfSha256: context.app_elf_sha256, trust });
    }
    if (request.method === "POST" && url.pathname === "/activate") {
      exactObject(await body(request), []);
      await verify();
      return send(response, 200, { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 });
    }
    if (request.method === "POST" && url.pathname === "/original-budget-context") {
      exactObject(await body(request), []);
      await verify();
      return send(response, 200, { campaignId: await readFrozenCampaign(context) });
    }
    if (request.method === "POST" && url.pathname === "/accounting") {
      await verify();
      await readFrozenCampaign(context);
      await recordQueue;
      return send(response, 200, await saveNoMiningAccounting(root, context, await body(request)));
    }
    if (request.method === "POST" && url.pathname === "/record") {
      const input = await body(request);
      exactObject(input, ["state"]);
      const state = validateState(input.state, context);
      requireCondition(!state.running && state.renewalsConfirmed === 0 && state.status !== "window_loaded", "no_mining_state_required");
      const operation = recordQueue.then(async () => {
        const count = (await readdir(root)).filter((file) => /^no-mining-state-[0-9]{4}\.json$/u.test(file)).length;
        requireCondition(count < 512, "sample_bound");
        await writeNew(resolve(root, `no-mining-state-${String(count + 1).padStart(4, "0")}.json`), {
          schema: "fixed-usb-no-mining-state-v1", context_sha256: digest(JSON.stringify(context)), sequence: count + 1, state });
        return { recorded: true, sequence: count + 1 };
      });
      recordQueue = operation.then(() => undefined, () => undefined);
      return send(response, 200, await operation);
    }
    if (request.method === "POST" && url.pathname === "/diagnostic-export") {
      await verify();
      const validate = operations.validateDiagnostics ?? ((input) => validateDiagnosticExport(input, context.gate_root, options.bun));
      return send(response, 200, await saveDiagnosticExport(root, context, await body(request), validate));
    }
    if (request.method === "GET" && url.pathname === "/supervisor-state") {
      return send(response, 200, { mode: "no-mining", mining_authorized: false, signing_available: false });
    }
    if (request.method === "GET" && url.pathname === "/no-mining-client.mjs") {
      const bytes = await readFile(resolve(SCRIPT_ROOT, "no-mining-client.mjs"));
      requireCondition(digest(bytes) === context.supervisor_client_sha256, "served_asset_drift");
      return send(response, 200, bytes, "text/javascript");
    }
    if (request.method === "GET" && ["/", `/${context.gate_page_relative_path}`, `/${BUNDLE}`].includes(url.pathname)) {
      const isPage = url.pathname !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(context.gate_root, isPage ? context.gate_page_relative_path : BUNDLE));
      requireCondition(digest(bytes) === (isPage ? context.gate_page_sha256 : context.gate_bundle_sha256), "served_asset_drift");
      if (isPage) bytes = Buffer.from(`${bytes.toString("utf8")}\n<script type="module" src="/no-mining-client.mjs"></script>`);
      return send(response, 200, bytes, isPage ? "text/html" : "text/javascript");
    }
    // This allowlist contains no signer, grant delivery, campaign, or work route.
    send(response, 404, { error: "route_unavailable" });
  }
  return server;
}
