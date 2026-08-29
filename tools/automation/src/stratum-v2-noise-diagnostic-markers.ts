export type JsonObject = Record<string, unknown>;

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("marker must be an object");
  }
  return value as JsonObject;
}

export function noiseStagesFromMonitor(output: string): JsonObject {
  const stages: JsonObject = {
    monitor_armed: false,
    noise_prepared: false,
    tcp_connected: false,
    act_one_created: false,
    act_one_sent: false,
    act_two_received: false,
    time_sampled: false,
    authenticated: false,
    encrypted_proof_sent: false,
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
    proof_write_ms: null,
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

function consistentBoundedNumber(values: readonly number[], maximum: number): number | null {
  const bounded = values.filter(value => Number.isInteger(value) && value >= 0 && value <= maximum);
  const distinct = new Set(bounded);
  return bounded.length > 0 && distinct.size === 1 ? bounded[0] ?? null : null;
}

export function noiseSendFromMonitor(output: string): JsonObject {
  const actOne: number[] = [];
  const proof: number[] = [];
  for (const match of output.matchAll(/stratum_v2_noise_send=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const count = marker["bytes_written"];
      if (typeof count !== "number") continue;
      if (marker["kind"] === "act_one") actOne.push(count);
      if (marker["kind"] === "proof") proof.push(count);
    } catch { continue; }
  }
  return {
    adapter: "std",
    authority_required: true,
    act_one_reported_bytes: consistentBoundedNumber(actOne, 64),
    proof_reported_bytes: consistentBoundedNumber(proof, 65_535),
  };
}

export function noiseSocketErrorsFromMonitor(output: string): JsonObject {
  const phases: Record<string, string[]> = {
    pre_act_one: [],
    post_act_one: [],
    post_act_two: [],
    post_proof: [],
  };
  const allowed = new Set([
    "none", "would_block", "not_connected", "out_of_memory", "invalid_input", "unsupported",
    "connection_aborted", "connection_reset", "broken_pipe", "timed_out", "other",
    "query_failed",
  ]);
  for (const match of output.matchAll(/stratum_v2_noise_socket_error=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const phase = String(marker["phase"] ?? "");
      const category = String(marker["category"] ?? "");
      if (Object.hasOwn(phases, phase) && allowed.has(category)) phases[phase]?.push(category);
    } catch { continue; }
  }
  const result: JsonObject = {};
  for (const [phase, values] of Object.entries(phases)) {
    const distinct = new Set(values);
    result[`${phase}_error`] = values.length > 0 && distinct.size === 1
      ? values[0]
      : "unavailable";
  }
  return result;
}
