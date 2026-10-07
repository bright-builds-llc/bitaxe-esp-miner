import test from "node:test";
import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { digest } from "../fixed-usb-qualification/contract.mjs";
import { signSoak } from "./authority.mjs";
import { requireIdleSoakLedger, requireSoakTask, soakAllowance, SOAK_MAXIMUM_ACTIVE_MS, SOAK_RENEWALS, SOAK_TASK, SOAK_TASK_LINE } from "./contract.mjs";
import { clockObservations, judgeSoak, unsafeSample } from "./judge.mjs";
import { createSoakObserver, idleProof } from "./observer.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);
const throwsWith = (operation, code) => assert.throws(operation, (error) => error.code === code);

async function privateDirectory() {
  return realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "soak-")));
}

async function tasksWith(body) {
  const root = await privateDirectory();
  await writeFile(resolve(root, "TASKS.md"), body);
  return root;
}

test("the soak task gate requires the exact enable line in the active soak block", async () => {
  // Arrange
  const enabled = await tasksWith(`## Active\n\n### ${SOAK_TASK} | 2026-10-06 | Soak\n\n${SOAK_TASK_LINE}\n\n## Future\n`);
  const disabled = await tasksWith(`## Active\n\n### ${SOAK_TASK} | 2026-10-06 | Soak\n\n## Future\n`);
  const elsewhere = await tasksWith(`## Active\n\n### ${SOAK_TASK} | x\n\n### other | x\n\n${SOAK_TASK_LINE}\n`);
  // Act / Assert
  await requireSoakTask(enabled);
  await rejectsWith(requireSoakTask(disabled), "soak_task_disabled");
  await rejectsWith(requireSoakTask(elsewhere), "soak_task_disabled");
});

test("a future soak task is not active", async () => {
  // Arrange
  const future = await tasksWith(`## Active\n\n## Future\n\n### ${SOAK_TASK} | x\n\n${SOAK_TASK_LINE}\n`);
  // Act / Assert
  await rejectsWith(requireSoakTask(future), "soak_task_ambiguous");
});

test("only an idle, consistent soak ledger yields the ordinal to sign", () => {
  // Arrange
  const idle = { schema: "worker-soak-ledger-v1", next_ordinal: 2, total_charged_ms: SOAK_MAXIMUM_ACTIVE_MS, pending: false, last_completed_ordinal: 1 };
  // Act / Assert
  assert.equal(requireIdleSoakLedger(idle), 2);
  throwsWith(() => requireIdleSoakLedger({ ...idle, pending: true, last_completed_ordinal: 0 }), "soak_ledger_not_idle");
  throwsWith(() => requireIdleSoakLedger({ ...idle, total_charged_ms: 600000 }), "soak_ledger_not_idle");
});

test("a soak allowance carries the exact budget and a fresh id", () => {
  // Arrange / Act
  const [first, second] = [soakAllowance(1), soakAllowance(1)];
  // Assert
  assert.equal(first.maximumActiveMilliseconds, 619050);
  assert.notEqual(first.id, second.id);
});

test("signing produces one upstream-default soak grant and 34 renewals at 60/20 s", async () => {
  // Arrange
  const operations = [];
  const sign = async (operation, input) => { operations.push(operation); return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: `signed-${input.request.leaseId}` }; };
  // Act
  const artifacts = await signSoak({ allowance: soakAllowance(3), challengeId: "challenge_x", binding: "B".repeat(43), stratum: { endpoint: "e", username: "u", password: "p" }, sign });
  // Assert
  assert.equal(artifacts.renewals.length, SOAK_RENEWALS);
  assert.deepEqual([artifacts.grant.hardwareProfile, artifacts.grant.soakAllowance.ordinal, artifacts.grant.renewAfterMilliseconds], ["upstream-default", 3, 20000]);
  assert.ok(artifacts.renewals.every((renewal) => renewal.leaseId === artifacts.grant.leaseId && renewal.durationMilliseconds === 60000));
  assert.equal(operations.filter((operation) => operation === "start").length, 1);
});

const idleLines = (overrides = []) => [
  { event: "connected", transport: "websocket", detail: { reconnect: false } },
  ...Array.from({ length: 20 }, () => ({ event: "sample", transport: "http", sample: { miningActive: false } })),
  ...Array.from({ length: 10 }, () => ({ event: "sample", transport: "websocket", sample: { miningActive: false } })),
  ...overrides,
];

test("the idle proof passes after 60 s of unbroken idle observation", () => {
  // Arrange / Act
  const proof = idleProof(idleLines(), 0, 60000);
  // Assert
  assert.equal(proof.passed, true);
});

test("a reconnect, a failure, mining or too little time fails the idle proof", () => {
  // Arrange
  const cases = [idleProof(idleLines([{ event: "connected", detail: { reconnect: true } }]), 0, 60000),
    idleProof(idleLines([{ event: "http_failed" }]), 0, 60000),
    idleProof(idleLines([{ event: "sample", transport: "http", sample: { miningActive: true } }]), 0, 60000),
    idleProof(idleLines(), 0, 59999)];
  // Act / Assert
  assert.deepEqual(cases.map((proof) => proof.passed), [false, false, false, false]);
});

