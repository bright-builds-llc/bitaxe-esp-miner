import { createHash } from "node:crypto";

export const CONTRACT_PATH = "docs/hardware/str005-v2-serial-qualification.md";
export const CONTRACT_SHA256 = "d23220a4ac5021470b18d7e97cb4b82ba28232d9d5a0f8340389852b39841c79";
export const AMENDMENT_PATH = "docs/hardware/str005-v2-serial-clock-amendment.md";
export const AMENDMENT_SHA256 = "4fbdfc754610855c904c6434a4da6ef8506323c95582a0b23a6873a79bc3aaef";
export const PROFILE = "bwg-worker-stratum-v2-standard/0.1";
export const SCOPES = Object.freeze(["channel", "share"]);
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function check(condition, code) {
  if (!condition) throw Object.assign(new Error(code), { code });
}
export function object(value, keys) {
  check(value !== null && typeof value === "object" && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every((key) => Object.hasOwn(value, key)), "v2_object_shape");
}
export function uint(value, maximum = Number.MAX_SAFE_INTEGER) {
  check(Number.isSafeInteger(value) && value >= 0 && value <= maximum, "v2_integer");
  return value;
}
export function boolean(value) { check(typeof value === "boolean", "v2_boolean"); return value; }
export function nullable(value, validate) { if (value !== null) validate(value); return value; }
export function digest(value) { check(typeof value === "string" && /^[a-f0-9]{64}$/u.test(value), "v2_digest"); return value; }
export function bytes(value, length) {
  check(typeof value === "string" && /^[A-Za-z0-9_-]+$/u.test(value), "v2_base64url");
  const decoded = Buffer.from(value, "base64url");
  check(decoded.length === length && decoded.toString("base64url") === value, "v2_base64url");
  return value;
}
export function ipv4(value) {
  check(typeof value === "string" && /^(?:0|[1-9][0-9]{0,2})(?:\.(?:0|[1-9][0-9]{0,2})){3}$/u.test(value), "v2_private_ipv4");
  const octets = value.split(".").map(Number);
  check(octets.every((part) => part <= 255) && (octets[0] === 10 ||
    (octets[0] === 172 && octets[1] >= 16 && octets[1] <= 31) ||
    (octets[0] === 192 && octets[1] === 168)), "v2_private_ipv4");
  return value;
}
export function port(value) { uint(value, 65535); check(value > 0, "v2_port"); return value; }
export function tuple(value) {
  object(value, ["localIpv4", "localPort", "remoteIpv4", "remotePort"]);
  ipv4(value.localIpv4); ipv4(value.remoteIpv4); port(value.localPort); port(value.remotePort);
}
export function list(value, maximum) { check(Array.isArray(value) && value.length <= maximum, "v2_array_bound"); return value; }

/** Runtime input only. Callers must never persist this object or its digest. */
export function parseStratum(value) {
  object(value, ["profile", "endpoint", "authorityPublicKey", "userIdentity"]);
  check(value.profile === PROFILE && typeof value.endpoint === "string", "v2_stratum_profile");
  const match = /^stratum\+tcp:\/\/([^/:]+):([1-9][0-9]{0,4})\/$/u.exec(value.endpoint);
  check(match !== null, "v2_endpoint"); ipv4(match[1]); port(Number(match[2]));
  bytes(value.authorityPublicKey, 32);
  check(typeof value.userIdentity === "string" && value.userIdentity.length > 0 &&
    Buffer.byteLength(value.userIdentity, "utf8") <= 255 && !value.userIdentity.includes("\u0000") &&
    Buffer.from(value.userIdentity, "utf8").toString("utf8") === value.userIdentity, "v2_user_identity");
  return structuredClone(value);
}

export function parseChannelStart(value) {
  object(value, ["schema", "attemptId", "expectedBootOrdinal", "networkObservedAtUs", "stratum"]);
  check(value.schema === "worker-stratum-v2-channel-start-v1", "v2_start_schema");
  bytes(value.attemptId, 16); uint(value.expectedBootOrdinal); uint(value.networkObservedAtUs);
  check(value.expectedBootOrdinal > 0, "v2_boot"); parseStratum(value.stratum);
  return structuredClone(value);
}
