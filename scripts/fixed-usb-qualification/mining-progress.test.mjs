import test from "node:test";
import assert from "node:assert/strict";
import { validateMiningProgress } from "./mining-progress.mjs";
import { judgeWindow, validateQualification } from "./judge.mjs";

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
test("v2 filter observations remain distinct from pool-qualified candidates", () => {
  // Arrange
  const mining = { ...progress(), schema: "worker-mining-progress-v2",
    expected_filter: "bm1366-ticket-256-leading-zero-40-v1", expected_filter_matches: "0", expected_filter_misses: "4" };
  // Act
  const value = validateQualification({ ...qualification(), mining_progress: mining });
  // Assert
  assert.equal(value.mining_progress.expected_filter_misses, "4");
  assert.equal(value.mining_progress.qualified_candidates, "0");
  assert.equal(value.accepted, 0);
});
test("v2 requires the fixed filter and bounded counts without raw candidate data", () => {
  // Arrange
  const input = { ...progress(), schema: "worker-mining-progress-v2",
    expected_filter: "bm1366-ticket-256-leading-zero-40-v1", expected_filter_matches: "0", expected_filter_misses: "4" };
  for (const mutation of [{ expected_filter: "arbitrary" }, { raw_hash: "private" },
    { expected_filter_matches: "01" }, { expected_filter_misses: "18446744073709551616" },
    { schema: "worker-mining-progress-v1" }]) {
    // Act / Assert
    assert.throws(() => validateMiningProgress({ ...input, ...mutation }, 3));
  }
});

function faultEvidence(index) {
  const purpose = index === 1 ? "foreground_loss" : "heartbeat_loss";
  const mining_progress = { ...progress(), schema: "worker-mining-progress-v2",
    expected_filter: "bm1366-ticket-256-leading-zero-40-v1", expected_filter_matches: "1", expected_filter_misses: "0" };
  const active = { ...qualification(), mining_progress };
  const terminal = { ...active, active_ms: 4000, generation_elapsed_ms: 5000, safe_stop_complete: true,
    safe_stop_stage: "fan_paused", revocation_reason: index === 1 ? "restoration_requested" : "heartbeat_timeout",
    gate_closed_ms: 4800, shutdown_started_ms: 4900, work_gate_remaining_ms: 0 };
  const records = [
    { sequence: 1, state: { running: true, heartbeatSuppressed: index === 2, qualification: active } },
    { sequence: 2, state: { running: false, deviceRestorationConfirmed: true, deviceLeaseInactive: true, qualification: terminal } },
  ];
  const fault = { window: index, generation: 3, after_sequence: 1,
    kind: index === 1 ? "visibility_hidden" : "heartbeats_suppressed" };
  return { purpose, records, fault };
}
for (const index of [1, 2]) test(`stop window ${index} can prove work from a pre-fault fixed-filter match`, () => {
  // Arrange
  const evidence = faultEvidence(index);
  // Act
  const result = judgeWindow(index, evidence.records, evidence.fault, { iterativePurpose: evidence.purpose });
  // Assert
  assert.equal(result.expected_filter_work_verified, true);
  assert.equal(result.accepted_share_verified, false);
  assert.equal(result.nonce_work_correlations, 0);
});
test("filter misses or matches appearing only after the fault cannot prove initial work", () => {
  // Arrange
  const evidence = faultEvidence(1);
  evidence.records[0].state.qualification.mining_progress = {
    ...evidence.records[0].state.qualification.mining_progress, expected_filter_matches: "0", expected_filter_misses: "1" };
  // Act / Assert
  assert.throws(() => judgeWindow(1, evidence.records, evidence.fault, { iterativePurpose: evidence.purpose }), /fault_work_not_observed/u);
  evidence.records[1].state.qualification.mining_progress = evidence.records[0].state.qualification.mining_progress;
  assert.throws(() => judgeWindow(1, evidence.records, evidence.fault, { iterativePurpose: evidence.purpose }), /mining_evidence_missing/u);
});
test("dispatch appearing only after the fault cannot prove initial work", () => {
  // Arrange
  const evidence = faultEvidence(1);
  evidence.records[0].state.qualification.work_dispatched = 0;
  // Act / Assert
  assert.throws(() => judgeWindow(1, evidence.records, evidence.fault, { iterativePurpose: evidence.purpose }), /fault_work_not_observed/u);
});
test("fixed-filter evidence does not replace the normal qualified-share requirement", () => {
  // Arrange
  const evidence = faultEvidence(1);
  for (const record of evidence.records) record.state.qualification.active_limit_ms = 180000;
  // Act / Assert
  assert.throws(() => judgeWindow(0, evidence.records, undefined, { iterativePurpose: "normal" }), /mining_evidence_missing/u);
});
test("fixed-filter stop evidence retains the three-second deadline and legacy judgment", () => {
  // Arrange
  const evidence = faultEvidence(2);
  // Act / Assert
  assert.throws(() => judgeWindow(2, evidence.records, evidence.fault), /mining_evidence_missing/u);
  evidence.records[1].state.qualification.shutdown_started_ms = 5001;
  assert.throws(() => judgeWindow(2, evidence.records, evidence.fault, { iterativePurpose: evidence.purpose }), /revocation_deadline_missed/u);
});
