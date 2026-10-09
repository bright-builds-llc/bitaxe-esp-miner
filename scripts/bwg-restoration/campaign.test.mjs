import test from "node:test";
import assert from "node:assert/strict";
import { activate, admitSigning, createCampaign, deliverWindow, finishScenario, markSegment, observeRecord, recordSigned, replayArtifact } from "./campaign.mjs";
import { operatorReady } from "./checkpoint.mjs";
import { SCENARIOS } from "./contract.mjs";
import { baseline, journal, pageState, rejectionReview } from "./fixtures.test-helper.mjs";

const throwsWith = (operation, code) => assert.throws(operation, (error) => error.code === code);
let scopes = 0;
const newScope = () => ({ challengeId: `challenge_${++scopes}`, retentionExpiryUnixSeconds: 1 });
const window = (renewals) => ({ grant: { leaseId: `lease_${scopes}`, authorization: "g" }, renewals: Array.from({ length: renewals }, () => ({ authorization: "r" })) });
const passed = (scenario) => ({ scenario, result: "passed", failures: [], facts: {}, carry: {} });

function sign(campaign, now = 0) {
  const request = admitSigning(campaign);
  const artifacts = window(request.renewals);
  recordSigned(campaign, request, artifacts, now);
  return { request, artifacts };
}

/** Advance a campaign through the first `count` scenarios with one signed and delivered window each. */
function advanceTo(campaign, name) {
  while (campaign.scenario.name !== name) {
    activate(campaign, newScope);
    if (campaign.scenario.maybeCheckpoint) { operatorReady(campaign.scenario.maybeCheckpoint, "awaiting_operator_ready", 0); markSegment(campaign, 0); }
    sign(campaign);
    deliverWindow(campaign);
    finishScenario(campaign, passed(campaign.scenario.name));
  }
}

const reviewRecord = (operation, result) => ({ operation, outcome: "ok", result, state: pageState({ entries: journal(["closed"], 100) }) });
/** The post-reboot status in the same connection that the firmware requires before N1. */
const reportReboot = (campaign, events = ["connected", "status_reviewed:reboot"]) =>
  observeRecord(campaign, { operation: "statusReview", outcome: "ok", result: baseline("reboot"), state: pageState({ entries: journal(events) }) }, 0);

test("one scope serves every reconnect within a scenario and a new scenario gets a new one", () => {
  // Arrange
  const campaign = createCampaign();
  // Act
  const first = activate(campaign, newScope), again = activate(campaign, newScope);
  finishScenario(campaign, passed("completion"));
  const next = activate(campaign, newScope);
  // Assert
  assert.deepEqual([first.created, again.created], [true, false]);
  assert.equal(again.scope, first.scope);
  assert.notEqual(next.scope.challengeId, first.scope.challengeId);
});

test("reboot and authorization_negatives share one scope", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "reboot");
  const reboot = activate(campaign, newScope).scope;
  operatorReady(campaign.scenario.maybeCheckpoint, "awaiting_operator_ready", 0);
  sign(campaign);
  finishScenario(campaign, passed("reboot"));
  // Act
  const negatives = activate(campaign, newScope);
  // Assert
  assert.equal(negatives.created, false);
  assert.equal(negatives.scope, reboot);
});

test("a signed window is delivered exactly once", () => {
  // Arrange
  const campaign = createCampaign();
  activate(campaign, newScope);
  sign(campaign);
  // Act
  const first = deliverWindow(campaign);
  // Assert
  assert.equal(first.renewals.length, 1);
  throwsWith(() => deliverWindow(campaign), "artifacts_unavailable");
  throwsWith(() => admitSigning(campaign), "scenario_already_signed");
});

test("a physical scenario signs only after the operator's readiness", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "disconnect");
  activate(campaign, newScope);
  // Act / Assert
  throwsWith(() => admitSigning(campaign), "operator_ready_required");
  operatorReady(campaign.scenario.maybeCheckpoint, "awaiting_operator_ready", 0);
  assert.equal(admitSigning(campaign).renewals, 0);
});

test("the attempt never signs more than ten Starts or two renewals", () => {
  // Arrange
  const campaign = createCampaign();
  activate(campaign, newScope);
  campaign.startsSigned = 10;
  // Act / Assert
  throwsWith(() => admitSigning(campaign), "start_cap");
  campaign.startsSigned = 0; campaign.renewalsSigned = 2;
  throwsWith(() => admitSigning(campaign), "renewal_cap");
});

test("the reboot Start is replayed once for N1 and then burned", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "authorization_negatives");
  activate(campaign, newScope);
  reportReboot(campaign);
  // Act
  const first = replayArtifact(campaign, 0);
  // Assert
  assert.equal(first.operation, "start");
  assert.equal(campaign.maybeRebootStart, null);
  throwsWith(() => replayArtifact(campaign, 0), "replay_unavailable");
});

