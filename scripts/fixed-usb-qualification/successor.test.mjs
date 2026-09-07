import test from "node:test";
import { once } from "node:events";
import { createSupervisor } from "./server.mjs";
import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { digest, writeNew } from "./contract.mjs";
import { createSuccessor, loadSuccessor, validateBudgetReview, validateCoolingReview } from "./successor.mjs";
import { judgeWindow } from "./judge.mjs";
import { selectedWindow } from "./store.mjs";
import { signWindow } from "./authority.mjs";

const report = { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 1,
  completed_mask: 1, charged_ms: 180000, pending: false };
const baseline = Buffer.alloc(16, 1).toString("base64url");
async function fixture(t) {
  const base = await mkdtemp(resolve(tmpdir(), "successor-test-"));
  await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, "attempt-new"), predecessor = resolve(base, "attempt-old");
  await mkdir(root, { mode: 0o700 }); await mkdir(predecessor, { mode: 0o700 });
  const context = { required_no_mining_cycles: 4, window_limits_ms: [180000, 30000, 30000], firmware_root: root, gate_root: root, campaign_id: Buffer.alloc(16, 9).toString("base64url"), firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64) };
  const old = { ...context, firmware_commit: "d".repeat(40) };
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await writeNew(resolve(predecessor, "context.json"), { context: old, sha256: digest(JSON.stringify(old)) });
  await writeNew(resolve(predecessor, "window-0.consumed.json"), { window: 0, delivery_attempted: true });
  await writeNew(resolve(predecessor, "window-0.first-failure.json"), { schema: "fixed-usb-live-failure-review-v1",
    window: 0, first_failure: "start_failed", active_milliseconds: "unverified", mining_retry: false });
  const state = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "ready", connected: true, running: false, heartbeatSuppressed: false,
    renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: false,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: baseline, settings_match: true,
      authorization_high_water_match: false, device_identity_match: true, mine_on_boot: false } };
  for (let cycle = 1; cycle <= 4; cycle += 1) await writeNew(resolve(root, `cycle-${cycle}.json`), {
    schema: "fixed-usb-cycle-report-v1", cycle, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
    baseline_id: baseline, browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
    device_identity_match: true, settings_match: true, authorization_high_water_match: true,
    probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false });
  const review = resolve(root, "budget-review-fixture.json");
  await writeNew(review, { schema: "fixed-usb-budget-review-v1", context_sha256: digest(JSON.stringify(context)), report, state });
  await mkdir(resolve(root, "firmware/bitaxe/bwg"), { recursive: true });
  await writeNew(resolve(root, "firmware/bitaxe/bwg/deployment-trust.json"), {});
  return { root, predecessor, context, review, state };
}

test("successor consumes no reservation and keeps original failure immutable", async (t) => {
  // Arrange
  const f = await fixture(t);
  const original = await readFile(resolve(f.predecessor, "window-0.first-failure.json"));
  // Act
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  // Assert
  assert.equal(await selectedWindow(f.root), 1);
  assert.deepEqual(await readFile(resolve(f.predecessor, "window-0.first-failure.json")), original);
  await assert.rejects(readFile(resolve(f.root, "window-0.result.json")), { code: "ENOENT" });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling));
});
test("a synthetic passing predecessor result cannot authorize successor", async (t) => {
  const f = await fixture(t);
  await writeNew(resolve(f.predecessor, "window-0.result.json"), { browser_report_accepted: true });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), /private_path_exists/u);
});
test("changed predecessor evidence invalidates existing successor", async (t) => {
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  await writeFile(resolve(f.predecessor, "window-0.consumed.json"), '{"window":0,"delivery_attempted":true,"changed":true}');
  await assert.rejects(loadSuccessor(f.root, f.context), /evidence_changed/u);
});
test("different campaign cannot reuse consumed window", async (t) => {
  const f = await fixture(t);
  await assert.rejects(createSuccessor(f.root, { ...f.context, campaign_id: Buffer.alloc(16, 2).toString("base64url") }, f.predecessor, f.review, f.cooling), /campaign_identity/u);
});
test("missing current-image cycle keeps successor blocked", async (t) => {
  const f = await fixture(t);
  await rm(resolve(f.root, "cycle-4.json"));
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), { code: "ENOENT" });
});
test("pending, refunded, mismatched and already consumed remaining windows reject", () => {
  for (const mutation of [{ pending: true }, { charged_ms: 0 }, { campaign_match: false }, { reserved_mask: 3 },
    { completed_mask: 0 }, { charged_ms: 180001 }, { reserved_mask: 9 }]) {
    assert.throws(() => validateBudgetReview({ ...report, ...mutation }), /successor_budget_state/u);
  }
  assert.deepEqual(validateBudgetReview(report), report);
  const second = { ...report, reserved_mask: 3, completed_mask: 3, charged_ms: 210000 };
  assert.deepEqual(validateBudgetReview(second, 2), second);
});
test("successor renewal happens before short-window shutdown headroom without extending reservation", async () => {
  const input = { campaignId: baseline, challengeId: "challenge_fixture", binding: Buffer.alloc(32, 1).toString("base64url"),
    stratum: { endpoint: "stratum+tcp://fixture.invalid:1234/", username: "fixture", password: "fixture" }, successor: true,
    sign: async (operation) => ({ profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }) };
  for (const index of [1, 2]) {
    const value = await signWindow({ ...input, index });
    assert.equal(value.grant.acceptanceCampaign.maximumActiveMilliseconds, 30000);
    assert.equal(value.grant.renewAfterMilliseconds, 5000);
    assert(5000 + 6000 < 30000 - 15550);
    assert(value.renewals.every((renewal) => renewal.renewAfterMilliseconds === 5000 && !Object.hasOwn(renewal, "acceptanceCampaign")));
  }
  await assert.rejects(signWindow({ ...input, index: 0 }), /successor_signing_window/u);
});

