// A soak whose lease interrupts local mining (ADR-0038 task 6): its own task gate, and refused
// Connects excused only before the observer started.
import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { INTERRUPT_TASK, INTERRUPT_TASK_LINE, INTERRUPT_TASKS, requireSoakTask, SOAK_MAXIMUM_ACTIVE_MS, SOAK_TASK, SOAK_TASK_LINE } from "./contract.mjs";
import { clockObservations, judgeSoak, refusedConnect, unsafeSample } from "./judge.mjs";
import { requireInterruptPlan } from "./preflight.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);

async function tasksWith(body) {
  const root = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "interrupt-")));
  await writeFile(resolve(root, "TASKS.md"), body);
  return root;
}

const refused = (sequence) => ({ sequence, receivedAtUnixMs: sequence,
  state: { running: false, failure: "connect_failed", admissionFailureStage: "hello" } });

/** A passing soak's terminal records after two refused Connects; the observer started after record 3. */
function interruptInput({ local = true, records = [refused(1), refused(2)] } = {}) {
  const qualification = { schema: "worker-qualification-v1", revocation_reason: "lease_or_budget_expired", active_limit_ms: SOAK_MAXIMUM_ACTIVE_MS,
    active_ms: 601000, accepted: 30, rejected: 0, work_dispatched: 500, nonce_work_correlations: 40, mine_on_boot: false,
    safe_stop_complete: true, safe_stop_stage: "fan_paused", voltage_volts: 5.0, power_watts: 12.5, chip_temp_celsius: 60, fan_rpm: 5000,
    voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true,
    soak: { ordinal: 1, maximum_active_ms: SOAK_MAXIMUM_ACTIVE_MS, reserved_ms: SOAK_MAXIMUM_ACTIVE_MS, complete: true, active_ms: 601000 } };
  const settings = { frequency: 485, coreVoltage: 1200, autofanspeed: 1 };
  return { context: { device_settings: settings, ...(local ? { local_mining_interrupt: true } : {}) },
    records: [...records, { sequence: 10, receivedAtUnixMs: 10, state: { running: false, renewalsConfirmed: 31, qualification } }],
    issuance: { ordinal: 1, ledger_before: { total_charged_ms: 0 } }, deviceSettings: { requested: settings, observed: settings },
    ledgerAfter: { schema: "worker-soak-ledger-v2", next_ordinal: 2, total_charged_ms: SOAK_MAXIMUM_ACTIVE_MS, pending: false, last_completed_ordinal: 1 },
    finalState: {}, windowJudgement: { passed: true, failures: [] }, observerClean: true, maybeObserverStartSequence: 3 };
}

test("the interruption check needs its own enable line, which never enables the standard soak", async () => {
  // Arrange
  const root = await tasksWith(`## Active\n\n### ${INTERRUPT_TASK} | 2026-10-16 | Soak\n\n${INTERRUPT_TASK_LINE}\n\n### ${SOAK_TASK} | x\n\n## Future\n`);
  // Act / Assert
  await requireSoakTask(root, true);
  await rejectsWith(requireSoakTask(root), "soak_task_disabled");
});

test("the standard soak's enable line never enables the interruption check", async () => {
  // Arrange
  const root = await tasksWith(`## Active\n\n### ${INTERRUPT_TASK} | x\n\n${SOAK_TASK_LINE}\n\n## Future\n`);
  // Act / Assert
  await rejectsWith(requireSoakTask(root, true), "soak_task_disabled");
});

test("refused Connects before the observer started are not a browser failure", () => {
  // Arrange / Act
  const result = judgeSoak(interruptInput());
  // Assert
  assert.ok(!result.failures.includes("browser_failure"), JSON.stringify(result.failures));
  assert.equal(result.refused_connect_records, 2);
});

test("a refused Connect after the observer started is a browser failure", () => {
  // Arrange
  const input = interruptInput({ records: [refused(1), refused(4)] });
  // Act
  const result = judgeSoak(input);
  // Assert
  assert.ok(result.failures.includes("browser_failure"));
});

test("another failure before the observer started is still a browser failure", () => {
  // Arrange
  const other = { sequence: 2, receivedAtUnixMs: 2, state: { running: false, failure: "connect_failed", admissionFailureStage: "manifest_identity" } };
  // Act
  const result = judgeSoak(interruptInput({ records: [refused(1), other] }));
  // Assert
  assert.ok(result.failures.includes("browser_failure"));
});

