import { boolean, bytes, check, digest, list, nullable, object, SCOPES, uint } from "./values.mjs";

export const STAGES = Object.freeze(["admitted", "preparing", "connected", "authenticated", "setup", "channel",
  "job", "target", "work_ready", "socket_closed", "worker_quiescent", "asic_dispatch", "nonce", "submission",
  "accepted", "revoked", "shutdown", "cooled"]);
export const OPERATIONS = Object.freeze(["initiator_construction", "act_one_construction", "connect", "act_one_write",
  "act_two_read", "act_two_authentication", "frame_encrypt", "frame_write", "frame_read", "header_decrypt",
  "payload_decrypt", "socket_close", "worker_join"]);
const FAILURES = ["admission", "clock", "allocation", "authority", "timeout", "eof", "extra", "authentication",
  "protocol", "channel_mismatch", "job_mismatch", "invalid_nonce", "rejected_share", "safety", "cleanup", "evidence"];
export const IDENTITY_KEYS = Object.freeze(["attemptId", "bootOrdinal", "workerGeneration", "serialTransportEpoch"]);
export const POOL_KEYS = Object.freeze(["poolSessionGeneration", "poolTransportEpoch"]);

export function failure(value) {
  object(value, ["stage", "category", "atDeviceUs"]);
  check(STAGES.includes(value.stage) && FAILURES.includes(value.category), "v2_failure");
  nullable(value.atDeviceUs, uint);
}
function event(value) {
  object(value, ["sequence", "atDeviceUs", "kind", "channelId", "jobId", "submissionSequence", "payloadSha256"]);
  uint(value.sequence); check(value.sequence > 0 && STAGES.includes(value.kind), "v2_event");
  nullable(value.atDeviceUs, uint);
  for (const key of ["channelId", "jobId", "submissionSequence"]) nullable(value[key], (v) => uint(v, 0xffffffff));
  nullable(value.payloadSha256, digest);
  if (["setup", "channel"].includes(value.kind)) check(value.payloadSha256 === null, "v2_secret_payload_digest");
}
function timing(value, clockFailed) {
  object(value, ["operation", "count", "failedCount", "maxDurationUs", "totalDurationUs",
    "firstStartedAtDeviceUs", "lastFinishedAtDeviceUs", "inFlightStartedAtDeviceUs"]);
  check(OPERATIONS.includes(value.operation), "v2_operation"); uint(value.count); uint(value.failedCount);
  check(value.failedCount <= value.count, "v2_failure_count");
  for (const key of ["maxDurationUs", "totalDurationUs", "firstStartedAtDeviceUs", "lastFinishedAtDeviceUs", "inFlightStartedAtDeviceUs"])
    nullable(value[key], uint);
  if (value.count === 0) check(value.firstStartedAtDeviceUs === null && value.maxDurationUs === null && value.totalDurationUs === null &&
    value.lastFinishedAtDeviceUs === null, "v2_unmeasured_timing");
  else if (!clockFailed) check(value.maxDurationUs !== null && value.totalDurationUs !== null &&
    value.firstStartedAtDeviceUs !== null && value.lastFinishedAtDeviceUs >= value.firstStartedAtDeviceUs &&
    value.totalDurationUs >= value.maxDurationUs, "v2_timing_missing");
  if (value.inFlightStartedAtDeviceUs !== null && value.lastFinishedAtDeviceUs !== null)
    check(value.inFlightStartedAtDeviceUs >= value.lastFinishedAtDeviceUs, "v2_timing_regression");
}
function shareFact(value, clockFailed) {
  object(value, ["dispatchSequence", "asicJobId", "workFieldsSha256", "dispatchedAtDeviceUs", "nonceAtDeviceUs",
    "writeStartedAtDeviceUs", "writeCompletedAtDeviceUs", "nonce", "versionBits", "asicIndex", "coreId", "smallCoreId",
    "channelId", "jobId", "submissionSequence", "ntime", "version", "ackAtDeviceUs", "ackLastSequence",
    "ackAcceptedCount", "ackSharesSum", "matchedSubmitCount"]);
  for (const key of ["dispatchSequence", "dispatchedAtDeviceUs", "nonceAtDeviceUs"]) uint(value[key]);
  for (const key of ["nonce", "versionBits", "channelId", "jobId", "submissionSequence", "ntime", "version"])
    uint(value[key], 0xffffffff);
  for (const key of ["asicJobId", "asicIndex", "coreId", "smallCoreId"]) uint(value[key], 255);
  digest(value.workFieldsSha256);
  for (const key of ["writeStartedAtDeviceUs", "writeCompletedAtDeviceUs", "ackAtDeviceUs", "ackLastSequence",
    "ackAcceptedCount", "ackSharesSum", "matchedSubmitCount"]) nullable(value[key], uint);
  if (!clockFailed) {
    check(value.nonceAtDeviceUs >= value.dispatchedAtDeviceUs, "v2_nonce_order");
    if (value.writeStartedAtDeviceUs !== null) check(value.writeStartedAtDeviceUs >= value.nonceAtDeviceUs, "v2_write_order");
    if (value.writeCompletedAtDeviceUs !== null) check(value.writeStartedAtDeviceUs !== null &&
      value.writeCompletedAtDeviceUs >= value.writeStartedAtDeviceUs, "v2_write_completion");
    if (value.ackAtDeviceUs !== null) check(value.writeCompletedAtDeviceUs !== null &&
      value.ackAtDeviceUs >= value.writeCompletedAtDeviceUs, "v2_ack_order");
  }
  const ackKeys = ["ackLastSequence", "ackAcceptedCount", "ackSharesSum", "matchedSubmitCount"];
  check((ackKeys.every((key) => value[key] === null) && value.ackAtDeviceUs === null) ||
    (ackKeys.every((key) => value[key] !== null) && (value.ackAtDeviceUs !== null || clockFailed)), "v2_partial_ack");
}

