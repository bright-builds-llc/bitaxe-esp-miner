import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, realpath, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { BUNDLE, digest, PAGE, readJson, writeNew } from "./contract.mjs";
import { iterativeBootstrap, iterativePreflight, requireIterativeTask, validateIterativeContext } from "./iterative-preflight.mjs";
import { requireIdleLedger, validateAttempt } from "./iterative-contract.mjs";
import { judgeIterative, finishIterative } from "./iterative-judge.mjs";
import { createIterativeSupervisor } from "./iterative-server.mjs";
import { signAttempt } from "./authority.mjs";
import { validateDiagnosticExport } from "./diagnostic-export.mjs";

const ID = Buffer.alloc(16, 1).toString("base64url");
const original = { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false };
const ledger = (ordinal = 1, charged = 0) => ({ schema: "worker-qualification-ledger-v1", next_ordinal: ordinal,
  total_charged_ms: charged, pending: false, last_completed_ordinal: ordinal - 1 });
function state(context, closed = false) {
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: closed ? "closed" : "ready", connected: !closed, running: false,
    heartbeatSuppressed: false, renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: closed,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: ID, device_identity_match: true, settings_match: true,
      authorization_high_water_match: true, mine_on_boot: false } };
}
async function fixture(t, purpose = "diagnostic") {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "iterative-fixture-"))); await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  const firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), authorityDirectory = resolve(base, "authority");
  for (const path of [firmwareRoot, gateRoot, authorityDirectory]) await mkdir(path, { mode: 0o700 });
  await writeFile(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-worker-preparation-panic-qualification | fixture\n");
  const old = { firmware_root: firmwareRoot, firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64), campaign_id: ID };
  const input = { original_context_path: resolve(base, "original-context.json"), budget_review_path: resolve(base, "original-budget.json"), final_state_path: resolve(base, "original-final.json") };
  await writeNew(input.original_context_path, { context: old, sha256: digest(JSON.stringify(old)) });
  await writeNew(input.budget_review_path, { context_sha256: digest(JSON.stringify(old)), report: original });
  await writeNew(input.final_state_path, state(old, true));
  const bootstrapInput = resolve(base, "bootstrap-input.json"), parent = resolve(base, "iterative");
  await writeNew(bootstrapInput, input); await iterativeBootstrap(parent, bootstrapInput, { ignored: () => undefined });
  const progress = resolve(base, "progress.json");
  await writeNew(progress, { schema: "worker-qualification-progress-v1", review: "verified", reason: "software_correction", evidence_sha256: ["d".repeat(64)] });
  const options = { privateRoot: resolve(parent, "attempt-001"), firmwareRoot, gateRoot, authorityDirectory, purpose,
    firmwareCommit: "d".repeat(40), gateCommit: old.gate_commit, manifest: resolve(base, "manifest.json"), previousReceipt: resolve(parent, "bootstrap.json"), input: progress };
  const snapshot = { firmware_commit: options.firmwareCommit, gate_commit: options.gateCommit, app_elf_sha256: old.app_elf_sha256,
    gate_page_relative_path: PAGE, gate_page_sha256: digest("page"), gate_bundle_sha256: digest("bundle") };
  const operations = { ignored: () => undefined, inspectSources: async () => snapshot };
  await iterativePreflight(options, operations);
  const { context } = await readJson(resolve(options.privateRoot, "context.json"));
  await mkdir(resolve(firmwareRoot, "firmware/bitaxe/bwg"), { recursive: true });
  await writeNew(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), {});
  for (const [file, bytes] of [[PAGE, "page"], [BUNDLE, "bundle"]]) { await mkdir(dirname(resolve(gateRoot, file)), { recursive: true }); await writeFile(resolve(gateRoot, file), bytes); }
  for (let cycle = 1; cycle <= 4; cycle += 1) await writeNew(resolve(options.privateRoot, `cycle-${cycle}.json`), {
    schema: "fixed-usb-cycle-report-v1", cycle, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, baseline_id: ID,
    browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true, device_identity_match: true,
    settings_match: true, authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false });
  return { base, parent, options, context, operations, originalInput: input };
}
function observations(context) {
  const a = context.qualification_attempt;
  const q = { schema: "worker-qualification-v1", generation: 1, active_ms: 1000, generation_elapsed_ms: 2000,
    budget_reserved_ms: 240000, budget_complete: true, active_limit_ms: a.maximumActiveMilliseconds, shutdown_budget_ms: 15550,
    work_gate_remaining_ms: 10000, submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 1,
    last_valid_heartbeat_ms: 1000, gate_closed_ms: null, shutdown_started_ms: null, safe_stop_complete: false,
    safe_stop_stage: "not_started", revocation_reason: "none", voltage_volts: 5, power_watts: 10, chip_temp_celsius: 40,
    fan_rpm: 2000, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false,
    attempt: { schema: "worker-qualification-observation-v1", ordinal: a.ordinal, purpose: a.purpose, maximum_active_ms: a.maximumActiveMilliseconds,
      reserved_ms: a.maximumActiveMilliseconds, complete: false, active_ms: 1000 } };
  return [{ sequence: 1, state: { ...state(context), running: true, qualification: q } }, { sequence: 2, state: {
    ...state(context), qualification: { ...q, gate_closed_ms: 1800, shutdown_started_ms: 1900, safe_stop_complete: true,
      safe_stop_stage: "fan_paused", revocation_reason: "restoration_requested", attempt: { ...q.attempt, complete: true } } } }];
}
test("iterative bootstrap leaves original allowance immutable and cannot mint a legacy campaign", async (t) => {
  const f = await fixture(t);
  assert(!(await readdir(f.parent)).includes("campaign.json"));
  await validateIterativeContext(f.options.privateRoot, f.context);
  assert.equal((await readJson(f.originalInput.budget_review_path)).report.charged_ms, 240000);
  await assert.rejects(iterativePreflight({ ...f.options, privateRoot: resolve(f.parent, "duplicate") }, f.operations));
});
test("attempt contracts reject wrong duration, ordinal exhaustion and legacy grant mixing", async () => {
  const attempt = { schema: "worker-qualification-attempt-v1", id: ID, ordinal: 1, purpose: "diagnostic", maximumActiveMilliseconds: 30000 };
  for (const value of [{ ...attempt, ordinal: 0 }, { ...attempt, ordinal: 0x100000000 }, { ...attempt, maximumActiveMilliseconds: 30001 }, { ...attempt, acceptanceCampaign: {} }]) assert.throws(() => validateAttempt(value));
  const artifacts = await signAttempt({ attempt, binding: Buffer.alloc(32, 2).toString("base64url"), challengeId: "challenge_fixture", stratum: {},
    sign: async (operation) => ({ profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }) });
  assert(!Object.hasOwn(artifacts.grant, "acceptanceCampaign"));
  assert.equal(artifacts.grant.qualificationAttempt.maximumActiveMilliseconds, 30000);
  assert(artifacts.renewals.every((value) => !Object.hasOwn(value, "qualificationAttempt")));
});
test("diagnostic succeeds on actual work and safe stop without requiring an accepted share", async (t) => {
  const f = await fixture(t);
  assert.equal(judgeIterative(f.context, observations(f.context)).diagnostic_only, true);
  const noWork = observations(f.context); for (const record of noWork) record.state.qualification.work_dispatched = 0;
  assert.throws(() => judgeIterative(f.context, noWork), /mining_evidence/u);
});
test("fresh ledger cannot refund, skip or retain a pending allowance", () => {
  for (const value of [{ ...ledger(), pending: true }, ledger(2, 30000), { ...ledger(), total_charged_ms: 1 }]) assert.throws(() => requireIdleLedger(value, 1, 0));
});
async function serverFixture(t) {
  const f = await fixture(t); let signs = 0;
  const server = await createIterativeSupervisor({ ...f.options, context: f.context }, {
    verifyFrozen: async () => ({}), readPool: async () => ({}),
    sign: async (operation) => { signs += 1; return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }; },
    validateDiagnostics: async (input) => { assert.deepEqual(input, { schema: "worker-diagnostic-export-v1", observations: [] }); return input; },
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  t.after(() => new Promise((done) => { server.close(done); server.closeAllConnections(); }));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const request = (path, input) => fetch(origin + path, { method: input === undefined ? "GET" : "POST",
    headers: { Origin: origin, "Content-Type": "application/json" }, body: input === undefined ? undefined : JSON.stringify(input) });
  const current = state(f.context), binding = Buffer.alloc(32, 2).toString("base64url");
  const review = async () => {
    const challenge = await (await request("/budget-review-context", {})).json();
    return request("/budget-review", { nonce: challenge.nonce, report: ledger(), controlSessionBindingSha256: binding, state: current });
  };
  const cool = async () => {
    const challenge = await (await request("/cooling-review-context", {})).json();
    return request("/cooling-review", { nonce: challenge.nonce, budget_before: ledger(), budget_after: ledger(), state: current,
      proof: { schema: "worker-cooling-proof-v1", fan_duty_percent: 100, fan_rpm: 2000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false },
      restoration: { schema: "worker-cooling-baseline-v1", fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false } });
  };
  return { ...f, request, review, cool, binding, signs: () => signs };
}
test("iterative server requires fan proof, fresh possession ledger and one delivery", async (t) => {
  const f = await serverFixture(t);
  await f.request("/activate", {});
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: f.binding })).status, 400);
  await f.review();
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: f.binding })).status, 400);
  assert.equal(f.signs(), 0);
  assert.equal((await f.cool()).status, 200);
  assert.equal((await f.review()).status, 200);
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: f.binding })).status, 200);
  const artifacts = await (await f.request("/window-artifacts")).json();
  assert.equal(artifacts.grant.qualificationAttempt.ordinal, 1);
  assert.equal((await f.request("/window-artifacts")).status, 400);
  assert(!(await readFile(resolve(f.options.privateRoot, "issued.json"), "utf8")).includes(f.binding));
});
async function completedDiagnostic(t) {
  const f = await serverFixture(t);
  await f.request("/activate", {}); await f.cool(); await f.review();
  await f.request("/authorization-context", { controlSessionBindingSha256: f.binding }); await f.request("/window-artifacts");
  for (const record of observations(f.context)) assert.equal((await f.request("/record", { state: record.state })).status, 200);
  const challenge = await (await f.request("/completion-context", {})).json();
  const response = await f.request("/completion-review", { nonce: challenge.nonce, ledger_after: ledger(2, 30000), original_budget: original, final_state: state(f.context, true) });
  assert.equal(response.status, 200);
  assert.equal((await response.json()).result, "passed");
  return f;
}
test("completion derives pass from work samples and binds the full nonrefunded ledger", async (t) => {
  const f = await completedDiagnostic(t);
  const result = await readJson(resolve(f.options.privateRoot, "result.json"));
  assert.equal(result.receipt.total_charged_ms, 30000);
  assert.equal(result.receipt.original_budget.charged_ms, 240000);
  const response = await f.request("/diagnostic-export", { schema: "worker-diagnostic-export-v1", observations: [] });
  const receipt = await response.json();
  assert.equal(receipt.diagnostic_export_saved, true);
  assert(!(await readFile(resolve(f.options.privateRoot, receipt.review_file), "utf8")).includes(ID));
});
test("same-runtime cycle reuse binds exact receipts and rejects changed runtime", async (t) => {
  const f = await completedDiagnostic(t);
  const progress = resolve(f.base, "normal-progress.json");
  await writeNew(progress, { schema: "worker-qualification-progress-v1", review: "verified", reason: "diagnostic_pass", evidence_sha256: ["e".repeat(64)] });
  const options = { ...f.options, input: progress, purpose: "normal", privateRoot: resolve(f.parent, "attempt-002"),
    previousReceipt: resolve(f.options.privateRoot, "result.json"), cyclesFrom: f.options.privateRoot };
  await iterativePreflight(options, f.operations);
  const record = await readJson(resolve(options.privateRoot, "context.json"));
  assert.equal(record.context.qualification_attempt.ordinal, 2);
  assert.equal(record.context.expected_charged_ms, 30000);
  for (let cycle = 1; cycle <= 4; cycle += 1) assert.deepEqual(await readFile(resolve(options.privateRoot, `cycle-${cycle}.json`)), await readFile(resolve(f.options.privateRoot, `cycle-${cycle}.json`)));
  await assert.rejects(iterativePreflight({ ...options, privateRoot: resolve(f.parent, "changed") }, {
    ...f.operations, inspectSources: async () => ({ ...record.context, firmware_commit: "e".repeat(40) }) }), /cycle_runtime_changed|iterative_next_progress/u);
});
test("normal acceptance cannot pass without a correlated accepted share and renewal", async (t) => {
  const f = await fixture(t);
  f.context.qualification_attempt.purpose = "normal"; f.context.qualification_attempt.maximumActiveMilliseconds = 180000;
  const records = observations(f.context);
  for (const record of records) { record.state.renewalsConfirmed = 1; record.state.qualification.nonce_work_correlations = 1; record.state.qualification.submitted = 1; }
  assert.throws(() => judgeIterative(f.context, records), /no_accepted_share/u);
  for (const record of records) record.state.qualification.accepted = 1;
  assert.equal(judgeIterative(f.context, records).accepted_share_verified, true);
  for (const record of records) record.state.renewalsConfirmed = 0;
  assert.throws(() => judgeIterative(f.context, records), /foreground_window/u);
});
test("failed cleanup and forged prior pass never authorize the next ordinal", async (t) => {
  const f = await completedDiagnostic(t), path = resolve(f.options.privateRoot, "result.json"), originalBytes = await readFile(path);
  const record = JSON.parse(originalBytes);
  record.receipt.final_state.deviceRestorationConfirmed = false; record.sha256 = digest(JSON.stringify(record.receipt));
  await writeFile(path, JSON.stringify(record));
  const options = { ...f.options, privateRoot: resolve(f.parent, "attempt-002"), previousReceipt: path };
  await assert.rejects(iterativePreflight(options, f.operations), /cleanup_required/u);
  await writeFile(path, originalBytes);
  await writeFile(resolve(f.options.privateRoot, "iterative.samples.jsonl"), '[]\n');
  await assert.rejects(iterativePreflight(options, f.operations), /samples_changed/u);
});
test("diagnostic export rejects raw fields and unbounded observations before invoking a validator", async () => {
  await assert.rejects(validateDiagnosticExport({ schema: "worker-diagnostic-export-v1", observations: [], raw: "private-fixture" }, "/unused"), /object_fields/u);
  await assert.rejects(validateDiagnosticExport({ schema: "worker-diagnostic-export-v1", observations: Array(41).fill({}) }, "/unused"), /diagnostic_export_shape/u);
});
test("initial parent cannot skip diagnostic qualification", async (t) => {
  await assert.rejects(fixture(t, "normal"), /iterative_initial_purpose/u);
});
test("earliest browser failure remains separate from missing qualification judgment", async (t) => {
  const f = await serverFixture(t);
  await f.request("/activate", {}); await f.cool(); await f.review();
  await f.request("/authorization-context", { controlSessionBindingSha256: f.binding }); await f.request("/window-artifacts");
  await f.request("/record", { state: { ...state(f.context), status: "failed", failure: "start_failed", serialFailureCategory: "timeout" } });
  const first = await readFile(resolve(f.options.privateRoot, "first-failure.json"));
  await f.request("/record", { state: { ...state(f.context), status: "failed", failure: "close_failed", serialFailureCategory: "fields" } });
  const challenge = await (await f.request("/completion-context", {})).json();
  const response = await f.request("/completion-review", { nonce: challenge.nonce, ledger_after: ledger(2, 30000), original_budget: original, final_state: state(f.context, true) });
  assert.equal(response.status, 200);
  const result = await response.json();
  assert.equal(result.result, "unverified");
  assert.equal(result.first_failure, "start_failed");
  assert.equal(result.judgment_failure, "iterative_observation_missing");
  assert.deepEqual(await readFile(resolve(f.options.privateRoot, "first-failure.json")), first);
});

