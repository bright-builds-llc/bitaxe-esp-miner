import { boolean, bytes, check, ipv4, object, port, SCOPES, uint } from "./values.mjs";
import { parseStatus } from "./device.mjs";

/** Runtime-only readiness; never write this object or a fingerprint of its endpoint. */
export function parseReady(value) {
  object(value, ["schema", "scope", "attemptId", "instanceId", "listenIpv4", "listenPort", "authorityPublicKey"]);
  check(value.schema === "str005-v2-fixture-ready-runtime-v1" && SCOPES.includes(value.scope), "v2_fixture_ready");
  bytes(value.attemptId, 16); bytes(value.instanceId, 16); bytes(value.authorityPublicKey, 32);
  ipv4(value.listenIpv4); port(value.listenPort); return structuredClone(value);
}
export function parseConnection(value) {
  object(value, ["schema", "scope", "attemptId", "instanceId", "connectionId", "observedAtFixtureUs",
    "localIpv4", "localPort", "peerIpv4", "peerPort"]);
  check(value.schema === "str005-v2-fixture-connection-runtime-v1" && SCOPES.includes(value.scope), "v2_fixture_connection");
  for (const key of ["attemptId", "instanceId", "connectionId"]) bytes(value[key], 16);
  uint(value.observedAtFixtureUs); ipv4(value.localIpv4); ipv4(value.peerIpv4); port(value.localPort); port(value.peerPort);
  return structuredClone(value);
}
export function parseConnectionFacts(value) {
  object(value, ["instanceId", "connectionId", "expectedPeerCount", "unexpectedPeerCount", "candidateOverflow", "expectedPeerMatch"]);
  bytes(value.instanceId, 16); if (value.connectionId !== null) bytes(value.connectionId, 16);
  uint(value.expectedPeerCount, 3); uint(value.unexpectedPeerCount, 3);
  boolean(value.candidateOverflow); boolean(value.expectedPeerMatch); return structuredClone(value);
}

/** Compare actual source observations in memory; return no endpoint or endpoint-derived hash. */
export function compareConnection(deviceStatus, fixtureConnection, fixtureReady) {
  deviceStatus = parseStatus(deviceStatus);
  const c = parseConnection(fixtureConnection), r = parseReady(fixtureReady), d = deviceStatus.connection;
  check(d !== null && deviceStatus.record !== null && c.scope === deviceStatus.scope && c.attemptId === deviceStatus.record.attemptId &&
    c.instanceId === r.instanceId && c.attemptId === r.attemptId && c.scope === r.scope, "v2_connection_identity");
  check(c.localIpv4 === r.listenIpv4 && c.localPort === r.listenPort && c.peerIpv4 === d.socket.localIpv4 &&
    c.peerPort === d.socket.localPort && c.localIpv4 === d.socket.remoteIpv4 && c.localPort === d.socket.remotePort,
    "v2_socket_tuple_conflict");
  return { connectionId: c.connectionId, instanceId: c.instanceId, tupleMatch: true };
}
