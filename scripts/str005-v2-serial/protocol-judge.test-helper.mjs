import { createHash } from "node:crypto";
import { OPERATIONS } from "./device-record.mjs";
import { SHARE_TARGET, workPayload, decodeJob } from "./job-proof.mjs";
import { sha256 } from "./values.mjs";
export const nonce = Buffer.alloc(16, 1).toString("base64url");
const instance = Buffer.alloc(16, 2).toString("base64url");
const connection = Buffer.alloc(16, 3).toString("base64url");
function frame(type, channel, payload) {
  const h = Buffer.alloc(6); h.writeUInt16LE(channel ? 0x8000 : 0); h[2] = type; h.writeUIntLE(payload.length, 3, 3);
  return Buffer.concat([h, payload]);
}
export function jobFixture() {
  const target = Buffer.from(SHARE_TARGET.toString(16).padStart(64, "0"), "hex").reverse();
  const channel = Buffer.alloc(45); channel.writeUInt32LE(11); channel.writeUInt32LE(12, 4); target.copy(channel, 8);
  const job = Buffer.alloc(45); job.writeUInt32LE(12); job.writeUInt32LE(13, 4); job.writeUInt32LE(0x20000000, 9); job.fill(4, 13);
  const difficulty = Buffer.alloc(36); difficulty.writeUInt32LE(12); target.copy(difficulty, 4);
  const previous = Buffer.alloc(48); previous.writeUInt32LE(12); previous.writeUInt32LE(13, 4); previous.fill(5, 8, 40); previous.writeUInt32LE(17, 40); previous.writeUInt32LE(0x207fffff, 44);
  const frames = [frame(0x11, false, channel), frame(0x15, true, job), frame(0x21, true, difficulty), frame(0x20, true, previous)];
  const h = createHash("sha256"); for (const value of frames) { const length = Buffer.alloc(4); length.writeUInt32LE(value.length); h.update(length).update(value); }
  return { connectionId: connection, jobCommitment: h.digest("hex"), ...Object.fromEntries(["channelSuccess", "newMiningJob", "setTarget", "setNewPrevHash"].map((key, i) => [key, frames[i].toString("base64")])) };
}
export function channelFixture() {
  const job = jobFixture();
  const stages = ["admitted", "preparing", "connected", "authenticated", "setup", "channel", "job", "target", "work_ready", "socket_closed", "worker_quiescent"];
  const final = { schema: "worker-v2-serial-evidence-v1", scope: "channel", attemptId: nonce, bootOrdinal: 1, workerGeneration: 2,
    serialTransportEpoch: 3, poolSessionGeneration: 4, poolTransportEpoch: 5, jobCommitment: job.jobCommitment,
    observedAtUs: 14000, state: "terminal", admittedAtDeviceUs: 1000, authorityDeadlineDeviceUs: 120001000,
    observationDeadlineDeviceUs: 125001000, terminalAtDeviceUs: 13000, outcome: "accepted",
    events: stages.map((kind, i) => ({ sequence: i + 1, atDeviceUs: 1000 * (i + 1), kind, channelId: null, jobId: null, submissionSequence: null, payloadSha256: null })),
    timings: OPERATIONS.map(operation => ({ operation, count: 1, failedCount: 0, maxDurationUs: 20, totalDurationUs: 20, firstStartedAtDeviceUs: operation === "connect" ? 1300 : 1100, lastFinishedAtDeviceUs: operation === "connect" ? 1320 : 1120, inFlightStartedAtDeviceUs: null })),
    shareFacts: [], firstFailure: null, secondaryFailures: [], resources: { socketClosed: true, workerQuiescent: true, fenceRetained: false, socketClosedAtUs: 10000, workerQuiescentAtUs: 11000 } };
  const admitted = structuredClone(final); Object.assign(admitted, { state: "admitted", observedAtUs: 1000, poolSessionGeneration: null, poolTransportEpoch: null, jobCommitment: null, outcome: null, terminalAtDeviceUs: null, timings: [], events: [final.events[0]], resources: { socketClosed: false, workerQuiescent: false, fenceRetained: true, socketClosedAtUs: null, workerQuiescentAtUs: null } });
  const kinds = ["setup_received", "channel_open_received", "channel_success_sent", "job_sent", "target_sent", "prev_hash_sent", "peer_eof", "listener_closed"];
  const events = kinds.map((kind, i) => ({ sequence: i + 1, atFixtureUs: i * 1000, kind,
    payloadSha256: i >= 2 && i <= 5 ? sha256(Buffer.from(job[["channelSuccess", "newMiningJob", "setTarget", "setNewPrevHash"][i - 2]], "base64")) : null, submissionSequence: null }));
  return { scope: "channel", context: { scope: "channel", attemptId: nonce }, deviceRecords: [admitted, final], job,
    fixtureEvents: { connectionId: connection, events }, fixtureShares: { connectionId: connection, shares: [] },
    fixtureTerminal: { instanceId: instance, connectionId: connection, outcome: "accepted", firstFailure: null, elapsedMs: 8, receivedShares: 0, acceptedShares: 0, rejectedShares: 0, duplicateShares: 0, peerClosed: true, socketClosed: true, listenerClosed: true },
    fixtureConnectionFacts: { instanceId: instance, connectionId: connection, expectedPeerCount: 1, unexpectedPeerCount: 0, candidateOverflow: false, expectedPeerMatch: true },
    connectionComparison: { attemptId: nonce, instanceId: instance, connectionId: connection, bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: 3, poolSessionGeneration: 4, poolTransportEpoch: 5,
      expectedPeerMatch: true, tupleMatch: true, expectedPeerCount: 1, unexpectedPeerCount: 0, candidateOverflow: false, readinessSha256: "a".repeat(64), deviceObservationSequence: 2, comparisonStartedAtMs: 1, comparisonCompletedAtMs: 2 } };
}
export function sharePair() {
  const job = jobFixture(), decoded = decodeJob(job);
  const fact = { dispatchSequence: 1, asicJobId: 8, workFieldsSha256: sha256(workPayload(decoded, 8)), dispatchedAtDeviceUs: 10000, nonceAtDeviceUs: 11000,
    writeStartedAtDeviceUs: 12000, writeCompletedAtDeviceUs: 12500, nonce: 3, versionBits: 0, asicIndex: 0, coreId: 1, smallCoreId: 2,
    channelId: 12, jobId: 13, submissionSequence: 1, ntime: 17, version: 0x20000000, ackAtDeviceUs: 13000, ackLastSequence: 1, ackAcceptedCount: 1, ackSharesSum: 1024, matchedSubmitCount: 1 };
  const share = { submission: { channelId: 12, sequenceNumber: 1, jobId: 13, nonce: 3, ntime: 17, version: 0x20000000 }, receivedAtFixtureUs: 6000,
    headerSha256d: "a".repeat(64), targetValid: true, writeStartedAtFixtureUs: 6500, writeCompletedAtFixtureUs: 7000, acceptedCount: 1, sharesSum: 1024 };
  return { job, fact, share };
}
