import { readFile } from "node:fs/promises";

type JsonObject = Record<string, unknown>;

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
): Promise<void> {
  const projection = object(JSON.parse(await readFile(candidate, "utf8")));
  const stages = object(projection["stages"]);
  const timings = object(projection["timings"]);
  const fixture = object(projection["fixture"]);
  const restoration = object(projection["restoration"]);
  const accepted = projection["status"] === "accepted";
  if (projection["schema_version"] !== "bitaxe-stratum-v2-noise-diagnostic-projection-v1"
    || projection["board"] !== 205
    || projection["diagnostic_ordinal"] !== expectedOrdinal
    || projection["source_commit"] !== expectedSource
    || typeof projection["reference_commit"] !== "string"
    || typeof projection["app_elf_sha256"] !== "string"
    || !["accepted", "failed"].includes(String(projection["status"] ?? ""))
    || typeof projection["terminal_category"] !== "string"
    || projection["redaction_complete"] !== true
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
    || restoration["owned_processes_remaining"] !== 0) {
    throw new Error("diagnostic projection contract mismatch");
  }
  const requiredStages = [
    "noise_prepared", "tcp_connected", "act_one_created", "act_one_sent", "act_two_received",
    "time_sampled", "authenticated",
  ];
  const requiredTimings = [
    "keypair_preparation_ms", "act_one_construction_ms", "connect_ms",
    "act_one_write_ms", "act_two_read_ms",
  ];
  const requiredFixture = [
    "listener_ready", "connection_accepted", "peer_matched", "act_one_received", "responder_created",
    "act_two_created", "act_two_sent", "client_authenticated",
  ];
  const timingsMalformed = requiredTimings.some(key => {
    const value = timings[key];
    return value !== null
      && (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 60_000);
  });
  if (timingsMalformed) throw new Error("diagnostic timing contract mismatch");
  if (accepted && (requiredStages.some(key => stages[key] !== true)
    || requiredTimings.some(key => typeof timings[key] !== "number")
    || Number(timings["keypair_preparation_ms"])
      + Number(timings["act_one_construction_ms"]) > 60_000
    || requiredFixture.some(key => fixture[key] !== true)
    || fixture["act_one_bytes_received"] !== 64
    || fixture["act_one_read_category"] !== "complete"
    || typeof fixture["accept_to_first_byte_millis"] !== "number"
    || Number(fixture["accept_to_first_byte_millis"]) > 10_000
    || typeof fixture["act_one_read_millis"] !== "number"
    || Number(fixture["act_one_read_millis"]) > 10_000
    || projection["terminal_category"] !== "accepted")) {
    throw new Error("accepted diagnostic evidence is incomplete");
  }
}
