// Local restoration supervisor on the Gate's fixed origin. The Gate page owns all USB; this server serves the
// pinned restoration page, keeps one scope per scenario (and across reboot → authorization_negatives), signs
// unbudgeted windows in memory, delivers each artifact once, drives the physical checkpoints through the
// presence watcher, records the closed page journal privately and judges each scenario.
import { createServer } from "node:http";
import { appendFile, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall, readPoolForSigning } from "../fixed-usb-qualification/authority.mjs";
import { canonicalBase64, digest, exactObject, nonce, QualificationError, readJson, requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
import { signRestorationWindow } from "./authority.mjs";
import { activate, admitSigning, createCampaign, deliverWindow, failCampaign, finishScenario, markSegment, observeRecord, recordSigned, replayArtifact,
  requireRunning } from "./campaign.mjs";
import { admitReconnect, checkpointFacts, operatorReady, view } from "./checkpoint.mjs";
import { BUNDLE, HOST_EVENTS, RESULT_SCHEMA, SCENARIO_RESULT_SCHEMA, SCENARIOS, scopeGroup, TOKEN } from "./contract.mjs";
import { supervisorState } from "./guidance.mjs";
import { judgeScenario } from "./judge.mjs";
import { verifyFrozenRestoration } from "./preflight.mjs";
import { parseCompletionReview, parseRecord, requireSecretsAbsent } from "./records.mjs";
import { createPhysicalOwner } from "./physical.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const COMPLETION_TTL_MS = 120000;

/** Host campaign events carry only closed names, scenario names, tokens, counts and booleans. */
function hostRow(sequence, atUnixMs, event, scenario, detail) {
  requireCondition(HOST_EVENTS.includes(event) && Object.values(detail).every((value) =>
    typeof value === "boolean" || Number.isSafeInteger(value) || (typeof value === "string" && TOKEN.test(value))), "host_event");
  return { sequence, atUnixMs, event, scenario, ...detail };
}

/** The private sealed result: closed categories and counts only. */
function finalResult(campaign, context) {
  return { schema: RESULT_SCHEMA, attempt: context.attempt, result: campaign.complete ? "passed" : "unverified", failure: campaign.maybeFailure,
    scenarios: campaign.results.map(({ scenario, result: outcome, failures }) => ({ scenario, result: outcome, failures })),
    starts_signed: campaign.startsSigned, renewals_signed: campaign.renewalsSigned, context_sha256: digest(JSON.stringify(context)), parity_promotion: false };
}

/** Judge the current scenario from its final-arm segment, and name its private result file. */
function scenarioJudgement(campaign, parsed, context) {
  const scenario = campaign.scenario;
  if (scenario.name === "authorization_negatives") requireCondition(scenario.leg === "complete", "negative_legs_incomplete");
  const judgement = judgeScenario({ scenario: scenario.name, records: scenario.records.filter((row) => row.receivedAtUnixMs >= scenario.segmentStartAt),
    reviews: parsed.reviews, finalState: parsed.final_state, segmentStartOrdinal: scenario.segmentStartOrdinal,
    checkpoint: scenario.maybeCheckpoint ? checkpointFacts(scenario.maybeCheckpoint) : null, legs: scenario.legs, carry: campaign.carry, safetySamples: [] });
  const file = `scenario-${String(campaign.index + 1).padStart(2, "0")}-${scenario.name}.json`;
  const value = { schema: SCENARIO_RESULT_SCHEMA, ...judgement, context_sha256: digest(JSON.stringify(context)),
    physical_identity_sha256: context.physical_identity_sha256, reviews: parsed.reviews, final_state: parsed.final_state };
  return { file, value, judgement };
}

/**
 * The HTTP shell: exact Host, same-origin page writes (a local tool may post operator checkpoints without an
 * Origin; a browser always sends one), the pinned page with the supervisor client appended, serialized writes.
 */
function createHttpServer({ context, trust, routes, serial }) {
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    const path = new URL(request.url, origin).pathname;
    const sameOrigin = request.headers.origin === origin || (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin");
    const operator = path.startsWith("/checkpoint/") && request.headers.origin === undefined && request.headers["sec-fetch-site"] === undefined;
    if (request.method === "POST" || path === "/scenario-artifacts" || path === "/replay-artifact") requireCondition(sameOrigin || operator, "origin_rejected");
    const input = request.method === "POST" ? await body(request) : undefined;
    if (path === "/context" && request.method === "GET") return send(response, 200, { expectedGateCommit: context.gate_commit,
      expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256, trust, restorationQualification: true });
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
    // Reads of supervisor state and the page's checkpoint poll stay outside the write queue.
    const serialized = request.method === "POST" || request.url === "/scenario-artifacts" || request.url === "/replay-artifact";
    const result = serialized ? serial.queue.then(operation) : operation();
    if (serialized) serial.queue = Promise.resolve(result).catch(() => undefined);
    Promise.resolve(result).catch((error) => {
      if (response.headersSent) { response.destroy(); return; }
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    });
  });
  return server;
}

