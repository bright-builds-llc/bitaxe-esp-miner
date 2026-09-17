import assert from "node:assert/strict";
import test from "node:test";
import { judgeV2Protocol, parseProtocolContext } from "./protocol-judge.mjs";
import { matchShareFields, judgeShares, judgeFault } from "./protocol-shares.mjs";
import { channelFixture, sharePair, nonce } from "./protocol-judge.test-helper.mjs";

test("complete coherent Channel protocol has no implied hardware or mining verdict", () => {
  // Arrange
  const input = channelFixture();
  // Act
  const result = judgeV2Protocol(input);
  // Assert
  assert.equal(result.acceptedProtocol, true); assert.equal(result.scope, "channel");
  assert.equal(Object.hasOwn(result, "hardware_qualified"), false); assert.equal(result.counts.submitted, 0);
  assert.equal(result.timings.maximumReadUs, 20);
});
for (const [name, mutate, code] of [
  ["missing admission", x => { x.deviceRecords = [x.deviceRecords[1]]; }, "v2_protocol_history_missing"],
  ["changed identity", x => { x.deviceRecords[1].workerGeneration++; }, "v2_job_changed"],
  ["missing timing", x => { x.deviceRecords[1].timings.pop(); }, "v2_timing_coverage"],
  ["unreleased scope", x => { x.deviceRecords[1].resources.workerQuiescent = false; x.deviceRecords[1].resources.workerQuiescentAtUs = null; }, "v2_protocol_incomplete"],
  ["unexpected peer", x => { x.fixtureConnectionFacts.unexpectedPeerCount = 1; }, "v2_fixture_inventory"],
  ["false tuple", x => { x.connectionComparison.tupleMatch = false; }, "v2_tuple_comparison"],
  ["wrong pool incarnation", x => { x.connectionComparison.poolTransportEpoch++; }, "v2_comparison_binding"],
  ["fixture deadline", x => { x.fixtureTerminal.elapsedMs = 150000; }, "v2_fixture_lifetime"],
  ["missing actual EOF", x => { x.fixtureTerminal.peerClosed = false; }, "v2_fixture_incomplete"],
  ["digesting secret frame", x => { x.fixtureEvents.events[0].payloadSha256 = "b".repeat(64); }, "v2_fixture_frame_commitment"],
  ["job not first", x => { [x.fixtureEvents.events[3].kind, x.fixtureEvents.events[4].kind] = [x.fixtureEvents.events[4].kind, x.fixtureEvents.events[3].kind]; }, "v2_fixture_protocol_order"],
  ["wrong received commitment", x => { x.deviceRecords[1].jobCommitment = "b".repeat(64); }, "v2_native_job_commitment"],
  ["private tuple leak", x => { x.connectionComparison.endpoint = "forbidden"; }, "v2_object_shape"],
]) test(`Channel rejects ${name}`, () => {
  const input = channelFixture(); mutate(input);
  assert.throws(() => judgeV2Protocol(input), { code });
});
test("earliest native typed failure survives later incomplete proof", () => {
  const input = channelFixture(); input.deviceRecords[0].firstFailure = { stage: "setup", category: "protocol", atDeviceUs: 1000 };
  input.fixtureTerminal.peerClosed = false;
  assert.throws(() => judgeV2Protocol(input), error => error.code === "v2_device_failure" && error.firstFailure.stage === "setup");
});
test("operation deadlines use device duration, never fixture receipt time", () => {
  const input = channelFixture(), final = input.deviceRecords[1];
  final.observedAtUs = 10000000;
  const connect = final.timings.find(row => row.operation === "connect");
  connect.maxDurationUs = connect.totalDurationUs = 5000001; connect.lastFinishedAtDeviceUs = connect.firstStartedAtDeviceUs + 5000001;
  assert.throws(() => judgeV2Protocol(input), { code: "v2_operation_deadline" });
});
test("context projection forbids channel allowance and unknown Share policy", () => {
  assert.throws(() => parseProtocolContext({ scope: "channel", attemptId: nonce, qualificationAttempt: {} }));
  const context = { scope: "share", attemptId: nonce, qualificationAttempt: { schema: "worker-qualification-attempt-v1", id: nonce, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 } };
  assert.deepEqual(parseProtocolContext(context), context);
  context.qualificationAttempt.maximumActiveMilliseconds = 30000; assert.throws(() => parseProtocolContext(context));
});
test("Share field joins distinguish device receipt from fixture ACK write", () => {
  const { fact, share } = sharePair();
  assert.deepEqual(matchShareFields(fact, share), share.submission);
  fact.ackAtDeviceUs = null; fact.ackLastSequence = fact.ackAcceptedCount = fact.ackSharesSum = fact.matchedSubmitCount = null;
  assert.deepEqual(matchShareFields(fact, share), share.submission);
});
for (const [name, mutate] of [
  ["fabricated nonce", (f, s) => { s.submission.nonce++; }],
  ["wrong accepted sum", f => { f.ackSharesSum = 1; }],
  ["queue-only write", f => { f.writeCompletedAtDeviceUs = null; }],
  ["duplicate credit", (f, s) => { s.acceptedCount = 2; }],
]) test(`Share rejects ${name}`, () => {
  const { fact, share } = sharePair(); mutate(fact, share);
  assert.throws(() => matchShareFields(fact, share));
});
test("claimed fixture validity never substitutes for independent finite-target hashing", () => {
  const { job, fact, share } = sharePair();
  assert.throws(() => judgeShares({ observedAtUs: 14000, events: [{ kind: "work_ready", atDeviceUs: 9000 }], shareFacts: [fact] }, job, { shares: [share], events: [] }), { code: "v2_unqualified_nonce" });
});
test("heartbeat safety permits a pre-revocation write to finish later without permitting new writes", () => {
  const { fact } = sharePair();
  const record = { workerGeneration: 2, poolSessionGeneration: 4, serialTransportEpoch: 3, poolTransportEpoch: 5,
    observedAtUs: 3010000, authorityDeadlineDeviceUs: 180001000, shareFacts: [fact], events: [{ kind: "revoked", atDeviceUs: 3005000 }, { kind: "shutdown", atDeviceUs: 3006000 }] };
  const fault = { workerGeneration: 2, poolSessionGeneration: 4, serialTransportEpoch: 3, poolTransportEpoch: 5,
    selectedDeviceAckSha256: "a".repeat(64), suppressionRequestedAtHostMs: 1, headroomObservedAtDeviceUs: 20000,
    leaseRemainingMs: 5000, workGateRemainingMs: 5000, revocationReason: "heartbeat_timeout", lastValidHeartbeatAtDeviceUs: 200000,
    gateClosedAtDeviceUs: 3000000, shutdownStartedAtDeviceUs: 3001000, observerTailMs: 5000 };
  const late = { ...fact, writeStartedAtDeviceUs: 2999000, writeCompletedAtDeviceUs: 3000500, dispatchedAtDeviceUs: 10000 };
  record.shareFacts.push(late);
  assert.equal(judgeFault(fault, record, { fact }).heartbeatToRevocationUs, 2800000);
  late.writeStartedAtDeviceUs = 3000001;
  assert.throws(() => judgeFault(fault, record, { fact }), { code: "v2_activity_after_revocation" });
});

