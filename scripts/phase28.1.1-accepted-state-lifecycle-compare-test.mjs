#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  ACCEPTED_STATE_LIFECYCLE_STAGES,
  compareAcceptedStateLifecycle,
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
    ...ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) =>
      marker(stage, overridesByStage[stage]),
    ),
  ].join("\n");
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
