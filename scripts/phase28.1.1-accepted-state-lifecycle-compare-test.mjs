#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  ACCEPTED_STATE_LIFECYCLE_STAGES,
  compareAcceptedStateLifecycle,
  AcceptedStateLifecycleError,
  parseAcceptedStateLifecycleMember,
  parsePlan13BootEvidenceMember,
  renderAcceptedStateLifecycle,
  unavailableAcceptedStateLifecycle,
} from "./phase28.1.1-accepted-state-lifecycle-compare.mjs";

function marker(stage, overrides = {}) {
  const values = {
    observation: "available",
    chip_count_class: "match",
    readable_responses: "1",
    error_counter_active: "false",
    domain_counter_active: "false",
    total_counter_active: "false",
    power_delta_class: "flat",
    result_correlated: "false",
    submit_observed: "false",
    redacted: "true",
    ...overrides,
  };
  return `accepted_state_snapshot stage=${stage} ${Object.entries(values)
    .map(([name, value]) => `${name}=${value}`)
    .join(" ")}`;
}

function completeLog(overridesByStage = {}) {
  const session = "c".repeat(32);
  return [
    "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
    "h4_continuous_result=listener_armed",
    bootEvidence(session, "booted"),
    bootEvidence(session, "listener_armed"),
    heartbeat(session, 0, 1_000, 1_000, false),
    heartbeat(session, 1, 120_000, 1_000, true),
    heartbeat(session, 2, 130_000, 10_000, true),
    ...ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) =>
      marker(stage, overridesByStage[stage]),
    ),
  ].join("\n");
}

function heartbeat(
  session,
  sequence,
  uptimeMs,
  cadenceMs,
  listenerArmed,
) {
  return `runtime_heartbeat session=${session} sequence=${sequence} uptime_ms=${uptimeMs} cadence_ms=${cadenceMs} listener_armed=${listenerArmed} redacted=true`;
}

function replaceHeartbeats(log, replacements) {
  return [
    ...log.split("\n").filter((line) => !line.includes("runtime_heartbeat")),
    ...replacements,
  ].join("\n");
}

function expectCode(callback, code) {
  assert.throws(callback, (error) => {
    assert.ok(error instanceof AcceptedStateLifecycleError);
    assert.equal(error.code, code);
    return true;
  });
}

function bootEvidence(session, state) {
  return `plan13_boot_evidence session=${session} state=${state} redacted=true`;
}

function expectFailure(callback, pattern) {
  assert.throws(callback, pattern);
}

{
  // Arrange
  const session = "a".repeat(32);
  const replayOnly = [
    bootEvidence(session, "booted"),
    bootEvidence(session, "listener_armed"),
    bootEvidence(session, "booted"),
    bootEvidence(session, "listener_armed"),
  ].join("\n");

  // Act
  const report = parsePlan13BootEvidenceMember(replayOnly, "cold-start", {
    requireOriginalMarkers: false,
  });

  // Assert
  assert.equal(report.bootSessionCount, 1);
  assert.equal(report.bootEvidenceStateCount, 2);
  assert.equal(report.equivalentDuplicates, true);
}

{
  // Arrange
  const session = "c".repeat(32);
  const coldHeartbeatOnly = [
    heartbeat(session, 0, 1_000, 1_000, false),
    heartbeat(session, 1, 130_000, 10_000, true),
    ...ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => marker(stage)),
  ].join("\n");

  // Act
  const report = compareAcceptedStateLifecycle(completeLog(), coldHeartbeatOnly);

  // Assert
  assert.equal(report.cold_start_listener_fallback_used, true);
  assert.equal(report.cold_start_boot_evidence_marker_count, 0);
  assert.equal(report.cold_start_heartbeat_uptime_category, "early_and_steady");
}

{
  // Arrange
  const session = "c".repeat(32);
  const mixedCold = [
    bootEvidence(session, "booted"),
    heartbeat(session, 0, 130_000, 10_000, true),
    ...ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => marker(stage)),
  ].join("\n");

  // Act
  const report = compareAcceptedStateLifecycle(completeLog(), mixedCold);

  // Assert
  assert.equal(report.cold_start_listener_fallback_used, true);
  assert.equal(report.cold_start_boot_evidence_state_count, 1);
}

