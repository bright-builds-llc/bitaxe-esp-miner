#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  classifyColdUartStream,
  validateUartConnectedPreflight,
} from "./ultra205-uart-capture-classifier.mjs";

const preflightSession = "0123456789abcdef0123456789abcdef";
const coldSession = "fedcba9876543210fedcba9876543210";
const stages = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function heartbeat(session, sequence, uptime) {
  const cadence = uptime <= 120_000 ? 1_000 : 10_000;
  return `runtime_heartbeat session=${session} sequence=${sequence} uptime_ms=${uptime} cadence_ms=${cadence} listener_armed=true redacted=true`;
}

function coldLog(session = coldSession) {
  return [
    "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
    "h4_continuous_result=listener_armed",
    `plan13_boot_evidence session=${session} state=booted redacted=true`,
    `plan13_boot_evidence session=${session} state=listener_armed redacted=true`,
    heartbeat(session, 1, 1_000),
    heartbeat(session, 2, 120_000),
    heartbeat(session, 3, 130_000),
    ...stages.map(
      (stage) =>
        `accepted_state_snapshot stage=${stage} observation=available redacted=true`,
    ),
  ].join("\n");
}

{
  // Arrange
  const native = heartbeat(preflightSession, 10, 130_000);
  const uart = heartbeat(preflightSession, 11, 140_000);

  // Act
  const result = validateUartConnectedPreflight(native, uart);

  // Assert
  assert.equal(result.passed, true);
  assert.equal(result.same_session, true);
  assert.equal(result.listener_ready, true);
}

{
  // Arrange / Act
  const result = validateUartConnectedPreflight(
    heartbeat(preflightSession, 10, 130_000),
    heartbeat(coldSession, 11, 140_000),
  );

  // Assert
  assert.equal(result.passed, false);
  assert.equal(result.same_session, false);
}

{
  // Arrange / Act
  const result = classifyColdUartStream(coldLog(), preflightSession);

  // Assert
  assert.equal(result.classification_category, "uart_cold_delivers");
  assert.equal(result.heartbeat_count, 3);
  assert.equal(result.accepted_state_stages_complete, true);
  assert.equal(result.new_session, true);
}

for (const [name, text] of [
  ["same-session", coldLog(preflightSession)],
  ["missing-boot", coldLog().replace(/bitaxe-rust boot[^\n]*\n/u, "")],
  ["missing-listener", coldLog().replace("h4_continuous_result=listener_armed\n", "")],
  ["missing-stage", coldLog().replace(/accepted_state_snapshot stage=post_max_baud[^\n]*\n/u, "")],
  ["malformed-evidence", coldLog().replace("state=booted redacted=true", "state=booted")],
  ["heartbeat-regression", coldLog().replace("sequence=3 uptime_ms=130000", "sequence=1 uptime_ms=900")],
  ["runtime-hazard", `${coldLog()}\nGuru Meditation Error: Core 0 panic'ed`],
]) {
  // Arrange / Act
  const result = classifyColdUartStream(text, preflightSession);

  // Assert
  assert.equal(result.classification_category, "cold_uart_evidence_invalid", name);
}

console.log("ultra205_uart_capture_classifier_test passed");
