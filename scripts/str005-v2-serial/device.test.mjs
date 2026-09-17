import assert from "node:assert/strict";
import test from "node:test";
import { collectedRecord, parseStatus, validateProgress } from "./device.mjs";
import { OPERATIONS } from "./device-record.mjs";
import { parseStratum, PROFILE } from "./values.mjs";

const nonce = Buffer.alloc(16, 1).toString("base64url");
const stratum = () => ({ profile: PROFILE, endpoint: "stratum+tcp://192.168.20.3:33000/",
  authorityPublicKey: Buffer.alloc(32, 2).toString("base64url"), userIdentity: "synthetic-fixture-user" });
function idle() {
  return { schema: "worker-stratum-v2-status-v1", scope: "channel", state: "idle", connection: null, record: null,
    observation: { bootOrdinal: 2, workerGeneration: 4, serialTransportEpoch: 6, observedAtUs: 1000,
      clockValid: true, stationIpv4: "192.168.20.4", wifiConnected: true, socket: null } };
}
function running() {
  const s = idle(); s.state = "running";
  s.record = { schema: "worker-v2-serial-evidence-v1", scope: "channel", attemptId: nonce, bootOrdinal: 2,
    workerGeneration: 4, serialTransportEpoch: 6, poolSessionGeneration: null, poolTransportEpoch: null,
    jobCommitment: null, observedAtUs: 1000, state: "running", admittedAtDeviceUs: 100,
    authorityDeadlineDeviceUs: 120000100, observationDeadlineDeviceUs: 125000100, terminalAtDeviceUs: null,
    outcome: null, events: [{ sequence: 1, atDeviceUs: 100, kind: "admitted", channelId: null, jobId: null,
      submissionSequence: null, payloadSha256: null }], timings: [], shareFacts: [], firstFailure: null,
    secondaryFailures: [], resources: { socketClosed: false, workerQuiescent: false, fenceRetained: true,
      socketClosedAtUs: null, workerQuiescentAtUs: null } };
  return s;
}
function connected() {
  const s = running(); s.record.poolSessionGeneration = 7; s.record.poolTransportEpoch = 9;
  const socket = { localIpv4: "192.168.20.4", localPort: 42000, remoteIpv4: "192.168.20.3", remotePort: 33000 };
  s.observation.socket = socket;
  s.connection = { observedAtUs: 200, bootOrdinal: 2, workerGeneration: 4, serialTransportEpoch: 6,
    poolSessionGeneration: 7, poolTransportEpoch: 9, socket };
  s.record.events.push({ sequence: 2, atDeviceUs: 200, kind: "connected", channelId: null,
    jobId: null, submissionSequence: null, payloadSha256: null });
  return s;
}

test("persistable record excludes current and retained endpoint observations", () => {
  // Arrange
  const status = connected();
  // Act
  const record = collectedRecord(status);
  const encoded = JSON.stringify(record);
  // Assert
  assert.equal(record.poolTransportEpoch, 9);
  for (const forbidden of ["192.168.20.3", "192.168.20.4", "33000", "42000", '"socket":', '"stationIpv4":'])
    assert.equal(encoded.includes(forbidden), false);
  assert.notEqual(record, status.record);
});

test("closed fast channel retains observed tuple without claiming an open socket", () => {
  // Arrange
  const status = connected(); status.observation.socket = null;
  status.record.resources.socketClosed = true; status.record.resources.socketClosedAtUs = 300;
  // Act
  const parsed = parseStatus(status);
  // Assert
  assert.equal(parsed.observation.socket, null);
  assert.equal(parsed.connection.socket.localPort, 42000);
});

test("new serial possession does not rewrite retained job identifiers", () => {
  // Arrange
  const before = connected(), after = structuredClone(before);
  after.observation.workerGeneration = 10; after.observation.serialTransportEpoch = 11;
  // Act / Assert
  assert.doesNotThrow(() => validateProgress(before, after));
  after.record.workerGeneration = 10;
  assert.throws(() => validateProgress(before, after), { code: "v2_connection_binding" });
});

test("connected evidence cannot silently lose the retained connection", () => {
  const status = connected(); status.connection = null;
  assert.throws(() => parseStatus(status), { code: "v2_connection_missing" });
});

test("idle clock loss is a failed command, not an apparently fresh observation", () => {
  const status = idle(); status.observation.clockValid = false; status.observation.observedAtUs = null;
  assert.throws(() => parseStatus(status), { code: "v2_idle_record" });
});