{
  // Arrange
  const withoutDedicated = completeLog()
    .split("\n")
    .filter((line) => !line.includes("plan13_boot_evidence"))
    .join("\n");

  // Act / Assert
  expectCode(
    () => compareAcceptedStateLifecycle(withoutDedicated, completeLog()),
    "reinit_listener_proof_absent",
  );
}

{
  // Arrange
  const session = "c".repeat(32);
  const coldFalseListener = replaceHeartbeats(completeLog(), [
    heartbeat(session, 0, 130_000, 10_000, false),
  ])
    .split("\n")
    .filter((line) => !line.includes("state=listener_armed"))
    .join("\n");

  // Act / Assert
  expectCode(
    () => compareAcceptedStateLifecycle(completeLog(), coldFalseListener),
    "cold_start_listener_proof_absent",
  );
}

for (const [name, reinitLog, coldLog, code] of [
  [
    "reinit malformed",
    `${completeLog()}\nruntime_heartbeat broken=true`,
    completeLog(),
    "reinit_heartbeat_malformed",
  ],
  [
    "cold malformed",
    completeLog(),
    `${completeLog()}\nruntime_heartbeat broken=true`,
    "cold_start_heartbeat_malformed",
  ],
  [
    "reinit session conflict",
    `${completeLog()}\n${heartbeat("d".repeat(32), 3, 140_000, 10_000, true)}`,
    completeLog(),
    "reinit_heartbeat_session_conflict",
  ],
  [
    "cold session conflict",
    completeLog(),
    `${completeLog()}\n${heartbeat("d".repeat(32), 3, 140_000, 10_000, true)}`,
    "cold_start_heartbeat_session_conflict",
  ],
  [
    "reinit monotonicity",
    `${completeLog()}\n${heartbeat("c".repeat(32), 2, 140_000, 10_000, true)}`,
    completeLog(),
    "reinit_heartbeat_monotonicity_failed",
  ],
  [
    "cold listener regression",
    completeLog(),
    `${completeLog()}\n${heartbeat("c".repeat(32), 3, 140_000, 10_000, false)}`,
    "cold_start_heartbeat_monotonicity_failed",
  ],
  [
    "reinit cadence",
    replaceHeartbeats(completeLog(), [
      heartbeat("c".repeat(32), 0, 130_000, 1_000, true),
    ]),
    completeLog(),
    "reinit_heartbeat_cadence_invalid",
  ],
  [
    "cold cadence",
    completeLog(),
    replaceHeartbeats(completeLog(), [
      heartbeat("c".repeat(32), 0, 120_000, 10_000, true),
    ]),
    "cold_start_heartbeat_cadence_invalid",
  ],
  [
    "reinit absent",
    replaceHeartbeats(completeLog(), []),
    completeLog(),
    "reinit_heartbeat_absent",
  ],
  [
    "cold absent",
    completeLog(),
    replaceHeartbeats(completeLog(), []),
    "cold_start_heartbeat_absent",
  ],
]) {
  // Arrange / Act / Assert
  assert.ok(name.length > 0);
  expectCode(() => compareAcceptedStateLifecycle(reinitLog, coldLog), code);
}

{
  // Arrange
  const malformedCold = `${replaceHeartbeats(completeLog(), [])}\nruntime_heartbeat broken=true`;
  const absentReinit = replaceHeartbeats(completeLog(), []);

  // Act / Assert
  expectCode(
    () => compareAcceptedStateLifecycle(absentReinit, malformedCold),
    "cold_start_heartbeat_malformed",
  );
  expectCode(
    () => compareAcceptedStateLifecycle(malformedCold, malformedCold),
    "reinit_heartbeat_malformed",
  );
}

{
  // Arrange
  const first = "a".repeat(32);
  const second = "b".repeat(32);
  const multipleSessions = [
    bootEvidence(first, "booted"),
    bootEvidence(first, "listener_armed"),
    bootEvidence(second, "booted"),
  ].join("\n");

  // Act / Assert
  expectFailure(
    () =>
      parsePlan13BootEvidenceMember(multipleSessions, "cold-start", {
        requireOriginalMarkers: false,
      }),
    /multiple boot sessions/u,
  );
}

