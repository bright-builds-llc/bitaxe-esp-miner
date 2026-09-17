import { proof } from "../str005-noise-serial/files.mjs";
import { readDeviceJournal } from "./journal.mjs";
import { failure } from "./device-record.mjs";
import { boolean, bytes, check, object, sha256, uint } from "./values.mjs";

export const judgmentCode = error => /^(?:v2|noise|iterative|original|observer)_[a-z_]+$/u.test(error?.code) ? error.code : "v2_evidence_incomplete";
async function maybeProof(root, path) {
  try { return await proof(root, path); }
  catch (error) { if (error.code === "ENOENT") return null; throw error; }
}
async function maybeFixtureCause(root) {
  const maybeTerminal = (await maybeProof(root, "fixture-run/fixture-terminal.json"))?.value;
  if (!maybeTerminal?.firstFailure) return null;
  object(maybeTerminal, ["instanceId", "connectionId", "outcome", "firstFailure", "elapsedMs", "receivedShares", "acceptedShares", "rejectedShares", "duplicateShares", "peerClosed", "socketClosed", "listenerClosed"]);
  bytes(maybeTerminal.instanceId, 16); if (maybeTerminal.connectionId !== null) bytes(maybeTerminal.connectionId, 16);
  check(maybeTerminal.outcome === "unverified", "v2_fixture_failure_outcome");
  for (const key of ["elapsedMs", "receivedShares", "acceptedShares", "rejectedShares", "duplicateShares"]) uint(maybeTerminal[key]);
  for (const key of ["peerClosed", "socketClosed", "listenerClosed"]) boolean(maybeTerminal[key]);
  const cause = maybeTerminal.firstFailure; object(cause, ["stage", "category", "atFixtureUs"]);
  check(["setup_received", "channel_open_received", "channel_success_sent", "job_sent", "target_sent", "prev_hash_sent", "share_received", "share_success_sent", "peer_eof", "listener_closed"].includes(cause.stage) &&
    ["admission", "clock", "allocation", "authority", "timeout", "eof", "extra", "authentication", "protocol", "channel_mismatch", "job_mismatch", "invalid_nonce", "rejected_share", "safety", "cleanup", "evidence"].includes(cause.category), "v2_fixture_failure_shape");
  if (cause.atFixtureUs !== null) uint(cause.atFixtureUs);
  const maybeReady = (await maybeProof(root, "fixture-ready.json"))?.value;
  if (maybeReady) check(maybeReady.instanceId === maybeTerminal.instanceId, "v2_fixture_failure_instance");
  return cause;
}
function observed(source, code, cause, sourceSequence, observedAtHostMs, availableCauses, ordering) {
  return { source, code, cause, sourceSequence, observedAtHostMs, availableCauses, ordering };
}
/** Recorded first observation and available typed causes are distinct; unrelated clocks are never ranked. */
async function deriveFirstFailure(root, context, code) {
  const maybeSaved = (await maybeProof(root, "failure.json"))?.value;
  if (maybeSaved) {
    object(maybeSaved, ["schema", "contextSha256", "code", "atHostMs", "deviceCause", "sourceSequence"]);
    check(maybeSaved.schema === "str005-v2-first-failure-v1" && maybeSaved.contextSha256 === sha256(JSON.stringify(context)) &&
      judgmentCode(maybeSaved) === maybeSaved.code, "v2_failure_binding"); uint(maybeSaved.atHostMs);
    if (maybeSaved.sourceSequence !== null) uint(maybeSaved.sourceSequence);
  }
  const devices = await readDeviceJournal(root, context);
  const maybeDevice = devices.find(row => row.record.firstFailure !== null), maybeFixture = await maybeFixtureCause(root);
  const available = { device: maybeDevice ? { sequence: maybeDevice.sequence, observedAtHostMs: maybeDevice.atHostMs, cause: maybeDevice.record.firstFailure } : null,
    fixture: maybeFixture };
  if (maybeSaved?.deviceCause) {
    failure(maybeSaved.deviceCause); uint(maybeSaved.sourceSequence);
    check(maybeDevice?.sequence === maybeSaved.sourceSequence && JSON.stringify(maybeDevice.record.firstFailure) === JSON.stringify(maybeSaved.deviceCause), "v2_failure_device_join");
    return observed("device", maybeSaved.code, maybeSaved.deviceCause, maybeSaved.sourceSequence, maybeSaved.atHostMs, available, "recorded-first-observation");
  }
  if (maybeSaved) return observed("supervisor", maybeSaved.code, null, maybeSaved.sourceSequence, maybeSaved.atHostMs, available, "recorded-first-observation");
  if (maybeDevice && maybeFixture) return observed("unresolved", "v2_producer_failures", null, null, null, available, "unproved");
  if (maybeDevice) return observed("device", "v2_device_failure", maybeDevice.record.firstFailure, maybeDevice.sequence, maybeDevice.atHostMs, available, "single-observed-producer");
  if (maybeFixture) return observed("fixture", "v2_fixture_failure", maybeFixture, null, null, available, "single-observed-producer");
  return observed("judge", code, null, null, null, available, "no-producer-cause");
}
export async function firstFailure(root, context, code) {
  try { return await deriveFirstFailure(root, context, code); }
  catch (error) {
    // Malformed or unjoinable evidence remains byte-bound in the seal, without a trusted typed interpretation.
    return observed("judge", judgmentCode(error), null, null, null, { device: null, fixture: null }, "unproved");
  }
}
export function failureOutcome(first, code) {
  const causes = [first.cause, first.availableCauses.device?.cause, first.availableCauses.fixture].filter(Boolean);
  if (causes.some(cause => cause.category === "authority") || /authority|task_inactive|source_changed|source_drift|permission/u.test(code)) return "stop_authority_boundary";
  if (causes.some(cause => cause.category !== "evidence")) return "stop_hardware_blocker";
  return "stop_evidence_incomplete";
}
