import { recoverSampleSeal, uniquePrefix } from "./sample-seal.mjs";
import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, realpath, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { BUNDLE, digest, PAGE, readJson, writeNew } from "./contract.mjs";
import { iterativeBootstrap, iterativePreflight, readPrevious, requireIterativeTask, validateIterativeContext, validateIterativePolicy } from "./iterative-preflight.mjs";
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
  const options = { suggestedDifficulty: "1000", privateRoot: resolve(parent, "attempt-001"), firmwareRoot, gateRoot, authorityDirectory, purpose,
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
    owner_resources: { schema: "worker-owner-resources-v1", generation: 1, phase: "active", observed_at_ms: "2000", heap_free_bytes: 50000, heap_largest_bytes: 12000, stack_free_bytes: 8192 },
    attempt: { schema: "worker-qualification-observation-v1", ordinal: a.ordinal, purpose: a.purpose, maximum_active_ms: a.maximumActiveMilliseconds,
      reserved_ms: a.maximumActiveMilliseconds, complete: false, active_ms: 1000 } };
  return [{ sequence: 1, state: { ...state(context), running: true, qualification: q } }, { sequence: 2, state: {
    ...state(context), qualification: { ...q, gate_closed_ms: 1800, shutdown_started_ms: 1900, safe_stop_complete: true,
      safe_stop_stage: "fan_paused", revocation_reason: "restoration_requested", owner_resources: { ...q.owner_resources, phase: "shutdown_complete" }, attempt: { ...q.attempt, complete: true } } } }];
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
  assert.equal(artifacts.grant.stratum.suggestedDifficulty, 1000);
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
  await assert.rejects(iterativePreflight(options, f.operations), /samples_changed|sealed_journal_prefix_changed/u);
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
test("v2 owner policy rejects low, stale, mismatched and malformed resource observations", async (t) => {
  const f = await fixture(t);
  assert.equal(f.context.schema, "fixed-usb-iterative-context-v3");
  assert.equal(f.context.owner_stack_minimum_bytes, 4096);
  for (const mutation of [{ stack_free_bytes: 28 }, { generation: 2 }, { phase: "preparation" },
    { heap_free_bytes: -1 }, { heap_largest_bytes: "123" }, { observed_at_ms: "18446744073709551616" }]) {
    const records = observations(f.context);
    Object.assign(records[0].state.qualification.owner_resources, mutation);
    assert.throws(() => judgeIterative(f.context, records), /owner_resources/u);
  }
  const stale = observations(f.context); delete stale[0].state.qualification.owner_resources;
  assert.throws(() => judgeIterative(f.context, stale), /owner_resources/u);
  const wrongTerminal = observations(f.context); wrongTerminal[1].state.qualification.owner_resources.phase = "active";
  assert.throws(() => judgeIterative(f.context, wrongTerminal), /owner_resources/u);
  const lowTerminal = observations(f.context); lowTerminal[1].state.qualification.owner_resources.stack_free_bytes = 28;
  assert.throws(() => judgeIterative(f.context, lowTerminal), /owner_resources/u);
  assert.equal(judgeIterative(f.context, observations(f.context)).diagnostic_only, true);
});
test("immutable v1 history retains its original judgment but cannot serve new allowances", async (t) => {
  const f = await fixture(t);
  const original = { ...f.context, schema: "fixed-usb-iterative-context-v1" };
  delete original.owner_stack_minimum_bytes;
  delete original.suggested_difficulty;
  const records = observations(original);
  for (const record of records) delete record.state.qualification.owner_resources;
  const originalBytes = JSON.stringify(records);
  assert.equal(judgeIterative(original, records).diagnostic_only, true);
  assert.equal(JSON.stringify(records), originalBytes);
  assert.throws(() => validateIterativePolicy(original, true), /policy_upgrade/u);
  await assert.rejects(createIterativeSupervisor({ ...f.options, context: original }), /policy_upgrade/u);
});
test("v2 cannot lower its stack floor or downgrade a required context field", async (t) => {
  const f = await fixture(t);
  for (const floor of [undefined, 28, 4095, 8192]) assert.throws(() => validateIterativePolicy({ ...f.context, owner_stack_minimum_bytes: floor }), /iterative_policy/u);
});

