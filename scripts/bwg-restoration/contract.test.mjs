import test from "node:test";
import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pinnedGateCommit, requireRestorationTask, RESTORATION_TASK, RESTORATION_TASK_LINE, SCENARIO_PLANS, SCENARIOS, WINDOWS } from "./contract.mjs";
import { privateDirectory } from "./fixtures.test-helper.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);

async function repoWith(name, body) {
  const root = await privateDirectory();
  await writeFile(resolve(root, name), body);
  return root;
}

test("the task gate requires the exact enable line inside the active restoration block", async () => {
  // Arrange
  const enabled = await repoWith("TASKS.md", `## Active\n\n### ${RESTORATION_TASK} | 2026-08-29 | x\n\n${RESTORATION_TASK_LINE}\n\n## Future\n`);
  const disabled = await repoWith("TASKS.md", `## Active\n\n### ${RESTORATION_TASK} | 2026-08-29 | x\n\nBWG-007 serial restoration hardware: disabled.\n`);
  const elsewhere = await repoWith("TASKS.md", `## Active\n\n### ${RESTORATION_TASK} | x\n\n### other | x\n\n${RESTORATION_TASK_LINE}\n`);
  // Act / Assert
  await requireRestorationTask(enabled);
  await rejectsWith(requireRestorationTask(disabled), "restoration_task_disabled");
  await rejectsWith(requireRestorationTask(elsewhere), "restoration_task_disabled");
});

test("a future or duplicated restoration task is not admitted", async () => {
  // Arrange
  const future = await repoWith("TASKS.md", `## Active\n\n## Future\n\n### ${RESTORATION_TASK} | x\n\n${RESTORATION_TASK_LINE}\n`);
  const twice = await repoWith("TASKS.md", `## Active\n\n### ${RESTORATION_TASK} | x\n\n${RESTORATION_TASK_LINE}\n\n### ${RESTORATION_TASK} | y\n`);
  // Act / Assert
  await rejectsWith(requireRestorationTask(future), "restoration_task_ambiguous");
  await rejectsWith(requireRestorationTask(twice), "restoration_task_ambiguous");
});

test("the pinned Gate commit is read from the single Gate archive pin", async () => {
  // Arrange
  const commit = "f".repeat(40);
  const root = await repoWith("MODULE.bazel", `http_archive(\n    strip_prefix = "bitaxe-turnstile-system-${commit}",\n)\n`);
  // Act / Assert
  assert.equal(await pinnedGateCommit(root), commit);
});

test("the scenarios run in the fixed order with their terminal reasons and windows", () => {
  // Arrange / Act
  const terminals = SCENARIOS.map((scenario) => SCENARIO_PLANS[scenario].terminal);
  // Assert
  assert.deepEqual(SCENARIOS, ["completion", "pause", "cancel", "expiry", "monotonic_uncertainty", "disconnect", "reboot", "authorization_negatives"]);
  assert.deepEqual(terminals, ["challenge_satisfied", "paused", "cancelled", "lease_expired", "monotonic_reset", "connectivity_lost", "reboot", "connectivity_lost"]);
  assert.deepEqual(WINDOWS.expiry, { durationMilliseconds: 30000, renewAfterMilliseconds: 10000 });
  assert.equal(SCENARIO_PLANS.expiry.renewals, 0);
});
