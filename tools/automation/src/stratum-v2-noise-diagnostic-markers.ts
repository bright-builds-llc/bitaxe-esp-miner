export type JsonObject = Record<string, unknown>;

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("marker must be an object");
  }
  return value as JsonObject;
}

export function noiseStagesFromMonitor(output: string): JsonObject {
  const stages: JsonObject = {
    noise_prepared: false,
    tcp_connected: false,
    act_one_created: false,
    act_one_sent: false,
    act_two_received: false,
    time_sampled: false,
    authenticated: false,
  };
  for (const match of output.matchAll(/stratum_v2_noise_diagnostic=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const stage = marker["stage"];
      if (typeof stage === "string" && Object.hasOwn(stages, stage)) stages[stage] = true;
    } catch { continue; }
  }
  return stages;
}

export function noiseTimingsFromMonitor(output: string): JsonObject {
  const timings: JsonObject = {
    keypair_preparation_ms: null,
    act_one_construction_ms: null,
    connect_ms: null,
    act_one_write_ms: null,
    act_two_read_ms: null,
  };
  for (const match of output.matchAll(/stratum_v2_noise_timing=(\{[^\r\n]+\})/gu)) {
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

export function noiseTerminalFromMonitor(output: string): JsonObject {
  const matches = [...output.matchAll(/stratum_v2_noise_terminal=(\{[^\r\n]+\})/gu)];
  const last = matches.at(-1)?.[1];
  if (last === undefined) return { category: "terminal_missing", accepted: false };
  try { return object(JSON.parse(last)); }
  catch { return { category: "terminal_malformed", accepted: false }; }
}