test("v1 completed receipt is revalidated under its historical resource policy", async (t) => {
  const f = await completedDiagnostic(t), root = f.options.privateRoot;
  const context = { ...f.context, schema: "fixed-usb-iterative-context-v1" };
  delete context.owner_stack_minimum_bytes;
  delete context.suggested_difficulty;
  const records = observations(context);
  for (const record of records) delete record.state.qualification.owner_resources;
  const sampleBytes = records.map((record) => JSON.stringify(record) + "\n").join("");
  await writeFile(resolve(root, "context.json"), JSON.stringify({ context, sha256: digest(JSON.stringify(context)) }));
  await writeFile(resolve(root, "iterative.samples.jsonl"), sampleBytes);
  const resultPath = resolve(root, "result.json"), record = await readJson(resultPath);
  record.receipt.schema = "worker-iterative-result-v1"; delete record.receipt.samples_file;
  record.receipt.context = context; record.receipt.context_sha256 = digest(JSON.stringify(context));
  record.receipt.samples_sha256 = digest(sampleBytes); record.sha256 = digest(JSON.stringify(record.receipt));
  await writeFile(resultPath, JSON.stringify(record));
  const before = await readFile(resultPath);
  assert.equal((await readPrevious(resultPath)).result, "passed");
  assert.deepEqual(await readFile(resultPath), before);
});
test("v2 retains a low-stack failure even when shutdown later reports healthy resources", async (t) => {
  const f = await fixture(t), records = observations(f.context);
  records[0].state.ownerResourceFailure = { schema: "worker-owner-resource-failure-v1", generation: 1,
    resources: { ...records[0].state.qualification.owner_resources, stack_free_bytes: 28 } };
  assert.throws(() => judgeIterative(f.context, records), /owner_resources_unqualified/u);
  records[0].state.ownerResourceFailure = { schema: "worker-owner-resource-failure-v1", generation: 1, resources: null };
  assert.throws(() => judgeIterative(f.context, records), /owner_resources_unqualified/u);
});

