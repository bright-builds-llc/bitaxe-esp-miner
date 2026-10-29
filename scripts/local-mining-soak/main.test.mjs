import assert from "node:assert/strict";
import test from "node:test";
import { baselineOfJournal, readLeaseResult, recover, restoreBaselineOfJournal, requireFirmwareVariant, requireSoakBoard, requireSoakTask, SOAK_TASK, soakTask, soakTaskLine } from "./main.mjs";
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

const journalOf = (...events) => `${events.map((event) => JSON.stringify(event)).join("\n")}\n`;
const CLEAR_BASELINE = { event: "baseline", sample: { frequencyMhz: 525, overheatMode: false, autoFanSpeed: true } };
const OVERHEAT_STOP = { event: "live_stop", code: "soak_overheat" };

test("pause restores a soak's clear baseline after the firmware's overheat trip stopped it", () => {
  // Arrange
  const text = journalOf(CLEAR_BASELINE, { event: "resumed" }, OVERHEAT_STOP);
  // Act
  const maybeBaseline = restoreBaselineOfJournal(SOAKS["gamma601-production-001"], text);
  // Assert
  assert.deepEqual(maybeBaseline, CLEAR_BASELINE.sample);
});

test("pause restores nothing for a soak that another stop ended", () => {
  // Arrange
  const text = journalOf(CLEAR_BASELINE, { event: "live_stop", code: "soak_power" });
  // Act / Assert
  assert.equal(restoreBaselineOfJournal(SOAKS["gamma601-production-001"], text), null);
});

test("pause never restores a baseline that was itself latched in overheat mode", () => {
  // Arrange
  const latched = { event: "baseline", sample: { frequencyMhz: 425, overheatMode: true, autoFanSpeed: false } };
  const text = journalOf(latched, OVERHEAT_STOP);
  // Act / Assert
  assert.equal(restoreBaselineOfJournal(SOAKS["gamma601-production-001"], text), null);
});

test("pause restores an overheat check's baseline whatever stopped it", () => {
  // Arrange
  const text = journalOf(CLEAR_BASELINE);
  // Act / Assert
  assert.deepEqual(restoreBaselineOfJournal(SOAKS["overheat-001"], text), CLEAR_BASELINE.sample);
});

test("pause restores a soak that died before journaling the trip the device still shows", () => {
  // Arrange
  const text = journalOf(CLEAR_BASELINE, { event: "resumed" });
  // Act
  const maybeBaseline = restoreBaselineOfJournal(SOAKS["gamma601-production-001"], text, true);
  // Assert
  assert.deepEqual(maybeBaseline, CLEAR_BASELINE.sample);
});

const PRODUCTION_601 = SOAKS["gamma601-production-001"];
/** A journaled 601 baseline: 525/1150, the fan automatic, the latch clear. */
const BASELINE_601 = { event: "baseline", sample: { frequencyMhz: 525, coreVoltageMv: 1150, autoFanSpeed: true, manualFanSpeed: 100,
  overheatMode: false } };

/**
 * A paused 601 an overheat trip left latched: 425/1050, the fan manual at 100%, `overheat_mode` set,
 * and with `selfPaused` the device's own overheat pause. It saves every patched setting.
 */
function trippedDevice({ board = { storedBoard: "601", boardIdentity: "match" }, selfPaused = false } = {}) {
  const clock = { now: 0 };
  const state = { paused: true, frequency: 425, coreVoltage: 1050, autofanspeed: 0, manualFanSpeed: 100, overheat_mode: 1,
    pauseReason: selfPaused ? "overheat" : null, calls: [] };
  const client = {
    async info() {
      return { ...board, miningPaused: state.paused, miningActivity: state.paused ? "paused" : "safe_blocked", startMiningOnBoot: false,
        power: 5, vrTemp: 40, frequency: state.frequency, coreVoltage: state.coreVoltage, autofanspeed: state.autofanspeed,
        manualFanSpeed: state.manualFanSpeed, overheat_mode: state.overheat_mode, pauseReason: state.pauseReason,
        powerStatus: { state: "fresh" }, vrTempStatus: { state: "fresh" },
        chipTempStatus: { state: "unavailable", reason: { kind: "unavailable", code: "asic_unpowered" } } };
    },
    async logs() { return "boot\nlocal_mining=overheat stays_stopped=false\nlocal_mining=overheat_recovered\n"; },
    async patch(settings) { state.calls.push("patch"); Object.assign(state, settings); },
    async command(name) {
      state.calls.push(name); state.paused = name === "pause";
      if (name === "resume") state.pauseReason = null;
    },
  };
  return { client, clock, state };
}

const recoverOn = (device, text) => recover({ client: device.client, now: () => device.clock.now,
  sleep: async (ms) => { device.clock.now += ms; }, note: async () => {}, soak: PRODUCTION_601, maybeJournalText: text });

test("the recovery restores a tripped soak's baseline settings and clears the latch", async () => {
  // Arrange
  const device = trippedDevice();
  // Act
  const outcome = await recoverOn(device, journalOf(BASELINE_601, OVERHEAT_STOP));
  // Assert
  assert.deepEqual([outcome.recovery_settled, outcome.settings_restored, outcome.stop_cleared], [true, true, true]);
  assert.deepEqual([device.state.frequency, device.state.coreVoltage, device.state.autofanspeed, device.state.overheat_mode],
    [525, 1150, 1, 0]);
});

test("the recovery restores no settings on another board, and records the refusal", async () => {
  // Arrange
  const device = trippedDevice({ board: { storedBoard: "205", boardIdentity: "match" } });
  // Act
  const outcome = await recoverOn(device, journalOf(BASELINE_601, OVERHEAT_STOP));
  // Assert
  assert.deepEqual([outcome.restore_failure, outcome.paused], ["soak_board_identity", true]);
  assert.ok(!device.state.calls.includes("patch"));
});

test("the overheat check's recovery still resumes once to clear the device's own pause", async () => {
  // Arrange
  const device = trippedDevice({ selfPaused: true });
  // Act
  await recover({ client: device.client, now: () => device.clock.now, sleep: async (ms) => { device.clock.now += ms; },
    note: async () => {}, soak: SOAKS["gamma601-overheat-001"], maybeJournalText: journalOf(BASELINE_601) });
  // Assert
  assert.ok(device.state.calls.includes("resume"));
  assert.equal(device.state.paused, true);
});

test("the recovery leaves a soak's self-paused overheat stop to the operator, never resuming", async () => {
  // Arrange
  const device = trippedDevice({ selfPaused: true });
  // Act
  const outcome = await recoverOn(device, journalOf(BASELINE_601, OVERHEAT_STOP));
  // Assert
  assert.equal(outcome.stop_cleared, false);
  assert.ok(!device.state.calls.includes("resume"));
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
