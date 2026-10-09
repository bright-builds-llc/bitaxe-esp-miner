// Pure campaign state: scenario order, persistent scopes, contract caps, one-time artifact delivery and the
// authorization_negatives legs. The HTTP shell (server.mjs) owns signing, files and the watcher process.
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { blocksActivation, createCheckpoint } from "./checkpoint.mjs";
import { EXPIRED_START_WAIT_MS, FRESH_POSSESSION_MS, MAXIMUM_RENEWALS, MAXIMUM_STARTS, PHYSICAL_PLANS, SCENARIO_PLANS, SCENARIOS,
  scopeGroup, SETTLED_ADMISSION_STAGES } from "./contract.mjs";

function freshScenario(name) {
  return { name, signed: 0, signedSinceReady: 0, maybePending: null, records: [], segmentStartOrdinal: 0, segmentStartAt: 0,
    maybeCheckpoint: PHYSICAL_PLANS[name] ? createCheckpoint(name) : null,
    leg: name === "authorization_negatives" ? "n1" : null, legs: {}, maybeExpiredStart: null, maybeReplayRenewal: null,
    maybeReplayDelivered: null, maybeLastReviewAt: null, n1StatusReported: false, maybeDeliveredAt: null };
}

export function createCampaign() {
  return { index: 0, maybeFailure: null, results: [], startsSigned: 0, renewalsSigned: 0, scopes: new Map(),
    scenario: freshScenario(SCENARIOS[0]), maybeRebootStart: null, maybeLastJournalOrdinal: 0, complete: false, carry: {} };
}

export function requireRunning(campaign) {
  requireCondition(!campaign.complete && campaign.maybeFailure === null, "campaign_stopped");
}

/** Every connect asks for a scope; one scope serves a scenario's reconnects and the reboot → negatives pair. */
export function activate(campaign, createScope) {
  requireRunning(campaign);
  const checkpoint = campaign.scenario.maybeCheckpoint;
  if (checkpoint) requireCondition(!blocksActivation(checkpoint), "physical_checkpoint_not_ready");
  const group = scopeGroup(campaign.scenario.name);
  const maybeScope = campaign.scopes.get(group);
  if (maybeScope) return { scope: maybeScope, created: false };
  const scope = createScope();
  campaign.scopes.set(group, scope);
  return { scope, created: true };
}

/**
 * True when this scenario's latest page record was taken while connected and shows the device's admission
 * diagnostic settled (`idle` or `complete`), so the previous lease's native shutdown has finished
 * (attempt-006). The diagnostic is non-authoritative: it can only refuse a lease, never admit one.
 */
export function admissionSettled(campaign) {
  const maybeState = campaign.scenario.records.at(-1)?.state;
  return maybeState?.connected === true && SETTLED_ADMISSION_STAGES.includes(maybeState.admission?.stage);
}

/** No deadline: the agent re-polls `admissionDiagnostic` until the device settles. */
function requireSettled(campaign) {
  requireCondition(admissionSettled(campaign), "settle_required");
}

/**
 * What the next signing request may produce, or a refusal. Caps count signed Starts and renewals for the
 * whole attempt; replays never sign.
 */
export function admitSigning(campaign) {
  requireRunning(campaign);
  const scenario = campaign.scenario, plan = SCENARIO_PLANS[scenario.name];
  requireCondition(campaign.scopes.has(scopeGroup(scenario.name)), "scope_missing");
  requireCondition(scenario.maybePending === null, "artifacts_pending");
  let request;
  if (scenario.name === "authorization_negatives") {
    requireCondition(scenario.leg === "n2_sign" || scenario.leg === "n4_sign", "negative_leg_not_signing");
    request = scenario.leg === "n2_sign" ? { window: "standard", renewals: 0, kind: "expired_start" } : { window: plan.window, renewals: 1, kind: "window" };
  } else if (scenario.maybeCheckpoint) {
    requireCondition(scenario.maybeCheckpoint.checkpoint === "ready_for_lease" && scenario.signedSinceReady === 0, "operator_ready_required");
    request = { window: plan.window, renewals: plan.renewals, kind: "window" };
  } else {
    requireCondition(scenario.signed === 0, "scenario_already_signed");
    request = { window: plan.window, renewals: plan.renewals, kind: "window" };
  }
  requireSettled(campaign);
  requireCondition(campaign.startsSigned + 1 <= MAXIMUM_STARTS, "start_cap");
  requireCondition(campaign.renewalsSigned + request.renewals <= MAXIMUM_RENEWALS, "renewal_cap");
  return request;
}