test("delayed benign close posts cannot modify a completed sample snapshot", async (t) => {
  const f = await completedDiagnostic(t), root = f.options.privateRoot;
  const receipt = (await readJson(resolve(root, "result.json"))).receipt;
  const before = await readFile(resolve(root, "sealed.samples.jsonl"));
  for (const status of ["closing", "closed"]) {
    const response = await f.request("/record", { state: { ...receipt.final_state, status } });
    assert.equal(response.status, 200); assert.equal((await response.json()).recorded, false);
  }
  assert.deepEqual(await readFile(resolve(root, "sealed.samples.jsonl")), before);
  assert.equal((await readPrevious(resolve(root, "result.json"))).result, "passed");
});
test("different late evidence is preserved as a conflict and blocks future allowance", async (t) => {
  const f = await completedDiagnostic(t), root = f.options.privateRoot;
  const receipt = (await readJson(resolve(root, "result.json"))).receipt;
  const response = await f.request("/record", { state: { ...receipt.final_state, failure: "window_control_failed" } });
  assert.equal(response.status, 400);
  assert.equal((await readJson(resolve(root, "post-seal-conflict.json"))).state.failure, "window_control_failed");
  await assert.rejects(readPrevious(resolve(root, "result.json")));
});
async function legacyTailFixture(t) {
  const f = await completedDiagnostic(t), root = f.options.privateRoot, resultPath = resolve(root, "result.json");
  const record = await readJson(resultPath), rows = observations(f.context);
  while (rows.length < 52) rows.push({ sequence: rows.length + 1, state: structuredClone(rows.at(-1).state) });
  const prefix = rows.map((row) => JSON.stringify(row) + "\n").join("");
  record.receipt.schema = "worker-iterative-result-v1"; delete record.receipt.samples_file;
  record.receipt.samples_sha256 = digest(prefix); record.sha256 = digest(JSON.stringify(record.receipt));
  await writeFile(resultPath, JSON.stringify(record));
  for (const status of ["closing", "closed"]) rows.push({ sequence: rows.length + 1, state: { ...record.receipt.final_state, status } });
  const journal = rows.map((row) => JSON.stringify(row) + "\n").join("");
  await writeFile(resolve(root, "iterative.samples.jsonl"), journal);
  await rm(resolve(root, "sealed.samples.jsonl")); await rm(resolve(root, "sample-seal-intent.json"));
  return { ...f, root, resultPath, prefix, journal, rows };
}
test("legacy exact 52-row prefix recovery preserves all original files and verdict", async (t) => {
  const f = await legacyTailFixture(t), before = await readFile(f.resultPath);
  await assert.rejects(readPrevious(f.resultPath), /samples_changed/u);
  const recovered = await recoverSampleSeal(f.root);
  assert.equal(recovered.prefix_rows, 52); assert.equal(recovered.journal_rows, 54);
  assert.deepEqual(await readFile(f.resultPath), before);
  assert.equal(await readFile(resolve(f.root, "iterative.samples.jsonl"), "utf8"), f.journal);
  assert.equal(await readFile(resolve(f.root, "sealed.samples.jsonl"), "utf8"), f.prefix);
  assert.equal((await readPrevious(f.resultPath)).result, "passed");
  await writeFile(resolve(f.root, "iterative.samples.jsonl"), f.journal + JSON.stringify({ sequence: 55, state: f.rows.at(-1).state }) + "\n");
  await assert.rejects(readPrevious(f.resultPath), /recovery_changed/u);
  await writeFile(resolve(f.root, "iterative.samples.jsonl"), f.prefix);
  await assert.rejects(readPrevious(f.resultPath), /recovery_changed/u);
});
test("legacy recovery rejects nonclose, active, failure and changed metric suffixes", async (t) => {
  const f = await legacyTailFixture(t), path = resolve(f.root, "iterative.samples.jsonl");
  for (const mutation of [{ status: "ready" }, { running: true }, { failure: "close_failed" }, { renewalsConfirmed: 1 }]) {
    const rows = structuredClone(f.rows); Object.assign(rows[52].state, mutation);
    await writeFile(path, rows.map((row) => JSON.stringify(row) + "\n").join(""));
    await assert.rejects(recoverSampleSeal(f.root), /nonclosing_suffix/u);
    await assert.rejects(readFile(resolve(f.root, "sealed.samples.jsonl")), { code: "ENOENT" });
  }
});
test("prefix recovery rejects missing or ambiguous matches", () => {
  const bytes = Buffer.from("one\ntwo\n");
  assert.throws(() => uniquePrefix(bytes, "missing"), /prefix_not_unique/u);
  assert.throws(() => uniquePrefix(bytes, "collision", () => "collision"), /prefix_not_unique/u);
});

test("a sealed snapshot cannot be changed by a later writer without blocking history", async (t) => {
  const f = await completedDiagnostic(t), root = f.options.privateRoot;
  const path = resolve(root, "sealed.samples.jsonl"), bytes = await readFile(path);
  await writeFile(path, Buffer.concat([bytes, Buffer.from("\n")]));
  await assert.rejects(readPrevious(resolve(root, "result.json")), /samples_changed/u);
});

test("new qualification requires the explicit fixed difficulty hint before creating an attempt", async (t) => {
  // Arrange
  const f = await fixture(t);
  // Act / Assert
  for (const suggestedDifficulty of [undefined, "0", "256", "1000.0", "1e3", "65536", "-1"]) {
    await assert.rejects(iterativePreflight({ ...f.options, suggestedDifficulty }, f.operations), /iterative_hint_policy/u);
  }
  assert.equal(f.context.suggested_difficulty, 1000);
  for (const suggested_difficulty of [undefined, null, 0, 256, 1000.5, "1000"]) {
    assert.throws(() => validateIterativePolicy({ ...f.context, suggested_difficulty }), /iterative_hint_policy/u);
  }
});

test("immutable v2 observations retain their judgment but cannot authorize hinted work", async (t) => {
  // Arrange
  const f = await fixture(t);
  const context = { ...f.context, schema: "fixed-usb-iterative-context-v2" };
  delete context.suggested_difficulty;
  const records = observations(context), original = JSON.stringify(records);
  // Act / Assert
  assert.equal(judgeIterative(context, records).diagnostic_only, true);
  assert.equal(JSON.stringify(records), original);
  assert.throws(() => validateIterativePolicy(context, true), /policy_upgrade/u);
  await assert.rejects(createIterativeSupervisor({ ...f.options, context }), /policy_upgrade/u);
});
