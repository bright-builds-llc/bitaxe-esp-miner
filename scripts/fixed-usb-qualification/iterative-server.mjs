import { createServer } from "node:http";
import { appendFile, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall, readPoolForSigning, signAttempt } from "./authority.mjs";
import { BUNDLE, canonicalBase64, contextPage, digest, exactObject, missing, nonce, protectedPath, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { body, send } from "./http.mjs";
import { validateState } from "./judge.mjs";
import { requireSuccessorBaseline } from "./successor.mjs";
import { requireIdleLedger, validateCooling } from "./iterative-contract.mjs";
import { validateIterativeContext } from "./iterative-preflight.mjs";
import { finishIterative } from "./iterative-judge.mjs";
import { verifyFrozen } from "./preflight.mjs";
import { saveDiagnosticExport, validateDiagnosticExport } from "./diagnostic-export.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function createIterativeSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = options.context, attempt = context.qualification_attempt;
  const verify = operations.verifyFrozen ?? (() => verifyFrozen(context, options.authorityDirectory, options.bun, {}, root));
  await verify(); await validateIterativeContext(root, context);
  const now = operations.now ?? Date.now, page = contextPage(context);
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  const sign = operations.sign ?? ((operation, input) => authorityCall(context.gate_root, options.authorityDirectory, `sign-${operation}`, input, options.bun));
  const readPool = operations.readPool ?? (() => readPoolForSigning(context.firmware_root, options.poolCredentials));
  const validateDiagnostics = operations.validateDiagnostics ?? ((input) => validateDiagnosticExport(input, context.gate_root, options.bun));
  let scope, challenge, review, pending, lastState, sequence = 0, lastRecord, queue = Promise.resolve();
  const samplesPath = resolve(root, "iterative.samples.jsonl");
  try {
    await protectedPath(samplesPath);
    const previous = (await readFile(samplesPath, "utf8")).trim().split("\n").filter(Boolean).map((line) => JSON.parse(line));
    requireCondition(previous.length <= 512 && previous.every((record, index) => record.sequence === index + 1), "sample_history_shape");
    for (const record of previous) validateState(record.state, context);
    sequence = previous.length; lastRecord = previous.at(-1); lastState = lastRecord?.state;
  } catch (error) { if (error.code !== "ENOENT") throw error; }
  const server = createServer((request, response) => {
    const operation = () => handle(request, response);
    const result = request.method === "POST" ? queue.then(operation) : operation();
    if (request.method === "POST") queue = Promise.resolve(result).catch(() => undefined);
    Promise.resolve(result).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    });
  });
  server.requestTimeout = 5000; server.headersTimeout = 10000;
  server.on("close", () => { scope = undefined; pending = undefined; review = undefined; challenge = undefined; });
  async function safeState(state) { validateState(state, context); await requireSuccessorBaseline(root, context, state); }
  function challengeFor(kind) {
    requireCondition(scope && !pending, "iterative_review_scope");
    review = undefined;
    challenge = { kind, nonce: nonce(), scope: scope.challengeId, expires: now() + 45000 };
    return { nonce: challenge.nonce, mode: "iterative" };
  }
  function consumeChallenge(kind, value) {
    const saved = challenge; challenge = undefined;
    requireCondition(saved && saved.kind === kind && saved.nonce === value && saved.scope === scope?.challengeId && saved.expires > now(), "iterative_review_challenge");
    return saved;
  }
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    const path = new URL(request.url, origin).pathname;
    if (request.method === "POST" || path === "/window-artifacts") requireCondition(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "origin_rejected");
    const input = request.method === "POST" ? await body(request) : undefined;
    if (path === "/context" && request.method === "GET") return send(response, 200, {
      expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256, trust });
    if (path === "/activate" && input) {
      exactObject(input, []); requireCondition(!pending, "iterative_pending");
      scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
      review = undefined; challenge = undefined; return send(response, 200, scope);
    }
    if (["/budget-review-context", "/cooling-review-context", "/completion-context"].includes(path) && input) {
      exactObject(input, []);
      const kind = path.slice(1).replace("-context", "");
      const value = challengeFor(kind);
      return send(response, 200, path === "/completion-context" ? { nonce: value.nonce, campaignId: context.original_campaign_id } : value);
    }
    if (path === "/cooling-review" && input) {
      exactObject(input, ["nonce", "proof", "restoration", "budget_before", "budget_after", "state"]);
      consumeChallenge("cooling-review", input.nonce);
      validateCooling(input.proof, input.restoration);
      requireIdleLedger(input.budget_before, attempt.ordinal, context.expected_charged_ms);
      requireIdleLedger(input.budget_after, attempt.ordinal, context.expected_charged_ms);
      await safeState(input.state);
      const { nonce: omitted, ...value } = input;
      await writeNew(resolve(root, "cooling.json"), { schema: "worker-iterative-cooling-v1", context_sha256: digest(JSON.stringify(context)), ...value });
      lastState = input.state;
      return send(response, 200, { cooling_review_saved: true, review_file: "cooling.json" });
    }
    if (path === "/budget-review" && input) {
      exactObject(input, ["nonce", "report", "controlSessionBindingSha256", "state"]);
      const saved = consumeChallenge("budget-review", input.nonce);
      requireIdleLedger(input.report, attempt.ordinal, context.expected_charged_ms);
      requireCondition(canonicalBase64(input.controlSessionBindingSha256, 32), "iterative_review_binding");
      await safeState(input.state);
      review = { binding: input.controlSessionBindingSha256, expires: saved.expires, report: input.report };
      lastState = input.state;
      return send(response, 200, { budget_review_saved: true });
    }
    if (path === "/authorization-context" && input) {
      exactObject(input, ["controlSessionBindingSha256"]);
      const saved = review; review = undefined;
      requireCondition(scope && !pending && saved && saved.binding === input.controlSessionBindingSha256 && saved.expires > now(), "iterative_fresh_review_required");
      await missing(resolve(root, "issued.json")); await safeState(lastState); await validateIterativeContext(root, context); await verify();
      await protectedPath(resolve(root, "cooling.json"));
      const cooling = await readJson(resolve(root, "cooling.json"));
      requireCondition(cooling.context_sha256 === digest(JSON.stringify(context)), "iterative_cooling_context");
      validateCooling(cooling.proof, cooling.restoration); await safeState(cooling.state);
      requireIdleLedger(cooling.budget_before, attempt.ordinal, context.expected_charged_ms);
      requireIdleLedger(cooling.budget_after, attempt.ordinal, context.expected_charged_ms);
      const artifacts = await signAttempt({ attempt, challengeId: scope.challengeId, binding: saved.binding, stratum: await readPool(), sign });
      requireCondition(saved.expires > now() && Buffer.byteLength(JSON.stringify(artifacts)) <= 65536, "iterative_signing_expired");
      await writeNew(resolve(root, "issued.json"), { schema: "worker-iterative-issuance-v1", ordinal: attempt.ordinal,
        context_sha256: digest(JSON.stringify(context)), ledger_before: saved.report, private_payload_persisted: false });
      pending = artifacts; return send(response, 200, { ready: true });
    }
    if (path === "/window-artifacts" && request.method === "GET") {
      requireCondition(pending, "iterative_artifacts_unavailable");
      await writeNew(resolve(root, "consumed.json"), { ordinal: attempt.ordinal, delivery_attempted: true });
      const artifacts = pending; pending = undefined; return send(response, 200, artifacts);
    }
    if (path === "/record" && input) {
      exactObject(input, ["state"]); validateState(input.state, context); lastState = input.state;
      if (!lastState.connected || lastState.failure) review = undefined;
      requireCondition(sequence < 512, "sample_bound");
      sequence += 1; lastRecord = { sequence, state: lastState };
      const file = resolve(root, "iterative.samples.jsonl");
      if (sequence > 1) await protectedPath(file);
      await appendFile(file, JSON.stringify(lastRecord) + "\n", { mode: 0o600 });
      if (lastState.failure) {
        const failurePath = resolve(root, "first-failure.json");
        try { await protectedPath(failurePath); }
        catch (error) {
          if (error.code !== "ENOENT") throw error;
          await writeNew(failurePath, { schema: "worker-iterative-first-failure-v1", ordinal: attempt.ordinal, sequence,
            browser: lastState.failure, serial: lastState.serialFailureCategory ?? null, admission: lastState.admissionFailureStage ?? null });
        }
      }
      return send(response, 200, { recorded: true, sequence });
    }
    if (path === "/fault" && input) {
      exactObject(input, ["kind", "running", "visibility", "heartbeatSuppressed", "generation"]);
      const expected = attempt.purpose === "foreground_loss" ? "visibility_hidden" : attempt.purpose === "heartbeat_loss" ? "heartbeats_suppressed" : undefined;
      requireCondition(expected && input.kind === expected && input.running === true &&
        (expected === "visibility_hidden" ? input.visibility === "hidden" : input.heartbeatSuppressed === true) &&
        lastRecord?.state.running && lastRecord.state.qualification?.generation === input.generation &&
        lastRecord.state.qualification.work_gate_remaining_ms > 3000, "iterative_fault_evidence");
      await writeNew(resolve(root, "iterative.fault.json"), { kind: input.kind, generation: input.generation, after_sequence: sequence });
      return send(response, 200, { recorded: true });
    }
    if (path === "/completion-review" && input) {
      exactObject(input, ["nonce", "ledger_after", "original_budget", "final_state"]); consumeChallenge("completion", input.nonce);
      const issuance = await readJson(resolve(root, "issued.json"));
      const file = resolve(root, "completion-input.json");
      await writeNew(file, { ledger_before: issuance.ledger_before, ledger_after: input.ledger_after, original_budget: input.original_budget, final_state: input.final_state });
      return send(response, 200, await finishIterative(root, context, file));
    }
    if (path === "/diagnostic-export" && input) {
      await verify(); return send(response, 200, await saveDiagnosticExport(root, context, input, validateDiagnostics));
    }
    if (path === "/supervisor-state" && request.method === "GET") return send(response, 200, {
      mode: "iterative", purpose: attempt.purpose, ordinal: attempt.ordinal, waiting_for_human_has_no_deadline: true, private_payload_pending_in_memory: Boolean(pending) });
    if (path === "/supervisor-client.mjs") return send(response, 200, await readFile(resolve(SCRIPT_ROOT, "client.mjs")), "text/javascript");
    if (["/", `/${page}`, `/${BUNDLE}`].includes(path)) {
      const isPage = path !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(context.gate_root, isPage ? page : BUNDLE));
      requireCondition(digest(bytes) === (isPage ? context.gate_page_sha256 : context.gate_bundle_sha256), "served_asset_drift");
      if (isPage) bytes = Buffer.from(bytes.toString("utf8") + '\n<script type="module" src="/supervisor-client.mjs"></script>');
      return send(response, 200, bytes, isPage ? "text/html" : "text/javascript");
    }
    send(response, 404, { error: "route_unavailable" });
  }
  return server;
}
