#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  classifyLateAttachStreams,
  qualifyOsNativeColdStream,
  validateConnectedPreflight,
} from "./ultra205-late-attach-classifier.mjs";

const session = "0123456789abcdef0123456789abcdef";
const coldSession = "fedcba9876543210fedcba9876543210";
const acceptedStages = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

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

function coldLog(coldSessionId = coldSession) {
  return [
    `plan13_boot_evidence session=${coldSessionId} state=booted redacted=true`,
    `plan13_boot_evidence session=${coldSessionId} state=listener_armed redacted=true`,
    heartbeat(10, 130_000, coldSessionId),
    heartbeat(11, 140_000, coldSessionId),
    heartbeat(12, 150_000, coldSessionId),
    ...acceptedStages.map(
      (stage) =>
        `accepted_state_snapshot stage=${stage} observation=available redacted=true`,
    ),
  ].join("\n");
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

{
  // Arrange
  const osNative = [
    "bitaxe-rust boot",
    heartbeat(1, 130_000),
    "ordinary firmware diagnostic",
    heartbeat(2, 140_000),
    heartbeat(3, 150_000),
  ].join("\n");

  // Act
  const result = validateConnectedPreflight("", osNative);

  // Assert
  assert.equal(result.passed, true);
  assert.equal(result.espflashHeartbeatCount, 0);
  assert.equal(result.osNativeListenerArmed, true);
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

{
  // Arrange
  const cold = coldLog();

  // Act
  const result = qualifyOsNativeColdStream(cold, session);

  // Assert
  assert.equal(result.category, "native_cold_delivers");
  assert.ok(result.application_byte_count > 0);
  assert.equal(result.heartbeat_count, 3);
  assert.equal(result.listener_armed, true);
  assert.equal(result.boot_evidence_replay_complete, true);
  assert.equal(result.accepted_state_replay_complete, true);
  assert.equal(result.new_cold_session, true);
}

for (const [name, cold] of [
  ["zero-bytes", ""],
  ["same-session", coldLog(session)],
  [
    "too-few-heartbeats",
    coldLog().replace(`${heartbeat(12, 150_000, coldSession)}\n`, ""),
  ],
  [
    "regression",
    coldLog().replace("sequence=12 uptime_ms=150000", "sequence=9 uptime_ms=120000"),
  ],
  [
    "malformed-heartbeat",
    coldLog().replace(heartbeat(11, 140_000, coldSession), "runtime_heartbeat malformed"),
  ],
  [
    "missing-boot-replay",
    coldLog().replace(/plan13_boot_evidence[^\n]*state=booted[^\n]*\n/u, ""),
  ],
  [
    "missing-listener-replay",
    coldLog().replace(/plan13_boot_evidence[^\n]*state=listener_armed[^\n]*\n/u, ""),
  ],
  [
    "mixed-session-replay",
    coldLog().replace(
      `session=${coldSession} state=listener_armed`,
      `session=${session} state=listener_armed`,
    ),
  ],
  [
    "missing-accepted-state-stage",
    coldLog().replace(/accepted_state_snapshot stage=post_max_baud[^\n]*\n/u, ""),
  ],
  [
    "unknown-accepted-state-stage",
    coldLog().replace("stage=post_max_baud", "stage=unknown"),
  ],
  ["runtime-hazard", `${coldLog()}\nGuru Meditation Error: Core 0 panic'ed`],
]) {
  // Arrange / Act
  const result = qualifyOsNativeColdStream(cold, session);

  // Assert
  assert.equal(result.category, "cold_native_evidence_invalid", name);
}

console.log("ultra205_late_attach_classifier_test passed");
