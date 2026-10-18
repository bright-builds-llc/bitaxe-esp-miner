// A soak whose lease interrupts local mining (ADR-0038 task 6): its own task gate, and refused
// Connects excused only before the observer started.
import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { INTERRUPT_TASK, INTERRUPT_TASK_LINE, requireSoakTask, SOAK_MAXIMUM_ACTIVE_MS, SOAK_TASK, SOAK_TASK_LINE } from "./contract.mjs";
import { judgeSoak, refusedConnect } from "./judge.mjs";
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