/** Account for one signed window and hold it in memory until delivery. */
export function recordSigned(campaign, request, artifacts, now) {
  const scenario = campaign.scenario;
  campaign.startsSigned += 1; campaign.renewalsSigned += artifacts.renewals.length;
  scenario.signed += 1; scenario.signedSinceReady += 1;
  if (request.kind === "expired_start") {
    scenario.maybeExpiredStart = { grant: artifacts.grant, signedAt: now, deliveries: 0 };
    scenario.leg = "n2_wait";
    return;
  }
  scenario.maybePending = artifacts;
  if (scenario.name === "reboot") campaign.maybeRebootStart = artifacts.grant;
  if (scenario.name === "authorization_negatives") { scenario.maybeReplayRenewal = artifacts.renewals[0]; scenario.leg = "n4_renew"; }
}

/** Signed windows are delivered exactly once; the delivery time bounds the earliest device lease deadline. */
export function deliverWindow(campaign, now) {
  requireRunning(campaign);
  const artifacts = campaign.scenario.maybePending;
  requireCondition(artifacts !== null, "artifacts_unavailable");
  requireSettled(campaign);
  campaign.scenario.maybePending = null;
  campaign.scenario.maybeDeliveredAt = now;
  return artifacts;
}

/** The one replay artifact the current negative leg allows; each is delivered once and then burned. */
export function replayArtifact(campaign, now) {
  requireRunning(campaign);
  const scenario = campaign.scenario;
  requireCondition(scenario.name === "authorization_negatives" && scenario.maybeReplayDelivered === null, "replay_unavailable");
  const deliver = (leg, artifact, next) => { scenario.leg = next; scenario.maybeReplayDelivered = leg; return artifact; };
  if (scenario.leg === "n1") {
    requireCondition(campaign.maybeRebootStart !== null, "replay_unavailable");
    // The device refuses a Start until its once-per-boot reboot report has been delivered and acknowledged: either the
    // passing reboot scenario received it (its carry) or a status in this connection reported it.
    requireCondition(scenario.n1StatusReported || campaign.carry.rebootReported === true, "n1_status_required");
    const grant = campaign.maybeRebootStart; campaign.maybeRebootStart = null;
    return deliver("n1", { operation: "start", grant }, "n1_review");
  }
  if (scenario.leg === "n2_wait" || scenario.leg === "n3_replay") {
    const held = scenario.maybeExpiredStart;
    requireCondition(held !== null && now - held.signedAt >= EXPIRED_START_WAIT_MS, "replay_not_expired");
    if (scenario.leg === "n3_replay") requireCondition(scenario.maybeLastReviewAt !== null && now - scenario.maybeLastReviewAt <= FRESH_POSSESSION_MS, "replay_possession_stale");
    held.deliveries += 1;
    const leg = scenario.leg === "n2_wait" ? "n2" : "n3";
    if (leg === "n3") scenario.maybeExpiredStart = null;
    return deliver(leg, { operation: "start", grant: held.grant }, `${leg}_review`);
  }
  requireCondition(scenario.leg === "n4_replay" && scenario.maybeReplayRenewal !== null, "replay_unavailable");
  const renewal = scenario.maybeReplayRenewal; scenario.maybeReplayRenewal = null;
  return deliver("n4", { operation: "renew", renewal }, "n4_review");
}

const NEXT_LEG = { n1_review: "n2_sign", n2_review: "n3_replay", n3_review: "n4_sign", n4_review: "complete" };

