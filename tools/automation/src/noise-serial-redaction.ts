/** Closed v2 public shape. This checks privacy, never hardware acceptance. */
export function validateNoiseSerialProjection(value: Record<string, unknown>): void {
  function object(input: unknown, keys: readonly string[]): Record<string, unknown> {
    if (!input || typeof input !== "object" || Array.isArray(input)) throw new Error("noise_projection_shape");
    const result = input as Record<string, unknown>;
    if (Object.keys(result).length !== keys.length || keys.some(key => !Object.hasOwn(result, key))) throw new Error("noise_projection_fields");
    return result;
  }
  const keys = ["schema_version", "status", "board", "diagnostic_ordinal", "source_commit", "gate_commit", "reference_commit", "provenance", "criteria", "timings_ms", "counts", "redaction_status"];
  object(value, keys);
  if (value["schema_version"] !== "bitaxe-stratum-v2-noise-serial-projection-v2" || value["status"] !== "accepted" || value["board"] !== 205 || value["redaction_status"] !== "passed") throw new Error("noise_projection_profile");
  function integer(input: unknown, maximum: number): void {
    if (typeof input !== "number" || !Number.isSafeInteger(input) || input < 0 || input > maximum) throw new Error("noise_projection_number");
  }
  integer(value["diagnostic_ordinal"], Number.MAX_SAFE_INTEGER);
  if (value["diagnostic_ordinal"] === 0) throw new Error("noise_projection_ordinal");
  for (const key of ["source_commit", "gate_commit", "reference_commit"]) if (typeof value[key] !== "string" || !/^[a-f0-9]{40}$/u.test(value[key])) throw new Error("noise_projection_commit");
  const provenance = object(value["provenance"], ["app_elf", "package_manifest", "contract", "fixture", "evaluator", "sealed_inventory", "private_result"]);
  if (Object.values(provenance).some(item => typeof item !== "string" || !/^[a-f0-9]{64}$/u.test(item))) throw new Error("noise_projection_digest");
  const criteria = object(value["criteria"], ["identity", "continuity", "authority", "tcp_delivery", "noise_authentication", "encrypted_proof", "no_new_work", "preservation", "accounting", "restoration", "cleanup", "privacy"]);
  if (Object.values(criteria).some(item => item !== true)) throw new Error("noise_projection_criteria");
  const limits = { preparation: 60000, connect: 5000, act_one_write: 2000, act_two_read: 10000, proof_write: 2000, diagnostic: 120000, device_cleanup: 120000, fixture_lifetime: 150000, host_cleanup: 5000 };
  const timings = object(value["timings_ms"], Object.keys(limits));
  for (const [key, limit] of Object.entries(limits)) integer(timings[key], limit);
  const counts = { exact_peer_connections: 1, unexpected_peer_connections: 0, act_one_written: 64, act_one_received: 64, proof_written: 22, proof_received: 22, new_work: 0, new_shares: 0 };
  const observed = object(value["counts"], Object.keys(counts));
  if (Object.entries(counts).some(([key, expected]) => observed[key] !== expected)) throw new Error("noise_projection_counts");
}
