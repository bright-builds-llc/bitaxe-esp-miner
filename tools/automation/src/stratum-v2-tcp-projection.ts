import { projectTcpPayloadConnection, type JsonObject } from "./stratum-v2-tcp-connection.js";

type TcpPayloadProjectionInput = {
  readonly sourceCommit: string;
  readonly referenceCommit: unknown;
  readonly appElfSha256: unknown;
  readonly planSha256: string;
  readonly packageManifestSha256: string;
  readonly evaluatorSha256: string;
  readonly earliestCategory: string;
  readonly stages: JsonObject;
  readonly timings: JsonObject;
  readonly socketErrors: JsonObject;
  readonly terminal: JsonObject;
  readonly monitorOutput: string;
  readonly fixtureTerminal: JsonObject;
  readonly fixtureProgress: JsonObject;
  readonly diagnosticAccepted: boolean;
  readonly restoration: JsonObject;
};

export function buildTcpPayloadProjection(input: TcpPayloadProjectionInput): JsonObject {
  const projected = projectTcpPayloadConnection(input.monitorOutput, input.fixtureProgress);
  const connection = projected.connection;
  const accepted = input.diagnosticAccepted
    && connection["tuple_match"] === true
    && connection["exact_peer_connection_count"] === 1
    && connection["candidate_overflow"] === false;
  const reportedBytes = typeof input.terminal["bytes_written"] === "number"
    && Number.isInteger(input.terminal["bytes_written"])
    && Number(input.terminal["bytes_written"]) >= 0
    && Number(input.terminal["bytes_written"]) <= 64
    ? Number(input.terminal["bytes_written"])
    : 0;
  return {
    schema_version: "bitaxe-stratum-v2-tcp-payload-projection-v2",
    status: accepted ? "accepted" : "failed",
    board: 205,
    diagnostic_ordinal: 9,
    source_commit: input.sourceCommit,
    reference_commit: input.referenceCommit,
    app_elf_sha256: input.appElfSha256,
    plan_sha256: input.planSha256,
    package_manifest_sha256: input.packageManifestSha256,
    payload_sha256: "fdeab9acf3710362bd2658cdc9a29e8f9c757fcf9811603a8c447cd1d9151108",
    evaluator_sha256: input.evaluatorSha256,
    terminal_category: accepted ? "accepted" : input.earliestCategory,
    stages: input.stages,
    timings: input.timings,
    connection,
    send: {
      adapter: "std",
      reported_bytes: reportedBytes,
      pre_send_error: input.socketErrors["pre_send"] ?? "unavailable",
      post_send_error: input.socketErrors["post_send"] ?? "unavailable",
      post_shutdown_error: input.socketErrors["post_shutdown"] ?? "unavailable",
      category: input.terminal["category"] ?? "terminal_missing",
    },
    fixture: projected.fixture,
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
