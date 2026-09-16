import { ACT_TWO_BYTES, STAGES, OUTCOMES, boolean, canonical, cause, digest, object,
  port, privateIpv4, requireValue as check, uint } from "./contract.mjs";

function release(value) {
  object(value, ["socketState", "workerState", "volatileInputsDisposed", "startedAtUs", "deadlineAtUs",
    "releasedAtUs", "deadlineMet", "failure"]);
  check(["not_created", "open", "closed"].includes(value.socketState) &&
    ["not_started", "running", "quiescent"].includes(value.workerState), "noise_resource_state");
  boolean(value.volatileInputsDisposed);
  for (const key of ["startedAtUs", "deadlineAtUs", "releasedAtUs"]) if (value[key] !== null) uint(value[key]);
  if (value.deadlineMet !== null) boolean(value.deadlineMet);
  if (value.failure !== null) cause(value.failure, true);
  if (value.startedAtUs !== null && value.deadlineAtUs !== null) {
    check(value.deadlineAtUs === value.startedAtUs + 5_000_000, "noise_cleanup_deadline");
  }
  if (value.deadlineMet === true) check(value.releasedAtUs !== null && value.deadlineAtUs !== null &&
    value.startedAtUs !== null && value.releasedAtUs >= value.startedAtUs && value.releasedAtUs <= value.deadlineAtUs &&
    value.socketState !== "open" && value.workerState !== "running" && value.volatileInputsDisposed && value.failure === null, "noise_release_claim");
}
function stages(job) {
  check(Array.isArray(job.stages) && job.stages.length <= 8, "noise_stage_count");
  let priorIndex = -1, priorTime = job.admittedAtUs;
  for (const [index, row] of job.stages.entries()) {
    object(row, ["stage", "sequence", "atUs", "durationUs", "bytes"]);
    const order = STAGES.indexOf(row.stage);
    check(order > priorIndex && row.sequence === index + 1, "noise_stage_order");
    priorIndex = order;
    if (row.atUs !== null) { uint(row.atUs); check(row.atUs >= priorTime, "noise_time_order"); priorTime = row.atUs; }
    if (row.durationUs !== null) { uint(row.durationUs); if (row.atUs !== null) check(row.durationUs <= row.atUs - job.admittedAtUs, "noise_duration_origin"); }
    if (["act_one_written", "act_two_received", "proof_written"].includes(row.stage)) {
      if (row.bytes !== null) uint(row.bytes);
    } else check(row.bytes === null, "noise_stage_bytes");
  }
}
function accepted(job) {
  const r = job.resources;
  check(job.firstFailure === null && r.failure === null && job.stages.length === 8 &&
    r.socketState === "closed" && r.workerState === "quiescent" && r.deadlineMet === true &&
    job.localSocketPort !== null && job.terminal.decidedAtUs !== null && r.releasedAtUs !== null &&
    job.terminal.decidedAtUs >= r.releasedAtUs && job.terminal.decidedAtUs <= job.authorityDeadlineUs,
  "noise_accepted_resources");
  const counts = { act_one_written: 64, act_two_received: ACT_TWO_BYTES, proof_written: 22 };
  const bounds = { noise_prepared: 60_000_000, tcp_connected: 5_000_000, act_one_written: 2_000_000,
    act_two_received: 10_000_000, proof_written: 2_000_000, socket_closed: 5_000_000, worker_quiescent: 5_000_000 };
  for (const row of job.stages) {
    check(row.atUs !== null && row.durationUs !== null && row.atUs <= job.terminal.decidedAtUs, "noise_accepted_timing");
    if (row.stage === "worker_quiescent") check(row.atUs === r.releasedAtUs &&
      row.durationUs === row.atUs - r.startedAtUs, "noise_quiescence_join");
    if (Object.hasOwn(counts, row.stage)) check(row.bytes === counts[row.stage], "noise_accepted_bytes");
    if (Object.hasOwn(bounds, row.stage)) check(row.durationUs <= bounds[row.stage], "noise_operation_deadline");
  }
}

