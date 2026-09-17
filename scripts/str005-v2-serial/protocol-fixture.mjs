import { bytes, check, digest, list, object, sha256, uint } from "./values.mjs";
import { parseConnectionFacts } from "./fixture.mjs";

const BEGIN = ["setup_received", "channel_open_received", "channel_success_sent", "job_sent", "target_sent", "prev_hash_sent"];
const STAGES = [...BEGIN, "share_received", "share_success_sent", "peer_eof", "listener_closed"];
const uintKeys = (value, keys) => { for (const key of keys) uint(value[key]); };

/** Strict native fixture projection; no private runtime readiness or socket tuple is accepted. */
export function judgeFixture(input, record, job) {
  const { fixtureEvents: trace, fixtureTerminal: terminal, fixtureConnectionFacts: counts, connectionComparison: compared, fixtureShares } = input;
  object(trace, ["connectionId", "events"]); bytes(trace.connectionId, 16);
  object(terminal, ["instanceId", "connectionId", "outcome", "firstFailure", "elapsedMs", "receivedShares", "acceptedShares", "rejectedShares", "duplicateShares", "peerClosed", "socketClosed", "listenerClosed"]);
  bytes(terminal.instanceId, 16); bytes(terminal.connectionId, 16);
  uintKeys(terminal, ["elapsedMs", "receivedShares", "acceptedShares", "rejectedShares", "duplicateShares"]);
  if (terminal.firstFailure !== null) {
    const failure = terminal.firstFailure; object(failure, ["stage", "category", "atFixtureUs"]);
    check(STAGES.includes(failure.stage) && ["admission", "clock", "allocation", "authority", "timeout", "eof", "extra", "authentication", "protocol", "channel_mismatch", "job_mismatch", "invalid_nonce", "rejected_share", "safety", "cleanup", "evidence"].includes(failure.category), "v2_fixture_failure_shape");
    if (failure.atFixtureUs !== null) uint(failure.atFixtureUs);
    throw Object.assign(new Error("v2_fixture_failure"), { code: "v2_fixture_failure", firstFailure: structuredClone(failure) });
  }
  check(terminal.outcome === "accepted" && terminal.firstFailure === null && terminal.peerClosed === true && terminal.socketClosed === true && terminal.listenerClosed === true, "v2_fixture_incomplete");
  check(terminal.elapsedMs < (input.scope === "channel" ? 150000 : 300000), "v2_fixture_lifetime");
  parseConnectionFacts(counts);
  check(counts.instanceId === terminal.instanceId && counts.connectionId === terminal.connectionId && counts.expectedPeerMatch && counts.expectedPeerCount === 1 && counts.unexpectedPeerCount === 0 && !counts.candidateOverflow, "v2_fixture_inventory");
  object(compared, ["attemptId", "instanceId", "connectionId", "bootOrdinal", "workerGeneration", "serialTransportEpoch", "poolSessionGeneration", "poolTransportEpoch", "expectedPeerMatch", "tupleMatch", "expectedPeerCount", "unexpectedPeerCount", "candidateOverflow", "readinessSha256", "deviceObservationSequence", "comparisonStartedAtMs", "comparisonCompletedAtMs"]);
  digest(compared.readinessSha256);
  uintKeys(compared, ["deviceObservationSequence", "comparisonStartedAtMs", "comparisonCompletedAtMs"]);
  check(compared.deviceObservationSequence > 0 && compared.comparisonCompletedAtMs >= compared.comparisonStartedAtMs, "v2_comparison_order");
  for (const key of ["attemptId", "bootOrdinal", "workerGeneration", "serialTransportEpoch", "poolSessionGeneration", "poolTransportEpoch"]) check(compared[key] === record[key], "v2_comparison_binding");
  check(compared.instanceId === counts.instanceId && compared.connectionId === counts.connectionId && compared.tupleMatch === true && compared.expectedPeerMatch === true && compared.expectedPeerCount === 1 && compared.unexpectedPeerCount === 0 && compared.candidateOverflow === false, "v2_tuple_comparison");
  check(trace.connectionId === terminal.connectionId && job.connectionId === terminal.connectionId, "v2_fixture_connection_join");
  object(fixtureShares, ["connectionId", "shares"]); check(fixtureShares.connectionId === terminal.connectionId, "v2_fixture_share_connection");
  list(fixtureShares.shares, 1024);
  check(terminal.receivedShares === fixtureShares.shares.length && terminal.acceptedShares === fixtureShares.shares.length && terminal.rejectedShares === 0 && terminal.duplicateShares === 0, "v2_fixture_share_counts");
  const events = list(trace.events, 1024);
  check(events.length >= 8 && events.length === 8 + fixtureShares.shares.length * 2, "v2_fixture_event_count");
  let previousUs = 0;
  events.forEach((event, index) => {
    object(event, ["sequence", "atFixtureUs", "kind", "payloadSha256", "submissionSequence"]);
    uint(event.atFixtureUs); check(event.sequence === index + 1 && STAGES.includes(event.kind) && event.atFixtureUs >= previousUs, "v2_fixture_event_order");
    check(Math.floor(event.atFixtureUs / 1000) <= terminal.elapsedMs, "v2_fixture_event_after_exit"); previousUs = event.atFixtureUs;
    if (event.payloadSha256 !== null) digest(event.payloadSha256);
    if (event.submissionSequence !== null) uint(event.submissionSequence, 0xffffffff);
  });
  for (const [index, kind] of BEGIN.entries()) {
    const event = events[index]; check(event.kind === kind && event.submissionSequence === null, "v2_fixture_protocol_order");
    const frameKey = [null, null, "channelSuccess", "newMiningJob", "setTarget", "setNewPrevHash"][index];
    check(event.payloadSha256 === (frameKey === null ? null : sha256(Buffer.from(input.job[frameKey], "base64"))), "v2_fixture_frame_commitment");
  }
  for (const [offset, kind] of ["peer_eof", "listener_closed"].entries()) {
    const event = events[events.length - 2 + offset];
    check(event.kind === kind && event.payloadSha256 === null && event.submissionSequence === null, "v2_fixture_close_order");
  }
  if (input.scope === "channel") check(fixtureShares.shares.length === 0, "v2_channel_submission");
  return { events, terminal, shares: fixtureShares.shares };
}

function frame(type, payload) {
  const head = Buffer.alloc(6); head.writeUInt16LE(0x8000); head[2] = type; head.writeUIntLE(payload.length, 3, 3);
  return Buffer.concat([head, payload]);
}
export function submissionFrame(submission) {
  const payload = Buffer.alloc(24);
  for (const [index, key] of ["channelId", "sequenceNumber", "jobId", "nonce", "ntime", "version"].entries()) payload.writeUInt32LE(submission[key], index * 4);
  return frame(0x1a, payload);
}
export function acknowledgementFrame(submission) {
  const payload = Buffer.alloc(20); payload.writeUInt32LE(submission.channelId); payload.writeUInt32LE(submission.sequenceNumber, 4);
  payload.writeUInt32LE(1, 8); payload.writeBigUInt64LE(1024n, 12);
  return frame(0x1c, payload);
}