test("forged remaining-window result cannot skip directly to heartbeat run", async (t) => {
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  await writeNew(resolve(f.root, "window-1.result.json"), { browser_report_accepted: true });
  await assert.rejects(selectedWindow(f.root));
});
test("fresh private review binds issuance to the current possession and is consumed once", async (t) => {
  // Arrange
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  let clock = 1000, signs = 0;
  const server = await createSupervisor({ privateRoot: f.root, context: f.context }, {
    verifyFrozen: async () => ({}), now: () => clock,
    readPool: async () => ({ endpoint: "stratum+tcp://fixture.invalid:1234/", username: "fixture", password: "fixture" }),
    sign: async (operation) => { signs += 1; return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }; },
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  t.after(() => new Promise((resolveDone) => { server.close(resolveDone); server.closeAllConnections(); }));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (route, body) => fetch(origin + route, { method: "POST", headers: { Origin: origin, "Content-Type": "application/json" }, body: JSON.stringify(body) });
  const binding = Buffer.alloc(32, 4).toString("base64url");
  let lastNonce;
  const review = async () => {
    const challenge = await (await post("/budget-review-context", {})).json();
    lastNonce = challenge.nonce;
    const response = await post("/budget-review", { nonce: challenge.nonce, report, controlSessionBindingSha256: binding, state: f.state });
    if (response.ok) {
      const receipt = await response.clone().json();
      const saved = await readFile(resolve(f.root, receipt.review_file), "utf8");
      assert(!saved.includes(binding));
      assert(!saved.includes(f.context.campaign_id));
    }
    return response;
  };
  await post("/activate", {});
  // Act / Assert: no review and wrong connection are closed failures.
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal((await review()).status, 200);
  assert.equal((await post("/budget-review", { nonce: lastNonce, report, controlSessionBindingSha256: binding, state: f.state })).status, 400);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: Buffer.alloc(32, 5).toString("base64url") })).status, 400);
  assert.equal(signs, 0);
  assert.equal((await review()).status, 200);
  await post("/record", { state: { ...f.state, connected: false, serialOwnershipReleased: true } });
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal((await review()).status, 200);
  clock += 45000;
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal(signs, 0);
  assert.equal((await review()).status, 200);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 200);
  assert(signs > 0);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
});

