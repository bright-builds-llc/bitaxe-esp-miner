// Synthetic vectors mirror Gate worker-noise-diagnostic.fixture.ts; no device claims.
import { STAGES } from "./contract.mjs";
import { startDigest } from "./evidence.mjs";
export function example() {
  const start = { schema: "worker-noise-diagnostic-start-v1", attemptId: "A".repeat(22),
    expectedBootOrdinal: 1, networkObservedAtUs: 100, fixtureIpv4: "192.168.1.20",
    fixturePort: 1234, authorityPublicKey: "A".repeat(43) };
  const device = { schema: "worker-noise-diagnostic-status-v1", state: "terminal",
    observation: { bootOrdinal: 1, workerGeneration: 7, transportEpoch: 2, observedAtUs: 11000,
      stationIpv4: "192.168.1.10", wifiConnected: true },
    job: { attemptId: start.attemptId, inputSha256: startDigest(start), bootOrdinal: 1, workerGeneration: 7,
      transportEpoch: 2, admittedAtUs: 1000, authorityDeadlineUs: 120001000, localSocketPort: 54321,
      stages: STAGES.map((stage, index) => ({ stage, sequence: index + 1, atUs: 2000 + index * 1000,
        durationUs: stage === "worker_quiescent" ? 2000 : 500,
        bytes: stage === "act_one_written" ? 64 : stage === "act_two_received" ? 234 : stage === "proof_written" ? 22 : null })),
      firstFailure: null, terminal: { outcome: "accepted", decidedAtUs: 10000 },
      resources: { socketState: "closed", workerState: "quiescent", volatileInputsDisposed: true, startedAtUs: 7000,
        deadlineAtUs: 5007000, releasedAtUs: 9000, deadlineMet: true, failure: null } } };
  const fixtureReady = { schema: "noise-serial-fixture-ready-v1", attemptId: start.attemptId,
    listenIpv4: start.fixtureIpv4, listenPort: start.fixturePort, authorityPublicKey: start.authorityPublicKey };
  const fixtureTerminal = { schema: "noise-serial-fixture-terminal-v1", attemptId: start.attemptId, outcome: "accepted",
    failure: null, elapsedMs: 1000, expectedPeerConnectionCount: 1, unexpectedPeerCount: 0, candidateOverflow: false,
    selectedIndex: 0, candidates: [{ remotePort: 54321, actOneBytes: 64, readOutcome: "complete" }],
    actTwoBytesWritten: 234, proofBytesReceived: 22, extraBytesReceived: 0, encryptedProofExact: true, peerClosed: true, socketClosed: true };
  return { start, device, fixtureReady, fixtureTerminal };
}
export function admitted() {
  const value = example().device;
  value.state = "admitted"; value.observation.observedAtUs = 1000;
  Object.assign(value.job, { localSocketPort: null, stages: [], terminal: null });
  value.job.resources = { socketState: "not_created", workerState: "not_started", volatileInputsDisposed: false,
    startedAtUs: null, deadlineAtUs: null, releasedAtUs: null, deadlineMet: null, failure: null };
  return value;
}
export function cleanup() {
  return { schema: "noise-serial-cleanup-v1", contextSha256: "a".repeat(64), browserClosed: true,
    browserOwnershipReleased: true, fixtureExited: true, fixtureExitCode: 0, supervisorExited: true,
    supervisorExitCode: 0, remainingOwnedProcesses: 0, listenerAbsent: true, serialHoldersAbsent: true,
    deviceResourcesReleased: true, observedAtHostUnixMs: 1000,
    witnesses: Object.fromEntries(["browser", "fixture", "supervisor", "resources"].map((key) => [key, "b".repeat(64)])) };
}
export function projection() {
  return { schema_version: "bitaxe-stratum-v2-noise-serial-projection-v1", status: "accepted", board: 205,
    diagnostic_ordinal: 1, source_commit: "a".repeat(40), gate_commit: "b".repeat(40), reference_commit: "c".repeat(40),
    provenance: Object.fromEntries(["app_elf", "package_manifest", "contract", "fixture", "evaluator", "sealed_inventory", "private_result"].map((key) => [key, "d".repeat(64)])),
    criteria: Object.fromEntries(["identity", "continuity", "authority", "tcp_delivery", "noise_authentication", "encrypted_proof", "no_new_work",
      "preservation", "accounting", "restoration", "cleanup", "privacy"].map((key) => [key, true])),
    timings_ms: Object.fromEntries(["preparation", "connect", "act_one_write", "act_two_read", "proof_write", "diagnostic", "device_cleanup", "fixture_lifetime", "host_cleanup"].map((key) => [key, 1])),
    counts: { exact_peer_connections: 1, unexpected_peer_connections: 0, act_one_written: 64,
      act_one_received: 64, proof_written: 22, proof_received: 22, new_work: 0, new_shares: 0 }, redaction_status: "passed" };
}