function consistency(j, observation) {
  const clockFailed = j.firstFailure?.category === "clock_invalid" || j.resources.failure?.category === "clock_invalid";
  let previousTime = j.admittedAtUs;
  for (const row of j.stages) {
    if (row.atUs === null || row.durationUs === null) check(clockFailed &&
      ["socket_closed", "worker_quiescent"].includes(row.stage), "noise_unavailable_clock");
    if (row.atUs !== null && row.durationUs !== null && row.stage !== "worker_quiescent")
      check(row.atUs - row.durationUs >= previousTime, "noise_operation_order");
    if (j.firstFailure?.atUs !== undefined && j.firstFailure.atUs !== null && row.atUs !== null &&
      !["socket_closed", "worker_quiescent"].includes(row.stage)) check(row.atUs <= j.firstFailure.atUs, "noise_protocol_after_failure");
    if (row.atUs !== null) previousTime = row.atUs;
  }
  if (!clockFailed) {
    const times = [j.admittedAtUs, ...j.stages.map((row) => row.atUs), j.terminal?.decidedAtUs,
      j.firstFailure?.atUs, j.resources.failure?.atUs, j.resources.releasedAtUs];
    check(times.every((time) => time === null || time === undefined || time <= observation.observedAtUs), "noise_observation_order");
    check(j.firstFailure?.atUs !== null && j.resources.failure?.atUs !== null && j.terminal?.decidedAtUs !== null, "noise_unavailable_clock");
  }
  if (j.terminal === null) return;
  if (j.terminal.decidedAtUs !== null && !clockFailed) {
    check(j.terminal.outcome === "incomplete" || j.terminal.decidedAtUs >= previousTime, "noise_terminal_order");
    if (j.firstFailure !== null) check(j.firstFailure.atUs <= j.terminal.decidedAtUs, "noise_first_failure_time");
  }
  if (j.terminal.outcome === "accepted") return;
  check(j.firstFailure !== null, "noise_first_failure_missing");
  if (j.terminal.outcome === "incomplete") {
    check(j.resources.failure !== null && j.resources.deadlineMet === false, "noise_incomplete_cleanup");
    return;
  }
  const first = j.firstFailure, outcome = j.terminal.outcome;
  if (["cancel_requested", "session_replaced"].includes(first.detail)) check(outcome === "cancelled", "noise_terminal_cause");
  else if (first.detail === "heartbeat_expired" || (first.category === "authority_lost" && first.detail === "timeout"))
    check(outcome === "expired", "noise_terminal_cause");
  else if (first.detail === "timeout" && ["preparation", "connect", "read", "write"].includes(first.category))
    check(["rejected", "expired"].includes(outcome), "noise_terminal_cause");
  else check(outcome === "rejected", "noise_terminal_cause");
  check(j.resources.socketState !== "open" && j.resources.workerState !== "running" &&
    j.resources.volatileInputsDisposed && j.resources.deadlineMet === true, "noise_terminal_cleanup");
}

