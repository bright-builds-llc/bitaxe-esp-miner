import assert from "node:assert/strict";
import test from "node:test";
import { baselineOfJournal, readLeaseResult, requireFirmwareVariant, requireSoakBoard, requireSoakTask, SOAK_TASK, soakTask, soakTaskLine } from "./main.mjs";
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

test("the 601 soaks run on the production firmware since the promotion", () => {
  // Arrange
  const soaks = ["gamma601-first-001", "gamma601-default-001", "gamma601-soak-001", "gamma601-lease-001",
    "gamma601-boot-resume-001", "gamma601-production-001"].map((id) => SOAKS[id]);
  // Act / Assert
  for (const soak of soaks) {
    assert.doesNotThrow(() => requireFirmwareVariant({ firmware_variant: "production" }, soak), soak.id);
    assert.doesNotThrow(() => requireFirmwareVariant({}, soak), soak.id);
    assert.throws(() => requireFirmwareVariant({ firmware_variant: "gamma601-mining" }, soak), { code: "soak_firmware_variant" }, soak.id);
  }
});

test("each soak admits only its own board's package", () => {
  // Arrange
  const packages = { "205": { image_metadata: { board: "205" } }, "601": { image_metadata: { board: "601" } } };
  // Act / Assert
  for (const soak of Object.values(SOAKS)) {
    const own = soak.requireStoredBoard ?? "205", other = own === "205" ? "601" : "205";
    assert.doesNotThrow(() => requireSoakBoard(packages[own], soak), soak.id);
    assert.throws(() => requireSoakBoard(packages[other], soak), { code: "soak_package_board" }, soak.id);
    assert.throws(() => requireSoakBoard({}, soak), { code: "soak_package_board" }, soak.id);
  }
});

test("the 601 soak has its own enable line in the 601 bring-up task", () => {
  // Arrange
  const block = `## Active\n\n### task-gamma-601-bring-up | x\n\nGamma 601 first local mining hardware: enabled.\n\n## Other\n`;
  // Act / Assert
  assert.equal(soakTaskLine("gamma601-first-001"), "Gamma 601 first local mining hardware: enabled.");
  assert.doesNotThrow(() => requireSoakTask(block, "gamma601-first-001"));
  assert.throws(() => requireSoakTask(block.replace("Gamma 601 first", "Ultra 205 first"), "gamma601-first-001"), { code: "soak_task_disabled" });
});

test("each firmware variant admits only its own soaks", () => {
  // Arrange
  const variants = ["production", "overheat-trip-test", "gamma601-overheat-trip-test"];
  // Act / Assert
  for (const variant of variants) {
    for (const soak of Object.values(SOAKS)) {
      const own = (soak.firmwareVariant ?? "production") === variant;
      const admit = () => requireFirmwareVariant({ firmware_variant: variant }, soak);
      if (own) assert.doesNotThrow(admit, `${variant} ${soak.id}`);
      else assert.throws(admit, { code: "soak_firmware_variant" }, `${variant} ${soak.id}`);
    }
  }
});

test("the 601 overheat check runs only on its own test firmware", () => {
  // Arrange
  const admitted = Object.values(SOAKS).filter((soak) => soak.firmwareVariant === "gamma601-overheat-trip-test").map((soak) => soak.id);
  // Act / Assert
  assert.deepEqual(admitted, ["gamma601-overheat-001"]);
});

test("the 601 overheat check has its own enable line in the 601 bring-up task", () => {
  // Arrange
  const line = "Gamma 601 local mining overheat check hardware: enabled.";
  const block = `## Active\n\n### task-gamma-601-bring-up | x\n\n${line}\n\n## Other\n`;
  // Act / Assert
  assert.equal(soakTaskLine("gamma601-overheat-001"), line);
  assert.doesNotThrow(() => requireSoakTask(block, "gamma601-overheat-001"));
  assert.throws(() => requireSoakTask(block.replace(line, soakTaskLine("overheat-001")), "gamma601-overheat-001"), { code: "soak_task_disabled" });
});

test("the 601 lease check is enabled by the same line as the Gate's 601 interruption check", async () => {
  // Arrange
  const { INTERRUPT_TASKS } = await import("../fixed-usb-soak/contract.mjs");
  const [task, line] = INTERRUPT_TASKS["601"];
  const block = `## Active\n\n### ${task} | x\n\n${line}\n\n## Other\n`;
  // Act / Assert
  assert.equal(soakTaskLine("gamma601-lease-001"), "Gamma 601 lease interrupt check hardware: enabled.");
  assert.deepEqual([soakTask("gamma601-lease-001"), soakTaskLine("gamma601-lease-001")], [task, line]);
  assert.doesNotThrow(() => requireSoakTask(block, "gamma601-lease-001"));
  assert.throws(() => requireSoakTask(block.replace(line, soakTaskLine("lease-001")), "gamma601-lease-001"), { code: "soak_task_disabled" });
});
