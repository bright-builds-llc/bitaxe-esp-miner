import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { admitTrust, nonce } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { loadContext, recheckNative, verifyEffectInputs } from "./context.mjs";
import { requireAuthorityOption } from "./contract.mjs";
import { failureRecorder } from "./failure.mjs";
import { startFixture } from "./fixture-owner.mjs";
import { baseline, createJournal, healthy, saveAccounting } from "./journal.mjs";
import { claimInstall, claimProbe, completeProbe, recordCycle, reviewInstall } from "./install.mjs";
import { createObserverRoutes } from "./observer.mjs";
import { createSupervisorCleanup } from "./supervisor-cleanup.mjs";
import { parseStatus } from "./device.mjs";
import { createSigner } from "./signing.mjs";
import { createShareRoutes } from "./share-routes.mjs";
import { configuration, serveAsset } from "./server-assets.mjs";
import { check, object, sha256 } from "./values.mjs";

/** This server owns host resources only; all device control stays on the admitted Gate page. */
export async function createSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = await loadContext(root, { operations });
  requireAuthorityOption(context.scope, options);
  await recheckNative(context, operations);
  const contextSha256 = sha256(JSON.stringify(context)), now = operations.now ?? (() => Math.floor(performance.now()));
  await writeNew(resolve(root, "server.claim.json"), { schema: "str005-v2-server-claim-v1", contextSha256 });
  const failures = failureRecorder(root, context, now), journal = await createJournal(root, context);
  const trust = JSON.parse(await readFile(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"), "utf8"));
  let phase = "before", maybeFixture = null, maybeFixtureStart = null, maybeScope = null, queue = Promise.resolve(), stopping = false;
  const verify = () => verifyEffectInputs(context, operations);
  const ready = () => check(!failures.failed() && !stopping, "v2_terminal_failure");
  let maybeSigner = null;
  if (context.scope === "share") {
    maybeSigner = await createSigner(root, context, options.authorityDirectory, failures.failed, operations);
    admitTrust(trust, await maybeSigner("public-trust"));
  }
  const observer = createObserverRoutes(root, context, { now, ready, failed: failures.failed, fail: failures.fail, journal, operations });
  const share = createShareRoutes(root, context, { now, ready: () => { ready(); check(phase === "candidate", "v2_candidate_required"); },
    failed: failures.failed, verify, journal, fixture: () => maybeFixture, activeScope: () => maybeScope,
    requireObserver: (observation, binding) => {
      observer.alive(); const admitted = observer.binding();
      check(admitted && admitted.bootOrdinal === observation.bootOrdinal && admitted.workerGeneration === observation.workerGeneration &&
        admitted.controlSessionBindingSha256 === binding, "v2_observer_session_changed");
    },
    sign: async (operation, input) => { check(maybeSigner !== null, "v2_signing_scope"); return maybeSigner(operation, input); } });
  const runtime = { root, context, contextSha256, now, ready, verify, journal, failures, observer,
    fixture: () => maybeFixture, phase: () => phase };
  const { createExecutionRoutes } = await import("./execution-routes.mjs");
  const execution = createExecutionRoutes(runtime);
  const server = createServer((request, response) => {
    const result = queue.then(() => handle(request, response));
    queue = result.then(() => undefined, async (error) => {
      failures.fail(error.code ?? "v2_request_rejected");
      try { await failures.settled(); }
      catch { response.destroy(); return; }
      if (response.destroyed || response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: failures.current().code });
    });
  });
  server.requestTimeout = 60000; server.headersTimeout = 10000;
  async function handle(request, response) {
    observer.observeFailure();
    const origin = `http://127.0.0.1:${server.address().port}`, path = new URL(request.url, origin).pathname;
    check(request.headers.host === `127.0.0.1:${server.address().port}`, "v2_host_rejected");
    if (request.method === "POST" || path === "/window-artifacts") check(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "v2_origin_rejected");
    if (request.method === "GET" && path === "/context") return send(response, 200, configuration(context, phase, trust));
    if (request.method === "GET" && path === "/supervisor-state") return send(response, 200, { scope: context.scope, phase, failed: failures.failed() });
    if (request.method === "GET" && await serveAsset(root, context, path, response)) return;
    if (request.method !== "POST" && path !== "/window-artifacts") return send(response, 404, { error: "v2_route_unavailable" });
    const input = request.method === "POST" ? await body(request) : undefined;
    const result = await route(path, input, request.method);
    send(response, 200, result);
  }
  async function route(path, input, method) {
    if (path === "/record") {
      object(input, ["state"]); const saved = await journal.state(phase, input.state, now());
      if (input.state.failure && !execution.expectedFault(input.state)) failures.fail("v2_browser_failed", null, saved.sequence);
      if (!input.state.connected) share.resetScope();
      return saved;
    }
    if (path === "/device/record") {
      object(input, ["status"]); const status = parseStatus(input.status), saved = await journal.device(status, now());
      if (status.record.firstFailure) failures.fail("v2_device_failed", status.record.firstFailure, saved.sequence);
      return saved;
    }
    if (path === "/client-failure") {
      object(input, ["phase", "code"]);
      check(["configuration", "accounting", "probe", "observer", "fixture", "admission", "start", "poll", "fault", "restoration", "journal"].includes(input.phase) &&
        ["timeout", "closed", "restoration_pending", "shape", "observation_horizon", "unexpected_terminal", "operation_failed"].includes(input.code), "v2_client_failure_shape");
      failures.fail(`v2_client_${input.phase}_${input.code}`); return { recorded: true };
    }
    if (path === "/activate") {
      object(input, []); share.resetScope();
      maybeScope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 };
      return maybeScope;
    }
    if (path === "/accounting-context") { object(input, []); return { campaignId: context.original_campaign_id }; }
    if (path === "/accounting") return saveAccounting(root, context, input, journal.lastState());
    if (path === "/cleanup/private-context") {
      object(input, []); check(maybeFixture !== null, "v2_fixture_not_started");
      return { schema: "str005-v2-cleanup-runtime-v1", contextSha256, scope: context.scope, attemptId: context.attemptId,
        fixtureInstanceId: maybeFixture.ready.instanceId, fixturePort: maybeFixture.ready.listenPort };
    }
    const recorded = await execution.handle(path, input);
    if (recorded !== undefined) return recorded;
    if (path === "/observer/finish") return observer.handle(path, input);
    ready();
    if (path === "/install/claim") { object(input, ["index"]); await verify(); ready(); return claimInstall(root, context, input.index, failures.failed, operations); }
    if (path === "/install/review") {
      object(input, ["index"]); await verify(); ready();
      const result = await reviewInstall(root, context, input.index, now(), operations); ready(); phase = "candidate"; return result;
    }
    if (path === "/candidate-context") { object(input, []); check(phase === "candidate", "v2_install_required"); return configuration(context, phase, trust); }
    if (path === "/probe/claim") { object(input, ["index", "status"]); return claimProbe(root, context, input.index, input.status, now()); }
    if (path === "/probe/complete") return completeProbe(root, context, input, now());
    if (path === "/cycle") { object(input, ["index"]); return recordCycle(root, context, input.index); }
    const observed = await observer.handle(path, input); if (observed !== undefined) return observed;
    if (path === "/fixture/start") {
      object(input, ["status"]); await verify(); ready(); check(phase === "candidate" && maybeFixture === null, "v2_fixture_admission");
      for (let index = 1; index <= 4; index++) await proof(root, `cycle-${index}.json`);
      await proof(root, "accounting-before.json"); healthy(journal.lastState()?.state);
      const status = parseStatus(input.status);
      check(status.scope === context.scope && status.state === "idle" && status.observation.wifiConnected && status.observation.stationIpv4 !== null,
        "v2_fixture_network");
      if (context.scope === "share") observer.alive();
      ready();
      maybeFixtureStart = startFixture(root, context, status.observation.stationIpv4, failures.fail, operations);
      maybeFixture = await maybeFixtureStart;
      // A stop may arrive while readiness/ownership is still being established.
      // Join and close the actual returned owner before rejecting its admission.
      if (stopping) await maybeFixture.close();
      ready();
      return { fixture_ready: true, attemptId: context.attemptId };
    }
    const shared = await share.handle(path, input, method); if (shared !== undefined) return shared;
    check(false, "v2_route_unavailable");
  }
  let resolveReady, rejectReady;
  server.qualificationReady = new Promise((done, reject) => { resolveReady = done; rejectReady = reject; });
  server.once("listening", () => {
    (async () => {
      const rows = await (operations.processSnapshot ?? processSnapshot)(), owner = rows.find((row) => row.pid === process.pid);
      check(owner, "v2_server_owner");
      await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256, owner,
        origin: `http://127.0.0.1:${server.address().port}`, port: server.address().port, atHostMs: now() });
    })().then(resolveReady).catch((error) => { failures.fail("v2_server_identity_failed"); server.close(); server.closeAllConnections(); rejectReady(error); });
  });
  server.closeQualificationResources = createSupervisorCleanup({
    stop() { stopping = true; server.close(); server.closeAllConnections(); },
    async closeFixture() {
      // Startup owns and reaps its own failed child; a successful pending start
      // must be joined before close so it cannot create an owner after cleanup.
      const fixture = maybeFixture ?? (maybeFixtureStart ? await maybeFixtureStart : null);
      if (fixture) await fixture.close();
    },
    closeObserver: () => observer.finish(), settleQueue: () => queue,
    fail: failures.fail, settled: failures.settled,
  });
  return server;
}
