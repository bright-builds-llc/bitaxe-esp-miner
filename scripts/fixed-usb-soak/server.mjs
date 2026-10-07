// Local soak supervisor on the Gate's fixed origin. The Gate owns all USB; this server serves the pinned
// page, reviews idle device state, signs one soak after the idle WebSocket proof, records Gate states and
// judges completion. Signed artifacts stay in memory and are delivered once.
import { execFile } from "node:child_process";
import { createServer } from "node:http";
import { appendFile, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { authorityCall, readPoolForSigning } from "../fixed-usb-qualification/authority.mjs";
import { BUNDLE, canonicalBase64, digest, exactObject, fileDigest, missing, nonce, QualificationError, readJson, requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { validateCooling } from "../fixed-usb-qualification/iterative-contract.mjs";
import { validateState } from "../fixed-usb-qualification/judge.mjs";
import { signSoak } from "./authority.mjs";
import { requireIdleSoakLedger, requireQualificationIdle, soakAllowance, SOAK_WORK_GATE_MS } from "./contract.mjs";
import { clockObservations, judgeSoak } from "./judge.mjs";
import { createSoakObserver } from "./observer.mjs";
import { verifyFrozenSoak } from "./preflight.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const MAX_RECORDS = 4096;
const CHALLENGE_TTL_MS = 45000;
/** Terminal samples after the gate closes that the window judge requires before the observer stops. */
const TERMINAL_OBSERVATION_MS = 20000;
const TERMINAL_WAIT_LIMIT_MS = 120000;
/** At least two HTTP polls after the restored state is reported. */
const POST_RESTORATION_OBSERVATION_MS = 5000;

export async function createSoakSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = options.context, now = operations.now ?? Date.now;
  const verify = operations.verifyFrozen ?? (() => verifyFrozenSoak(context, options.authorityDirectory, options.bun));
  await verify();
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  const sign = operations.sign ?? ((operation, input) => authorityCall(context.gate_root, options.authorityDirectory, `sign-${operation}`, input, options.bun));
  const readPool = operations.readPool ?? (() => readPoolForSigning(context.firmware_root, options.poolCredentials));
  const observer = (operations.createObserver ?? createSoakObserver)(root, context, operations);
  const recordsPath = resolve(root, "soak.samples.jsonl");
  let scope, challenge, review, pending, cooling, lastState, records = [], queue = Promise.resolve();

  function challengeFor(kind) {
    requireCondition(scope && !pending, "soak_review_scope");
    review = undefined;
    challenge = { kind, nonce: nonce(), scope: scope.challengeId, expires: now() + CHALLENGE_TTL_MS };
    return challenge.nonce;
  }
  function consumeChallenge(kind, value) {
    const saved = challenge; challenge = undefined;
    requireCondition(saved && saved.kind === kind && saved.nonce === value && saved.scope === scope?.challengeId && saved.expires > now(), "soak_review_challenge");
    return saved;
  }
  function idleState(state) {
    validateState(state, context);
    requireCondition(state.connected && !state.running && !state.failure && state.deviceLeaseInactive, "soak_device_not_idle");
  }

  async function authorize(input) {
    exactObject(input, ["controlSessionBindingSha256"]);
    const saved = review; review = undefined;
    requireCondition(scope && !pending && saved && saved.binding === input.controlSessionBindingSha256 && saved.expires > now(), "soak_fresh_review_required");
    requireCondition(cooling, "soak_cooling_required");
    const idle = await observer.idle();
    requireCondition(idle.passed, "soak_idle_proof_failed");
    await missing(resolve(root, "issued.json"));
    await verify();
    const allowance = soakAllowance(saved.ordinal);
    const stratum = { ...await readPool(), suggestedDifficulty: context.suggested_difficulty };
    const artifacts = await signSoak({ allowance, challengeId: scope.challengeId, binding: saved.binding, stratum, sign });
    requireCondition(saved.expires > now() && Buffer.byteLength(JSON.stringify(artifacts)) <= 65536, "soak_signing_expired");
    await writeNew(resolve(root, "idle-proof.json"), idle);
    await writeNew(resolve(root, "issued.json"), { schema: "fixed-usb-soak-issuance-v1", ordinal: allowance.ordinal, allowance_id: allowance.id,
      context_sha256: digest(JSON.stringify(context)), ledger_before: saved.report, renewals: artifacts.renewals.length, private_payload_persisted: false });
    pending = artifacts;
    return { ready: true };
  }

  async function record(input) {
    exactObject(input, ["state"]);
    validateState(input.state, context);
    requireCondition(records.length < MAX_RECORDS, "sample_bound");
    lastState = input.state;
    if (!lastState.connected || lastState.failure) review = undefined;
    const entry = { sequence: records.length + 1, receivedAtUnixMs: now(), state: lastState };
    records.push(entry);
    await appendFile(recordsPath, JSON.stringify(entry) + "\n", { mode: 0o600 });
    return { recorded: true, sequence: entry.sequence };
  }

  async function runWindowJudge() {
    const clockPath = resolve(root, "gate-clock.json");
    await writeNew(clockPath, { schema: "soak-gate-clock-v1", observations: clockObservations(records) });
    if (await fileDigest(context.soak_judge.path).catch(() => null) !== context.soak_judge.sha256) return null;
    try {
      const { stdout } = await promisify(execFile)(context.soak_judge.path,
        [observer.journalPath, clockPath, context.firmware_commit, context.app_elf_sha256], { env: {}, maxBuffer: 1048576 });
      return JSON.parse(stdout);
    } catch (error) {
      // The judge exits 1 for a failed soak and still prints its numeric judgement.
      try { const value = JSON.parse(String(error.stdout ?? "")); return value.schema === "soak-judge-v1" && Array.isArray(value.failures) ? value : null; } catch { return null; }
    }
  }

  async function complete(input) {
    exactObject(input, ["nonce", "ledger_after", "final_state"]);
    consumeChallenge("completion", input.nonce);
    const issuance = await readJson(resolve(root, "issued.json"));
    const origin = Math.min(...clockObservations(records).map((value) => value.observedUnixMs - value.activeMs));
    // Keep observing past restoration too, so each transport's last sample shows the restored baseline.
    const until = Math.max(Number.isFinite(origin) ? origin + SOAK_WORK_GATE_MS + TERMINAL_OBSERVATION_MS : 0, now() + POST_RESTORATION_OBSERVATION_MS);
    const deadline = now() + TERMINAL_WAIT_LIMIT_MS;
    while (now() < until && now() < deadline) await new Promise((done) => setTimeout(done, 250));
    const observerClean = await observer.stop().then(() => true, () => false);
    await writeNew(resolve(root, "completion-input.json"), { ledger_after: input.ledger_after, final_state: input.final_state });
    const windowJudgement = await runWindowJudge();
    const result = judgeSoak({ context, records, issuance, ledgerAfter: input.ledger_after, finalState: input.final_state, windowJudgement, observerClean });
    await writeNew(resolve(root, "result.json"), { result, sha256: digest(JSON.stringify(result)) });
    return { result: result.result, ordinal: result.ordinal, cumulative_charged_ms: result.cumulative_charged_ms, cleanup_confirmed: result.cleanup_confirmed };
  }

  const routes = {
    "POST /activate": async (input) => {
      exactObject(input, []); requireCondition(!pending, "soak_pending");
      scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
      // A new scope needs its own cooling proof; an earlier session's proof never authorizes signing.
      review = undefined; challenge = undefined; cooling = false; return scope;
    },
    "POST /cooling-review-context": async (input) => { exactObject(input, []); return { nonce: challengeFor("cooling-review"), mode: "iterative" }; },
    "POST /budget-review-context": async (input) => { exactObject(input, []); return { nonce: challengeFor("budget-review"), mode: "soak" }; },
    "POST /completion-context": async (input) => { exactObject(input, []); return { nonce: challengeFor("completion") }; },
    "POST /cooling-review": async (input) => {
      exactObject(input, ["nonce", "proof", "restoration", "budget_before", "budget_after", "state"]);
      consumeChallenge("cooling-review", input.nonce);
      validateCooling(input.proof, input.restoration);
      requireQualificationIdle(input.budget_before); requireQualificationIdle(input.budget_after);
      idleState(input.state);
      const { nonce: omitted, ...value } = input;
      await writeNew(resolve(root, "cooling.json"), { schema: "fixed-usb-soak-cooling-v1", context_sha256: digest(JSON.stringify(context)), ...value });
      cooling = true; lastState = input.state;
      return { cooling_review_saved: true, review_file: "cooling.json" };
    },
    "POST /observer/start": async (input) => {
      exactObject(input, ["endpoint", "state"]);
      idleState(input.state);
      const endpoint = input.endpoint;
      requireCondition(endpoint?.schema === "worker-telemetry-endpoint-v1" && canonicalBase64(endpoint.controlSessionBindingSha256, 32), "observer_endpoint");
      return observer.start({ ipv4: endpoint.ipv4, httpPort: endpoint.httpPort });
    },
    "GET /observer/idle-proof": async () => observer.idle(),
    "POST /budget-review": async (input) => {
      exactObject(input, ["nonce", "report", "controlSessionBindingSha256", "state"]);
      const saved = consumeChallenge("budget-review", input.nonce);
      const ordinal = requireIdleSoakLedger(input.report);
      requireCondition(canonicalBase64(input.controlSessionBindingSha256, 32), "soak_review_binding");
      idleState(input.state);
      review = { binding: input.controlSessionBindingSha256, expires: saved.expires, report: input.report, ordinal };
      lastState = input.state;
      return { budget_review_saved: true };
    },
    "POST /authorization-context": authorize,
    "GET /window-artifacts": async () => {
      requireCondition(pending, "soak_artifacts_unavailable");
      const issuance = await readJson(resolve(root, "issued.json"));
      await writeNew(resolve(root, "consumed.json"), { ordinal: issuance.ordinal, delivery_attempted: true });
      const artifacts = pending; pending = undefined; return artifacts;
    },
    "POST /record": record,
    "POST /completion-review": complete,
    "GET /supervisor-state": async () => ({ mode: "soak", waiting_for_human_has_no_deadline: true, private_payload_pending_in_memory: Boolean(pending),
      observer_started: observer.started() !== undefined, records: records.length }),
  };

  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    const path = new URL(request.url, origin).pathname;
    if (request.method === "POST" || path === "/window-artifacts") requireCondition(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "origin_rejected");
    const input = request.method === "POST" ? await body(request) : undefined;
    if (path === "/context" && request.method === "GET") return send(response, 200, { expectedGateCommit: context.gate_commit,
      expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256, trust, soakQualification: true });
    const maybeRoute = routes[`${request.method} ${path}`];
    if (maybeRoute) return send(response, 200, await maybeRoute(input));
    if (path === "/supervisor-client.mjs") return send(response, 200, await readFile(resolve(SCRIPT_ROOT, "client.mjs")), "text/javascript");
    if (["/", `/${context.gate_page_relative_path}`, `/${BUNDLE}`].includes(path)) {
      const isPage = path !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(context.gate_root, isPage ? context.gate_page_relative_path : BUNDLE));
      requireCondition(digest(bytes) === (isPage ? context.gate_page_sha256 : context.gate_bundle_sha256), "served_asset_drift");
      if (isPage) bytes = Buffer.from(bytes.toString("utf8") + '\n<script type="module" src="/supervisor-client.mjs"></script>');
      return send(response, 200, bytes, isPage ? "text/html" : "text/javascript");
    }
    send(response, 404, { error: "route_unavailable" });
  }

  const server = createServer((request, response) => {
    const operation = () => handle(request, response);
    const result = request.method === "POST" ? queue.then(operation) : operation();
    if (request.method === "POST") queue = Promise.resolve(result).catch(() => undefined);
    Promise.resolve(result).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    });
  });
  // Completion waits up to two minutes for terminal samples, so requests get a longer bound than reviews.
  server.requestTimeout = 180000; server.headersTimeout = 10000;
  server.closeSoakResources = () => observer.stop().catch(() => undefined);
  server.on("close", () => { scope = undefined; pending = undefined; review = undefined; challenge = undefined; });
  return server;
}
