export type JsonObject = Record<string, unknown>;

function maybeObject(value: unknown): JsonObject | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as JsonObject
    : undefined;
}

function boundedCount(value: unknown, maximum = 65_535): number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= maximum
    ? value
    : 0;
}

function closedReadCategory(value: unknown): string {
  const category = String(value ?? "unavailable");
  return ["complete", "timeout", "eof", "io", "extra", "observation_end"].includes(category)
    ? category
    : "unavailable";
}

function privateLocalPort(output: string): {
  readonly localPort: number | null;
  readonly consistent: boolean;
} {
  const ports: number[] = [];
  for (const match of output.matchAll(/stratum_v2_noise_connection_private=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = maybeObject(JSON.parse(match[1] ?? ""));
      const port = marker?.["local_port"];
      if (typeof port === "number" && Number.isInteger(port) && port >= 1 && port <= 65_535) {
        ports.push(port);
      }
    } catch { continue; }
  }
  const distinct = new Set(ports);
  return {
    localPort: distinct.size === 1 ? ports[0] ?? null : null,
    consistent: ports.length > 0 && distinct.size === 1,
  };
}

export function projectNoiseAuthConnection(
  monitorOutput: string,
  privateFixtureProgress: JsonObject,
): { readonly connection: JsonObject; readonly fixture: JsonObject } {
  const identity = privateLocalPort(monitorOutput);
  const candidateValues = privateFixtureProgress["noise_candidates"];
  const candidates = Array.isArray(candidateValues)
    ? candidateValues
      .map(value => maybeObject(value))
      .filter((value): value is JsonObject => value !== undefined)
    : [];
  const correlated = identity.localPort === null
    ? undefined
    : candidates.find(candidate => candidate["remote_port"] === identity.localPort);
  const exactPeerConnectionCount = boundedCount(
    privateFixtureProgress["exact_peer_connection_count"],
    3,
  );
  const tupleMatch = identity.consistent && correlated !== undefined;
  const candidateOverflow = privateFixtureProgress["candidate_overflow"] === true;
  const safeCandidate = correlated ?? candidates[0];
  return {
    connection: {
      tuple_match: tupleMatch,
      local_marker_consistent: identity.consistent,
      exact_peer_connection_count: exactPeerConnectionCount,
      other_exact_peer_connection_count: tupleMatch
        ? Math.max(0, exactPeerConnectionCount - 1)
        : exactPeerConnectionCount,
      candidate_overflow: candidateOverflow,
      correlated_candidate_found: correlated !== undefined,
    },
    fixture: {
      listener_ready: privateFixtureProgress["listener_ready"] === true,
      connection_accepted: privateFixtureProgress["connection_accepted"] === true,
      peer_matched: privateFixtureProgress["peer_matched"] === true,
      unexpected_peer_count: boundedCount(privateFixtureProgress["unexpected_peer_count"]),
      act_one_bytes_received: boundedCount(safeCandidate?.["act_one_bytes_received"], 64),
      act_one_read_category: closedReadCategory(safeCandidate?.["act_one_read_category"]),
      responder_created: privateFixtureProgress["responder_created"] === true,
      act_two_created: privateFixtureProgress["act_two_created"] === true,
      act_two_sent: privateFixtureProgress["act_two_sent"] === true,
      client_authenticated: privateFixtureProgress["client_authenticated"] === true,
      noise_authenticated: privateFixtureProgress["noise_authenticated"] === true,
      encrypted_proof_exact: privateFixtureProgress["encrypted_proof_exact"] === true,
    },
  };
}
