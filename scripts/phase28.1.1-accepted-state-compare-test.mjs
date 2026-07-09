#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  compareAcceptedState,
  parseAcceptedStateLog,
  renderAcceptedStateReport,
} from "./phase28.1.1-accepted-state-compare.mjs";

function marker(overrides = {}) {
  const values = {
    stage: "post_first_work",
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

// Arrange
const upstreamActive = marker({ total_counter_active: "true" });
const rustInactive = marker();

// Act
const divergence = compareAcceptedState(upstreamActive, rustInactive);

// Assert
assert.equal(divergence.accepted_state_status, "mismatch");
assert.equal(
  divergence.recommended_investigation,
  "accepted_state_transition_divergence",
);

// Arrange
const rustUnavailable = marker({
  observation: "unavailable",
  chip_count_class: "unavailable",
  readable_responses: "0",
  power_delta_class: "unavailable",
});

// Act
const unavailable = compareAcceptedState(marker(), rustUnavailable);

// Assert
assert.equal(unavailable.accepted_state_status, "unavailable");
assert.equal(
  unavailable.recommended_investigation,
  "cold_boot_recovery_lifecycle_parity",
);

// Arrange
const matchingIdle = marker();

// Act
const matching = compareAcceptedState(matchingIdle, matchingIdle);

// Assert
assert.equal(matching.accepted_state_status, "match");
assert.equal(
  matching.recommended_investigation,
  "cold_boot_recovery_lifecycle_parity",
);

// Arrange
const otherMismatchRust = marker({ power_delta_class: "falling" });

// Act
const otherMismatch = compareAcceptedState(marker(), otherMismatchRust);

// Assert
assert.equal(
  otherMismatch.recommended_investigation,
  "upstream_init_transcript_prefix_bisection",
);

// Arrange
const progressed = marker({
  result_correlated: "true",
  submit_observed: "true",
  power_delta_class: "rising_hashing",
});

// Act
const progress = compareAcceptedState(marker(), progressed);

// Assert
assert.equal(progress.recommended_investigation, "none");
assert.equal(progress.result_correlated, true);
assert.equal(progress.fake_pool_submit_observed, true);

// Arrange / Act / Assert
assert.throws(
  () =>
    parseAcceptedStateLog(
      marker({ stage: "unknown_stage" }),
    ),
  /unknown stage/u,
);

// Arrange
const sensitiveNoise =
  "poolPassword=do-not-copy device_url=http://192.0.2.1 " + marker();

// Act
const rendered = renderAcceptedStateReport(
  compareAcceptedState(marker(), sensitiveNoise),
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

process.stdout.write("accepted_state_compare_test: passed\n");
