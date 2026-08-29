import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

type JsonObject = Record<string, unknown>;

const evaluatorSources = [
  "firmware/bitaxe/src/settings_adapter/tcp_payload_diagnostic.rs",
  "firmware/bitaxe/src/startup.rs",
  "firmware/bitaxe/src/stratum_v2_tcp_payload_diagnostic.rs",
  "firmware/bitaxe/src/stratum_v2_tcp_payload_replay.rs",
  "tools/automation/src/stratum-v2-tcp-connection.ts",
  "tools/automation/src/stratum-v2-tcp-fixture.ts",
  "tools/automation/src/stratum-v2-tcp-payload-markers.ts",
  "tools/automation/src/stratum-v2-tcp-payload-process.ts",
  "tools/automation/src/stratum-v2-tcp-projection.ts",
  "tools/automation/src/stratum-v2-tcp-payload-validator.ts",
  "tools/automation/src/stratum-v2-tcp-payload.ts",
  "tools/automation/src/stratum-v2-tcp-recovery-tooling.ts",
  "tools/automation/src/stratum-v2-tcp-recovery-readiness.ts",
  "tools/automation/src/stratum-v2-tcp-restore-preflight.ts",
  "tools/flash/src/tcp_payload_diagnostic.rs",
  "tools/flash/src/wifi.rs",
  "tools/stratum-v2-fixture/src/main.rs",
  "tools/stratum-v2-fixture/src/tcp_payload.rs",
  "tools/stratum-v2-fixture/src/tcp_payload_inventory.rs",
] as const;

export async function tcpPayloadEvaluatorIdentity(workspace: string): Promise<string> {
  const hash = createHash("sha256");
  for (const relative of evaluatorSources) {
    const source = await readFile(path.join(workspace, relative));
    hash.update(relative);
    hash.update("\0");
    hash.update(source);
    hash.update("\0");
  }
  return hash.digest("hex");
}

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("diagnostic projection must be an object");
  }
  return value as JsonObject;
}

export async function validateTcpPayloadDiagnosticProjection(
  candidate: string,
  expectedSource: string,
  expectedOrdinal: number,
  workspace = process.cwd(),
): Promise<void> {
  const projection = object(JSON.parse(await readFile(candidate, "utf8")));
  const stages = object(projection["stages"]);
  const timings = object(projection["timings"]);
  const connection = object(projection["connection"]);
  const send = object(projection["send"]);
  const fixture = object(projection["fixture"]);
  const restoration = object(projection["restoration"]);
  const accepted = projection["status"] === "accepted";
  const socketErrors = new Set([
    "none", "would_block", "not_connected", "out_of_memory", "invalid_input",
    "unsupported", "connection_aborted", "connection_reset", "broken_pipe", "timed_out",
    "query_failed", "other", "unavailable",
  ]);
  const encoded = JSON.stringify(projection);
  if (projection["schema_version"] !== "bitaxe-stratum-v2-tcp-payload-projection-v2"
    || projection["board"] !== 205
    || projection["diagnostic_ordinal"] !== expectedOrdinal
    || projection["source_commit"] !== expectedSource
    || typeof projection["reference_commit"] !== "string"
    || typeof projection["app_elf_sha256"] !== "string"
    || projection["payload_sha256"] !== "fdeab9acf3710362bd2658cdc9a29e8f9c757fcf9811603a8c447cd1d9151108"
    || projection["evaluator_sha256"] !== await tcpPayloadEvaluatorIdentity(workspace)
    || !["accepted", "failed"].includes(String(projection["status"] ?? ""))
    || typeof projection["terminal_category"] !== "string"
    || projection["redaction_complete"] !== true
    || projection["redaction_status"] !== "passed"
    || projection["campaign_started"] !== false
    || projection["mining_started"] !== false
    || projection["asic_touched"] !== false
    || projection["fan_touched"] !== false
    || projection["voltage_touched"] !== false
    || restoration["identity_exact"] !== true
    || restoration["settings_exact"] !== true
    || restoration["mineonboot_disabled"] !== true
    || restoration["mining_inactive"] !== true
    || restoration["zero_work"] !== true
    || restoration["usb_cleanup_complete"] !== true
    || restoration["owned_processes_remaining"] !== 0
    || typeof connection["tuple_match"] !== "boolean"
    || typeof connection["local_marker_consistent"] !== "boolean"
    || typeof connection["correlated_candidate_found"] !== "boolean"
    || typeof connection["candidate_overflow"] !== "boolean"
    || typeof connection["exact_peer_connection_count"] !== "number"
    || Number(connection["exact_peer_connection_count"]) < 0
    || Number(connection["exact_peer_connection_count"]) > 3
    || typeof connection["other_exact_peer_connection_count"] !== "number"
    || !["std", "lwip_direct"].includes(String(send["adapter"] ?? ""))
    || typeof send["reported_bytes"] !== "number"
    || !Number.isInteger(send["reported_bytes"])
    || Number(send["reported_bytes"]) < 0
    || Number(send["reported_bytes"]) > 64
    || !socketErrors.has(String(send["pre_send_error"] ?? ""))
    || !socketErrors.has(String(send["post_send_error"] ?? ""))
    || !socketErrors.has(String(send["post_shutdown_error"] ?? ""))
    || /local_port|remote_port|tcp_candidates/u.test(encoded)) {
    throw new Error("diagnostic projection contract mismatch");
  }
  const requiredStages = [
    "monitor_armed", "resolved", "tcp_connected", "payload_sent", "write_half_closed",
    "receipt_acknowledged",
  ];
  const requiredTimings = ["connect_ms", "write_ms"];
  const requiredFixture = [
    "listener_ready", "connection_accepted", "peer_matched", "payload_digest_match",
  ];
  const timingsMalformed = requiredTimings.some(key => {
    const value = timings[key];
    return value !== null
      && (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 60_000);
  });
  if (timingsMalformed) throw new Error("diagnostic timing contract mismatch");
  if (accepted && (requiredStages.some(key => stages[key] !== true)
    || requiredTimings.some(key => typeof timings[key] !== "number")
    || requiredFixture.some(key => fixture[key] !== true)
    || fixture["unexpected_peer_count"] !== 0
    || fixture["payload_bytes_received"] !== 64
    || fixture["payload_read_category"] !== "complete"
    || fixture["extra_bytes_received"] !== 0
    || fixture["receipt_ack_sent"] !== true
    || connection["tuple_match"] !== true
    || connection["local_marker_consistent"] !== true
    || connection["exact_peer_connection_count"] !== 1
    || connection["other_exact_peer_connection_count"] !== 0
    || connection["candidate_overflow"] !== false
    || connection["correlated_candidate_found"] !== true
    || send["adapter"] !== "std"
    || send["reported_bytes"] !== 64
    || projection["terminal_category"] !== "accepted")) {
    throw new Error("accepted diagnostic evidence is incomplete");
  }
}
