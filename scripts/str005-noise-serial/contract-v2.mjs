// Pure closed-schema parsing. These functions never establish hardware authority.
export const BASE_CONTRACT_SHA256 = "0da417fc198a89042eb62902999ac822be5365a5f0333df8220f15afe3447a62";
export const ACT_TWO_BYTES = 234; // noise_sv2 1.4.2: 64 + (64 + 16) + (74 + 16).
export const STAGES = ["noise_prepared", "tcp_connected", "act_one_written", "act_two_received",
  "authority_verified", "proof_written", "socket_closed", "worker_quiescent"];
export const OUTCOMES = ["accepted", "rejected", "cancelled", "expired", "incomplete"];
const CATEGORIES = ["preparation", "connect", "write", "read", "authentication", "proof",
  "authority_lost", "clock_invalid", "cleanup", "evidence_incomplete"];
const DETAILS = ["timeout", "eof", "partial", "extra", "malformed", "io", "wrong_authority",
  "certificate_time", "session_replaced", "heartbeat_expired", "cancel_requested", "clock_discontinuity",
  "delivery_ambiguous", "resource_unreleased", "identity_conflict", "peer_conflict", "overflow",
  "missing", "stale", "duplicate", "rng", "allocation", "before_epoch", "time_overflow"];

export function requireValue(condition, code) {
  if (!condition) throw Object.assign(new Error(code), { code });
}
export function object(value, keys, code = "noise_object_shape") {
  requireValue(value !== null && typeof value === "object" && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every((key) => Object.hasOwn(value, key)), code);
}
export function uint(value, maximum = Number.MAX_SAFE_INTEGER) {
  requireValue(Number.isSafeInteger(value) && value >= 0 && value <= maximum, "noise_integer");
}
export function boolean(value) { requireValue(typeof value === "boolean", "noise_boolean"); }
export function digest(value) { requireValue(typeof value === "string" && /^[a-f0-9]{64}$/u.test(value), "noise_digest"); }
export function canonical(value, bytes) {
  requireValue(typeof value === "string" && /^[A-Za-z0-9_-]+$/u.test(value) &&
    Buffer.from(value, "base64url").length === bytes && Buffer.from(value, "base64url").toString("base64url") === value,
  "noise_canonical_encoding");
}
export function port(value) { uint(value, 65535); requireValue(value > 0, "noise_port"); }
export function privateIpv4(value) {
  requireValue(typeof value === "string" && /^(?:0|[1-9][0-9]{0,2})(?:\.(?:0|[1-9][0-9]{0,2})){3}$/u.test(value), "noise_ipv4");
  const bytes = value.split(".").map(Number);
  requireValue(bytes.every((n) => n <= 255) && (bytes[0] === 10 || (bytes[0] === 172 && bytes[1] >= 16 && bytes[1] <= 31) ||
    (bytes[0] === 192 && bytes[1] === 168)), "noise_ipv4");
}
export function cause(value, device = false) {
  object(value, ["stage", "category", "detail", ...(device ? ["atUs"] : [])]);
  requireValue([...STAGES, "admission", "fixture_ready", "candidate_inventory", "proof_received", "cleanup", "evidence"].includes(value.stage) &&
    CATEGORIES.includes(value.category) && DETAILS.includes(value.detail), "noise_cause");
  if (device && value.atUs !== null) uint(value.atUs);
}
export function parseStartV2(value) {
  object(value, ["schema", "attemptId", "expectedBootOrdinal", "networkObservedAtUs", "fixtureIpv4", "fixturePort", "authorityPublicKey"]);
  requireValue(value.schema === "worker-noise-diagnostic-start-v2", "noise_start_schema");
  canonical(value.attemptId, 16); canonical(value.authorityPublicKey, 32);
  uint(value.expectedBootOrdinal); requireValue(value.expectedBootOrdinal > 0, "noise_boot");
  uint(value.networkObservedAtUs); privateIpv4(value.fixtureIpv4); port(value.fixturePort);
  return structuredClone(value);
}

export { parseStartV2 as parseStart };