test("the negative legs advance only on the page's replay and review records", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "authorization_negatives");
  activate(campaign, newScope);
  reportReboot(campaign);
  replayArtifact(campaign, 0);
  // Act
  observeRecord(campaign, reviewRecord("authorizationRejectionReview", rejectionReview()), 10);
  const beforeReplay = campaign.scenario.leg;
  observeRecord(campaign, reviewRecord("replayArtifact", { operation: "start", outcome: "rejected", category: "authentication_failed" }), 20);
  observeRecord(campaign, reviewRecord("authorizationRejectionReview", rejectionReview()), 30);
  // Assert
  assert.equal(beforeReplay, "n1_review");
  assert.equal(campaign.scenario.leg, "n2_sign");
});

test("N2 waits past the admission age and N3 needs a fresh possession", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "authorization_negatives");
  activate(campaign, newScope);
  Object.assign(campaign.scenario, { leg: "n2_sign" });
  sign(campaign, 1000);
  // Act / Assert
  throwsWith(() => replayArtifact(campaign, 1000 + 60999), "replay_not_expired");
  assert.equal(replayArtifact(campaign, 1000 + 61000).operation, "start");
  observeRecord(campaign, reviewRecord("replayArtifact", { operation: "start", outcome: "rejected", category: "admission_required" }), 70000);
  observeRecord(campaign, reviewRecord("authorizationRejectionReview", rejectionReview()), 70000);
  assert.equal(campaign.scenario.leg, "n3_replay");
  throwsWith(() => replayArtifact(campaign, 70000 + 45001), "replay_possession_stale");
  assert.equal(replayArtifact(campaign, 70000 + 45000).operation, "start");
  assert.equal(campaign.scenario.maybeExpiredStart, null);
});

test("N4 replays its accepted renewal once, only after renewOnce", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "authorization_negatives");
  activate(campaign, newScope);
  Object.assign(campaign.scenario, { leg: "n4_sign" });
  sign(campaign);
  deliverWindow(campaign);
  // Act / Assert
  throwsWith(() => replayArtifact(campaign, 0), "replay_unavailable");
  observeRecord(campaign, reviewRecord("renewOnce", null), 1);
  const replay = replayArtifact(campaign, 2);
  assert.equal(replay.operation, "renew");
  throwsWith(() => replayArtifact(campaign, 3), "replay_unavailable");
});

test("the first unverified scenario stops the attempt", () => {
  // Arrange
  const campaign = createCampaign();
  // Act
  finishScenario(campaign, { scenario: "completion", result: "unverified", failures: ["terminal_reason_mismatch"], facts: {}, carry: {} });
  // Assert
  assert.deepEqual(campaign.maybeFailure, { scenario: "completion", category: "terminal_reason_mismatch" });
  throwsWith(() => activate(campaign, newScope), "campaign_stopped");
  assert.equal(SCENARIOS.length, 8);
});

test("a scenario's closing journal entries stay out of the next scenario's segment", () => {
  // Arrange
  const campaign = createCampaign();
  // Act
  finishScenario(campaign, passed("completion"), 17);
  // Assert
  assert.deepEqual([campaign.scenario.name, campaign.scenario.segmentStartOrdinal], ["pause", 17]);
});

test("N1 is released by the passing reboot scenario's carried report", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "reboot");
  activate(campaign, newScope);
  operatorReady(campaign.scenario.maybeCheckpoint, "awaiting_operator_ready", 0);
  markSegment(campaign, 0);
  sign(campaign);
  deliverWindow(campaign);
  finishScenario(campaign, { ...passed("reboot"), carry: { rebootReported: true } });
  activate(campaign, newScope);
  // Act
  const artifact = replayArtifact(campaign, 0);
  // Assert
  assert.equal(artifact.operation, "start");
});

test("N1 waits for a status that reported the reboot in the same connection", () => {
  // Arrange
  const campaign = createCampaign();
  advanceTo(campaign, "authorization_negatives");
  activate(campaign, newScope);
  // Act / Assert
  throwsWith(() => replayArtifact(campaign, 0), "n1_status_required");
  reportReboot(campaign, ["connected", "status_reviewed:reboot", "disconnected", "connected"]);
  throwsWith(() => replayArtifact(campaign, 0), "n1_status_required");
  reportReboot(campaign, ["connected", "status_reviewed:reboot", "disconnected", "connected", "status_reviewed:reboot"]);
  assert.equal(replayArtifact(campaign, 0).operation, "start");
});
