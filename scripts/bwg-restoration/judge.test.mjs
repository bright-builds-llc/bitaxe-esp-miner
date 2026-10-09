import test from "node:test";
import assert from "node:assert/strict";
import { SCENARIOS } from "./contract.mjs";
import { baseline, journal, pageState, passingInput, rejectionReview, stimulusReview } from "./fixtures.test-helper.mjs";
import { judgeScenario, liveSafetyFailures, preRebootEpoch, segment } from "./judge.mjs";

for (const scenario of SCENARIOS) {
  test(`a complete ${scenario} scenario passes`, () => {
    // Arrange
    const input = passingInput(scenario);
    // Act
    const judgement = judgeScenario(input);
    // Assert
    assert.deepEqual(judgement.failures, []);
    assert.equal(judgement.result, "passed");
  });
}

test("a wrong terminal reason fails the scenario", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.finalState = { ...input.finalState, device: baseline("connectivity_lost") };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("terminal_reason_mismatch"));
});

test("a restore relabelled as monotonic_reset is not monotonic evidence", () => {
  // Arrange
  const input = passingInput("monotonic_uncertainty");
  const entries = input.finalState.journal.entries.map((entry) => entry.event === "device_baseline_observed" ? { ...entry, event: "restored", category: undefined } : entry)
    .map(({ category, ...entry }) => category === undefined ? entry : { ...entry, category });
  input.finalState = pageState({ entries, device: baseline("monotonic_reset") });
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.equal(judgement.result, "unverified");
  assert.ok(judgement.failures.includes("fact_noRelabel"));
  assert.ok(judgement.failures.includes("fact_deviceEndedLease"));
});

test("a missing discontinuity counter increment fails monotonic_uncertainty", () => {
  // Arrange
  const input = passingInput("monotonic_uncertainty");
  input.reviews = { ...input.reviews, stimulus: stimulusReview("consumed", 0) };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_monotonicDetectionCounted"));
});

test("a device stop observed after the stimulus bound fails monotonic_uncertainty", () => {
  // Arrange
  const input = passingInput("monotonic_uncertainty");
  input.records = input.records.map((record) => record.operation === "statusReview" ? { ...record, receivedAtUnixMs: 3000 + 15001 } : record);
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_endedWithinBound"));
});

test("a reconnect between the stimulus and the stop fails monotonic_uncertainty", () => {
  // Arrange
  const input = passingInput("monotonic_uncertainty");
  const events = ["connected", "stimulus_reviewed:idle", "start_prepared", "lease_loaded:no_renewal", "lease_started", "stimulus_acknowledged", "disconnected",
    "connected", "device_baseline_observed:monotonic_reset", "status_reviewed:monotonic_reset", "closed"];
  input.finalState = pageState({ entries: journal(events), device: baseline("monotonic_reset") });
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_sameSession"));
});

test("N1 with the high-water advanced this boot is not durable replay evidence", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.legs.n1.review.highWater.advancedThisBoot = true;
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_durableReplayAttributed"));
});

test("N1 whose fingerprint was first seen after the reboot breaks high-water continuity", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.legs.n1.review.highWater.fingerprintFirstObservedEpoch = 4;
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_highWaterUnchangedAcrossReboot"));
});

test("a cross-context leg attributed to the replay guard fails N3", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.legs.n3.review.last.replayGuard = "at_or_below_durable_high_water";
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_crossContextAttributed"));
});

test("an accepted replay fails the negatives", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.legs.n2.replay = { operation: "start", outcome: "accepted" };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_expiredContextAttributed"));
});

test("an expiry observed before the 30 s window fails expiry", () => {
  // Arrange
  const input = passingInput("expiry");
  input.records[2] = { ...input.records[2], receivedAtUnixMs: 1000 + 27000 };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_expiredAfterWindow"));
});

test("a start operation that includes lease preparation does not shorten the expiry window", () => {
  // Arrange: attempt-001's shape, where the start record landed 8.88 s after loading and expiry was seen 27.98 s later.
  const input = passingInput("expiry");
  input.records[1] = { ...input.records[1], receivedAtUnixMs: 1000 + 8880 };
  input.records[2] = { ...input.records[2], receivedAtUnixMs: 1000 + 8880 + 27983 };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(!judgement.failures.includes("fact_expiredAfterWindow"));
});

test("an expiry without a loaded-lease record fails expiry", () => {
  // Arrange
  const input = passingInput("expiry");
  input.records.splice(0, 1);
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_expiredAfterWindow"));
});

