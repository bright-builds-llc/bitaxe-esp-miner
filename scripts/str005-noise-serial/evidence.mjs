import { createHash } from "node:crypto";
import { parseNoiseStatus } from "./device.mjs";
import { parseFixtureReady, parseFixtureTerminal } from "./fixture.mjs";
import { boolean, digest, object, parseStart, requireValue as check, uint } from "./contract.mjs";

/** Canonical Start JSON matches the existing Controller sorted-key encoder. */
export function startDigest(value) {
  const parsed = parseStart(value);
  const ordered = Object.fromEntries(Object.keys(parsed).sort().map((key) => [key, parsed[key]]));
  return createHash("sha256").update(JSON.stringify(ordered)).digest("hex");
}

/** Pure cross-field consistency ONLY. Does not admit artifacts, owners or hardware. */
export function inspectExchangeShape(input) {
  object(input, ["start", "device", "fixtureReady", "fixtureTerminal"]);
  const start = parseStart(input.start), status = parseNoiseStatus(input.device);
  const ready = parseFixtureReady(input.fixtureReady), terminal = parseFixtureTerminal(input.fixtureTerminal);
  check(status.state === "terminal" && status.job.terminal.outcome === "accepted" && terminal.outcome === "accepted", "noise_protocol_incomplete");
  const job = status.job;
  check(start.attemptId === job.attemptId && job.attemptId === ready.attemptId && ready.attemptId === terminal.attemptId &&
    startDigest(start) === job.inputSha256, "noise_attempt_join");
  check(start.expectedBootOrdinal === job.bootOrdinal && status.observation.bootOrdinal === job.bootOrdinal &&
    start.networkObservedAtUs <= job.admittedAtUs && job.admittedAtUs - start.networkObservedAtUs <= 5_000_000,
  "noise_admission_join");
  check(start.fixtureIpv4 === ready.listenIpv4 && start.fixturePort === ready.listenPort &&
    start.authorityPublicKey === ready.authorityPublicKey, "noise_fixture_binding");
  check(job.localSocketPort === terminal.candidates[0].remotePort, "noise_connection_join");
  return { schema: "noise-serial-shape-check-v1", protocol_consistent: true, hardware_qualified: false };
}

/** Validate the frozen parent-witness shape without interpreting witness authenticity. */
export function parseCleanupReceipt(value) {
  object(value, ["schema", "contextSha256", "browserClosed", "browserOwnershipReleased", "fixtureExited",
    "fixtureExitCode", "supervisorExited", "supervisorExitCode", "remainingOwnedProcesses", "listenerAbsent",
    "serialHoldersAbsent", "deviceResourcesReleased", "observedAtHostUnixMs", "witnesses"]);
  check(value.schema === "noise-serial-cleanup-v1", "noise_cleanup_schema"); digest(value.contextSha256);
  for (const key of ["browserClosed", "browserOwnershipReleased", "fixtureExited", "supervisorExited", "listenerAbsent", "serialHoldersAbsent", "deviceResourcesReleased"]) boolean(value[key]);
  for (const key of ["fixtureExitCode", "supervisorExitCode"]) if (value[key] !== null) {
    check(Number.isInteger(value[key]) && value[key] >= -2147483648 && value[key] <= 2147483647, "noise_exit_code");
  }
  uint(value.remainingOwnedProcesses); uint(value.observedAtHostUnixMs);
  object(value.witnesses, ["browser", "fixture", "supervisor", "resources"]);
  Object.values(value.witnesses).forEach(digest);
  return structuredClone(value);
}

/** Shape-only redacted projection parser. No result or publication writer exists yet. */
export function parseProjection(value) {
  object(value, ["schema_version", "status", "board", "diagnostic_ordinal", "source_commit", "gate_commit", "reference_commit",
    "provenance", "criteria", "timings_ms", "counts", "redaction_status"]);
  check(value.schema_version === "bitaxe-stratum-v2-noise-serial-projection-v1" && value.status === "accepted" &&
    value.board === 205 && value.redaction_status === "passed", "noise_projection_schema");
  uint(value.diagnostic_ordinal); check(value.diagnostic_ordinal > 0, "noise_ordinal");
  for (const key of ["source_commit", "gate_commit", "reference_commit"]) check(typeof value[key] === "string" && /^[a-f0-9]{40}$/u.test(value[key]), "noise_commit");
  object(value.provenance, ["app_elf", "package_manifest", "contract", "fixture", "evaluator", "sealed_inventory", "private_result"]);
  Object.values(value.provenance).forEach(digest);
  object(value.criteria, ["identity", "continuity", "authority", "tcp_delivery", "noise_authentication", "encrypted_proof",
    "no_new_work", "preservation", "accounting", "restoration", "cleanup", "privacy"]);
  check(Object.values(value.criteria).every((v) => v === true), "noise_projection_criteria");
  object(value.timings_ms, ["preparation", "connect", "act_one_write", "act_two_read", "proof_write", "diagnostic",
    "device_cleanup", "fixture_lifetime", "host_cleanup"]);
  const bounds = { preparation: 60000, connect: 5000, act_one_write: 2000, act_two_read: 10000, proof_write: 2000,
    diagnostic: 120000, device_cleanup: 5000, fixture_lifetime: 150000, host_cleanup: 5000 };
  for (const [key, bound] of Object.entries(bounds)) uint(value.timings_ms[key], bound);
  const counts = { exact_peer_connections: 1, unexpected_peer_connections: 0, act_one_written: 64,
    act_one_received: 64, proof_written: 22, proof_received: 22, new_work: 0, new_shares: 0 };
  object(value.counts, Object.keys(counts));
  check(Object.entries(counts).every(([key, count]) => value.counts[key] === count), "noise_projection_counts");
  return structuredClone(value);
}
