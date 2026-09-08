import { exactObject, requireCondition } from "./contract.mjs";

const COUNTS = ["poll_requested", "poll_idle", "poll_nonce", "poll_register", "stale_completion",
  "qualified_candidates", "below_pool_target", "duplicate_candidates"];
const DISCARDS = ["invalid_length", "invalid_preamble", "invalid_crc", "job_lookup", "core",
  "address_interval", "register_response", "parser_invariant"];
const BLOCKED = ["wrong_session", "job_lookup", "work_stale", "target_mismatch", "other"];
const u64 = (value) => typeof value === "string" && /^(0|[1-9][0-9]{0,19})$/u.test(value) &&
  BigInt(value) <= 18446744073709551615n;

/** Validates diagnostic counters only; they cannot establish work or lease authority. */
export function validateMiningProgress(value, generation) {
  const v2 = value?.schema === "worker-mining-progress-v2";
  exactObject(value, ["schema", "generation", "observed_at_ms", ...COUNTS, "discards", "blocked",
    ...(v2 ? ["expected_filter", "expected_filter_matches", "expected_filter_misses"] : [])]);
  exactObject(value.discards, DISCARDS);
  exactObject(value.blocked, BLOCKED);
  requireCondition(value.schema === (v2 ? "worker-mining-progress-v2" : "worker-mining-progress-v1") &&
    Number.isInteger(value.generation) && value.generation > 0 && value.generation <= 0xffffffff &&
    value.generation === generation && u64(value.observed_at_ms) &&
    COUNTS.every((key) => u64(value[key])) && DISCARDS.every((key) => u64(value.discards[key])) &&
    BLOCKED.every((key) => u64(value.blocked[key])), "mining_progress_shape");
  if (v2) requireCondition(value.expected_filter === "bm1366-ticket-256-leading-zero-40-v1" &&
    u64(value.expected_filter_matches) && u64(value.expected_filter_misses), "mining_filter_shape");
  return value;
}
