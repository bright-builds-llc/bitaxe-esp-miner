export type JsonObject = Record<string, unknown>;

function object(value: unknown): JsonObject | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as JsonObject
    : undefined;
}

export function tcpPayloadPrivateLocalPortFromMonitor(output: string): {
  readonly localPort: number | null;
  readonly markerCount: number;
  readonly consistent: boolean;
} {
  const ports: number[] = [];
  for (const match of output.matchAll(/stratum_v2_tcp_connection_private=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""));
      const port = marker?.["local_port"];
      if (typeof port === "number" && Number.isInteger(port) && port >= 1 && port <= 65_535) {
        ports.push(port);
      }
    } catch { continue; }
  }
  const distinct = new Set(ports);
  return {
    localPort: distinct.size === 1 ? ports[0] ?? null : null,
    markerCount: ports.length,
    consistent: ports.length > 0 && distinct.size === 1,
  };
}

function boundedCount(value: unknown, maximum = 65_535): number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= maximum
    ? value
    : 0;
}

function closedReadCategory(value: unknown): string {
  const category = String(value ?? "unavailable");
  return [
    "complete", "timeout", "eof", "io", "mismatch", "extra", "observation_end",
  ].includes(category)
    ? category
    : "unavailable";
}

export function projectTcpPayloadConnection(
  monitorOutput: string,
  privateFixtureProgress: JsonObject,
): { readonly connection: JsonObject; readonly fixture: JsonObject } {
  const identity = tcpPayloadPrivateLocalPortFromMonitor(monitorOutput);
  const candidatesValue = privateFixtureProgress["tcp_candidates"];
  const candidates: JsonObject[] = Array.isArray(candidatesValue)
    ? candidatesValue
      .map(candidate => object(candidate))
      .filter((candidate): candidate is JsonObject => candidate !== undefined)
    : [];
  const correlated = identity.localPort === null
    ? undefined
    : candidates.find(candidate => candidate["remote_port"] === identity.localPort);
  const exactPeerConnectionCount = boundedCount(
    privateFixtureProgress["exact_peer_connection_count"],
    3,
  );
  const candidateOverflow = privateFixtureProgress["candidate_overflow"] === true;
  const tupleMatch = identity.consistent && correlated !== undefined;
  const safeCandidate = correlated ?? candidates[0];
  const payloadBytesReceived = boundedCount(safeCandidate?.["payload_bytes_received"], 65);
  const extraBytesReceived = boundedCount(safeCandidate?.["extra_bytes_received"], 1);
  const payloadReadCategory = closedReadCategory(safeCandidate?.["payload_read_category"]);
  const payloadDigestMatch = safeCandidate?.["payload_digest_match"] === true;
  const receiptAckSent = safeCandidate?.["receipt_ack_sent"] === true;
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
      payload_bytes_received: payloadBytesReceived,
      payload_read_category: payloadReadCategory,
      payload_digest_match: payloadDigestMatch,
      extra_bytes_received: extraBytesReceived,
      receipt_ack_sent: receiptAckSent,
    },
  };
}