{
  // Arrange
  const session = "a".repeat(32);
  const missingListener = bootEvidence(session, "booted");

  // Act / Assert
  expectFailure(
    () =>
      parsePlan13BootEvidenceMember(missingListener, "cold-start", {
        requireOriginalMarkers: false,
      }),
    /listener proof is absent/u,
  );
}

{
  // Arrange
  const reinit = completeLog();
  const coldStart = `${completeLog()}\n${marker("post_enumerate")}`;

  // Act
  const report = compareAcceptedStateLifecycle(reinit, coldStart);

  // Assert
  assert.equal(report.lifecycle_status, "match");
  assert.equal(report.reinit_stage_count, 5);
  assert.equal(report.cold_start_stage_count, 5);
  assert.equal(report.cold_start_marker_count, 6);
  assert.equal(report.cold_start_equivalent_duplicates, true);
}

{
  // Arrange
  const reinit = completeLog();
  const coldStartReplayOnly = completeLog()
    .split("\n")
    .filter(
      (line) =>
        !line.includes("bitaxe-rust boot") &&
        !line.includes("h4_continuous_result=listener_armed"),
    )
    .join("\n");

  // Act
  const report = compareAcceptedStateLifecycle(reinit, coldStartReplayOnly);

  // Assert
  assert.equal(report.lifecycle_status, "match");
  assert.equal(report.cold_start_boot_session_count, 1);
  assert.equal(report.cold_start_boot_evidence_state_count, 2);
}

{
  // Arrange
  const conflicting = `${completeLog()}\n${marker("post_enumerate", {
    readable_responses: "2",
  })}`;

  // Act / Assert
  expectFailure(
    () => parseAcceptedStateLifecycleMember(conflicting, "cold-start"),
    /conflicting duplicate/u,
  );
}

{
  // Arrange
  const missing = ACCEPTED_STATE_LIFECYCLE_STAGES.slice(1)
    .map((stage) => marker(stage))
    .join("\n");

  // Act / Assert
  expectFailure(
    () => parseAcceptedStateLifecycleMember(missing, "cold-start"),
    /stage set is incomplete/u,
  );
}

{
  // Arrange
  const extra = `${completeLog()}\n${marker("post_unknown")}`;

  // Act / Assert
  expectFailure(
    () => parseAcceptedStateLifecycleMember(extra, "cold-start"),
    /unknown stage/u,
  );
}

{
  // Arrange
  const unavailable = completeLog({
    post_first_work: {
      observation: "unavailable",
      chip_count_class: "unavailable",
      readable_responses: "0",
    },
  });

  // Act / Assert
  expectFailure(
    () => parseAcceptedStateLifecycleMember(unavailable, "cold-start"),
    /contains unavailable observation/u,
  );
}

{
  // Arrange
  const unredacted = completeLog({ post_first_work: { redacted: "false" } });

  // Act / Assert
  expectFailure(
    () => parseAcceptedStateLifecycleMember(unredacted, "cold-start"),
    /marker is not redacted/u,
  );
}

{
  // Arrange
  const reinit = completeLog();
  const coldStart = completeLog({
    post_first_work: { power_delta_class: "rising_hashing" },
  });

  // Act
  const report = compareAcceptedStateLifecycle(reinit, coldStart);

  // Assert
  assert.equal(report.lifecycle_status, "mismatch");
  assert.equal(report.stage_post_first_work, "mismatch");
}

{
  // Arrange
  const secretNoise = [
    "poolPassword=secret-sentinel",
    "https://private.invalid/device",
    completeLog(),
  ].join("\n");

  // Act
  const rendered = renderAcceptedStateLifecycle(
    compareAcceptedStateLifecycle(completeLog(), secretNoise),
  );

  // Assert
  assert.equal(rendered.includes("secret-sentinel"), false);
  assert.equal(rendered.includes("private.invalid"), false);
  assert.match(rendered, /^lifecycle_status: match$/mu);
  assert.match(rendered, /^redacted: true$/mu);
}

{
  // Arrange / Act
  const report = unavailableAcceptedStateLifecycle();

  // Assert
  assert.equal(report.lifecycle_status, "unavailable");
  assert.equal(report.reinit_stage_count, 0);
  assert.equal(report.redacted, true);
}

process.stdout.write("accepted_state_lifecycle_compare_test: passed\n");
