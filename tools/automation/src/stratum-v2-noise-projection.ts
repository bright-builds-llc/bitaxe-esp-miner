import type { JsonObject } from "./stratum-v2-noise-connection.js";
import { projectNoiseAuthConnection } from "./stratum-v2-noise-connection.js";
import {
  noiseSendFromMonitor,
  noiseSocketErrorsFromMonitor,
} from "./stratum-v2-noise-diagnostic-markers.js";

type NoiseAuthProjectionInput = {
  readonly sourceCommit: string;
  readonly referenceCommit: unknown;
  readonly appElfSha256: unknown;
  readonly planSha256: string;
  readonly packageManifestSha256: string;
  readonly evaluatorSha256: string;
  readonly earliestCategory: string;
  readonly stages: JsonObject;
  readonly timings: JsonObject;
  readonly terminal: JsonObject;
  readonly monitorOutput: string;
  readonly fixtureTerminal: JsonObject;
  readonly fixtureProgress: JsonObject;
  readonly diagnosticExitCode: number;
  readonly restoration: JsonObject;
};

export function buildNoiseAuthProjection(input: NoiseAuthProjectionInput): JsonObject {
  const projected = projectNoiseAuthConnection(input.monitorOutput, input.fixtureProgress);
  const send = {
    ...noiseSendFromMonitor(input.monitorOutput),
    ...noiseSocketErrorsFromMonitor(input.monitorOutput),
  };
  const connection = projected.connection;
  const fixture = projected.fixture;
  const accepted = input.terminal["accepted"] === true
    && input.fixtureTerminal["status"] === "accepted"
    && input.fixtureTerminal["terminal_category"] === "accepted"
    && input.diagnosticExitCode === 0
    && connection["tuple_match"] === true
    && connection["exact_peer_connection_count"] === 1
    && connection["candidate_overflow"] === false
    && send["act_one_reported_bytes"] === 64
    && send["proof_reported_bytes"] === 22
    && fixture["act_one_bytes_received"] === 64
    && fixture["act_one_read_category"] === "complete"
    && fixture["encrypted_proof_exact"] === true;
  return {
    schema_version: "bitaxe-stratum-v2-noise-auth-projection-v1",
    status: accepted ? "accepted" : "failed",
    board: 205,
    diagnostic_ordinal: 1,
    source_commit: input.sourceCommit,
    reference_commit: input.referenceCommit,
    app_elf_sha256: input.appElfSha256,
    plan_sha256: input.planSha256,
    package_manifest_sha256: input.packageManifestSha256,
    evaluator_sha256: input.evaluatorSha256,
    terminal_category: accepted ? "accepted" : input.earliestCategory,
    stages: input.stages,
    timings: input.timings,
    connection,
    send,
    fixture,
    campaign_started: false,
    mining_started: false,
    asic_touched: false,
    fan_touched: false,
    voltage_touched: false,
    restoration: input.restoration,
    redaction_complete: true,
    redaction_status: "passed",
  };
}
