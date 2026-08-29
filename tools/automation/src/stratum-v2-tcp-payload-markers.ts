export type JsonObject = Record<string, unknown>;

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("marker must be an object");
  }
  return value as JsonObject;
}

export function tcpPayloadStagesFromMonitor(output: string): JsonObject {
  const stages: JsonObject = {
    monitor_armed: false,
    resolved: false,
    tcp_connected: false,
    payload_sent: false,
    write_half_closed: false,
    receipt_acknowledged: false,
  };
  for (const match of output.matchAll(/stratum_v2_tcp_payload=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const stage = marker["stage"];
      if (typeof stage === "string" && Object.hasOwn(stages, stage)) stages[stage] = true;
    } catch { continue; }
  }
  return stages;
}

export function tcpPayloadTimingsFromMonitor(output: string): JsonObject {
  const timings: JsonObject = {
    connect_ms: null,
    write_ms: null,
  };
  for (const match of output.matchAll(/stratum_v2_tcp_payload_timing=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const key = `${String(marker["phase"] ?? "")}_ms`;
      const duration = marker["duration_ms"];
      if (Object.hasOwn(timings, key)
        && typeof duration === "number"
        && Number.isInteger(duration)
        && duration >= 0
        && duration <= 60_000) {
        timings[key] = duration;
      }
    } catch { continue; }
  }
  return timings;
}

export function tcpPayloadTerminalFromMonitor(output: string): JsonObject {
  const matches = [...output.matchAll(/stratum_v2_tcp_payload_terminal=(\{[^\r\n]+\})/gu)];
  const last = matches.at(-1)?.[1];
  if (last === undefined) return { category: "terminal_missing", accepted: false };
  try { return object(JSON.parse(last)); }
  catch { return { category: "terminal_malformed", accepted: false }; }
}

export function tcpPayloadSocketErrorsFromMonitor(output: string): JsonObject {
  const errors: JsonObject = {
    pre_send: "unavailable",
    post_send: "unavailable",
    post_shutdown: "unavailable",
  };
  const admitted = new Set([
    "none", "would_block", "not_connected", "out_of_memory", "invalid_input",
    "unsupported", "connection_aborted", "connection_reset", "broken_pipe", "timed_out",
    "query_failed", "other",
  ]);
  for (const match of output.matchAll(/stratum_v2_tcp_socket_error=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const phase = marker["phase"];
      const category = marker["category"];
      if (typeof phase === "string" && Object.hasOwn(errors, phase)
        && typeof category === "string" && admitted.has(category)) {
        errors[phase] = category;
      }
    } catch { continue; }
  }
  return errors;
}