test("later clock failure preserves the earliest protocol cause and actual release", () => {
  // Arrange
  const status = connected(); status.state = "terminal"; status.record.state = "terminal";
  status.record.firstFailure = { stage: "setup", category: "protocol", atDeviceUs: 250 };
  status.record.secondaryFailures = [{ stage: "socket_closed", category: "clock", atDeviceUs: null }];
  status.record.outcome = "rejected"; status.record.terminalAtDeviceUs = null; status.record.observedAtUs = null;
  status.observation.clockValid = false; status.observation.observedAtUs = null; status.observation.socket = null;
  status.record.resources = { socketClosed: true, workerQuiescent: true, fenceRetained: false,
    socketClosedAtUs: null, workerQuiescentAtUs: null };
  // Act
  const result = collectedRecord(status);
  // Assert
  assert.equal(result.firstFailure.category, "protocol");
  assert.equal(result.resources.workerQuiescent, true);
  assert.equal(result.resources.workerQuiescentAtUs, null);
});

test("credential-bearing unexpected fields and payload digests are rejected before persistence", () => {
  // Arrange
  const status = running(); status.record.events[0].kind = "setup";
  status.record.events[0].payloadSha256 = "a".repeat(64);
  // Act / Assert
  assert.throws(() => collectedRecord(status), { code: "v2_secret_payload_digest" });
  status.record.events[0].payloadSha256 = null; status.record.endpoint = stratum().endpoint;
  assert.throws(() => collectedRecord(status), { code: "v2_object_shape" });
});

test("altering already observed stages is rejected", () => {
  const before = running(), after = structuredClone(before);
  after.record.events[0].atDeviceUs += 1;
  assert.throws(() => validateProgress(before, after), { code: "v2_history_changed" });
});

test("a transport epoch is not interchangeable with a worker generation", () => {
  const status = connected(); status.connection.poolTransportEpoch = status.record.workerGeneration;
  assert.throws(() => parseStatus(status), { code: "v2_connection_binding" });
});

test("each timing operation has exactly one bounded aggregate", () => {
  const status = running();
  const timing = { operation: OPERATIONS[0], count: 0, failedCount: 0, maxDurationUs: null, totalDurationUs: null,
    firstStartedAtDeviceUs: null, lastFinishedAtDeviceUs: null, inFlightStartedAtDeviceUs: null };
  status.record.timings = [timing, structuredClone(timing)];
  assert.throws(() => parseStatus(status), { code: "v2_duplicate_timing" });
});

test("V2 endpoint parsing rejects normalization, public routes and downgrade fields", () => {
  for (const endpoint of ["stratum+tcp://192.168.020.3:33000/", "stratum+tcp://127.0.0.1:33000/",
    "stratum+tcp://8.8.8.8:33000/", "stratum+tcp://192.168.20.3:33000/?x=1", "stratum+tcp://pool.test:33000/"])
    assert.throws(() => parseStratum({ ...stratum(), endpoint }));
  assert.throws(() => parseStratum({ ...stratum(), username: "mixed-v1" }));
  assert.throws(() => parseStratum({ ...stratum(), profile: null }));
  assert.deepEqual(parseStratum(stratum()), stratum());
});

test("UTF8 size and validity are checked before handing identities to the native parser", () => {
  assert.throws(() => parseStratum({ ...stratum(), userIdentity: "🧪".repeat(64) }));
  assert.throws(() => parseStratum({ ...stratum(), userIdentity: "\ud800" }));
  assert.doesNotThrow(() => parseStratum({ ...stratum(), userIdentity: "🧪".repeat(63) }));
});

function preparingShare() {
  const status = running(); status.scope = "share"; status.record.scope = "share";
  status.record.authorityDeadlineDeviceUs = null; status.record.observationDeadlineDeviceUs = null;
  return status;
}

test("Share preparation has no fabricated reservation deadline", () => {
  const status = preparingShare();
  assert.equal(parseStatus(status).record.authorityDeadlineDeviceUs, null);
  status.record.state = "terminal"; status.state = "terminal"; status.record.outcome = "cancelled";
  status.record.terminalAtDeviceUs = 500;
  status.record.firstFailure = { stage: "preparing", category: "authority", atDeviceUs: 500 };
  assert.equal(parseStatus(status).record.authorityDeadlineDeviceUs, null);
});

test("only first guarded dispatch may establish the previously unknown Share deadline", () => {
  // Arrange
  const before = preparingShare(), after = structuredClone(before);
  after.record.authorityDeadlineDeviceUs = 180001000;
  // Act / Assert
  assert.doesNotThrow(() => validateProgress(before, after));
  const changed = structuredClone(after); changed.record.authorityDeadlineDeviceUs += 1000;
  assert.throws(() => validateProgress(after, changed), { code: "v2_retained_fact_changed" });
  changed.record.authorityDeadlineDeviceUs = null;
  assert.throws(() => validateProgress(after, changed), { code: "v2_retained_fact_changed" });
});

test("actual work cannot be recorded while its native reservation clock is unknown", () => {
  const status = preparingShare();
  status.record.events.push({ sequence: 2, atDeviceUs: 500, kind: "asic_dispatch", channelId: 1,
    jobId: 2, submissionSequence: null, payloadSha256: null });
  assert.throws(() => parseStatus(status), { code: "v2_unarmed_work_evidence" });
});