test("typed device failure is preserved before malformed later snapshots are considered", () => {
  const input = channelFixture(); input.deviceRecords[0].firstFailure = { stage: "setup", category: "authentication", atDeviceUs: 1000 };
  input.deviceRecords[1] = { incomplete: true };
  assert.throws(() => judgeV2Protocol(input), error => error.code === "v2_device_failure" && error.firstFailure.category === "authentication");
});
test("typed fixture failure remains distinct from generic missing cleanup", () => {
  const input = channelFixture(); input.fixtureTerminal.firstFailure = { stage: "peer_eof", category: "eof", atFixtureUs: 6000 };
  input.fixtureTerminal.peerClosed = false;
  assert.throws(() => judgeV2Protocol(input), error => error.code === "v2_fixture_failure" && error.firstFailure.category === "eof");
});
test("a repeated connect cannot masquerade as one admitted exchange", () => {
  const input = channelFixture(); input.deviceRecords[1].timings.find(row => row.operation === "connect").count = 2;
  assert.throws(() => judgeV2Protocol(input), { code: "v2_single_exchange" });
});

test("causal post-ACK headroom honors status millisecond precision without widening write authority", () => {
  const { fact } = sharePair(); fact.ackAtDeviceUs = 20999;
  const record = { workerGeneration: 2, poolSessionGeneration: 4, serialTransportEpoch: 3, poolTransportEpoch: 5,
    observedAtUs: 3010000, authorityDeadlineDeviceUs: 180001000, shareFacts: [fact],
    events: [{ kind: "revoked", atDeviceUs: 3005000 }, { kind: "shutdown", atDeviceUs: 3006000 }] };
  const fault = { workerGeneration: 2, poolSessionGeneration: 4, serialTransportEpoch: 3, poolTransportEpoch: 5,
    selectedDeviceAckSha256: "a".repeat(64), suppressionRequestedAtHostMs: 1, headroomObservedAtDeviceUs: 20000,
    leaseRemainingMs: 5000, workGateRemainingMs: 5000, revocationReason: "heartbeat_timeout", lastValidHeartbeatAtDeviceUs: 200000,
    gateClosedAtDeviceUs: 3000000, shutdownStartedAtDeviceUs: 3001000, observerTailMs: 5000 };
  assert.doesNotThrow(() => judgeFault(fault, record, { fact }));
  fact.writeStartedAtDeviceUs = 3000001;
  assert.throws(() => judgeFault(fault, record, { fact }), { code: "v2_activity_after_revocation" });
});
