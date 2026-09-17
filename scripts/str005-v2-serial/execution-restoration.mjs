import { isDeepStrictEqual as equal } from "node:util";
import { proof } from "../str005-noise-serial/files.mjs";
import { restoredBaseline } from "./journal.mjs";
import { binding } from "./execution-receipts.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** Read-only restoration proof, including failed runs without claiming protocol acceptance. */
export async function inspectRestoration(root, context, states, devices) {
  const value = (await proof(root, "restoration.json")).value;
  object(value, ["schema", "contextSha256", "closedSequence", "observedSequence", "deviceSequence", "state", "recordSha256"]);
  for (const key of ["closedSequence", "observedSequence", "deviceSequence"]) uint(value[key]);
  const row = states[value.observedSequence - 1], closed = states[value.closedSequence - 1], device = devices[value.deviceSequence - 1];
  const before = (await proof(root, "accounting-before.json")).value;
  check(value.schema === "str005-v2-restoration-v1" && value.contextSha256 === sha256(JSON.stringify(context)) && row && closed && device &&
    row.phase === "candidate" && closed.phase === "candidate" && closed.sequence > before.observedSequence && closed.sequence < row.sequence &&
    device.atHostMs >= closed.atHostMs && device.atHostMs <= row.atHostMs && equal(value.state, row.state) &&
    value.recordSha256 === sha256(JSON.stringify(device.record)), "v2_restoration_join");
  check(closed.state.status === "closed" && !closed.state.connected && closed.state.serialOwnershipReleased &&
    row.state.preservation?.baseline_id === before.state.preservation?.baseline_id, "v2_restoration_fresh_session");
  const record = device.record;
  check(record.state === "terminal" && record.resources.socketClosed && record.resources.workerQuiescent && !record.resources.fenceRetained,
    "v2_restoration_resource");
  restoredBaseline(row.state, context);
  return value;
}

/** Same source-bound atoms are used by the live producer and independent reader. */
export function faultFacts(record, state, claim, confirmed, stop) {
  const q = state.qualification;
  check(q?.generation === record.workerGeneration && q.revocation_reason === "heartbeat_timeout" &&
    q.last_valid_heartbeat_ms !== null && q.gate_closed_ms !== null && q.shutdown_started_ms !== null, "v2_fault_native_stop");
  const facts = { ...binding(record), selectedDeviceAckSha256: claim.selectedDeviceAckSha256,
    suppressionRequestedAtHostMs: claim.requestedAtHostMs,
    ...Object.fromEntries(["headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs"].map(key => [key, confirmed.headroom[key]])),
    revocationReason: "heartbeat_timeout", lastValidHeartbeatAtDeviceUs: q.last_valid_heartbeat_ms * 1000,
    gateClosedAtDeviceUs: q.gate_closed_ms * 1000, shutdownStartedAtDeviceUs: q.shutdown_started_ms * 1000, observerTailMs: stop.tailMs };
  for (const key of ["lastValidHeartbeatAtDeviceUs", "gateClosedAtDeviceUs", "shutdownStartedAtDeviceUs", "observerTailMs"]) uint(facts[key]);
  check(facts.gateClosedAtDeviceUs >= facts.lastValidHeartbeatAtDeviceUs && facts.gateClosedAtDeviceUs - facts.lastValidHeartbeatAtDeviceUs <= 3000000 &&
    facts.shutdownStartedAtDeviceUs >= facts.gateClosedAtDeviceUs && facts.shutdownStartedAtDeviceUs - facts.lastValidHeartbeatAtDeviceUs <= 3000000 &&
    facts.observerTailMs >= 5000, "v2_fault_safety_join");
  return facts;
}
