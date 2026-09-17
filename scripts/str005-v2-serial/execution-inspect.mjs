import { readdir } from "node:fs/promises";
import { isDeepStrictEqual as equal } from "node:util";
import { proof } from "../str005-noise-serial/files.mjs";
import { RECEIPT_FILES, validateEnvelope, selectedFacts } from "./execution-receipts.mjs";
import { completedShares, inspectLiveSelection } from "./execution-selection.mjs";
import { faultFacts, inspectRestoration } from "./execution-restoration.mjs";
import { inspectShareStart } from "./start-observation.mjs";
import { baseline } from "./journal.mjs";
import { bytes, check, object, sha256, uint } from "./values.mjs";
export { inspectRestoration } from "./execution-restoration.mjs";

function source(rows, sequence, label) {
  uint(sequence); const row = rows[sequence - 1];
  check(row && row.sequence === sequence, `v2_execution_${label}_source`); return row;
}
function bound(value, schema, context, keys) {
  object(value, ["schema", "contextSha256", ...keys]);
  check(value.schema === schema && value.contextSha256 === sha256(JSON.stringify(context)), "v2_execution_source_binding");
}
async function receipts(root, context) {
  const names = Object.keys(RECEIPT_FILES).slice(0, context.scope === "channel" ? 2 : undefined);
  const all = await readdir(root);
  check(all.filter(name => Object.hasOwn(RECEIPT_FILES, name)).length === names.length, "v2_execution_receipt_membership");
  const result = {};
  for (const [index, name] of names.entries()) {
    const row = await proof(root, name); validateEnvelope(row.value, context, RECEIPT_FILES[name]);
    check(row.value.sequence === index + 1, "v2_execution_receipt_order"); result[name] = row;
  }
  return result;
}
function receiptMatches(receipt, clock, at, facts) {
  check(receipt.value.clock === clock && receipt.value.at === at && equal(receipt.value.facts, facts), "v2_execution_receipt_facts");
}
async function channelClaim(root, context, states, devices, ready) {
  const value = (await proof(root, "start.claim.json")).value;
  bound(value, "str005-v2-start-claim-v1", context, ["atHostMs", "observedStateSequence", "bootOrdinal", "workerGeneration", "serialTransportEpoch",
    "networkObservedAtDeviceUs", "fixtureReadySha256", "clientSha256", "attemptId"]);
  const state = source(states, value.observedStateSequence, "start"), first = devices[0];
  for (const key of ["atHostMs", "networkObservedAtDeviceUs", "bootOrdinal", "workerGeneration", "serialTransportEpoch"]) uint(value[key]);
  baseline(state.state);
  check(state.phase === "candidate" && value.attemptId === context.attemptId && value.clientSha256 === context.client_sha256 &&
    value.fixtureReadySha256 === ready.sha256 && value.atHostMs >= ready.value.readyAtMs && value.atHostMs - ready.value.readyAtMs <= 10000 &&
    value.atHostMs >= state.atHostMs && value.atHostMs <= first.atHostMs &&
    ["bootOrdinal", "workerGeneration", "serialTransportEpoch"].every(key => value[key] === first.record[key]) &&
    value.networkObservedAtDeviceUs <= first.record.admittedAtDeviceUs && first.record.admittedAtDeviceUs - value.networkObservedAtDeviceUs <= 5000000,
    "v2_channel_claim_join");
  return value;
}
async function selection(root, context, states, devices, stored, restoration) {
  const claimProof = await proof(root, "fault.claim.json"), claim = claimProof.value;
  bound(claim, "str005-v2-fault-claim-v1", context, ["nonce", "requestedAtHostMs", "selectedDeviceAckSha256", "selectionDeviceSequence", "stateSequence"]);
  bytes(claim.nonce, 16); uint(claim.requestedAtHostMs);
  const selectedDevice = source(devices, claim.selectionDeviceSequence, "selection"), claimedState = source(states, claim.stateSequence, "fault");
  check(selectedDevice.record.state !== "terminal" && selectedDevice.atHostMs <= claim.requestedAtHostMs && claimedState.atHostMs <= claim.requestedAtHostMs &&
    claimedState.state.running && !claimedState.state.heartbeatSuppressed &&
    claimedState.state.qualification?.generation === selectedDevice.record.workerGeneration, "v2_fault_claim_source");
  const selected = await inspectLiveSelection(root, context, selectedDevice.record);
  check(selected, "v2_selected_share_missing");
  const fact = selected.fact, nonce = stored["selected-nonce.json"], submission = stored["selected-submission.json"], ack = stored["selected-device-ack.json"];
  const facts = selectedFacts(selected.record, selected.job, fact, selected.share, nonce.sha256, submission.sha256);
  receiptMatches(stored["selected-dispatch.json"], "device-us", fact.dispatchedAtDeviceUs, facts.dispatch);
  receiptMatches(nonce, "device-us", fact.nonceAtDeviceUs, facts.nonce);
  receiptMatches(submission, "device-us", fact.writeCompletedAtDeviceUs, facts.submission);
  receiptMatches(ack, "device-us", fact.ackAtDeviceUs, facts.deviceAck);
  receiptMatches(stored["selected-fixture-ack.json"], "fixture-us", selected.share.writeCompletedAtFixtureUs, facts.fixtureAck);
  check(claim.selectedDeviceAckSha256 === ack.sha256, "v2_selected_ack_hash");
  const confirmed = (await proof(root, "fault-confirmed.json")).value;
  bound(confirmed, "str005-v2-fault-confirmed-v1", context, ["claimSha256", "confirmedAtHostMs", "headroom", "observedStateSequence"]);
  uint(confirmed.confirmedAtHostMs); const observed = source(states, confirmed.observedStateSequence, "confirmation"), h = confirmed.headroom;
  object(h, ["schema", "workerGeneration", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs"]);
  for (const key of ["workerGeneration", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs"]) uint(h[key]);
  check(confirmed.claimSha256 === claimProof.sha256 && confirmed.confirmedAtHostMs >= claim.requestedAtHostMs &&
    confirmed.confirmedAtHostMs - claim.requestedAtHostMs <= 45000 && observed.sequence > claimedState.sequence &&
    observed.atHostMs >= claim.requestedAtHostMs && observed.atHostMs <= confirmed.confirmedAtHostMs &&
    h.schema === "worker-v2-fault-headroom-v1" && h.workerGeneration === selected.record.workerGeneration &&
    h.headroomObservedAtDeviceUs >= Math.floor(fact.ackAtDeviceUs / 1000) * 1000 && h.leaseRemainingMs >= 5000 && h.leaseRemainingMs <= 60000 &&
    h.workGateRemainingMs >= 5000 && h.workGateRemainingMs <= 164450, "v2_fault_confirmation_join");
  const q = observed.state.qualification;
  check(observed.state.running && observed.state.heartbeatSuppressed && !observed.state.failure && q?.generation === h.workerGeneration &&
    q.revocation_reason === "none" && !q.safe_stop_complete && q.work_gate_remaining_ms === h.workGateRemainingMs, "v2_fault_confirmation_state");
  const restoredState = source(states, restoration.observedSequence, "restoration"), restoredDevice = source(devices, restoration.deviceSequence, "restoration");
  check(restoredState.atHostMs >= confirmed.confirmedAtHostMs + 145000, "v2_restoration_wait");
  const stop = (await proof(root, "observer-stop.json")).value;
  check(stop.contextSha256 === sha256(JSON.stringify(context)) && stop.tailMs === stop.requestedAtHostMs - confirmed.confirmedAtHostMs, "v2_fault_observer_join");
  const fault = faultFacts(restoredDevice.record, restoredState.state, claim, confirmed, stop);
  receiptMatches(stored["fault.json"], "device-us", fault.shutdownStartedAtDeviceUs, fault);
  return { selection: { submissionSequence: fact.submissionSequence, deviceSequence: selectedDevice.sequence, selectedDeviceAckSha256: ack.sha256 },
    fault, faultClaim: claim, faultConfirmed: confirmed };
}

/** Independently join immutable files. Private socket tuples were intentionally never persisted. */
export async function inspectExecution(root, context, states, devices) {
  check(states.length > 0 && devices.length >= 2, "v2_execution_journals_missing");
  const stored = await receipts(root, context), ready = await proof(root, "fixture-ready.json");
  const restoration = await inspectRestoration(root, context, states, devices);
  const comparison = stored["connection.json"].value.facts, observed = source(devices, comparison.deviceObservationSequence, "connection");
  receiptMatches(stored["connection.json"], "supervisor-ms", comparison.comparisonCompletedAtMs, comparison);
  check(comparison.readinessSha256 === ready.sha256 && ready.value.contextSha256 === sha256(JSON.stringify(context)) &&
    ready.value.scope === context.scope && ready.value.attemptId === context.attemptId && ready.value.instanceId === comparison.instanceId &&
    comparison.comparisonStartedAtMs >= observed.atHostMs && comparison.comparisonStartedAtMs >= ready.value.readyAtMs &&
    comparison.comparisonCompletedAtMs >= comparison.comparisonStartedAtMs &&
    ["attemptId", "bootOrdinal", "workerGeneration", "serialTransportEpoch", "poolSessionGeneration", "poolTransportEpoch"].every(key => comparison[key] === observed.record[key]),
    "v2_connection_source_join");
  const job = await proof(root, "fixture-run/job.json"), terminal = await proof(root, "fixture-run/fixture-terminal.json"), shares = await proof(root, "fixture-run/shares.json");
  const events = await proof(root, "fixture-run/fixture-events.json"), counts = await proof(root, "fixture-run/connection-facts.json");
  const workReady = observed.record.events.find(event => event.kind === "work_ready") ?? devices.at(-1).record.events.find(event => event.kind === "work_ready");
  check(workReady?.atDeviceUs !== null && workReady?.atDeviceUs !== undefined, "v2_job_source_missing");
  receiptMatches(stored["job-receipt.json"], "device-us", workReady.atDeviceUs, job.value);
  const incremental = await completedShares(root);
  check(Array.isArray(shares.value.shares) && incremental.length === shares.value.shares.length &&
    incremental.every((row, index) => row.value.connectionId === shares.value.connectionId && equal(row.share, shares.value.shares[index])), "v2_incremental_share_join");
  const completion = (await proof(root, "protocol-complete.json")).value;
  bound(completion, "str005-v2-protocol-complete-v1", context, ["atHostMs", "observedStateSequence", "observedDeviceSequence", "fixtureTerminalSha256", "fixtureSharesSha256"]);
  uint(completion.atHostMs);
  const completedState = source(states, completion.observedStateSequence, "complete"), completedDevice = source(devices, completion.observedDeviceSequence, "complete");
  const closed = source(states, restoration.closedSequence, "closed"), exit = (await proof(root, "fixture-exit.json")).value, reap = (await proof(root, "fixture-reap.json")).value;
  check(completion.fixtureTerminalSha256 === terminal.sha256 && completion.fixtureSharesSha256 === shares.sha256 &&
    completion.atHostMs >= comparison.comparisonCompletedAtMs && completion.atHostMs >= completedState.atHostMs && completion.atHostMs >= completedDevice.atHostMs &&
    completion.atHostMs >= exit.atHostMs && completion.atHostMs >= reap.completedAtHostMs && completion.atHostMs <= closed.atHostMs &&
    completedDevice.sequence <= restoration.deviceSequence, "v2_protocol_completion_join");
  if (context.scope === "channel") {
    const claim = await channelClaim(root, context, states, devices, ready);
    check(completedDevice.atHostMs >= claim.atHostMs && completedDevice.atHostMs - claim.atHostMs <= 125000, "v2_channel_observation_horizon");
    check(completedDevice.record.state === "terminal" && completedDevice.record.outcome === "accepted" && completedDevice.record.resources.socketClosed &&
      completedDevice.record.resources.workerQuiescent && !completedDevice.record.resources.fenceRetained, "v2_channel_completion");
  }
  const startObserved = context.scope === "share" ? await inspectShareStart(root, context, states, devices) : null;
  const selected = context.scope === "share" ? await selection(root, context, states, devices, stored, restoration) : {};
  const projectedContext = { scope: context.scope, attemptId: context.attemptId, ...(context.scope === "share" ? { qualificationAttempt: context.qualificationAttempt } : {}) };
  const protocolInput = { scope: context.scope, context: projectedContext, deviceRecords: devices.map(row => row.record), job: job.value,
    fixtureEvents: events.value, fixtureTerminal: terminal.value, fixtureConnectionFacts: counts.value,
    connectionComparison: comparison, fixtureShares: shares.value, ...(context.scope === "share" ? { fault: selected.fault } : {}) };
  return { protocolInput, restoration, completion, startObserved, ...selected };
}
