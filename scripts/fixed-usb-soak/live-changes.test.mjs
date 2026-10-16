import assert from "node:assert/strict";
import test from "node:test";
import { dueLiveChange, judgeLiveChanges, LIVE_SETTLE_MS, SOAK_LIVE_CHANGES } from "./live-changes.mjs";

const initial = { frequency: 485, coreVoltage: 1200, autofanspeed: 1 };
const running = (activeMs, overrides = {}) => ({ running: true, qualification: { soak: { ordinal: 1 }, revocation_reason: "none",
  work_gate_remaining_ms: 600000 - activeMs, active_ms: activeMs, ...overrides } });

/** One HTTP sample a second from `startMs`; settings and measured voltage follow the active pair. */
function journal({ startMs = 0, endMs = 600000, pairAt = () => initial, measuredFor = (pair) => pair.coreVoltage - 3 } = {}) {
  const lines = [];
  for (let t = startMs; t < endMs; t += 1000) {
    const pair = pairAt(t);
    lines.push({ event: "sample", transport: "http", hostUnixMs: t, sample: { miningActive: true, frequencyMhz: pair.frequency,
      coreVoltageMv: pair.coreVoltage, maybeCoreVoltageActualMv: measuredFor(pair), powerWatts: pair.frequency / 40, hashRateGhs: pair.frequency } });
  }
  return lines;
}
const recorded = [
  { index: 0, requested_at_unix_ms: 200000, completed_at_unix_ms: 200100 },
  { index: 1, requested_at_unix_ms: 400000, completed_at_unix_ms: 400100 },
];
const followed = (t) => (t < 200100 ? initial : t < 400100 ? SOAK_LIVE_CHANGES[0].settings : SOAK_LIVE_CHANGES[1].settings);
const judged = (overrides = {}) => judgeLiveChanges({ initial, planned: SOAK_LIVE_CHANGES, recorded, lines: journal({ pairAt: followed }),
  maybeGateOpenUntilUnixMs: 600000, ...overrides });

test("a change is not due before the work gate reaches its active time", () => {
  // Arrange / Act / Assert
  assert.equal(dueLiveChange(SOAK_LIVE_CHANGES, 0, running(199999)), undefined);
});

test("a change is due once the work gate reaches its active time", () => {
  // Arrange / Act / Assert
  assert.equal(dueLiveChange(SOAK_LIVE_CHANGES, 0, running(200000)), 0);
});

test("the second change waits for its own active time, and nothing follows the plan", () => {
  // Arrange / Act / Assert
  assert.deepEqual([dueLiveChange(SOAK_LIVE_CHANGES, 1, running(399000)), dueLiveChange(SOAK_LIVE_CHANGES, 2, running(500000))], [undefined, undefined]);
});

test("no change is sent once the soak is revoked or its work gate has closed", () => {
  // Arrange / Act / Assert
  assert.equal(dueLiveChange(SOAK_LIVE_CHANGES, 0, running(300000, { revocation_reason: "lease_or_budget_expired" })), undefined);
  assert.equal(dueLiveChange(SOAK_LIVE_CHANGES, 0, running(300000, { work_gate_remaining_ms: 0 })), undefined);
  assert.equal(dueLiveChange(SOAK_LIVE_CHANGES, 0, { ...running(300000), running: false }), undefined);
});

test("settings, measured voltage and hashrate that follow each change pass with a summary per segment", () => {
  // Arrange / Act
  const result = judged();
  // Assert
  assert.deepEqual(result.failures, []);
  assert.deepEqual(result.segments.map((segment) => [segment.target.frequency, segment.measured_core_mv.median, segment.median_hash_rate_ghs]),
    [[485, 1197, 485], [400, 1097, 400], [485, 1197, 485]]);
});

test("a fixed offset in the measured voltage does not fail the steps", () => {
  // Arrange
  const lines = journal({ pairAt: followed, measuredFor: (pair) => pair.coreVoltage - 60 });
  // Act
  const result = judged({ lines });
  // Assert
  assert.deepEqual(result.failures, []);
});

test("a hashrate that never follows the frequency is not confirmed", () => {
  // Arrange
  const lines = journal({ pairAt: followed }).map((line) => ({ ...line, sample: { ...line.sample, hashRateGhs: 485 } }));
  // Act
  const result = judged({ lines });
  // Assert
  assert.deepEqual(result.failures, ["live_change_frequency_unconfirmed"]);
});

test("missing voltage or hashrate readings fail before any step is judged", () => {
  // Arrange
  const lines = journal({ pairAt: followed }).map((line) => ({ ...line, sample: { ...line.sample, maybeCoreVoltageActualMv: null } }));
  // Act
  const result = judged({ lines });
  // Assert
  assert.deepEqual(result.failures, ["live_change_readings_missing"]);
});

test("without an open-gate time the changes are unobserved", () => {
  // Arrange / Act
  const result = judged({ maybeGateOpenUntilUnixMs: undefined });
  // Assert
  assert.deepEqual(result.failures, ["live_change_unobserved"]);
});

test("a measured voltage that never follows the change is not confirmed", () => {
  // Arrange
  const lines = journal({ pairAt: followed, measuredFor: () => 1197 });
  // Act
  const result = judged({ lines });
  // Assert
  assert.deepEqual(result.failures, ["live_change_voltage_unconfirmed"]);
});

test("saved settings that never change are unobserved", () => {
  // Arrange
  const lines = journal();
  // Act
  const result = judged({ lines });
  // Assert
  assert.ok(result.failures.includes("live_change_unobserved"));
});

test("a missing or failed change record fails before any journal reading", () => {
  // Arrange / Act
  const missing = judged({ recorded: [recorded[0]] });
  const failed = judged({ recorded: [recorded[0], { ...recorded[1], failure: "soak_device_settings_patch" }] });
  // Assert
  assert.deepEqual([missing.failures, failed.failures], [["live_change_missing"], ["live_change_failed"]]);
});

test("samples during the settle time and after the gate closes are not judged", () => {
  // Arrange: the voltage lags each change, and the shutdown lowers it after the gate closes.
  const lagging = (t) => (t < 200100 + LIVE_SETTLE_MS - 1000 ? initial : t < 400100 + LIVE_SETTLE_MS - 1000 ? SOAK_LIVE_CHANGES[0].settings : SOAK_LIVE_CHANGES[1].settings);
  const lines = journal({ endMs: 630000, pairAt: (t) => followed(t), measuredFor: (pair) => pair.coreVoltage })
    .map((line) => ({ ...line, sample: { ...line.sample, maybeCoreVoltageActualMv: line.hostUnixMs > 600000 ? 1000 : lagging(line.hostUnixMs).coreVoltage,
      hashRateGhs: line.hostUnixMs > 600000 ? 50 : lagging(line.hostUnixMs).frequency } }));
  // Act
  const result = judged({ lines });
  // Assert
  assert.deepEqual(result.failures, []);
});

test("a soak without planned changes has nothing to judge", () => {
  // Arrange / Act
  const result = judgeLiveChanges({ initial, planned: [], recorded: [], lines: [], maybeGateOpenUntilUnixMs: undefined });
  // Assert
  assert.deepEqual(result, { failures: [], segments: [] });
});