test("iterative hardware owner must occur exactly once in Active", async (t) => {
  const f = await fixture(t);
  const path = resolve(f.options.firmwareRoot, "TASKS.md");
  for (const contents of ["## Active\n", "## Future\n### task-worker-preparation-panic-qualification | fixture\n",
    "## Active\n### task-worker-preparation-panic-qualification | one\n### task-worker-preparation-panic-qualification | two\n",
    "## Active\n### task-worker-preparation-panic-qualification | one\n## Future\n### task-worker-preparation-panic-qualification | duplicate\n"]) {
    await writeFile(path, contents);
    await assert.rejects(requireIterativeTask(f.options.firmwareRoot), /iterative_active_task_required/u);
    await assert.rejects(validateIterativeContext(f.options.privateRoot, f.context), /iterative_active_task_required/u);
  }
});
test("fast diagnostic accepts authenticated stopping work before a running UI publication", async (t) => {
  // Arrange
  const f = await fixture(t), records = observations(f.context);
  records[0].state.running = false; records[0].state.status = "stopping";
  // Act / Assert
  assert.equal(judgeIterative(f.context, records).diagnostic_only, true);
});
test("fast diagnostic rejects stopping without work or a qualification record", async (t) => {
  const f = await fixture(t);
  for (const missing of [false, true]) {
    const records = observations(f.context);
    records[0].state.running = false; records[0].state.status = "stopping";
    if (missing) delete records[0].state.qualification;
    else records[0].state.qualification.work_dispatched = 0;
    assert.throws(() => judgeIterative(f.context, records), /running_device_evidence_missing/u);
  }
});
test("fast diagnostic checks fresh safety on its initial stopping work sample", async (t) => {
  const f = await fixture(t);
  for (const mutation of [{ voltage_volts: 4 }, { power_watts: 16 }, { chip_temp_celsius: 75 }, { fan_rpm: 0 },
    { watchdog_alive: false }, { fan_fresh: false, fan_rpm: null }]) {
    const records = observations(f.context);
    records[0].state.running = false; records[0].state.status = "stopping";
    Object.assign(records[0].state.qualification, mutation);
    assert.throws(() => judgeIterative(f.context, records), /safety_evidence_failed/u);
  }
});
test("normal acceptance cannot substitute stopping for running evidence", async (t) => {
  const f = await fixture(t);
  f.context.qualification_attempt.purpose = "normal"; f.context.qualification_attempt.maximumActiveMilliseconds = 180000;
  const records = observations(f.context);
  records[0].state.running = false; records[0].state.status = "stopping";
  assert.throws(() => judgeIterative(f.context, records), /running_device_evidence_missing/u);
});
