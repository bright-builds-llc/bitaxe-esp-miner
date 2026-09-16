import { ACT_TWO_BYTES, OUTCOMES, boolean, canonical, cause, object, port, privateIpv4,
  requireValue as check, uint } from "./contract.mjs";

/** Closed fixture metadata, never a substitute for process/source evidence. */
export function parseFixtureReady(value) {
  object(value, ["schema", "attemptId", "listenIpv4", "listenPort", "authorityPublicKey"]);
  check(value.schema === "noise-serial-fixture-ready-v1", "noise_fixture_ready_schema");
  canonical(value.attemptId, 16); canonical(value.authorityPublicKey, 32);
  privateIpv4(value.listenIpv4); port(value.listenPort);
  return structuredClone(value);
}
export function parseFixtureTerminal(value) {
  object(value, ["schema", "attemptId", "outcome", "failure", "elapsedMs", "expectedPeerConnectionCount",
    "unexpectedPeerCount", "candidateOverflow", "selectedIndex", "candidates", "actTwoBytesWritten",
    "proofBytesReceived", "extraBytesReceived", "encryptedProofExact", "peerClosed", "socketClosed"]);
  check(value.schema === "noise-serial-fixture-terminal-v1" && OUTCOMES.includes(value.outcome), "noise_fixture_terminal_schema");
  canonical(value.attemptId, 16);
  for (const key of ["elapsedMs", "expectedPeerConnectionCount", "unexpectedPeerCount", "actTwoBytesWritten", "proofBytesReceived", "extraBytesReceived"]) uint(value[key]);
  for (const key of ["candidateOverflow", "encryptedProofExact", "peerClosed", "socketClosed"]) boolean(value[key]);
  check(Array.isArray(value.candidates) && value.candidates.length <= 3, "noise_candidate_bound");
  for (const candidate of value.candidates) {
    object(candidate, ["remotePort", "actOneBytes", "readOutcome"]); port(candidate.remotePort); uint(candidate.actOneBytes);
    check(["complete", "partial", "eof", "timeout", "io", "malformed"].includes(candidate.readOutcome), "noise_candidate_outcome");
  }
  if (value.selectedIndex !== null) { uint(value.selectedIndex); check(value.selectedIndex < value.candidates.length, "noise_candidate_index"); }
  if (value.failure !== null) cause(value.failure);
  if (value.outcome === "accepted") {
    check(value.failure === null && value.elapsedMs <= 150000 && value.expectedPeerConnectionCount === 1 &&
      value.unexpectedPeerCount === 0 && !value.candidateOverflow && value.candidates.length === 1 && value.selectedIndex === 0 &&
      value.candidates[0].actOneBytes === 64 && value.candidates[0].readOutcome === "complete" &&
      value.actTwoBytesWritten === ACT_TWO_BYTES && value.proofBytesReceived === 22 && value.extraBytesReceived === 0 &&
      value.encryptedProofExact && value.peerClosed && value.socketClosed, "noise_fixture_positive_shape");
  } else check(value.failure !== null, "noise_fixture_failure_missing");
  return structuredClone(value);
}