async function lastWindowFixture(t) {
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  await writeNew(resolve(f.root, "window-1.issued.json"), { window: 1, maximum_active_ms: 30000 });
  await writeNew(resolve(f.root, "window-1.consumed.json"), { window: 1, delivery_attempted: true });
  await writeNew(resolve(f.root, "window-1.first-failure.json"), { schema: "fixed-usb-live-failure-review-v1",
    window: 1, first_failure: "start_failed", active_milliseconds: "unverified", foreground_fault_triggered: false, running_observed: false });
  await writeNew(resolve(f.root, "window-1.recovery.json"), { schema: "fixed-usb-recovery-review-v1", window: 1,
    context_sha256: digest(JSON.stringify(f.context)), restoration_confirmed: true, lease_inactive: true, mine_on_boot: false,
    serial_ownership_released: true, device_identity_match: true, settings_match: true, active_milliseconds: 0, work_dispatched: 0,
    hardware_preparation_started: false, readiness_mask: 55, first_device_failure: "readiness", fan_rpm: 0 });
  const remaining = { ...report, reserved_mask: 3, completed_mask: 3, charged_ms: 210000 };
  await writeNew(resolve(f.root, "window-1.budget-review.json"), { schema: "fixed-usb-budget-review-v1",
    context_sha256: digest(JSON.stringify(f.context)), report: remaining, state: f.state });
  const root = resolve(f.root, "../attempt-last");
  await mkdir(root, { mode: 0o700 });
  const context = { ...f.context, firmware_commit: "e".repeat(40) };
  const state = { ...f.state, expectedFirmwareSourceCommit: context.firmware_commit };
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  for (let index = 1; index <= 4; index += 1) {
    const cycle = JSON.parse(await readFile(resolve(f.root, `cycle-${index}.json`), "utf8"));
    await writeNew(resolve(root, `cycle-${index}.json`), { ...cycle, firmware_commit: context.firmware_commit });
  }
  const review = resolve(root, "budget-review-last.json");
  await writeNew(review, { schema: "fixed-usb-budget-review-v1", context_sha256: digest(JSON.stringify(context)), report: remaining, state });
  const cooling = resolve(root, "cooling-review-fixture.json");
  await writeNew(cooling, { schema: "fixed-usb-cooling-review-v1", context_sha256: digest(JSON.stringify(context)),
    ...coolingReview(state) });
  return { root, context, review, cooling, predecessor: f.root, original: f.predecessor };
}

test("last-window successor retains both consumed failures and selects only original window two", async (t) => {
  // Arrange
  const f = await lastWindowFixture(t);
  const before = await readFile(resolve(f.predecessor, "successor.json"));
  // Act
  const result = await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  // Assert
  assert.deepEqual(result.remaining_windows, [2]);
  assert.equal(await selectedWindow(f.root), 2);
  assert.equal((await loadSuccessor(f.root, f.context)).original_foreground_loss_window, "consumed_unverified");
  assert.deepEqual(await readFile(resolve(f.predecessor, "successor.json")), before);
  for (const index of [0, 1]) await assert.rejects(readFile(resolve(f.root, `window-${index}.result.json`)), { code: "ENOENT" });
});
test("last-window successor rejects transitive original evidence tampering", async (t) => {
  const f = await lastWindowFixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  await writeFile(resolve(f.original, "window-0.consumed.json"), '{"window":0,"delivery_attempted":true,"changed":true}');
  await assert.rejects(loadSuccessor(f.root, f.context), /evidence_changed/u);
});
test("last-window successor requires acknowledged recovery and completed original reservation", async (t) => {
  const f = await lastWindowFixture(t);
  const path = resolve(f.predecessor, "window-1.recovery.json");
  const recovery = JSON.parse(await readFile(path, "utf8"));
  await writeFile(path, JSON.stringify({ ...recovery, restoration_confirmed: false }));
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), /recovery_evidence/u);
  await writeFile(path, JSON.stringify(recovery));
  const budgetPath = resolve(f.predecessor, "window-1.budget-review.json");
  const budget = JSON.parse(await readFile(budgetPath, "utf8"));
  await writeFile(budgetPath, JSON.stringify({ ...budget, report: { ...budget.report, pending: true } }));
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), /budget_state/u);
});
test("last-window successor cannot admit a forged window-one pass or reuse its reservation", async (t) => {
  const f = await lastWindowFixture(t);
  await writeNew(resolve(f.predecessor, "window-1.result.json"), { browser_report_accepted: true });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), /private_path_exists/u);
  await rm(resolve(f.predecessor, "window-1.result.json"));
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  await writeNew(resolve(f.root, "window-1.issued.json"), { window: 1 });
  await assert.rejects(selectedWindow(f.root), /private_path_exists/u);
});