export async function createRestorationSupervisor(options, operations = {}) {
  const root = resolve(options.privateRoot), context = options.context, now = operations.now ?? Date.now;
  const announce = operations.announce ?? ((line) => process.stdout.write(`${line}\n`));
  const verify = operations.verifyFrozen ?? (() => verifyFrozenRestoration(context, options.authorityDirectory, options.bun));
  await verify();
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  const sign = operations.sign ?? ((operation, input) => authorityCall(context.gate_root, options.authorityDirectory, `sign-${operation}`, input, options.bun));
  const readPool = operations.readPool ?? (() => readPoolForSigning(context.firmware_root, options.poolCredentials));
  const campaign = createCampaign(), secrets = new Set();
  let completion, hostSequence = 0, finalWritten = false;
  const serial = { queue: Promise.resolve() };
  const hostEvent = (event, detail = {}) => appendFile(resolve(root, "campaign-events.jsonl"),
    `${JSON.stringify(hostRow(++hostSequence, now(), event, campaign.scenario.name, detail))}\n`, { mode: 0o600 });
  const physical = createPhysicalOwner({ campaign, context, root, now, hostEvent, announce, serial,
    onFailed: async (category) => { failCampaign(campaign, category); await writeFinal(); } }, operations);

  async function writeFinal() {
    if (finalWritten || !(campaign.complete || campaign.maybeFailure)) return;
    finalWritten = true;
    await physical.stopWatcher();
    const result = finalResult(campaign, context);
    await writeNew(resolve(root, "result.json"), { result, sha256: digest(JSON.stringify(result)) });
    await hostEvent(campaign.complete ? "campaign_completed" : "campaign_failed", campaign.maybeFailure ? { category: campaign.maybeFailure.category } : {});
  }

  const refuseLeak = async (input) => {
    try { requireSecretsAbsent(input, secrets); } catch (error) { failCampaign(campaign, "credential_in_record"); await writeFinal(); throw error; }
  };

  async function authorize(input) {
    exactObject(input, ["controlSessionBindingSha256"]);
    requireCondition(canonicalBase64(input.controlSessionBindingSha256, 32), "binding_shape");
    const request = admitSigning(campaign);
    await verify();
    const stratum = await readPool();
    for (const value of [stratum.endpoint, stratum.username, stratum.password, input.controlSessionBindingSha256]) secrets.add(value);
    const scope = campaign.scopes.get(scopeGroup(campaign.scenario.name));
    const artifacts = await signRestorationWindow({ ...request, challengeId: scope.challengeId, binding: input.controlSessionBindingSha256, stratum, sign });
    for (const artifact of [artifacts.grant, ...artifacts.renewals]) { secrets.add(artifact.authorization); secrets.add(artifact.leaseId); }
    recordSigned(campaign, request, artifacts, now());
    await hostEvent("artifacts_signed", { kind: request.kind, renewals: artifacts.renewals.length });
    return { authorization_context_saved: true };
  }

  async function record(input) {
    await refuseLeak(input);
    const parsed = parseRecord(input, context);
    await appendFile(resolve(root, "records.jsonl"), `${JSON.stringify({ receivedAtUnixMs: now(), scenario: campaign.scenario.name, ...parsed })}\n`, { mode: 0o600 });
    observeRecord(campaign, parsed, now());
    return { recorded: true };
  }

  async function complete(input) {
    await refuseLeak(input);
    const parsed = parseCompletionReview(input, context);
    const saved = completion; completion = undefined;
    requireCondition(saved && saved.nonce === parsed.nonce && saved.scenario === campaign.scenario.name && saved.expires > now(), "completion_challenge");
    requireRunning(campaign);
    const { file, value, judgement } = scenarioJudgement(campaign, parsed, context);
    await writeNew(resolve(root, file), { value, sha256: digest(JSON.stringify(value)) });
    await physical.stopWatcher();
    // Logged before finishScenario advances, so the row names the judged scenario rather than the next one.
    await hostEvent("scenario_judged", { result: judgement.result });
    finishScenario(campaign, judgement, parsed.final_state.journal.entries.at(-1)?.ordinal ?? 0);
    await writeFinal();
    return { result: judgement.result, scenario: judgement.scenario, cleanup_confirmed: true };
  }

  const routes = {
    "POST /activate": async (input) => {
      exactObject(input, []);
      const state = physical.checkpoint();
      if (state?.checkpoint === "reconnect_ready") { admitReconnect(state, now()); await physical.stopWatcher(); }
      const { scope, created } = activate(campaign, () => ({ challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 }));
      secrets.add(scope.challengeId);
      await hostEvent(created ? "scope_created" : "scope_reused");
      return { ...scope };
    },
    "POST /authorization-context": authorize,
    "GET /scenario-artifacts": async () => { const artifacts = deliverWindow(campaign, now()); await hostEvent("artifacts_delivered"); return artifacts; },
    "GET /replay-artifact": async () => { const artifact = replayArtifact(campaign, now()); await hostEvent("replay_delivered", { operation: artifact.operation }); return artifact; },
    "POST /physical-window": async (input) => {
      exactObject(input, ["event"]);
      requireRunning(campaign);
      requireCondition(input.event === "begin" || input.event === "arm", "physical_window_event");
      return input.event === "begin" ? physical.begin() : physical.arm();
    },
    "GET /physical-window": async () => {
      const state = physical.checkpoint();
      if (!state) return { checkpoint: "none" };
      await physical.settle();
      return { checkpoint: view(state, now()).checkpoint };
    },
    "POST /record": record,
    "POST /completion-context": async (input) => {
      exactObject(input, []); requireRunning(campaign);
      completion = { nonce: nonce(), scenario: campaign.scenario.name, expires: now() + COMPLETION_TTL_MS };
      return { nonce: completion.nonce };
    },
    "POST /completion-review": complete,
    "POST /checkpoint/ready": async (input) => {
      exactObject(input, ["scenario", "checkpoint"]);
      requireRunning(campaign);
      requireCondition(input.scenario === campaign.scenario.name, "checkpoint_scenario");
      await physical.stopWatcher();
      const state = physical.requireCheckpoint();
      operatorReady(state, input.checkpoint, now());
      markSegment(campaign, now());
      await hostEvent("operator_ready", { rearms: state.rearms });
      return { checkpoint: state.checkpoint };
    },
    "POST /checkpoint/cancel": async (input) => {
      exactObject(input, ["scenario"]);
      requireCondition(SCENARIOS.includes(input.scenario) && input.scenario === campaign.scenario.name, "checkpoint_scenario");
      await hostEvent("operator_cancelled");
      failCampaign(campaign, "operator_cancelled");
      await writeFinal();
      return { cancelled: true };
    },
    "GET /supervisor-state": async () => { await physical.settle(); return supervisorState(campaign, now()); },
  };

  const server = createHttpServer({ context, trust, routes, serial });
  server.requestTimeout = 60000; server.headersTimeout = 10000;
  server.closeRestorationResources = async () => { await physical.stopWatcher().catch(() => undefined); };
  server.campaign = campaign;
  return server;
}