test("an interruption check without a refused Connect fails", () => {
  // Arrange / Act
  const result = judgeSoak(interruptInput({ records: [] }));
  // Assert
  assert.ok(result.failures.includes("interrupt_refusal_unobserved"));
});

test("a standard soak never excuses a refused Connect", () => {
  // Arrange / Act
  const result = judgeSoak(interruptInput({ local: false }));
  // Assert
  assert.ok(result.failures.includes("browser_failure"));
  assert.equal(result.refused_connect_records, undefined);
});

test("a refused Connect is recognized only by its exact failure and stage", () => {
  // Arrange / Act / Assert
  assert.equal(refusedConnect(refused(1), 3), true);
  assert.equal(refusedConnect(refused(1), undefined), false);
  assert.equal(refusedConnect({ sequence: 1, state: { failure: "close_failed", admissionFailureStage: "hello" } }, 3), false);
});

test("the interruption check refuses live changes and other profiles", () => {
  // Arrange
  const interrupt = { interruptLocalMining: true };
  // Act / Assert
  assert.doesNotThrow(() => requireInterruptPlan(interrupt, undefined));
  assert.throws(() => requireInterruptPlan(interrupt, "http"), { code: "soak_interrupt_plan" });
  assert.throws(() => requireInterruptPlan({ ...interrupt, profile: "catalog-maximum-frequency" }, undefined), { code: "soak_interrupt_plan" });
  assert.doesNotThrow(() => requireInterruptPlan({ profile: "catalog-maximum-frequency" }, "http"));
});

test("the 601's interruption check is enabled by its own line in the 601 bring-up task", async () => {
  // Arrange
  const [task, line] = INTERRUPT_TASKS["601"];
  const root = await tasksWith(`## Active\n\n### ${task} | x\n\n${line}\n\n### ${INTERRUPT_TASK} | x\n\n${INTERRUPT_TASK_LINE}\n\n## Future\n`);
  const only205 = await tasksWith(`## Active\n\n### ${task} | x\n\n${INTERRUPT_TASK_LINE}\n\n## Future\n`);
  // Act / Assert
  assert.equal(line, "Gamma 601 lease interrupt check hardware: enabled.");
  await requireSoakTask(root, true, "601");
  await rejectsWith(requireSoakTask(only205, true, "601"), "soak_task_disabled");
});

test("only an interrupting soak runs on the 601", async () => {
  // Arrange
  const [task, line] = INTERRUPT_TASKS["601"];
  const root = await tasksWith(`## Active\n\n### ${task} | x\n\n${line}\n\n## Future\n`);
  // Act / Assert
  await rejectsWith(requireSoakTask(root, false, "601"), "soak_board");
});

/**
 * A 601 Gate observation. Before the chip's init (preparation) and after the gate closes (safe-stop)
 * its ASIC is unpowered, so the chip temperature reads unavailable (null, not fresh).
 */
function gamma601(phase, overrides = {}) {
  const base = { schema: "worker-qualification-v1", revocation_reason: "none", work_gate_remaining_ms: 600000, active_ms: 0,
    active_limit_ms: SOAK_MAXIMUM_ACTIVE_MS, accepted: 0, rejected: 0, work_dispatched: 0, nonce_work_correlations: 0, mine_on_boot: false,
    safe_stop_complete: false, safe_stop_stage: "not_started", voltage_volts: 5.0, power_watts: 5.1, chip_temp_celsius: null, fan_rpm: 4000,
    voltage_fresh: true, power_fresh: true, temperature_fresh: false, fan_fresh: true, watchdog_alive: true,
    soak: { ordinal: 1, maximum_active_ms: SOAK_MAXIMUM_ACTIVE_MS, reserved_ms: SOAK_MAXIMUM_ACTIVE_MS, complete: false, active_ms: 0 } };
  const phases = {
    // The protocol's work gate is null until the first dispatch.
    preparation: { work_gate_remaining_ms: null },
    active: { active_ms: 120000, power_watts: 16, chip_temp_celsius: 58, temperature_fresh: true, accepted: 4, work_dispatched: 90,
      nonce_work_correlations: 6, work_gate_remaining_ms: 480000, soak: { ...base.soak, active_ms: 120000 } },
    stopping: { revocation_reason: "lease_or_budget_expired", work_gate_remaining_ms: 0, active_ms: 601000, accepted: 30, work_dispatched: 500,
      nonce_work_correlations: 40, safe_stop_stage: "disable_asic", soak: { ...base.soak, active_ms: 601000 } },
    stopped: { revocation_reason: "lease_or_budget_expired", work_gate_remaining_ms: 0, active_ms: 601000, accepted: 30, work_dispatched: 500,
      nonce_work_correlations: 40, safe_stop_complete: true, safe_stop_stage: "fan_paused", soak: { ...base.soak, active_ms: 601000, complete: true } },
  };
  return { ...base, ...phases[phase], ...overrides };
}