test("a physical scenario without the restore-watcher token ordering fails", () => {
  // Arrange
  const input = passingInput("reboot");
  input.checkpoint = { ...input.checkpoint, restoreTokenBeforeRestoreInstruction: false };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_restoreTokenBeforeRestoreInstruction"));
});

test("a reboot that kept the stimulus consumed did not reboot", () => {
  // Arrange
  const input = passingInput("reboot");
  input.reviews = { ...input.reviews, stimulus: stimulusReview("consumed", 1) };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_rebootClearedStimulus"));
});

test("the reboot carries the pre-reboot epoch and its rejection count to the negatives", () => {
  // Arrange
  const input = passingInput("reboot");
  // Act
  const { carry } = judgeScenario(input);
  // Assert
  assert.deepEqual([carry.preRebootEpoch, carry.rebootRejections], [3, 0]);
  assert.equal(preRebootEpoch([]), null);
});

test("a page operation failure in the segment fails an operator scenario", () => {
  // Arrange
  const input = passingInput("pause");
  const entries = journal(["connected", "start_prepared", "lease_loaded:no_renewal", "lease_started", "renew_failed:request_failed", "paused", "closed"]);
  input.finalState = pageState({ entries, device: baseline("paused") });
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("page_operation_failed"));
});

test("a truncated journal cannot be judged", () => {
  // Arrange
  const state = pageState({ entries: journal(["closed"], 10) });
  state.journal.dropped = 5;
  // Act
  const result = segment(state, 2);
  // Assert
  assert.equal(result.truncated, true);
});

test("live safety limits reuse the soak sample rule", () => {
  // Arrange
  const safe = { voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, voltage_volts: 5, power_watts: 12,
    chip_temp_celsius: 60, fan_rpm: 4000, watchdog_alive: true };
  // Act / Assert
  assert.deepEqual(liveSafetyFailures([safe]), []);
  assert.deepEqual(liveSafetyFailures([{ ...safe, chip_temp_celsius: 75 }]), ["unsafe_sample"]);
});

test("the stimulus counter must stay unchanged before monotonic_uncertainty", () => {
  // Arrange
  const input = passingInput("cancel");
  input.reviews = { stimulus: stimulusReview("idle", 1), rejection: rejectionReview() };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_stimulusCounterConsistent"));
});

test("N1 replayed before a same-connection reboot status fails the negatives", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  const entries = input.finalState.journal.entries.filter((entry) => !(entry.event === "status_reviewed" && entry.category === "reboot"));
  input.finalState = pageState({ entries, device: baseline("control_failed") });
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_rebootReportedBeforeN1"));
});

test("N1 after the reboot scenario's carried report passes without a same-connection status", () => {
  // Arrange: attempt-003, where the reboot scenario's completion had already received the once-per-boot report.
  const input = passingInput("authorization_negatives");
  const entries = input.finalState.journal.entries.filter((entry) => !(entry.event === "status_reviewed" && entry.category === "reboot"));
  input.finalState = pageState({ entries, device: baseline("control_failed") });
  input.carry = { ...input.carry, rebootReported: true };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(!judgement.failures.includes("fact_rebootReportedBeforeN1"));
});

test("a carried reboot report does not cover a lease started before N1", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  const entries = input.finalState.journal.entries.map((entry) => entry.event === "status_reviewed" && entry.category === "reboot"
    ? { ordinal: entry.ordinal, event: "lease_started" } : entry);
  input.finalState = pageState({ entries, device: baseline("control_failed") });
  input.carry = { ...input.carry, rebootReported: true };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_rebootReportedBeforeN1"));
});

test("a passing reboot scenario carries the reboot report and a failing one does not", () => {
  // Arrange
  const passing = passingInput("reboot");
  const failing = passingInput("reboot");
  failing.finalState = { ...failing.finalState, device: baseline("connectivity_lost") };
  // Act
  const carried = [judgeScenario(passing), judgeScenario(failing)].map((judgement) => judgement.carry.rebootReported);
  // Assert
  assert.deepEqual(carried, [true, false]);
});

test("an expired-context replay must carry the device's admission_required category", () => {
  // Arrange
  const input = passingInput("authorization_negatives");
  input.legs.n2.replay = { ...input.legs.n2.replay, category: "authentication_failed" };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_expiredContextAttributed"));
});

test("a stimulus that expired instead of firing is not monotonic evidence", () => {
  // Arrange
  const input = passingInput("monotonic_uncertainty");
  input.reviews = { ...input.reviews, stimulus: stimulusReview("expired", 1) };
  // Act
  const judgement = judgeScenario(input);
  // Assert
  assert.ok(judgement.failures.includes("fact_monotonicDetectionCounted"));
});
