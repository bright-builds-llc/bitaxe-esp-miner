import test from "node:test";
import assert from "node:assert/strict";
import { validateMiningProgress } from "./mining-progress.mjs";
import { validateQualification } from "./judge.mjs";

function progress() {
  return { schema: "worker-mining-progress-v1", generation: 3, observed_at_ms: "4567",
    poll_requested: "20", poll_idle: "15", poll_nonce: "4", poll_register: "1", stale_completion: "0",
    qualified_candidates: "0", below_pool_target: "4", duplicate_candidates: "0",
    discards: { invalid_length: "0", invalid_preamble: "0", invalid_crc: "0", job_lookup: "0",
      core: "0", address_interval: "0", register_response: "0", parser_invariant: "0" },
    blocked: { wrong_session: "0", job_lookup: "0", work_stale: "0", target_mismatch: "0", other: "0" } };
}
function qualification() {
  return { schema: "worker-qualification-v1", generation: 3, active_ms: 1000, generation_elapsed_ms: 2000,
    budget_reserved_ms: 240000, budget_complete: true, submitted: 0, accepted: 0, rejected: 0,
    nonce_work_correlations: 0, work_dispatched: 1, last_valid_heartbeat_ms: 2000,
    safe_stop_complete: false, voltage_fresh: true, power_fresh: true, temperature_fresh: true,
    fan_fresh: true, watchdog_alive: true, mine_on_boot: false, voltage_volts: 5, power_watts: 9,
    chip_temp_celsius: 40, fan_rpm: 7000, gate_closed_ms: null, shutdown_started_ms: null,
    safe_stop_stage: "not_started", revocation_reason: "none", active_limit_ms: 30000,
    shutdown_budget_ms: 15550, work_gate_remaining_ms: 10000 };
}
test("below-target ASIC responses remain diagnostic and preserve zero qualified candidates", () => {
  // Arrange
  const value = { ...qualification(), mining_progress: progress() };
  // Act
  const result = validateQualification(value);
  // Assert
  assert.equal(result.mining_progress.poll_nonce, "4");
  assert.equal(result.nonce_work_correlations, 0);
  assert.equal(result.submitted, 0);
  assert.equal(result.accepted, 0);
});
test("historical qualification without mining progress remains readable", () => {
  assert.equal(validateQualification(qualification()).mining_progress, undefined);
});
test("mining diagnostics reject private or unknown fields at every level", () => {
  for (const mutate of [value => { value.rawJob = "private"; },
    value => { value.discards.poolUser = "private"; }, value => { value.blocked.lease = "private"; }]) {
    const value = progress(); mutate(value);
    assert.throws(() => validateMiningProgress(value, 3));
  }
});
test("mining diagnostics require the current generation and canonical bounded counts", () => {
  for (const generation of [0, 2, 0x100000000]) {
    assert.throws(() => validateMiningProgress({ ...progress(), generation }, 3));
  }
  for (const value of [0, "-1", "01", "1.5", "18446744073709551616", "secret"]) {
    assert.throws(() => validateMiningProgress({ ...progress(), poll_nonce: value }, 3));
  }
  const maximum = { ...progress(), poll_nonce: "18446744073709551615" };
  assert.equal(validateMiningProgress(maximum, 3).poll_nonce, maximum.poll_nonce);
});
