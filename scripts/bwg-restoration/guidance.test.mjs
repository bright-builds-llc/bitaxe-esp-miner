import test from "node:test";
import assert from "node:assert/strict";
import { createCampaign, finishScenario } from "./campaign.mjs";
import { SCENARIOS } from "./contract.mjs";
import { supervisorState } from "./guidance.mjs";

const passed = (scenario) => ({ scenario, result: "passed", failures: [], facts: {}, carry: {} });

/** A campaign at `scenario` whose physical checkpoint is ready for the reconnect. */
function reconnectReady(scenario) {
  const campaign = createCampaign();
  for (const name of SCENARIOS.slice(0, SCENARIOS.indexOf(scenario))) finishScenario(campaign, passed(name));
  campaign.scenario.maybeCheckpoint.checkpoint = "reconnect_ready";
  return campaign;
}

test("the reboot reconnect reviews the boot between the status and the completion", () => {
  // Arrange
  const campaign = reconnectReady("reboot");
  // Act
  const state = supervisorState(campaign, 0);
  // Assert
  assert.deepEqual([state.checkpoint, state.local_action], ["reconnect_ready", "connect (Connect click), statusReview, bootReview, submitCompletion."]);
});

test("the disconnect reconnect needs no boot review", () => {
  // Arrange
  const campaign = reconnectReady("disconnect");
  // Act
  const state = supervisorState(campaign, 0);
  // Assert
  assert.equal(state.local_action, "connect (Connect click), statusReview, submitCompletion.");
});
