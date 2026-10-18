import assert from "node:assert/strict";
import test from "node:test";
import { requireSoakTask, SOAK_TASK, SOAK_TASK_LINE } from "./main.mjs";

const tasks = (section, line = SOAK_TASK_LINE) => `## ${section}\n\n### ${SOAK_TASK} | 2026-10-16 | Soak\n\n${line}\n\n## Other\n`;

test("an active task with the exact line enables the soak", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireSoakTask(tasks("Active")));
});

test("a future task does not enable the soak", () => {
  // Arrange / Act / Assert
  assert.throws(() => requireSoakTask(tasks("Future")), { code: "soak_task_ambiguous" });
});

test("a missing enable line refuses", () => {
  // Arrange / Act / Assert
  assert.throws(() => requireSoakTask(tasks("Active", "Ultra 205 local mining soak 002 hardware: enabled.")), { code: "soak_task_disabled" });
});
