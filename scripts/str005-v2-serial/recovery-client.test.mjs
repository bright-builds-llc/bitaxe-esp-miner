import assert from "node:assert/strict";
import test from "node:test";
import { createRecoveryCollector, installRecoveryControls } from "./recovery-client.mjs";

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
