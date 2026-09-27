import assert from "node:assert/strict";
import test from "node:test";
import { createRecoveryBootstrap, createRecoveryCollector, installRecoveryControls } from "./recovery-client.mjs";

function fixture(failedStage) {
  const calls = [], saved = [];
  const operation = name => async () => { calls.push(name); if (name === failedStage) throw Error("private-secret"); return { name }; };
  const gate = Object.fromEntries(["reviewQualificationAttempts", "reviewBudget", "exportDiagnostics", "stratumV2Possession",
    "stratumV2Status", "stop", "refresh", "close"].map(name => [name, operation(name)]));
  const collect = createRecoveryCollector({ gate, campaignId: "test", attemptId: "test", published: () => ({ failure: "window_control_failed" }),
    save: async (stage, value) => { saved.push({ stage, value }); } });
  return { collect, calls, saved, gate };
}
test("failure-latched main UI still exposes one-shot recovery and no work action", async () => {
  // Arrange
  const f = fixture(), nodes = [];
  const document = { createElement() { const node = { setAttribute() {}, addEventListener(_, fn) { this.click = fn; } }; nodes.push(node); return node; }, body: { append() {} } };
  installRecoveryControls(document, f.collect);
  // Act
  await nodes[0].click(); await nodes[0].click();
  // Assert
  assert.equal(f.calls.filter(name => name === "stop").length, 1);
  assert.equal(nodes[0].disabled, true);
  assert.equal(f.saved.find(row => row.stage === "state").value.failure, "window_control_failed");
  assert.deepEqual(f.calls, ["reviewQualificationAttempts", "reviewBudget", "exportDiagnostics", "stop", "refresh", "stratumV2Possession", "stratumV2Status", "close"]);
});
for (const boundary of ["reviewQualificationAttempts", "reviewBudget", "exportDiagnostics", "stratumV2Possession", "stratumV2Status", "stop", "refresh"]) {
  test(`partial failure at ${boundary} preserves independent reads and always closes`, async () => {
    // Arrange
    const f = fixture(boundary);
    // Act
    const result = await f.collect();
    // Assert
    assert.equal(result.complete, false); assert.equal(f.calls.at(-1), "close");
    assert.equal(f.calls.includes("stop"), true); assert.equal(JSON.stringify(f.saved).includes("private-secret"), false);
    await assert.rejects(f.collect(), /recovery_consumed/u);
  });
}
test("persistence failure cannot bypass restoration or close", async () => {
  const f = fixture();
  const collect = createRecoveryCollector({ gate: f.gate, published: () => ({}), save: async () => { throw Error("disk"); } });
  await assert.rejects(collect());
  assert.equal(f.calls.at(-1), "close"); assert.equal(f.calls.includes("stop"), true);
});
test("stale session rejection cannot become a retained status proof", async () => {
  const f = fixture("stratumV2Status");
  const result = await f.collect();
  assert.deepEqual(result.failures, ["status"]);
  assert.equal(f.saved.some(row => row.stage === "status"), false);
});

test("idle discovery never demands a lost historical record after reboot", async () => {
  const f = fixture(), queried = [];
  f.gate.stratumV2Status = async (_, id) => { queried.push(id); return { state: "idle", record: null }; };
  await f.collect();
  assert.deepEqual(queried, [null]);
  assert.equal(f.saved.find(row => row.stage === "status").value.record, null);
});

test("only the exact non-idle correlation failure admits one retained query", async () => {
  const f = fixture(), queried = [];
  f.gate.stratumV2Status = async (_, id) => {
    queried.push(id);
    if (id === null) throw Object.assign(Error("closed"), { category: "v2_idle_correlation" });
    return { state: "terminal" };
  };
  await f.collect();
  assert.deepEqual(queried, [null, "test"]);
});

test("Close failure still persists observed release state and keeps the failure", async () => {
  const f = fixture("close");
  const result = await f.collect();
  assert.deepEqual(result.failures, ["closed"]);
  assert.ok(f.saved.some(row => row.stage === "closed"));
});

test("bootstrap requires connected before baseline and a separate native candidate reconnect", async () => {
  // Arrange
  const config = { expectedGateCommit: "gate", expectedFirmwareSourceCommit: "firmware", expectedAppElfSha256: "elf" };
  const current = { status: "configured", connected: false, running: false, serialOwnershipReleased: true,
    deviceBaselineConfirmed: true, deviceLeaseInactive: true, gateCommit: "gate", expectedFirmwareSourceCommit: "firmware",
    expectedAppElfSha256: "elf", preservation: { settings_match: true, device_identity_match: true,
      authorization_high_water_match: true, mine_on_boot: false, baseline_id: "same" } };
  const calls = [];
  const bootstrap = createRecoveryBootstrap({ candidateConfiguration: config, published: () => current,
    gate: { async close() { calls.push("close"); Object.assign(current, { status: "closed", connected: false, serialOwnershipReleased: true }); },
      configure() { calls.push("configure"); current.status = "configured"; } }, collect: async () => { calls.push("collect"); } });
  // Act / Assert
  await assert.rejects(bootstrap.prepare(), /before_baseline/u);
  Object.assign(current, { status: "ready", connected: true, serialOwnershipReleased: false });
  await bootstrap.prepare();
  await assert.rejects(bootstrap.collect(), /reconnect_required/u);
  Object.assign(current, { status: "ready", connected: true, serialOwnershipReleased: false });
  current.preservation.baseline_id = "changed";
  await assert.rejects(bootstrap.collect(), /reconnect_required/u);
  current.preservation.baseline_id = "same";
  await bootstrap.collect();
  await assert.rejects(bootstrap.collect(), /reconnect_required/u);
  assert.deepEqual(calls, ["close", "configure", "collect"]);
});
