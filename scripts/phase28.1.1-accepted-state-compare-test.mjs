#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  compareAcceptedState,
  parseAcceptedStateLog,
  renderAcceptedStateReport,
} from "./phase28.1.1-accepted-state-compare.mjs";

const STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function marker(stage, overrides = {}) {
  const values = {
    stage,
    observation: "available",
    chip_count_class: "match",
    readable_responses: "7",
    error_counter_active: "false",
    domain_counter_active: "false",
    total_counter_active: "false",
    power_delta_class: "flat",
    result_correlated: "false",
    submit_observed: "false",
    ...overrides,
  };
  return `accepted_state_snapshot ${Object.entries(values)
    .map(([name, value]) => `${name}=${value}`)
    .join(" ")} redacted=true`;
}

function completeLog(overridesByStage = {}) {
  return STAGES.map((stage) => marker(stage, overridesByStage[stage])).join(
    "\n",
  );
}

// Arrange
const upstreamActive = completeLog({
  post_first_work: { total_counter_active: "true" },
});
const rustInactive = completeLog();

// Act
const divergence = compareAcceptedState(upstreamActive, rustInactive);

// Assert
assert.equal(divergence.accepted_state_status, "mismatch");
assert.equal(
  divergence.recommended_investigation,
  "accepted_state_transition_divergence",
);

// Arrange
const rustUnavailable = completeLog({
  post_first_work: {
    observation: "unavailable",
    chip_count_class: "unavailable",
    readable_responses: "0",
    power_delta_class: "unavailable",
  },
});

// Act
const unavailable = compareAcceptedState(completeLog(), rustUnavailable);

// Assert
assert.equal(unavailable.accepted_state_status, "unavailable");
assert.equal(
  unavailable.recommended_investigation,
  "cold_boot_recovery_lifecycle_parity",
);

// Arrange
const matchingIdle = completeLog();

// Act
const matching = compareAcceptedState(matchingIdle, matchingIdle);

// Assert
assert.equal(matching.accepted_state_status, "match");
assert.equal(
  matching.recommended_investigation,
  "cold_boot_recovery_lifecycle_parity",
);

// Arrange
const otherMismatchRust = completeLog({
  post_first_work: { power_delta_class: "falling" },
});

// Act
const otherMismatch = compareAcceptedState(completeLog(), otherMismatchRust);

// Assert
assert.equal(
  otherMismatch.recommended_investigation,
  "upstream_init_transcript_prefix_bisection",
);

// Arrange
const progressed = completeLog({
  post_first_work: {
    result_correlated: "true",
    submit_observed: "true",
    power_delta_class: "rising_hashing",
  },
});

// Act
const progress = compareAcceptedState(completeLog(), progressed);

// Assert
assert.equal(progress.recommended_investigation, "none");
assert.equal(progress.result_correlated, true);
assert.equal(progress.fake_pool_submit_observed, true);

// Arrange / Act / Assert
assert.throws(
  () =>
    parseAcceptedStateLog(
      marker("unknown_stage"),
    ),
  /unknown stage/u,
);

// Arrange
const sensitiveNoise =
  `poolPassword=do-not-copy device_url=http://192.0.2.1\n${completeLog()}`;

// Act
const rendered = renderAcceptedStateReport(
  compareAcceptedState(completeLog(), sensitiveNoise),
);

// Assert
for (const forbidden of [
  "poolPassword",
  "do-not-copy",
  "device_url",
  "192.0.2.1",
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
  "ticket_mask_asic_difficulty",
  "count_asic_chips_rx_loop_parity",
  "negotiated_version_mask_work_field_parity",
  "pool_negotiated_mask_asic_reload",
]) {
  assert.equal(rendered.includes(forbidden), false);
}

{
  // Arrange
  const upstream = "";
  const rust = "";

  // Act
  const report = compareAcceptedState(upstream, rust);

  // Assert
  assert.equal(report.accepted_state_status, "unavailable");
  assert.equal(report.first_divergent_stage, "post_enumerate");
  assert.equal(
    report.recommended_investigation,
    "cold_boot_recovery_lifecycle_parity",
  );
}

{
  // Arrange
  const partial = STAGES.slice(0, 3)
    .map((stage) => marker(stage))
    .join("\n");

  // Act
  const report = compareAcceptedState(partial, partial);

  // Assert
  assert.equal(report.accepted_state_status, "unavailable");
  assert.equal(report.first_divergent_stage, "post_mask_reload");
}

{
  // Arrange
  const upstream = completeLog();
  const rust = STAGES.slice(0, 4)
    .map((stage) => marker(stage))
    .join("\n");

  // Act
  const report = compareAcceptedState(upstream, rust);

  // Assert
  assert.equal(report.accepted_state_status, "unavailable");
  assert.equal(report.first_divergent_stage, "post_first_work");
}

process.stdout.write("accepted_state_compare_test: passed\n");
