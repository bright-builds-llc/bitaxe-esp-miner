type JsonObject = Record<string, unknown>;

const keys = [
  "schema_version", "status", "board", "source_commit", "reference_commit",
  "package_manifest_sha256", "fixture_accepted",
  "share_target_valid", "safe_stop_complete", "settings_restored", "package_restored",
  "mineonboot_false", "mining_inactive", "mining_activity_category", "zero_hashrate",
  "zero_shares", "usb_cleanup_ready", "redaction_status", "exact_non_claims",
] as const;

const nonClaims = [
  "external_production_pool", "mixed_protocol_live_fallback", "other_boards",
  "unbounded_mining", "ota", "release_readiness",
] as const;

export function validateStratumV2CampaignProjection(
  value: unknown,
  expectedSource: string,
  expectedReference: string,
  expectedManifestSha256: string,
): void {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("projection must be an object");
  }
  const projection = value as JsonObject;
  const actualKeys = Object.keys(projection).sort();
  const expectedKeys = [...keys].sort();
  if (JSON.stringify(actualKeys) !== JSON.stringify(expectedKeys)) {
    throw new Error("projection field inventory mismatch");
  }
  if (projection["schema_version"] !== "bitaxe-stratum-v2-campaign-projection-v1"
    || projection["status"] !== "accepted"
    || projection["board"] !== 205
    || projection["source_commit"] !== expectedSource
    || projection["reference_commit"] !== expectedReference
    || projection["package_manifest_sha256"] !== expectedManifestSha256
    || !["paused", "safe_blocked"].includes(String(projection["mining_activity_category"] ?? ""))
    || projection["redaction_status"] !== "passed") {
    throw new Error("projection identity or status mismatch");
  }
  for (const key of [
    "fixture_accepted", "share_target_valid", "safe_stop_complete", "settings_restored",
    "package_restored", "mineonboot_false", "mining_inactive", "zero_hashrate", "zero_shares",
    "usb_cleanup_ready",
  ]) {
    if (projection[key] !== true) throw new Error("projection proof is incomplete");
  }
  if (JSON.stringify(projection["exact_non_claims"]) !== JSON.stringify(nonClaims)) {
    throw new Error("projection non-claim inventory mismatch");
  }
  const serialized = JSON.stringify(projection).toLowerCase();
  for (const forbidden of [
    "password", "credential", "ssid", "device_url", "authority_public_key", "poolurl",
    "pooluser", "port_path", "nonce", "private_key", "ip_address", "mac_address",
  ]) {
    if (serialized.includes(forbidden)) throw new Error("projection contains a forbidden field");
  }
}