test("the 601 needs a fresh chip temperature only while the gate is open and the chip active", () => {
  // Arrange / Act / Assert
  assert.equal(unsafeSample(gamma601("preparation"), "601"), false);
  assert.equal(unsafeSample(gamma601("active"), "601"), false);
  assert.equal(unsafeSample(gamma601("stopping"), "601"), false);
  assert.equal(unsafeSample(gamma601("active", { chip_temp_celsius: null, temperature_fresh: false }), "601"), true);
});

test("the 205 still needs a fresh chip temperature on every running sample", () => {
  // Arrange / Act / Assert
  assert.equal(unsafeSample(gamma601("preparation")), true);
  assert.equal(unsafeSample(gamma601("stopping"), "205"), true);
});

test("the 601's power stop is 22 W and the 205's 15 W", () => {
  // Arrange
  const at = (watts) => gamma601("active", { power_watts: watts });
  // Act / Assert
  assert.deepEqual([unsafeSample(at(16), "601"), unsafeSample(at(22), "601"), unsafeSample(at(23), "601")], [false, false, true]);
  assert.deepEqual([unsafeSample(at(15), "205"), unsafeSample(at(16), "205")], [false, true]);
});

test("a 601 reported chip temperature is judged even outside the active gate", () => {
  // Arrange / Act / Assert
  assert.equal(unsafeSample(gamma601("stopping", { chip_temp_celsius: 76, temperature_fresh: true }), "601"), true);
});

/** A passing 601 interrupt soak: a refused Connect, preparation without a chip reading, mining at 16 W, then the safe-stop. */
function gamma601Input(board) {
  const records = [refused(1),
    ...[["preparation", 4], ["preparation", 5], ["active", 6], ["active", 7], ["stopping", 8]].map(([phase, sequence]) =>
      ({ sequence, receivedAtUnixMs: sequence * 1000, state: { running: true, renewalsConfirmed: 5, qualification: gamma601(phase) } })),
    { sequence: 9, receivedAtUnixMs: 9000, state: { running: false, renewalsConfirmed: 31, qualification: gamma601("stopped") } }];
  const settings = { frequency: 400, coreVoltage: 1150, autofanspeed: 1 };
  return { ...interruptInput({ records }), context: { device_settings: settings, local_mining_interrupt: true, ...(board ? { board } : {}) },
    deviceSettings: { requested: settings, observed: settings } };
}

test("a 601 soak's active time starts only after the chip's init, and its preparation is safe", () => {
  // Arrange
  const input = gamma601Input("601");
  // Act
  const result = judgeSoak(input);
  const observations = clockObservations(input.records);
  // Assert
  assert.ok(!result.failures.includes("unsafe_sample"), JSON.stringify(result.failures));
  assert.deepEqual(observations.map((value) => value.observedUnixMs), [6000, 7000]);
  assert.ok(observations.every((value) => value.activeMs > 0));
});

test("the same 601 records judged as a 205 soak are unsafe", () => {
  // Arrange / Act
  const result = judgeSoak(gamma601Input(undefined));
  // Assert
  assert.ok(result.failures.includes("unsafe_sample"));
});

test("a 601 running sample with no chip reading and an unknown work gate is unsafe", () => {
  // Arrange
  const q = { voltage_fresh: true, power_fresh: true, temperature_fresh: false, fan_fresh: true,
    voltage_volts: 5.2, power_watts: 16, chip_temp_celsius: null, fan_rpm: 4000, watchdog_alive: true,
    revocation_reason: "none", active_ms: 1000, safe_stop_stage: "not_started" };
  // Act
  const unsafe = unsafeSample(q, "601");
  // Assert
  assert.equal(unsafe, true);
});

test("a 601 preparation sample with a null work gate and no chip reading is excused", () => {
  // Arrange
  const q = { voltage_fresh: true, power_fresh: true, temperature_fresh: false, fan_fresh: true,
    voltage_volts: 5.2, power_watts: 9, chip_temp_celsius: null, fan_rpm: 4000, watchdog_alive: true,
    revocation_reason: "none", work_gate_remaining_ms: null, active_ms: 0, work_dispatched: 0,
    safe_stop_stage: "not_started" };
  // Act
  const unsafe = unsafeSample(q, "601");
  // Assert
  assert.equal(unsafe, false);
});
