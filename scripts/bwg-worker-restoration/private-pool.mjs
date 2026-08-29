import { createHash } from "node:crypto";
import { lookup } from "node:dns/promises";

export async function resolvePrivatePoolEndpoint(credentials, expectedSetSha256) {
  const addresses = await lookup(credentials.poolURL, { all: true, verbatim: true });
  if (addresses.length === 0 || addresses.some((entry) =>
    entry.family !== 4 || !privateIpv4(entry.address))) {
    throw new Error("pool_endpoint_not_private");
  }
  const endpoints = [...new Set(addresses.map((entry) =>
    `${entry.address}:${credentials.poolPort}`))].sort();
  const digest = createHash("sha256").update(endpoints.join("\n")).digest("hex");
  if (digest !== expectedSetSha256) throw new Error("pool_resolution_drift");
  return endpoints[0].slice(0, endpoints[0].lastIndexOf(":"));
}

export function privateIpv4(value) {
  const octets = value.split(".").map(Number);
  if (octets.length !== 4 || octets.some((octet) =>
    !Number.isInteger(octet) || octet < 0 || octet > 255)) return false;
  const [first, second] = octets;
  return first === 10 ||
    (first === 172 && second >= 16 && second <= 31) ||
    (first === 192 && second === 168);
}
