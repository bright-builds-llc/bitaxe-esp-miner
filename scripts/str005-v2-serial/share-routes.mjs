import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { missing, nonce } from "../fixed-usb-qualification/contract.mjs";
import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger, validateAttempt, validateCooling } from "../fixed-usb-qualification/iterative-contract.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { baseline, checkedState, healthy } from "./journal.mjs";
import { parseStatus } from "./device.mjs";
import { validateStartTiming } from "./start-observation.mjs";
import { signV2Attempt } from "./signing.mjs";
import { bytes, check, digest, object, sha256, uint } from "./values.mjs";

const PATHS = new Set(["/budget-review-context", "/budget-review", "/cooling-review-context", "/cooling-review", "/start/network", "/authorization-context", "/window-artifacts", "/share/start-observed"]);
/** One active server scope, no signing material owned here and no grants persisted anywhere. */
export function createShareRoutes(root, context, operations) {
  const contextSha256 = sha256(JSON.stringify(context));
  let maybeChallenge = null, maybeReview = null, maybeNetwork = null, maybePending = null;
  let issuanceClaimed = false, deliveryClaimed = false, epoch = 0, reviewSequence = 0;
  function assertReady() {
    check(context.scope === "share", "v2_share_route_forbidden");
    check(!operations.failed(), "v2_terminal_failure"); operations.ready();
    const scope = operations.activeScope();
    check(scope && typeof scope.challengeId === "string", "v2_share_scope");
    return scope.challengeId;
  }
  function currentTime() { return uint(operations.now()); }
  function fresh(saved, scope) {
    const at = currentTime();
    check(saved && saved.epoch === epoch && saved.scope === scope && at >= saved.createdAt && at < saved.expiresAt, "v2_share_review_expired");
  }
  function ledger(value) {
    requireIdleLedger(context.expectedLedgerBefore, 18, 1560000);
    requireIdleLedger(value, 18, 1560000);
    check(equal(value, context.expectedLedgerBefore), "v2_share_ledger_changed");
    validateAttempt(context.qualificationAttempt);
    check(context.qualificationAttempt.id === context.attemptId && context.qualificationAttempt.ordinal === 18 && context.qualificationAttempt.purpose === "normal" && context.qualificationAttempt.maximumActiveMilliseconds === 180000, "v2_share_attempt");
  }
  async function prerequisites(scope, { accounting = false, cooling = false } = {}) {
    const last = operations.journal.lastState();
    check(last?.phase === "candidate", "v2_share_candidate_required"); healthy(last.state);
    const baselineId = last.state.preservation.baseline_id;
    let previous, previousSequence = 0;
    for (let index = 1; index <= 4; index++) {
      const row = (await proof(root, `cycle-${index}.json`)).value;
      object(row, ["schema", "contextSha256", "beforeSequence", "afterSequence", "installReviewSha256", "report"]);
      check(row.schema === "str005-v2-cycle-v1" && row.contextSha256 === contextSha256 && row.report.cycle === index &&
        row.report.baseline_id === baselineId && row.beforeSequence >= previousSequence && row.afterSequence > row.beforeSequence && row.afterSequence <= last.sequence,
      "v2_share_cycle_continuity");
      digest(row.installReviewSha256);
      previous = validateCycle(row.report, context, previous); previousSequence = row.afterSequence;
    }
    if (accounting) {
      const saved = (await proof(root, "accounting-before.json")).value;
      object(saved, ["schema", "contextSha256", "observedSequence", "stage", "state", "ledger", "original_budget"]);
      check(saved.schema === "str005-v2-accounting-v1" && saved.contextSha256 === contextSha256 && saved.stage === "before" &&
        saved.observedSequence >= previousSequence && saved.observedSequence <= last.sequence && saved.state.preservation.baseline_id === baselineId,
      "v2_share_accounting_join");
      checkedState(saved.state, context, "candidate"); baseline(saved.state); ledger(saved.ledger); requireExhaustedOriginal(saved.original_budget);
    }
    if (cooling) {
      const saved = (await proof(root, "cooling.json")).value;
      object(saved, ["schema", "contextSha256", "scopeChallengeId", "observedSequence", "atHostMs", "proof", "restoration", "budget_before", "budget_after", "state"]);
      check(saved.schema === "str005-v2-cooling-v1" && saved.contextSha256 === contextSha256 && saved.scopeChallengeId === scope &&
        saved.observedSequence >= previousSequence && saved.observedSequence <= last.sequence && saved.state.preservation.baseline_id === baselineId,
      "v2_share_cooling_join");
      checkedState(saved.state, context, "candidate"); baseline(saved.state);
      validateCooling(saved.proof, saved.restoration); ledger(saved.budget_before); ledger(saved.budget_after);
    }
    check(assertReady() === scope, "v2_share_scope_changed");
    return last;
  }
  async function challenge(kind) {
    const scope = assertReady();
    check(!issuanceClaimed && !maybePending, "v2_share_issuance_consumed");
    await missing(resolve(root, "issuance.claim.json"));
    await prerequisites(scope);
    if (kind === "cooling") await missing(resolve(root, "cooling.json"));
    const createdAt = currentTime();
    maybeReview = null; maybeNetwork = null;
    maybeChallenge = { kind, nonce: nonce(), scope, epoch, createdAt, expiresAt: createdAt + 45000 };
    return { nonce: maybeChallenge.nonce, mode: "iterative" };
  }
  function consumeChallenge(kind, supplied, scope) {
    const saved = maybeChallenge; maybeChallenge = null;
    fresh(saved, scope); check(saved.kind === kind && saved.nonce === supplied, "v2_share_review_nonce");
    return saved;
  }
  async function recordBaseline(state) {
    checkedState(state, context, "candidate"); healthy(state);
    const result = await operations.journal.state("candidate", state, currentTime());
    return result.sequence;
  }
  async function coolingReview(input) {
    object(input, ["nonce", "proof", "restoration", "budget_before", "budget_after", "state"]);
    const scope = assertReady(), saved = consumeChallenge("cooling", input.nonce, scope);
    validateCooling(input.proof, input.restoration); ledger(input.budget_before); ledger(input.budget_after);
    const observedSequence = await recordBaseline(input.state);
    await prerequisites(scope); fresh(saved, scope);
    await writeNew(resolve(root, "cooling.json"), { schema: "str005-v2-cooling-v1", contextSha256, scopeChallengeId: scope,
      observedSequence, atHostMs: currentTime(), proof: input.proof, restoration: input.restoration,
      budget_before: input.budget_before, budget_after: input.budget_after, state: input.state });
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope);
    return { cooling_review_saved: true, review_file: "cooling.json" };
  }
  async function budgetReview(input) {
    object(input, ["nonce", "report", "controlSessionBindingSha256", "state"]);
    const scope = assertReady(), saved = consumeChallenge("budget", input.nonce, scope);
    bytes(input.controlSessionBindingSha256, 32); ledger(input.report);
    const observedSequence = await recordBaseline(input.state);
    await prerequisites(scope, { accounting: true, cooling: true }); fresh(saved, scope);
    check(reviewSequence < 1024, "v2_share_review_bound");
    const file = `budget-review-${String(++reviewSequence).padStart(4, "0")}.json`;
    await writeNew(resolve(root, file), { schema: "str005-v2-budget-review-v1", contextSha256, observedSequence,
      atHostMs: currentTime(), expiresAtHostMs: saved.expiresAt, ledger: input.report });
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope);
    maybeReview = { ...saved, binding: input.controlSessionBindingSha256, report: structuredClone(input.report), file, observedSequence };
    return { budget_review_saved: true };
  }
  async function network(input) {
    object(input, ["status", "controlSessionBindingSha256"]);
    const scope = assertReady(); bytes(input.controlSessionBindingSha256, 32); fresh(maybeReview, scope);
    check(!issuanceClaimed && maybeReview.binding === input.controlSessionBindingSha256, "v2_share_network_binding");
    const status = parseStatus(input.status), owner = operations.fixture();
    check(status.scope === "share" && status.state === "idle" && status.observation.wifiConnected && status.observation.stationIpv4 !== null && owner, "v2_share_network_idle");
    owner.alive(); owner.validateStation(status.observation.stationIpv4);
    operations.requireObserver(status.observation, input.controlSessionBindingSha256);
    check(networkSequence < 1024, "v2_share_network_bound");
    const observedAtHostMs = currentTime();
    maybeNetwork = { observation: structuredClone(status.observation), binding: input.controlSessionBindingSha256, scope, epoch, observedAtHostMs };
    // Only the non-address portion of the actual observation may cross the write boundary.
    await writeNew(resolve(root, `share-network-${String(++networkSequence).padStart(4, "0")}.json`), {
      schema: "str005-v2-share-network-v1", contextSha256, observedAtHostMs, reviewFile: maybeReview.file,
      bootOrdinal: status.observation.bootOrdinal, workerGeneration: status.observation.workerGeneration,
      serialTransportEpoch: status.observation.serialTransportEpoch, observedAtDeviceUs: status.observation.observedAtUs, clockValid: true });
    check(assertReady() === scope && maybeNetwork?.epoch === epoch, "v2_share_scope_changed"); fresh(maybeReview, scope);
    operations.requireObserver(status.observation, input.controlSessionBindingSha256);
    return { network_reviewed: true };
  }
  let networkSequence = 0;
  function requireNetwork(scope, binding, owner) {
    baseline(operations.journal.lastState()?.state);
    check(maybeNetwork && maybeNetwork.epoch === epoch && maybeNetwork.scope === scope && maybeNetwork.binding === binding &&
      currentTime() >= maybeNetwork.observedAtHostMs && currentTime() - maybeNetwork.observedAtHostMs <= 5000, "v2_share_network_stale");
    operations.requireObserver(maybeNetwork.observation, binding);
    owner.alive(); owner.validateStation(maybeNetwork.observation.stationIpv4); owner.requireStartWindow();
  }
  async function authorize(input) {
    object(input, ["controlSessionBindingSha256"]);
    const scope = assertReady(), saved = maybeReview; maybeReview = null;
    fresh(saved, scope); bytes(input.controlSessionBindingSha256, 32);
    check(!issuanceClaimed && !maybePending && input.controlSessionBindingSha256 === saved.binding, "v2_share_issuance_consumed");
    await prerequisites(scope, { accounting: true, cooling: true }); await operations.verify();
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope);
    const owner = operations.fixture(); check(owner, "v2_fixture_missing"); requireNetwork(scope, saved.binding, owner);
    issuanceClaimed = true;
    await writeNew(resolve(root, "issuance.claim.json"), { schema: "str005-v2-issuance-claim-v1", contextSha256,
      ordinal: context.qualificationAttempt.ordinal, atHostMs: currentTime(), reviewFile: saved.file, observedSequence: saved.observedSequence,
      deviceReservationObserved: false });
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope); requireNetwork(scope, saved.binding, owner);
    const artifacts = await signV2Attempt({ attempt: context.qualificationAttempt, challengeId: scope, binding: saved.binding,
      stratum: owner.stratum, sign: operations.sign, failed: () => {
        if (operations.failed() || epoch !== saved.epoch || operations.activeScope()?.challengeId !== scope) return true;
        fresh(saved, scope); requireNetwork(scope, saved.binding, owner); return false;
      } });
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope); requireNetwork(scope, saved.binding, owner);
    await operations.verify(); check(assertReady() === scope, "v2_share_scope_changed");
    fresh(saved, scope); requireNetwork(scope, saved.binding, owner);
    check(Buffer.byteLength(JSON.stringify(artifacts)) <= 65536, "v2_share_artifact_bound");
    await writeNew(resolve(root, "issued.json"), { schema: "str005-v2-issuance-v1", contextSha256, ordinal: context.qualificationAttempt.ordinal,
      atHostMs: currentTime(), ledgerBefore: saved.report, authorizationCount: 10, privatePayloadPersisted: false, deviceReservationObserved: false });
    check(assertReady() === scope, "v2_share_scope_changed"); fresh(saved, scope); requireNetwork(scope, saved.binding, owner);
    maybePending = artifacts;
    return { ready: true };
  }
  async function deliver() {
    const scope = assertReady();
    check(issuanceClaimed && !deliveryClaimed && maybePending, "v2_share_artifacts_unavailable");
    await operations.verify(); check(assertReady() === scope, "v2_share_scope_changed");
    check(issuanceClaimed && !deliveryClaimed && maybePending, "v2_share_artifacts_unavailable");
    const owner = operations.fixture(); check(owner, "v2_fixture_missing");
    requireNetwork(scope, maybeNetwork?.binding, owner);
    deliveryClaimed = true; const artifacts = maybePending; maybePending = null;
    await writeNew(resolve(root, "consumed.json"), { schema: "str005-v2-artifact-delivery-v1", contextSha256,
      ordinal: context.qualificationAttempt.ordinal, atHostMs: currentTime(), deliveryAttempted: true, deviceReservationObserved: false });
    check(assertReady() === scope, "v2_share_scope_changed"); owner.alive(); owner.requireStartWindow();
    return artifacts;
  }
  async function startObserved(input) {
    object(input, ["status", "timing"]); const scope = assertReady();
    check(issuanceClaimed && deliveryClaimed && maybeNetwork?.scope === scope && maybeNetwork.epoch === epoch, "v2_share_start_issuance");
    validateStartTiming(input.timing);
    const status = parseStatus(input.status), owner = operations.fixture();
    check(status.scope === "share" && status.state === "idle" && status.observation.wifiConnected && status.observation.stationIpv4 !== null && owner,
      "v2_share_network_idle");
    const observation = status.observation;
    check(["bootOrdinal", "workerGeneration", "serialTransportEpoch"].every(key => observation[key] === maybeNetwork.observation[key]) &&
      observation.observedAtUs >= maybeNetwork.observation.observedAtUs, "v2_share_start_binding");
    owner.validateStation(observation.stationIpv4); operations.requireObserver(observation, maybeNetwork.binding);
    const last = operations.journal.lastState();
    check(last?.phase === "candidate" && last.state.running && !last.state.heartbeatSuppressed && !last.state.failure &&
      last.state.qualification?.generation === observation.workerGeneration, "v2_share_start_state");
    const ready = await proof(root, "fixture-ready.json"), consumed = await proof(root, "consumed.json");
    check(assertReady() === scope, "v2_share_scope_changed"); operations.requireObserver(observation, maybeNetwork.binding);
    await writeNew(resolve(root, "share-start-observed.json"), { schema: "str005-v2-share-start-observed-v1", contextSha256,
      clientSha256: context.client_sha256, fixtureReadySha256: ready.sha256, consumedSha256: consumed.sha256,
      atHostMs: currentTime(), observedStateSequence: last.sequence, bootOrdinal: observation.bootOrdinal,
      workerGeneration: observation.workerGeneration, serialTransportEpoch: observation.serialTransportEpoch,
      networkObservedAtDeviceUs: observation.observedAtUs, timing: input.timing });
    return { start_observed: true };
  }
  return {
    async handle(path, input, method = "POST") {
      if (!PATHS.has(path)) return undefined;
      assertReady();
      check(method === (path === "/window-artifacts" ? "GET" : "POST"), "v2_share_route_method");
      if (path === "/window-artifacts") { check(input === undefined, "v2_share_get_body"); return deliver(); }
      if (path.endsWith("-review-context")) { object(input, []); return challenge(path === "/cooling-review-context" ? "cooling" : "budget"); }
      if (path === "/cooling-review") return coolingReview(input);
      if (path === "/budget-review") return budgetReview(input);
      if (path === "/start/network") return network(input);
      if (path === "/share/start-observed") return startObserved(input);
      return authorize(input);
    },
    resetScope() { epoch++; maybeChallenge = null; maybeReview = null; maybeNetwork = null; maybePending = null; },
  };
}
