import { boolean, check, ipv4, nullable, object, SCOPES, tuple, uint } from "./values.mjs";
import { IDENTITY_KEYS, parseDeviceRecord, POOL_KEYS } from "./device-record.mjs";

/** Parse RAM-only transport observations. Persist only the separately validated record. */
export function parseStatus(value) {
  check(Buffer.byteLength(JSON.stringify(value), "utf8") <= 65536, "v2_reply_bound");
  object(value, ["schema", "scope", "state", "observation", "connection", "record"]);
  check(value.schema === "worker-stratum-v2-status-v1" && SCOPES.includes(value.scope) &&
    ["idle", "admitted", "running", "terminal"].includes(value.state), "v2_status_schema");
  const o = value.observation;
  object(o, ["bootOrdinal", "workerGeneration", "serialTransportEpoch", "observedAtUs", "clockValid",
    "stationIpv4", "wifiConnected", "socket"]);
  for (const key of ["bootOrdinal", "workerGeneration", "serialTransportEpoch"]) uint(o[key]);
  check(o.bootOrdinal > 0, "v2_boot"); boolean(o.clockValid); boolean(o.wifiConnected);
  nullable(o.observedAtUs, uint); nullable(o.stationIpv4, ipv4); nullable(o.socket, tuple);
  check(o.clockValid === (o.observedAtUs !== null) && (o.wifiConnected || o.stationIpv4 === null), "v2_observation_state");
  if (value.state === "idle") {
    check(value.record === null && value.connection === null && o.socket === null && o.clockValid, "v2_idle_record");
    return structuredClone(value);
  }
  const r = parseDeviceRecord(value.record);
  check(r.scope === value.scope && r.state === value.state && r.bootOrdinal === o.bootOrdinal, "v2_record_binding");
  const clockFailed = [r.firstFailure, ...r.secondaryFailures].some((f) => f?.category === "clock");
  check(o.clockValid || clockFailed, "v2_status_clock");
  if (value.connection !== null) {
    const c = value.connection;
    object(c, ["observedAtUs", "bootOrdinal", "workerGeneration", "serialTransportEpoch", ...POOL_KEYS, "socket"]);
    uint(c.observedAtUs); tuple(c.socket);
    for (const key of ["bootOrdinal", "workerGeneration", "serialTransportEpoch", ...POOL_KEYS])
      check(c[key] === r[key] && c[key] !== null, "v2_connection_binding");
    check(c.observedAtUs >= r.admittedAtDeviceUs && (o.observedAtUs === null || c.observedAtUs <= o.observedAtUs), "v2_connection_time");
  }
  if (r.events.some((row) => row.kind === "connected")) check(value.connection !== null, "v2_connection_missing");
  if (o.observedAtUs !== null && r.observedAtUs !== null) check(r.observedAtUs <= o.observedAtUs, "v2_snapshot_time");
  return structuredClone(value);
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
function immutableDefined(before, after, keys, code) {
  for (const key of keys) if (before[key] !== null) check(same(before[key], after[key]), code);
}

/** Validate append-only facts; reconnect cannot relabel an earlier job as a fresh one. */
export function validateProgress(previous, current) {
  const before = parseStatus(previous), after = parseStatus(current);
  check(before.scope === after.scope && before.observation.bootOrdinal === after.observation.bootOrdinal, "v2_progress_scope");
  if (before.record === null) return;
  check(after.record !== null, "v2_record_lost");
  if (before.connection !== null) check(same(before.connection, after.connection), "v2_connection_changed");
  validateRecordProgress(before.record, after.record);
}

/** The sealed reader rechecks history without requiring intentionally unpersisted tuples. */
export function validateRecordProgress(previous, current) {
  const a = parseDeviceRecord(previous), b = parseDeviceRecord(current);
  check(a.scope === b.scope, "v2_progress_scope");
  check(["admitted", "running", "terminal"].indexOf(b.state) >= ["admitted", "running", "terminal"].indexOf(a.state),
    "v2_state_regression");
  for (const key of [...IDENTITY_KEYS, "admittedAtDeviceUs", "observationDeadlineDeviceUs"])
    check(same(a[key], b[key]), "v2_job_changed");
  immutableDefined(a, b, [...POOL_KEYS, "authorityDeadlineDeviceUs", "jobCommitment", "firstFailure", "outcome", "terminalAtDeviceUs"], "v2_retained_fact_changed");
  for (const key of ["events", "secondaryFailures"])
    check(b[key].length >= a[key].length && a[key].every((row, index) => same(row, b[key][index])), "v2_history_changed");
  if (a.outcome !== null || a.firstFailure !== null) {
    check(b.events.slice(a.events.length).every((row) =>
      ["socket_closed", "worker_quiescent", "revoked", "shutdown", "cooled", "accepted"].includes(row.kind)), "v2_activity_after_terminal");
    check(b.shareFacts.length === a.shareFacts.length, "v2_new_share_after_terminal");
  }
  check(b.shareFacts.length >= a.shareFacts.length, "v2_share_fact_lost");
  a.shareFacts.forEach((row, index) => immutableDefined(row, b.shareFacts[index], Object.keys(row), "v2_share_fact_changed"));
  for (const [flag, time] of [["socketClosed", "socketClosedAtUs"], ["workerQuiescent", "workerQuiescentAtUs"]]) {
    if (a.resources[flag]) check(b.resources[flag], "v2_release_regression");
    immutableDefined(a.resources, b.resources, [time], "v2_release_changed");
  }
  check(a.resources.fenceRetained || !b.resources.fenceRetained, "v2_fence_rearmed");
  for (const prior of a.timings) {
    const next = b.timings.find((row) => row.operation === prior.operation);
    check(next && next.count >= prior.count && next.failedCount >= prior.failedCount, "v2_timing_regression");
    for (const key of ["maxDurationUs", "totalDurationUs", "lastFinishedAtDeviceUs"])
      if (prior[key] !== null && next[key] !== null) check(next[key] >= prior[key], "v2_timing_regression");
    if (prior.firstStartedAtDeviceUs !== null) check(next.firstStartedAtDeviceUs === prior.firstStartedAtDeviceUs, "v2_timing_origin_changed");
  }
}

/** The only persistable device projection deliberately contains no network observation. */
export function collectedRecord(status) {
  const parsed = parseStatus(status);
  check(parsed.record !== null, "v2_record_missing");
  return parsed.record;
}
