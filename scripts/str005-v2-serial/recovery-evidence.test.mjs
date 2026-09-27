import assert from "node:assert/strict";
import test from "node:test";
import { ledger, original, state } from "../str005-noise-serial/test-fixture.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { projectRecoveryPart, recoveryConclusion, validateRecoveryParts } from "./recovery-evidence.mjs";

const context = { scope: "share", attemptId: Buffer.alloc(16, 1).toString("base64url"),
  gate_commit: "b".repeat(40), firmware_commit: "a".repeat(40), app_elf_sha256: "c".repeat(64) };
function status(retained = true) {
  const record = channelFixture().deviceRecords.at(-1);
  Object.assign(record, { scope: "share", authorityDeadlineDeviceUs: 180001000, observationDeadlineDeviceUs: null,
    poolSessionGeneration: 2, poolTransportEpoch: 3 });
  record.events = record.events.filter(row => row.kind !== "connected").map((row, i) => ({ ...row, sequence: i + 1 }));
  return { schema: "worker-stratum-v2-status-v1", scope: "share", state: retained ? "terminal" : "idle", connection: null,
    record: retained ? record : null, observation: { bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: 3,
      observedAtUs: 15000, clockValid: true, stationIpv4: "192.168.1.2", wifiConnected: true, socket: null } };
}
function parts() {
  return { finished: { failures: [] }, ...Object.fromEntries(Object.entries({ ledger, original_budget: original, state: state(context),
    closed: state(context, "candidate", true), status: status(),
    diagnostics: { schema: "worker-diagnostic-export-v1", observations: [] } })
    .map(([stage, value]) => [stage, projectRecoveryPart(stage, value, context)])) };
}

test("accounting preserves actual ordinals and unresolved pending reservation", () => {
  // Arrange
  const observed = { ...ledger, next_ordinal: 23, last_completed_ordinal: 21, total_charged_ms: 2100000, pending: true };
  // Act
  const projected = projectRecoveryPart("ledger", observed, context);
  // Assert
  assert.deepEqual(projected, observed);
  assert.ok(recoveryConclusion({ ...parts(), ledger: projected }).blockers.includes("recovery_accounting_pending"));
});

test("status projection omits network observations and rejects a different attempt", () => {
  // Arrange
  const input = status();
  // Act
  const projected = projectRecoveryPart("status", input, context);
  // Assert
  assert.equal(JSON.stringify(projected).includes("192.168"), false);
  assert.equal(Object.hasOwn(projected, "connection"), false);
  input.record.attemptId = Buffer.alloc(16, 2).toString("base64url");
  assert.throws(() => projectRecoveryPart("status", input, context), { code: "recovery_status_attempt" });
});

test("idle after reboot does not prove retained resource release", () => {
  // Arrange
  const input = { ...parts(), status: projectRecoveryPart("status", status(false), context) };
  // Act
  const result = recoveryConclusion(input);
  // Assert
  assert.equal(result.complete, false);
  assert.equal(result.device_resources_released, false);
  assert.deepEqual(result.blockers, ["recovery_retained_record_unavailable"]);
});

test("complete evidence requires actual serial closure and retained terminal resources", () => {
  // Arrange
  const input = parts();
  // Act
  const result = recoveryConclusion(input);
  // Assert
  assert.equal(result.complete, true);
  assert.equal(result.parity_promotion, false);
  input.closed.serialOwnershipReleased = false;
  assert.deepEqual(recoveryConclusion(input).blockers, ["recovery_serial_release_unconfirmed"]);
});

test("failure latch does not discard confirmed recovery facts", () => {
  // Arrange
  const input = parts();
  input.state.failure = "start_failed";
  input.closed.failure = "start_failed";
  // Act
  const retained = projectRecoveryPart("state", input.state, context);
  // Assert
  assert.equal(retained.failure, "start_failed");
  assert.equal(recoveryConclusion(input).complete, true);
});

test("stale candidate identity is rejected before persistence", () => {
  const input = state(context);
  input.expectedFirmwareSourceCommit = "d".repeat(40);
  assert.throws(() => projectRecoveryPart("state", input, context));
});

test("partial collection reports each missing proof without inventing success", () => {
  const result = recoveryConclusion({ ledger: projectRecoveryPart("ledger", ledger, context) });
  assert.equal(result.accounting_measured, false);
  assert.deepEqual(result.blockers, ["recovery_missing_original_budget", "recovery_missing_state", "recovery_missing_closed",
    "recovery_missing_status", "recovery_missing_diagnostics", "recovery_missing_finished"]);
});

test("diagnostics omit unknown private strings and retain only bounded boot categories", () => {
  // Arrange
  const boot = { category: "boot", authoritative: false, boot_ordinal: 13, reset_reason: "panic", uptime_ms: 2000 };
  const input = { schema: "worker-diagnostic-export-v1", observations: [boot, { category: "private", endpoint: "secret" }] };
  // Act
  const projected = projectRecoveryPart("diagnostics", input, context);
  // Assert
  assert.deepEqual(projected.observations, [boot]);
  assert.equal(projected.omitted_count, 1);
  assert.equal(JSON.stringify(projected).includes("secret"), false);
  input.observations = [{ ...boot, password: "secret" }];
  assert.throws(() => projectRecoveryPart("diagnostics", input, context));
});

test("changed baseline and missing resource release remain independent blockers", () => {
  // Arrange
  const input = parts();
  input.closed.preservation.baseline_id = Buffer.alloc(16, 9).toString("base64url");
  input.status.record.resources.workerQuiescent = false;
  // Act
  const result = recoveryConclusion(input);
  // Assert
  assert.deepEqual(result.blockers, ["recovery_device_resources_unreleased", "recovery_baseline_changed"]);
});

test("missing completion and reported collection failure cannot yield a complete result", () => {
  // Arrange
  const input = parts();
  delete input.finished;
  // Act / Assert
  assert.deepEqual(recoveryConclusion(input).blockers, ["recovery_missing_finished"]);
  input.finished = { failures: ["status"] };
  assert.deepEqual(recoveryConclusion(input).blockers, ["recovery_collection_failed_status"]);
});

test("persisted projections revalidate identity, resource record and privacy", () => {
  // Arrange
  const input = parts();
  // Act / Assert
  assert.deepEqual(validateRecoveryParts(input, context), input);
  for (const mutate of [
    value => { value.state.gateCommit = "wrong"; },
    value => { value.status.record.attemptId = Buffer.alloc(16, 8).toString("base64url"); },
    value => { value.status.observation.network = "secret"; },
    value => { value.status.observation.clockValid = false; },
    value => { value.diagnostics.observations = [{ category: "private", password: "secret" }]; },
    value => { value.finished.failures = ["secret"]; },
    value => { value.finished.failures = ["state", "state"]; },
  ]) {
    const changed = structuredClone(input); mutate(changed);
    assert.throws(() => validateRecoveryParts(changed, context));
  }
});
