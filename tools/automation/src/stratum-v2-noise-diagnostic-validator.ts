import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

type JsonObject = Record<string, unknown>;

const evaluatorSources = [
  "firmware/bitaxe/src/settings_adapter/noise_diagnostic.rs",
  "firmware/bitaxe/src/startup.rs",
  "firmware/bitaxe/src/stratum_v2_noise_diagnostic.rs",
  "firmware/bitaxe/src/stratum_v2_session/transport.rs",
  "firmware/bitaxe/src/stratum_v2_tcp_payload_replay.rs",
  "tools/automation/src/stratum-v2-noise-connection.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic-cli.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic-markers.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic-process.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic-validator.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic-validator-cli.ts",
  "tools/automation/src/stratum-v2-noise-diagnostic.ts",
  "tools/automation/src/stratum-v2-noise-finalize.ts",
  "tools/automation/src/stratum-v2-noise-projection.ts",
  "tools/automation/src/stratum-v2-noise-publish.ts",
  "tools/automation/src/stratum-v2-noise-recovery.ts",
  "tools/automation/src/stratum-v2-tcp-recovery-readiness.ts",
  "tools/automation/src/stratum-v2-tcp-recovery-tooling.ts",
  "tools/automation/src/stratum-v2-tcp-restore-preflight.ts",
  "tools/flash/src/noise_diagnostic.rs",
  "tools/flash/src/restore_installed.rs",
  "tools/flash/src/restore_installed/contract.rs",
  "tools/flash/src/wifi.rs",
  "tools/stratum-v2-fixture/src/main.rs",
  "tools/stratum-v2-fixture/src/noise_auth_inventory.rs",
  "tools/stratum-v2-fixture/src/noise_frame.rs",
] as const;

export async function noiseAuthEvaluatorIdentity(workspace: string): Promise<string> {
  const hash = createHash("sha256");
  for (const relative of evaluatorSources) {
    hash.update(relative);
    hash.update("\0");
    hash.update(await readFile(path.join(workspace, relative)));
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

export async function validateNoiseDiagnosticProjection(
  candidate: string,
  expectedSource: string,
  expectedOrdinal: number,
  maybeWorkspace?: string,
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
    "none", "would_block", "not_connected", "out_of_memory", "invalid_input", "unsupported",
    "connection_aborted", "connection_reset", "broken_pipe", "timed_out", "other",
    "query_failed", "unavailable",
  ]);
  const maybeActOneBytes = send["act_one_reported_bytes"];
  const maybeProofBytes = send["proof_reported_bytes"];
  if (projection["schema_version"] !== "bitaxe-stratum-v2-noise-auth-projection-v1"
    || projection["board"] !== 205
    || projection["diagnostic_ordinal"] !== expectedOrdinal
    || projection["source_commit"] !== expectedSource
    || typeof projection["reference_commit"] !== "string"
    || typeof projection["app_elf_sha256"] !== "string"
    || typeof projection["plan_sha256"] !== "string"
    || typeof projection["package_manifest_sha256"] !== "string"
    || typeof projection["evaluator_sha256"] !== "string"
    || (maybeWorkspace !== undefined
      && projection["evaluator_sha256"] !== await noiseAuthEvaluatorIdentity(maybeWorkspace))
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
    || !Number.isInteger(connection["exact_peer_connection_count"])
    || Number(connection["exact_peer_connection_count"]) < 0
    || Number(connection["exact_peer_connection_count"]) > 3
    || !Number.isInteger(connection["other_exact_peer_connection_count"])
    || send["adapter"] !== "std"
    || send["authority_required"] !== true
    || (maybeActOneBytes !== null
      && (!Number.isInteger(maybeActOneBytes) || Number(maybeActOneBytes) < 0
        || Number(maybeActOneBytes) > 64))
    || (maybeProofBytes !== null
      && (!Number.isInteger(maybeProofBytes) || Number(maybeProofBytes) < 0
        || Number(maybeProofBytes) > 65_535))
    || !socketErrors.has(String(send["pre_act_one_error"] ?? ""))
    || !socketErrors.has(String(send["post_act_one_error"] ?? ""))
    || !socketErrors.has(String(send["post_act_two_error"] ?? ""))
    || !socketErrors.has(String(send["post_proof_error"] ?? ""))
    || !Number.isInteger(fixture["unexpected_peer_count"])
    || Number(fixture["unexpected_peer_count"]) < 0
    || !Number.isInteger(fixture["act_one_bytes_received"])
    || Number(fixture["act_one_bytes_received"]) < 0
    || Number(fixture["act_one_bytes_received"]) > 64
    || !["complete", "timeout", "eof", "io", "extra", "observation_end", "unavailable"]
      .includes(String(fixture["act_one_read_category"] ?? ""))) {
    throw new Error("diagnostic projection contract mismatch");
  }
  const requiredStages = [
    "monitor_armed", "noise_prepared", "tcp_connected", "act_one_created", "act_one_sent",
    "act_two_received", "time_sampled", "authenticated", "encrypted_proof_sent",
  ];
  const requiredTimings = [
    "keypair_preparation_ms", "act_one_construction_ms", "connect_ms",
    "act_one_write_ms", "act_two_read_ms", "proof_write_ms",
  ];
  const requiredFixture = [
    "listener_ready", "connection_accepted", "peer_matched", "responder_created",
    "act_two_created", "act_two_sent", "client_authenticated", "noise_authenticated",
    "encrypted_proof_exact",
  ];
  const timingsMalformed = requiredTimings.some(key => {
    const value = timings[key];
    return value !== null
      && (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 60_000);
  });
  if (timingsMalformed) throw new Error("diagnostic timing contract mismatch");
  const encoded = JSON.stringify(projection);
  if (/local_port|remote_port|noise_candidates|authority_public_key|certificate|endpoint|credential/iu
    .test(encoded)) {
    throw new Error("diagnostic projection contains protected fields");
  }
  if (accepted && (requiredStages.some(key => stages[key] !== true)
    || requiredTimings.some(key => typeof timings[key] !== "number")
    || Number(timings["keypair_preparation_ms"])
      + Number(timings["act_one_construction_ms"]) > 60_000
    || requiredFixture.some(key => fixture[key] !== true)
    || fixture["unexpected_peer_count"] !== 0
    || connection["tuple_match"] !== true
    || connection["local_marker_consistent"] !== true
    || connection["exact_peer_connection_count"] !== 1
    || connection["other_exact_peer_connection_count"] !== 0
    || connection["candidate_overflow"] !== false
    || connection["correlated_candidate_found"] !== true
    || send["adapter"] !== "std"
    || send["authority_required"] !== true
    || send["act_one_reported_bytes"] !== 64
    || send["proof_reported_bytes"] !== 22
    || send["pre_act_one_error"] !== "none"
    || send["post_act_one_error"] !== "none"
    || send["post_act_two_error"] !== "none"
    || send["post_proof_error"] !== "none"
    || fixture["act_one_bytes_received"] !== 64
    || fixture["act_one_read_category"] !== "complete"
    || projection["terminal_category"] !== "accepted")) {
    throw new Error("accepted diagnostic evidence is incomplete");
  }
}