test("a real observer process receives the handoff, journals privately and stops on request", async () => {
  // Arrange
  const root = await privateDirectory(), bin = resolve(root, "bin");
  await mkdir(bin, { mode: 0o700 });
  const helper = fileURLToPath(new URL("./fake-observer.test-helper.mjs", import.meta.url));
  const node = process.env.JS_BINARY__NODE_BINARY ?? process.execPath;
  const wrapper = resolve(bin, "soak_observer");
  await writeFile(wrapper, `#!/bin/sh\nexec '${node}' '${helper}'\n`, { mode: 0o700 });
  const observer = createSoakObserver(root, { soak_observer: { path: wrapper, sha256: digest(await readFile(wrapper)) } });
  // Act
  await observer.start({ ipv4: "192.168.1.20", httpPort: 80 });
  await new Promise((done) => setTimeout(done, 500));
  const result = await observer.stop();
  // Assert
  assert.equal(result.stopped_on_request, true);
  const journal = await readFile(resolve(root, "soak-observer.jsonl"), "utf8");
  assert.ok(!journal.includes("192.168.1.20"));
  assert.match(journal, /"event":"stopped"/u);
});

test("the observer refuses a public endpoint", async () => {
  // Arrange
  const observer = createSoakObserver(await privateDirectory(), { soak_observer: { path: "/nonexistent", sha256: "0".repeat(64) } });
  // Act / Assert
  await rejectsWith(observer.start({ ipv4: "8.8.8.8", httpPort: 80 }), "observer_endpoint");
});

function qualification(overrides = {}) {
  return { schema: "worker-qualification-v1", revocation_reason: "none", active_limit_ms: SOAK_MAXIMUM_ACTIVE_MS, budget_reserved_ms: 240000,
    active_ms: 100000, accepted: 30, rejected: 0, work_dispatched: 500, nonce_work_correlations: 40, mine_on_boot: false,
    safe_stop_complete: false, safe_stop_stage: "not_started", voltage_volts: 5.0, power_watts: 12.5, chip_temp_celsius: 60, fan_rpm: 5000,
    voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true,
    soak: { schema: "worker-soak-observation-v1", ordinal: 1, maximum_active_ms: SOAK_MAXIMUM_ACTIVE_MS, reserved_ms: SOAK_MAXIMUM_ACTIVE_MS, complete: false, active_ms: 100000 },
    ...overrides };
}

function soakInput(overrides = {}) {
  const records = [
    { receivedAtUnixMs: 1, state: { running: true, renewalsConfirmed: 5, qualification: qualification() } },
    { receivedAtUnixMs: 2, state: { running: false, renewalsConfirmed: 31, qualification: qualification({ revocation_reason: "lease_or_budget_expired",
      active_ms: 601000, safe_stop_complete: true, safe_stop_stage: "fan_paused", soak: { ...qualification().soak, complete: true, active_ms: 601000 } }) } },
  ];
  return { context: {}, records, issuance: { ordinal: 1, ledger_before: { total_charged_ms: 0 } },
    ledgerAfter: { schema: "worker-soak-ledger-v1", next_ordinal: 2, total_charged_ms: SOAK_MAXIMUM_ACTIVE_MS, pending: false, last_completed_ordinal: 1 },
    finalState: {}, windowJudgement: { passed: true, failures: [] }, observerClean: true, ...overrides };
}

test("the terminal judgement fails closed on each missing criterion", () => {
  // Arrange
  const base = soakInput();
  const cases = {
    soak_not_ended_by_budget: soakInput({ records: [base.records[0]] }),
    soak_ledger_not_charged_once: soakInput({ ledgerAfter: { ...base.ledgerAfter, total_charged_ms: 600000 } }),
    window_websocket_gap_exceeded: soakInput({ windowJudgement: { passed: false, failures: ["websocket_gap_exceeded"] } }),
    window_judge_unavailable: soakInput({ windowJudgement: null }),
    observer_unclean: soakInput({ observerClean: false }),
  };
  // Act / Assert
  for (const [category, input] of Object.entries(cases)) {
    const result = judgeSoak(input);
    assert.equal(result.result, "unverified");
    assert.ok(result.failures.includes(category), category);
  }
});

test("an unreleased final state is never a passed soak", () => {
  // Arrange / Act
  const result = judgeSoak(soakInput());
  // Assert
  assert.equal(result.cleanup_confirmed, false);
  assert.ok(result.failures.includes("cleanup_unconfirmed"));
});

test("fresh readings outside the live stop limits are unsafe", () => {
  // Arrange / Act / Assert
  assert.equal(unsafeSample(qualification()), false);
  assert.equal(unsafeSample(qualification({ power_watts: 15.5 })), true);
  assert.equal(unsafeSample(qualification({ chip_temp_celsius: 75 })), true);
  assert.equal(unsafeSample(qualification({ voltage_volts: 4.4 })), true);
  assert.equal(unsafeSample(qualification({ fan_fresh: false })), true);
});

test("clock observations stop at the halt, before the device freezes active time", () => {
  // Arrange
  const open = { receivedAtUnixMs: 1000, state: { running: true, qualification: qualification({ work_gate_remaining_ms: 400000 }) } };
  const halted = { receivedAtUnixMs: 9000, state: { running: true, qualification: qualification({ active_ms: 600010, work_gate_remaining_ms: 0, revocation_reason: "lease_or_budget_expired", safe_stop_stage: "cooling_proof" }) } };
  // Act
  const observations = clockObservations([open, halted]);
  // Assert
  assert.deepEqual(observations, [{ observedUnixMs: 1000, activeMs: 100000 }]);
});