/** Validate collected device facts without treating structural validity as acceptance. */
export function parseDeviceRecord(value) {
  object(value, ["schema", "scope", ...IDENTITY_KEYS, ...POOL_KEYS, "jobCommitment", "observedAtUs", "state",
    "admittedAtDeviceUs", "authorityDeadlineDeviceUs", "observationDeadlineDeviceUs", "terminalAtDeviceUs", "outcome",
    "events", "timings", "shareFacts", "firstFailure", "secondaryFailures", "resources"]);
  check(value.schema === "worker-v2-serial-evidence-v1" && SCOPES.includes(value.scope), "v2_record_schema");
  bytes(value.attemptId, 16);
  for (const key of ["bootOrdinal", "workerGeneration", "serialTransportEpoch", "admittedAtDeviceUs"])
    uint(value[key]);
  check(value.bootOrdinal > 0, "v2_boot");
  for (const key of [...POOL_KEYS, "observedAtUs", "terminalAtDeviceUs", "observationDeadlineDeviceUs", "authorityDeadlineDeviceUs"]) nullable(value[key], uint);
  check((value.poolSessionGeneration === null) === (value.poolTransportEpoch === null), "v2_pool_binding");
  nullable(value.jobCommitment, digest); nullable(value.firstFailure, failure);
  list(value.secondaryFailures, 16).forEach(failure);
  const clockFailed = [value.firstFailure, ...value.secondaryFailures].some((f) => f?.category === "clock");
  check(value.observedAtUs !== null || clockFailed, "v2_clock_missing");
  check(["admitted", "running", "terminal"].includes(value.state), "v2_record_state");
  if (value.scope === "channel") check(value.authorityDeadlineDeviceUs === value.admittedAtDeviceUs + 120000000 &&
    value.observationDeadlineDeviceUs === value.admittedAtDeviceUs + 125000000, "v2_channel_deadline");
  else check((value.authorityDeadlineDeviceUs === null ||
    (value.authorityDeadlineDeviceUs > value.admittedAtDeviceUs && value.authorityDeadlineDeviceUs % 1000 === 0)) &&
    value.observationDeadlineDeviceUs === null, "v2_share_deadline");
  check((value.state === "terminal") === (value.outcome !== null), "v2_terminal_state");
  if (value.outcome !== null) check(["accepted", "rejected", "expired", "cancelled", "incomplete"].includes(value.outcome) &&
    (value.terminalAtDeviceUs !== null || clockFailed), "v2_terminal_outcome");
  let priorTime = value.admittedAtDeviceUs;
  list(value.events, 64).forEach((row, index) => {
    event(row); check(row.sequence === index + 1, "v2_event_sequence");
    if (row.atDeviceUs === null) check(clockFailed, "v2_event_clock");
    else { check(row.atDeviceUs >= priorTime, "v2_event_time"); priorTime = row.atDeviceUs; }
  });
  const seen = new Set();
  list(value.timings, OPERATIONS.length).forEach((row) => {
    timing(row, clockFailed); check(!seen.has(row.operation), "v2_duplicate_timing"); seen.add(row.operation);
  });
  list(value.shareFacts, 16).forEach((row) => shareFact(row, clockFailed));
  if (value.scope === "channel") check(value.shareFacts.length === 0, "v2_channel_work");
  else {
    if (value.shareFacts.length > 0 || value.outcome === "accepted" ||
      value.events.some((row) => ["asic_dispatch", "nonce", "submission", "accepted"].includes(row.kind)))
      check(value.authorityDeadlineDeviceUs !== null, "v2_unarmed_work_evidence");
    if (value.authorityDeadlineDeviceUs !== null) {
      const epochUs = value.authorityDeadlineDeviceUs - 180000000;
      check(epochUs >= Math.floor(value.admittedAtDeviceUs / 1000) * 1000 &&
        value.shareFacts.every((row) => row.dispatchedAtDeviceUs >= epochUs), "v2_dispatch_epoch");
    }
  }
  const r = value.resources;
  object(r, ["socketClosed", "workerQuiescent", "fenceRetained", "socketClosedAtUs", "workerQuiescentAtUs"]);
  for (const key of ["socketClosed", "workerQuiescent", "fenceRetained"]) boolean(r[key]);
  for (const [flag, time] of [["socketClosed", "socketClosedAtUs"], ["workerQuiescent", "workerQuiescentAtUs"]]) {
    nullable(r[time], uint);
    check(r[flag] ? r[time] !== null || clockFailed : r[time] === null, "v2_release_time");
  }
  // The network job can be quiescent while the independent ASIC restoration still owns the fence.
  return structuredClone(value);
}