/** Parse only a closed device observation; this grants no authority or hardware verdict. */
export function parseNoiseStatus(value) {
  object(value, ["schema", "state", "observation", "job"]);
  check(value.schema === "worker-noise-diagnostic-status-v1" &&
    ["idle", "admitted", "running", "cancelling", "terminal"].includes(value.state), "noise_status_schema");
  const o = value.observation;
  object(o, ["bootOrdinal", "workerGeneration", "transportEpoch", "observedAtUs", "stationIpv4", "wifiConnected"]);
  for (const key of ["bootOrdinal", "workerGeneration", "transportEpoch", "observedAtUs"]) uint(o[key]);
  check(o.bootOrdinal > 0, "noise_boot"); boolean(o.wifiConnected);
  if (o.stationIpv4 !== null) privateIpv4(o.stationIpv4);
  check(o.wifiConnected || o.stationIpv4 === null, "noise_station_state");
  if (value.state === "idle") { check(value.job === null, "noise_idle_job"); return structuredClone(value); }
  const j = value.job;
  object(j, ["attemptId", "inputSha256", "bootOrdinal", "workerGeneration", "transportEpoch", "admittedAtUs",
    "authorityDeadlineUs", "localSocketPort", "stages", "firstFailure", "terminal", "resources"]);
  canonical(j.attemptId, 16); digest(j.inputSha256);
  for (const key of ["bootOrdinal", "workerGeneration", "transportEpoch", "admittedAtUs", "authorityDeadlineUs"]) uint(j[key]);
  check(j.bootOrdinal > 0 && j.authorityDeadlineUs === j.admittedAtUs + 120_000_000, "noise_authority_deadline");
  if (j.localSocketPort !== null) port(j.localSocketPort);
  if (j.firstFailure !== null) cause(j.firstFailure, true);
  release(j.resources); stages(j);
  check(j.bootOrdinal === o.bootOrdinal, "noise_boot_binding");
  if (["admitted", "running"].includes(value.state)) check(j.workerGeneration === o.workerGeneration &&
    j.transportEpoch === o.transportEpoch, "noise_live_binding");
  if (j.stages.some((row) => row.stage === "tcp_connected")) check(j.localSocketPort !== null, "noise_socket_tuple");
  if (j.terminal === null) check(value.state !== "terminal", "noise_terminal_missing");
  else {
    object(j.terminal, ["outcome", "decidedAtUs"]);
    check(value.state === "terminal" && OUTCOMES.includes(j.terminal.outcome), "noise_terminal_state");
    if (j.terminal.decidedAtUs !== null) uint(j.terminal.decidedAtUs);
    if (j.terminal.outcome === "accepted") accepted(j);
    else check(j.firstFailure !== null || j.resources.failure !== null, "noise_failure_missing");
  }
  consistency(j, o);

  return structuredClone(value);
}

/** Check append-only retention across reads; session replacement cannot rewrite the job. */
export function validateNoiseProgress(previous, current) {
  const before = parseNoiseStatus(previous), after = parseNoiseStatus(current);
  if (before.job === null) return;
  check(after.job !== null, "noise_job_lost");
  for (const key of ["attemptId", "inputSha256", "bootOrdinal", "workerGeneration", "transportEpoch", "admittedAtUs", "authorityDeadlineUs"]) {
    check(before.job[key] === after.job[key], "noise_job_changed");
  }
  check(after.job.stages.length >= before.job.stages.length && before.job.stages.every((row, index) =>
    JSON.stringify(row) === JSON.stringify(after.job.stages[index])), "noise_stage_changed");
  if (before.job.firstFailure !== null) check(JSON.stringify(before.job.firstFailure) === JSON.stringify(after.job.firstFailure), "noise_first_failure_changed");
  if (before.job.terminal !== null) check(JSON.stringify(before.job.terminal) === JSON.stringify(after.job.terminal), "noise_terminal_changed");
  if (before.job.firstFailure !== null || before.job.terminal !== null) check(after.job.stages.slice(before.job.stages.length)
    .every((row) => ["socket_closed", "worker_quiescent"].includes(row.stage)), "noise_protocol_after_failure");
  const br = before.job.resources, ar = after.job.resources;
  for (const key of ["startedAtUs", "deadlineAtUs", "releasedAtUs", "deadlineMet", "failure"]) {
    if (br[key] !== null) check(JSON.stringify(br[key]) === JSON.stringify(ar[key]), "noise_release_changed");
  }
  if (before.job.localSocketPort !== null) check(before.job.localSocketPort === after.job.localSocketPort, "noise_socket_changed");
  check(!br.volatileInputsDisposed || ar.volatileInputsDisposed, "noise_resources_regressed");
  for (const [key, states] of [["socketState", ["not_created", "open", "closed"]], ["workerState", ["not_started", "running", "quiescent"]]]) {
    check(states.indexOf(ar[key]) >= states.indexOf(br[key]), "noise_resources_regressed");
    if (before.job.terminal !== null && br[key] === states[0]) check(ar[key] === states[0], "noise_owner_after_terminal");
  }
}
