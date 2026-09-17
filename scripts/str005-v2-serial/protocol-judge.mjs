import { bytes, check, list, object, SCOPES, uint } from "./values.mjs";
import { parseDeviceRecord, OPERATIONS } from "./device-record.mjs";
import { validateRecordProgress } from "./device.mjs";
import { decodeJob } from "./job-proof.mjs";
import { judgeFixture } from "./protocol-fixture.mjs";
import { judgeFault, judgeShares } from "./protocol-shares.mjs";

const STAGES = ["admitted", "preparing", "connected", "authenticated", "setup", "channel", "job", "target", "work_ready"];
/** The file wrapper derives this projection only after authenticating the complete frozen context. */
export function parseProtocolContext(context) {
  check(SCOPES.includes(context?.scope), "v2_protocol_scope");
  object(context, context.scope === "share" ? ["scope", "attemptId", "qualificationAttempt"] : ["scope", "attemptId"]);
  bytes(context.attemptId, 16);
  if (context.scope === "share") {
    const attempt = context.qualificationAttempt;
    object(attempt, ["schema", "id", "ordinal", "purpose", "maximumActiveMilliseconds"]);
    uint(attempt.ordinal, 0xffffffff);
    check(attempt.schema === "worker-qualification-attempt-v1" && attempt.id === context.attemptId && attempt.ordinal > 0 && attempt.purpose === "normal" && attempt.maximumActiveMilliseconds === 180000, "v2_protocol_allowance");
  }
  return structuredClone(context);
}
function records(input, context) {
  const values = [];
  for (const [index, value] of list(input, 1024).entries()) {
    const record = parseDeviceRecord(value);
    check(record.scope === context.scope && record.attemptId === context.attemptId, "v2_protocol_record_binding");
    if (record.firstFailure) throw Object.assign(new Error("v2_device_failure"), { code: "v2_device_failure", firstFailure: structuredClone(record.firstFailure) });
    check(record.secondaryFailures.length === 0, "v2_device_secondary_failure");
    if (index) validateRecordProgress(values[index - 1], record);
    values.push(record);
  }
  check(values.length >= 2, "v2_protocol_history_missing");
  check(context.scope === "channel" ? values[0].state === "admitted" : ["admitted", "running"].includes(values[0].state), "v2_protocol_admission_missing");
  const final = values.at(-1);
  check(final.state === "terminal" && final.outcome === "accepted" && final.resources.socketClosed && final.resources.workerQuiescent && !final.resources.fenceRetained, "v2_protocol_incomplete");
  check(final.poolSessionGeneration !== null && final.poolTransportEpoch !== null && final.jobCommitment !== null, "v2_protocol_session_missing");
  return { values, final };
}
function deviceStages(record, job) {
  const events = record.events;
  let previous = -1;
  for (const kind of [...STAGES, "socket_closed", "worker_quiescent"]) {
    const indices = events.map((event, index) => event.kind === kind ? index : -1).filter(index => index >= 0);
    check(indices.length === 1 && indices[0] > previous, "v2_device_stage_order"); previous = indices[0];
  }
  check(events[0].kind === "admitted" && events[0].atDeviceUs === record.admittedAtDeviceUs, "v2_device_admission_epoch");
  for (const event of events) {
    check(event.atDeviceUs !== null && event.atDeviceUs >= record.admittedAtDeviceUs && event.atDeviceUs <= record.observedAtUs, "v2_device_stage_time");
    if (event.channelId !== null) check(event.channelId === job.channelId, "v2_device_channel_join");
    if (event.jobId !== null) check(event.jobId === job.jobId, "v2_device_job_join");
  }
  check(record.resources.socketClosedAtUs !== null && record.resources.workerQuiescentAtUs !== null && record.resources.socketClosedAtUs <= record.resources.workerQuiescentAtUs && record.resources.workerQuiescentAtUs <= record.observedAtUs, "v2_device_resource_time");
  if (record.scope === "channel") {
    check(record.shareFacts.length === 0 && events.every(event => !["asic_dispatch", "nonce", "submission", "accepted", "revoked", "shutdown", "cooled"].includes(event.kind)), "v2_channel_asic_effect");
    check(record.terminalAtDeviceUs <= record.authorityDeadlineDeviceUs && record.resources.workerQuiescentAtUs <= record.authorityDeadlineDeviceUs, "v2_channel_authority_deadline");
  }
}
function timings(record) {
  const found = new Map(record.timings.map(row => [row.operation, row]));
  check(found.size === OPERATIONS.length, "v2_timing_coverage");
  for (const operation of OPERATIONS) {
    const row = found.get(operation);
    check(row && row.count > 0 && row.inFlightStartedAtDeviceUs === null && row.maxDurationUs !== null && row.totalDurationUs !== null && row.firstStartedAtDeviceUs !== null && row.lastFinishedAtDeviceUs !== null && row.lastFinishedAtDeviceUs <= record.observedAtUs, "v2_timing_incomplete");
    if (["initiator_construction", "act_one_construction", "connect", "act_one_write", "act_two_read", "act_two_authentication"].includes(operation)) check(row.count === 1 && row.failedCount === 0, "v2_single_exchange");
    if (record.scope === "channel") check(row.failedCount === 0, "v2_channel_operation_failed");
    const limit = operation === "connect" ? 5000000 : ["act_two_read", "frame_read"].includes(operation) ? 10000000 : ["act_one_write", "frame_write"].includes(operation) ? 2000000 : null;
    if (limit !== null) check(row.maxDurationUs <= limit, "v2_operation_deadline");
  }
  const prepared = found.get("connect").firstStartedAtDeviceUs - record.admittedAtDeviceUs;
  check(prepared >= 0 && prepared <= 60000000 && found.get("initiator_construction").totalDurationUs + found.get("act_one_construction").totalDurationUs <= 60000000, "v2_preparation_deadline");
  return { preparationUs: prepared, maximumReadUs: Math.max(found.get("act_two_read").maxDurationUs, found.get("frame_read").maxDurationUs),
    maximumWriteUs: Math.max(found.get("act_one_write").maxDurationUs, found.get("frame_write").maxDurationUs), connectUs: found.get("connect").maxDurationUs,
    resourceReleaseUs: record.resources.workerQuiescentAtUs - record.admittedAtDeviceUs };
}
/** Pure protocol judgment only. Provenance, cycles, native fit, ledgers and cleanup are separate joins. */
export function judgeV2Protocol(input) {
  const expectedKeys = ["scope", "context", "deviceRecords", "job", "fixtureEvents", "fixtureTerminal", "fixtureConnectionFacts", "connectionComparison", "fixtureShares"];
  if (input?.scope === "share") expectedKeys.push("fault");
  object(input, expectedKeys);
  const context = parseProtocolContext(input.context); check(input.scope === context.scope, "v2_protocol_scope");
  const { values, final } = records(input.deviceRecords, context), job = decodeJob(input.job);
  check(final.jobCommitment === job.jobCommitment, "v2_native_job_commitment");
  deviceStages(final, job);
  const measured = timings(final), fixture = judgeFixture(input, final, job);
  const counts = { deviceRecords: values.length, deviceEvents: final.events.length, fixtureEvents: fixture.events.length,
    connections: 1, submitted: fixture.shares.length, deviceAcknowledged: 0, unsubmitted: 0 };
  if (input.scope === "channel") return { scope: "channel", acceptedProtocol: true, counts, timings: measured };
  const shares = judgeShares(final, input.job, fixture), fault = judgeFault(input.fault, final, shares.selected);
  counts.deviceAcknowledged = shares.acknowledged; counts.unsubmitted = shares.unsubmitted;
  return { scope: "share", acceptedProtocol: true, counts, timings: { ...measured, ...fault }, selectedShare: {
    submissionSequence: shares.selected.fact.submissionSequence, deviceAckAtUs: shares.selected.fact.ackAtDeviceUs,
    headerSha256: shares.selected.headerSha256, sha256d: shares.selected.sha256d } };
}
