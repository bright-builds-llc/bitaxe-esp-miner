#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  classifyLateAttachStreams,
  validateConnectedPreflight,
} from "./ultra205-late-attach-classifier.mjs";

const session = "0123456789abcdef0123456789abcdef";

function heartbeat(sequence, uptimeMs, sessionId = session) {
  const cadence = uptimeMs <= 120_000 ? 1_000 : 10_000;
  return `runtime_heartbeat session=${sessionId} sequence=${sequence} uptime_ms=${uptimeMs} cadence_ms=${cadence} listener_armed=true redacted=true`;
}

function classify(pattern) {
  const streams = [
    pattern[0] ? heartbeat(1, 130_000) : "",
    pattern[1] ? heartbeat(2, 140_000) : "",
    pattern[2] ? heartbeat(3, 150_000) : "",
  ];
  return classifyLateAttachStreams(...streams);
}

{
  // Arrange
  const espflash = heartbeat(1, 130_000);
  const osNative = heartbeat(2, 140_000);

  // Act
  const result = validateConnectedPreflight(espflash, osNative);

  // Assert
  assert.equal(result.passed, true);
  assert.equal(result.sameSession, true);
}

for (const [pattern, expected] of [
  ["111", "all_readers_deliver"],
  ["010", "espflash_reader_silent"],
  ["011", "os_open_activates_transport"],
  ["000", "late_attach_transport_silent"],
  ["101", "os_reader_silent"],
  ["110", "inconclusive_mixed_delivery"],
]) {
  // Arrange / Act
  const result = classify(pattern.split("").map((value) => value === "1"));

  // Assert
  assert.equal(result.category, expected, pattern);
}

{
  // Arrange
  const secondSession = "fedcba9876543210fedcba9876543210";

  // Act
  const result = classifyLateAttachStreams(
    heartbeat(1, 130_000),
    heartbeat(2, 140_000, secondSession),
    heartbeat(3, 150_000),
  );

  // Assert
  assert.equal(result.category, "inconclusive_mixed_delivery");
  assert.equal(result.same_session, false);
}

{
  // Arrange / Act
  const result = classifyLateAttachStreams(
    heartbeat(2, 140_000),
    heartbeat(1, 130_000),
    "",
  );

  // Assert
  assert.equal(result.category, "inconclusive_mixed_delivery");
  assert.equal(result.monotonic, false);
}

{
  // Arrange / Act
  const result = classifyLateAttachStreams(
    `${heartbeat(1, 130_000)}\nunexpected output`,
    heartbeat(2, 140_000),
    heartbeat(3, 150_000),
  );

  // Assert
  assert.equal(result.category, "unexpected_non_heartbeat_bytes");
}

{
  // Arrange / Act
  const result = validateConnectedPreflight(heartbeat(1, 130_000), "");

  // Assert
  assert.equal(result.passed, false);
}

console.log("ultra205_late_attach_classifier_test passed");
