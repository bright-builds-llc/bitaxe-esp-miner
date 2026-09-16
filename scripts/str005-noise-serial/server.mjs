import { createServer } from "node:http";
import { resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { exactObject, nonce } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { configuration, serveAsset } from "./server-assets.mjs";
import { saveFirstFailure } from "./first-failure.mjs";
import { loadContext, recheckNative, verifyEffectInputs } from "./context.mjs";
import { baseline, healthy, readJournal, readNoiseJournal, recordNoise, recordState, saveAccounting } from "./journal.mjs";
import { claimInstall, reviewInstall, recordCycle, claimProbe, completeProbe } from "./install.mjs";
import { startFixture, selectInterface } from "./fixture-owner.mjs";
import { parseStartV2 } from "./contract-v2.mjs";
import { parseNoiseStatusV2 } from "./device-v2.mjs";
import { canonical, check, digest, proof, readJson, writeNew } from "./files.mjs";
import { processSnapshot } from "./host-resources.mjs";

export async function createSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = await loadContext(root, { operations });
  await recheckNative(context, operations);
  await writeNew(resolve(root, "server.claim.json"), { schema: "noise-serial-server-claim-v2", contextSha256: digest(JSON.stringify(context)) });
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  const now = operations.now ?? (() => Math.floor(performance.now()));
  let phase = "before", failed = false, firstFailure = null, maybeFixture = null, queue = Promise.resolve();
  let failureWrite = Promise.resolve(), startReserved = false;
  function fail(code, maybeCause, maybeProvenance) {
    if (failed) return;
    failed = true; firstFailure = /^[a-z_]+$/u.test(code) ? code : "noise_operation_failed";
    failureWrite = saveFirstFailure(root, context, firstFailure, now(), maybeCause, maybeProvenance);
  }
  const verify = () => verifyEffectInputs(context, operations);
  function ready() { check(!failed, "noise_terminal_failure"); }
  const config = (mode) => configuration(context, mode, trust);
  const server = createServer((request, response) => {
    handle(request, response).catch(async (error) => {
      fail(error.code ?? "noise_request_rejected");
      try { await failureWrite; } catch { response.destroy(); return; }
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: firstFailure });
    });
  });
  server.requestTimeout = 60000; server.headersTimeout = 10000;
  function enqueue(operation) { const result = queue.then(operation); queue = result.then(() => undefined, () => undefined); return result; }
  async function lastBaseline(closed = false) {
    const row = (await readJournal(root, context)).at(-1); check(row, "noise_baseline_missing"); baseline(row.state, closed); return row;
  }
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`, url = new URL(request.url, origin);
    check(request.headers.host === `127.0.0.1:${server.address().port}`, "noise_host_rejected");
    if (request.method === "POST") check(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "noise_origin_rejected");
    if (request.method === "GET" && url.pathname === "/context") { await verify(); return send(response, 200, config(phase)); }
    if (request.method === "GET" && url.pathname === "/supervisor-state") return send(response, 200, { phase, failed, signing_available: false, mining_authorized: false });
    if (request.method === "GET" && await serveAsset(root, context, url.pathname, response)) return;
    if (request.method !== "POST") return send(response, 404, { error: "noise_route_unavailable" });
    const input = await body(request);
    const result = await enqueue(async () => {
      if (url.pathname === "/record") {
        exactObject(input, ["state"]);
        const saved = await recordState(root, context, phase, input.state, now());
        if (input.state.failure) fail("noise_browser_failed");
        if (input.state.status === "restoration_pending") fail("noise_restoration_pending",
          { stage: "cleanup", category: "cleanup", detail: "resource_unreleased" }, { kind: "browser", sequence: saved.sequence });
        return saved;
      }
      if (url.pathname === "/noise/record") {
        exactObject(input, ["status"]); const saved = await recordNoise(root, context, input.status, now());
        if (input.status.job?.firstFailure || (input.status.job?.terminal && input.status.job.terminal.outcome !== "accepted")) {
          const typed = input.status.job.firstFailure;
          fail("noise_device_failed", typed ? { stage: typed.stage, category: typed.category, detail: typed.detail } : undefined,
            { kind: "device", sequence: saved.sequence, atDeviceUs: typed?.atUs ?? null });
        }
        return saved;
      }
      if (url.pathname === "/client-failure") {
        exactObject(input, ["phase", "code"]);
        check(["accounting_before", "fixture_ready", "start_admission", "start_request", "status_poll", "completion", "restoration", "accounting_after", "journal", "probe", "configuration"].includes(input.phase) &&
          ["serial_timeout", "serial_closed", "restoration_pending", "shape", "binding_mismatch", "observation_horizon", "record_rejected", "unexpected_terminal", "operation_failed"].includes(input.code), "noise_client_failure_shape");
        const cause = input.phase === "start_request" ? { stage: "admission", category: "authority_lost", detail: "delivery_ambiguous" } :
          input.phase === "restoration" ? { stage: "cleanup", category: "cleanup", detail: "resource_unreleased" } :
            { stage: "evidence", category: "evidence_incomplete", detail: "missing" };
        fail("noise_client_failed", cause, { kind: "browser-client", phase: input.phase, code: input.code, clientSha256: context.client_sha256 });
        return { recorded: true };
      }
      // Restoration/accounting remain available after failure, but no new effects.
      if (url.pathname === "/activate") { exactObject(input, []); await verify(); return { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 }; }
      if (url.pathname === "/accounting-context") { exactObject(input, []); await verify(); return { campaignId: context.original_campaign_id }; }
      if (url.pathname === "/accounting") { await verify(); return saveAccounting(root, context, input); }
      if (url.pathname === "/restoration/context") { exactObject(input, []); return { attemptId: context.attempt_id }; }
      if (url.pathname === "/restoration") {
        exactObject(input, ["status", "state"]);
        const rows = await readJournal(root, context), last = rows.at(-1), status = parseNoiseStatusV2(input.status);
        baseline(input.state, false);
        check(last && canonical(last.state) === canonical(input.state), "noise_restoration_unpublished");
        let boundary;
        try { boundary = (await proof(root, "diagnostic-complete.json")).value.observedSequence; }
        catch (error) { if (error.code !== "ENOENT") throw error; boundary = (await proof(root, "accounting-before.json")).value.observedSequence; }
        const closed = rows.find((row) => row.sequence > boundary && row.sequence < last.sequence &&
          row.state.status === "closed" && row.state.serialOwnershipReleased && !row.state.connected);
        check(closed && status.job?.attemptId === context.attempt_id && status.job.terminal !== null && status.job.resources.socketState === "closed" &&
          status.job.resources.workerState === "quiescent" && status.job.resources.volatileInputsDisposed &&
          status.observation.bootOrdinal === status.job.bootOrdinal && status.observation.transportEpoch !== status.job.transportEpoch,
        "noise_fresh_restoration_required");
        await writeNew(resolve(root, "restoration.json"), { schema: "noise-serial-restoration-v2", contextSha256: digest(JSON.stringify(context)),
          closedSequence: closed.sequence, observedSequence: last.sequence, status: input.status, state: input.state });
        return { restoration_recorded: true };
      }
      ready();
      if (url.pathname === "/install/claim") { exactObject(input, ["index"]); await verify(); ready(); return claimInstall(root, context, input.index, () => failed, operations); }
      if (url.pathname === "/install/review") {
        exactObject(input, ["index"]); await verify(); ready();
        const result = await reviewInstall(root, context, input.index, now(), operations); ready();
        phase = "candidate"; return result;
      }
      if (url.pathname === "/candidate-context") { exactObject(input, []); check(phase === "candidate", "noise_install_required"); return config("candidate"); }
      if (url.pathname === "/probe/claim") { exactObject(input, ["index", "status"]); return claimProbe(root, context, input.index, input.status, now()); }
      if (url.pathname === "/probe/complete") return completeProbe(root, context, input, now());
      if (url.pathname === "/cycle") { exactObject(input, ["index"]); return recordCycle(root, context, input.index); }
      if (url.pathname === "/fixture/start") {
        exactObject(input, ["status"]); await verify(); ready(); check(phase === "candidate", "noise_candidate_required");
        for (let index = 1; index <= 4; index++) await proof(root, `cycle-${index}.json`);
        await proof(root, "accounting-before.json"); healthy((await lastBaseline()).state);
        const status = parseNoiseStatusV2(input.status); check(status.state === "idle" && status.observation.wifiConnected && status.observation.stationIpv4 !== null, "noise_network_missing");
        check(maybeFixture === null, "noise_fixture_consumed");
        maybeFixture = await startFixture(root, context, status.observation.stationIpv4, fail, operations); ready();
        return { fixture_ready: true };
      }
      if (url.pathname === "/start/claim") {
        exactObject(input, ["status"]); check(!startReserved && maybeFixture, "noise_start_consumed");
        await verify(); ready(); maybeFixture.alive(); healthy((await lastBaseline()).state); ready();
        check((await readNoiseJournal(root, context)).length === 0, "noise_unexpected_prior_job");
        const status = parseNoiseStatusV2(input.status);
        check(status.state === "idle" && status.observation.stationIpv4 !== null && status.observation.wifiConnected, "noise_start_baseline");
        const selected = selectInterface(status.observation.stationIpv4, operations.networkInterfaces?.());
        check(selected.address === maybeFixture.ready.listenIpv4, "noise_network_changed");
        const start = parseStartV2({ schema: "worker-noise-diagnostic-start-v2", attemptId: context.attempt_id,
          expectedBootOrdinal: status.observation.bootOrdinal, networkObservedAtUs: status.observation.observedAtUs,
          fixtureIpv4: maybeFixture.ready.listenIpv4, fixturePort: maybeFixture.ready.listenPort, authorityPublicKey: maybeFixture.ready.authorityPublicKey });
        startReserved = true;
        ready(); maybeFixture.alive();
        await writeNew(resolve(root, "start.claim.json"), { schema: "noise-serial-start-claim-v2", contextSha256: digest(JSON.stringify(context)),
          atHostMs: now(), observedSequence: (await readJournal(root, context)).at(-1).sequence, inputSha256: digest(canonical(start)), start, observation: status });
        ready(); maybeFixture.alive(); return start;
      }
      if (url.pathname === "/diagnostic/complete") {
        exactObject(input, []); check(startReserved && maybeFixture, "noise_start_missing");
        const status = (await readNoiseJournal(root, context)).at(-1)?.status;
        check(status?.job?.terminal?.outcome === "accepted", "noise_positive_terminal_missing");
        const terminal = await maybeFixture.finish(); ready();
        await writeNew(resolve(root, "diagnostic-complete.json"), { schema: "noise-serial-complete-v2", contextSha256: digest(JSON.stringify(context)),
          atHostMs: now(), observedSequence: (await readJournal(root, context)).at(-1).sequence,
          statusSha256: digest(canonical(status)), fixtureSha256: digest(canonical(terminal)) });
        return { diagnostic_complete: true };
      }
      check(false, "noise_route_unavailable");
    });
    send(response, 200, result);
  }
  let resolveReady, rejectReady;
  server.qualificationReady = new Promise((resolvePromise, rejectPromise) => { resolveReady = resolvePromise; rejectReady = rejectPromise; });
  server.once("listening", () => {
    (async () => {
      const rows = await (operations.processSnapshot ?? processSnapshot)(), owner = rows.find((row) => row.pid === process.pid);
      check(owner, "noise_supervisor_identity");
      await writeNew(resolve(root, "server-owner.json"), { schema: "noise-serial-server-owner-v2", contextSha256: digest(JSON.stringify(context)), owner,
        origin: `http://127.0.0.1:${server.address().port}`, port: server.address().port, atHostMs: now() });
    })().then(resolveReady).catch((error) => {
      fail("noise_supervisor_identity_failed");
      server.close(); server.closeAllConnections(); rejectReady(error);
    });
  });
  server.closeQualificationResources = async () => {
    await queue; if (maybeFixture) await maybeFixture.close(); await failureWrite;
  };
  return server;
}