test("last-window successor cannot repeat an already issued heartbeat window", async (t) => {
  const f = await lastWindowFixture(t);
  await writeNew(resolve(f.predecessor, "window-2.issued.json"), { window: 2 });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling), /private_path_exists/u);
});
test("last heartbeat report records foreground prefix without promoting either failed window", () => {
  // Arrange
  const initial = { generation: 4, active_ms: 5000, generation_elapsed_ms: 6000, budget_reserved_ms: 240000,
    budget_complete: false, safe_stop_complete: false, mine_on_boot: false, gate_closed_ms: null, shutdown_started_ms: null,
    safe_stop_stage: "not_started", active_limit_ms: 30000, shutdown_budget_ms: 15550, work_gate_remaining_ms: 9450,
    submitted: 3, accepted: 1, rejected: 0, work_dispatched: 5, nonce_work_correlations: 3, last_valid_heartbeat_ms: 6000,
    revocation_reason: "none", voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true,
    voltage_volts: 5, power_watts: 10, chip_temp_celsius: 40, fan_rpm: 1500, watchdog_alive: true };
  const final = { ...initial, active_ms: 8000, generation_elapsed_ms: 12000, budget_complete: true, safe_stop_complete: true,
    safe_stop_stage: "fan_paused", gate_closed_ms: 8800, shutdown_started_ms: 8900, revocation_reason: "heartbeat_timeout" };
  const records = [{ sequence: 1, state: { running: true, renewalsConfirmed: 1, qualification: initial } },
    { sequence: 2, state: { running: true, heartbeatSuppressed: true, renewalsConfirmed: 1, qualification: initial } },
    { sequence: 3, state: { running: false, deviceRestorationConfirmed: true, deviceLeaseInactive: true, qualification: final } }];
  const fault = { window: 2, kind: "heartbeats_suppressed", generation: 4, after_sequence: 2 };
  // Act
  const result = judgeWindow(2, records, fault, { successor: true, lastWindowOnly: true });
  // Assert
  assert.equal(result.original_normal_window, "consumed_unverified");
  assert.equal(result.original_foreground_loss_window, "consumed_unverified");
  assert.equal(result.foreground_prefix_renewal_verified, true);
  assert.equal(result.foreground_prefix_accepted_share_verified, true);
});

function coolingReview(state) {
  const budget = { ...report, reserved_mask: 3, completed_mask: 3, charged_ms: 210000 };
  return { proof: { schema: "worker-cooling-proof-v1", fan_duty_percent: 100, fan_rpm: 4200,
    post_command_fan_proven: true, asic_effects: false, budget_reserved: false },
  restoration: { schema: "worker-cooling-baseline-v1", fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false },
  budget_before: budget, budget_after: budget, state };
}
test("last-window successor requires an explicit cooling receipt and detects later modification", async (t) => {
  const f = await lastWindowFixture(t);
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review), /cooling_required/u);
  await createSuccessor(f.root, f.context, f.predecessor, f.review, f.cooling);
  const receipt = JSON.parse(await readFile(f.cooling, "utf8"));
  await writeFile(f.cooling, JSON.stringify({ ...receipt, proof: { ...receipt.proof, fan_rpm: 4100 } }));
  await assert.rejects(loadSuccessor(f.root, f.context), /cooling_changed/u);
});
test("cooling review rejects fake fan proof, reservation changes and ASIC effects", () => {
  const review = coolingReview({});
  for (const mutation of [{ fan_duty_percent: 30 }, { fan_rpm: 0 }, { post_command_fan_proven: false },
    { asic_effects: true }, { budget_reserved: true }]) {
    assert.throws(() => validateCoolingReview({ ...review, proof: { ...review.proof, ...mutation } }));
  }
  for (const mutation of [{ fan_duty_percent: 100 }, { cooling_proven: false }, { asic_effects: true }, { budget_reserved: true }]) {
    assert.throws(() => validateCoolingReview({ ...review, restoration: { ...review.restoration, ...mutation } }));
  }
  assert.throws(() => validateCoolingReview({ ...review, budget_after: { ...review.budget_after, charged_ms: 240000 } }));
});
test("cooling endpoint records bounded proof once and expires stale challenges", async (t) => {
  // Arrange
  const f = await fixture(t);
  let clock = 1000;
  const server = await createSupervisor({ privateRoot: f.root, context: f.context }, { verifyFrozen: async () => ({}), now: () => clock });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  t.after(() => new Promise((done) => { server.close(done); server.closeAllConnections(); }));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (route, body) => fetch(origin + route, { method: "POST", headers: { Origin: origin, "Content-Type": "application/json" }, body: JSON.stringify(body) });
  // Act / Assert
  assert.equal((await post("/cooling-review-context", {})).status, 400);
  await post("/activate", {});
  const expired = await (await post("/cooling-review-context", {})).json();
  clock += 45000;
  assert.equal((await post("/cooling-review", { nonce: expired.nonce, ...coolingReview(f.state) })).status, 400);
  const challenge = await (await post("/cooling-review-context", {})).json();
  const response = await post("/cooling-review", { nonce: challenge.nonce, ...coolingReview(f.state) });
  assert.equal(response.status, 200);
  const receipt = await response.json();
  assert.equal(receipt.cooling_review_saved, true);
  const saved = await readFile(resolve(f.root, receipt.review_file), "utf8");
  assert(!saved.includes(f.context.campaign_id));
  assert.equal((await post("/cooling-review", { nonce: challenge.nonce, ...coolingReview(f.state) })).status, 400);
});
