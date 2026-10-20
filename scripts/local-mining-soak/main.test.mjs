import assert from "node:assert/strict";
import test from "node:test";
import { baselineOfJournal, readLeaseResult, requireFirmwareVariant, requireSoakTask, SOAK_TASK, soakTask, soakTaskLine } from "./main.mjs";
import { SOAKS } from "./judge.mjs";

const tasks = (section, line = soakTaskLine("001")) => `## ${section}\n\n### ${SOAK_TASK} | 2026-10-16 | Soak\n\n${line}\n\n## Other\n`;

test("an active task with the exact line enables the soak", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireSoakTask(tasks("Active"), "001"));
});

test("a future task does not enable the soak", () => {
  // Arrange / Act / Assert
  assert.throws(() => requireSoakTask(tasks("Future"), "001"), { code: "soak_task_ambiguous" });
});

test("a missing enable line refuses", () => {
  // Arrange / Act / Assert
  assert.throws(() => requireSoakTask(tasks("Active", soakTaskLine("002")), "001"), { code: "soak_task_disabled" });
});

test("the lease check is enabled by the same line as the Gate's interruption check", async () => {
  // Arrange
  const { INTERRUPT_TASK, INTERRUPT_TASK_LINE } = await import("../fixed-usb-soak/contract.mjs");
  // Act / Assert
  assert.equal(soakTaskLine("lease-001"), INTERRUPT_TASK_LINE);
  assert.equal(soakTask("lease-001"), INTERRUPT_TASK);
});

test("the lease result reads only a complete Gate soak result", async () => {
  // Arrange
  const { mkdtemp, writeFile } = await import("node:fs/promises");
  const { tmpdir } = await import("node:os");
  const { resolve } = await import("node:path");
  const root = await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "lease-"));
  // Act / Assert
  assert.equal(await readLeaseResult(root), null);
  await writeFile(resolve(root, "result.json"), '{"result":{"result":"pa');
  assert.equal(await readLeaseResult(root), null);
  await writeFile(resolve(root, "result.json"), JSON.stringify({ result: { result: "passed", failures: ["a_b", "Not A Category"] }, sha256: "x" }));
  assert.deepEqual(await readLeaseResult(root), { result: "passed", failures: ["a_b"] });
});

test("a soak with its own task is enabled only in that task", () => {
  // Arrange
  const own = `## Active\n\n### ${soakTask("handover-001")} | x\n\n${soakTaskLine("handover-001")}\n\n## Other\n`;
  const shared = tasks("Active", soakTaskLine("handover-001"));
  // Act / Assert
  assert.doesNotThrow(() => requireSoakTask(own, "handover-001"));
  assert.throws(() => requireSoakTask(shared, "handover-001"), { code: "soak_task_ambiguous" });
});

test("a production soak runs on a production package", () => {
  // Act / Assert
  assert.doesNotThrow(() => requireFirmwareVariant({ firmware_variant: "production" }, SOAKS["006"]));
});

test("a manifest without a variant is production", () => {
  // Act / Assert
  assert.doesNotThrow(() => requireFirmwareVariant({}, SOAKS["006"]));
});

test("a production soak refuses the test-only firmware", () => {
  // Act / Assert
  assert.throws(() => requireFirmwareVariant({ firmware_variant: "overheat-trip-test" }, SOAKS["006"]), { code: "soak_firmware_variant" });
});

test("the overheat check refuses production firmware", () => {
  // Act / Assert
  assert.throws(() => requireFirmwareVariant({ firmware_variant: "production" }, SOAKS["overheat-001"]), { code: "soak_firmware_variant" });
});

test("a torn last journal line still yields the baseline", () => {
  // Arrange
  const text = `${JSON.stringify({ event: "baseline", sample: { frequencyMhz: 485 } })}\n{"event":"sam`;
  // Act
  const baseline = baselineOfJournal(text);
  // Assert
  assert.deepEqual(baseline, { frequencyMhz: 485 });
});

test("a journal without a baseline yields none", () => {
  // Act / Assert
  assert.equal(baselineOfJournal(`${JSON.stringify({ event: "pool_saved" })}\n`), null);
});