/**
 * True when the page's latest connection has reviewed a status reporting the reboot: the last `connected`
 * precedes a `status_reviewed:reboot` and no `disconnected` follows that connection.
 */
export function rebootReportedInConnection(entries) {
  const connected = entries.findLastIndex((entry) => entry.event === "connected");
  const reported = entries.findLastIndex((entry) => entry.event === "status_reviewed" && entry.category === "reboot");
  const lost = entries.findLastIndex((entry) => entry.event === "disconnected");
  return connected >= 0 && reported > connected && lost < connected;
}

/** Advance the negative legs from the page's own recorded results; a review only counts after its replay. */
function observeNegativeLeg(scenario, record, now) {
  if (scenario.leg === "n1") scenario.n1StatusReported = rebootReportedInConnection(record.state.journal.entries);
  const leg = scenario.maybeReplayDelivered;
  if (record.operation === "renewOnce" && record.outcome === "ok" && scenario.leg === "n4_renew") { scenario.leg = "n4_replay"; return; }
  if (!leg || !scenario.leg.endsWith("_review") || record.outcome !== "ok") return;
  if (record.operation === "replayArtifact" && !scenario.legs[leg]) { scenario.legs[leg] = { replay: record.result, replayAt: now }; return; }
  if (record.operation !== "authorizationRejectionReview" || !scenario.legs[leg] || scenario.legs[leg].review) return;
  scenario.legs[leg].review = record.result;
  scenario.legs[leg].reviewAt = now;
  scenario.maybeLastReviewAt = now;
  scenario.maybeReplayDelivered = null;
  scenario.leg = NEXT_LEG[scenario.leg];
}

/** Append one validated page record to the current scenario. */
export function observeRecord(campaign, record, now) {
  const scenario = campaign.scenario;
  const entries = record.state.journal.entries;
  const maybeLast = entries.at(-1)?.ordinal ?? 0;
  requireCondition(maybeLast >= campaign.maybeLastJournalOrdinal, "journal_regressed");
  campaign.maybeLastJournalOrdinal = maybeLast;
  scenario.records.push({ ...record, receivedAtUnixMs: now });
  if (scenario.name === "authorization_negatives") observeNegativeLeg(scenario, record, now);
  if (record.operation === "authorizationRejectionReview" && record.outcome === "ok") scenario.maybeLastReviewAt = now;
}

/** The operator's readiness starts a new journal segment for the physical scenario's final arm. */
export function markSegment(campaign, now) {
  campaign.scenario.segmentStartOrdinal = campaign.maybeLastJournalOrdinal;
  campaign.scenario.segmentStartAt = now;
  campaign.scenario.signedSinceReady = 0;
}

/** Record a judged scenario; the first unverified scenario stops the attempt, a pass advances it. */
export function finishScenario(campaign, judgement, finalJournalOrdinal = 0) {
  // The completion's own closing entries belong to this scenario, not to the next one's segment.
  campaign.maybeLastJournalOrdinal = Math.max(campaign.maybeLastJournalOrdinal, finalJournalOrdinal);
  campaign.results.push(judgement);
  campaign.carry = judgement.carry;
  if (judgement.result !== "passed") { campaign.maybeFailure = { scenario: judgement.scenario, category: judgement.failures[0] ?? "scenario_unverified" }; return; }
  const next = campaign.index + 1;
  if (next >= SCENARIOS.length) { campaign.complete = true; return; }
  const previous = campaign.scenario;
  campaign.index = next;
  campaign.scenario = freshScenario(SCENARIOS[next]);
  campaign.scenario.segmentStartOrdinal = campaign.maybeLastJournalOrdinal;
  // reboot hands its in-memory Start to the next scenario's N1; any other scenario's leftovers are burned.
  if (previous.name !== "reboot") campaign.maybeRebootStart = null;
}

export function failCampaign(campaign, category) {
  campaign.maybeFailure ??= { scenario: campaign.scenario.name, category };
  campaign.scenario.maybePending = null;
  campaign.maybeRebootStart = null;
  campaign.scenario.maybeExpiredStart = null;
  campaign.scenario.maybeReplayRenewal = null;
}
