import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { CADENCE_LIMITS, CADENCE_SCHEMA, CADENCE_TASK, requireCadenceTask, validateCadencePolicy } from "./cadence-contract.mjs";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { digest, fileDigest, writeNew } from "./contract.mjs";
import { closeUnissued, readUnissued } from "./cadence-unissued.mjs";

async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-preflight-"))); await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  for (const path of ["firmware/scripts/fixed-usb-qualification", "firmware/bazel-bin/tools/http-transport", "gate", "authority", "attempts/prior"]) await mkdir(resolve(base, path), { recursive: true, mode: 0o700 });
  const firmwareRoot = resolve(base, "firmware");
  await writeFile(resolve(firmwareRoot, "TASKS.md"), `## Active\n### ${CADENCE_TASK} | fixture\n`);
  await writeFile(resolve(firmwareRoot, "scripts/fixed-usb-qualification/fixture.mjs"), "export const bounded = true;\n");
  const previousReceipt = resolve(base, "attempts/prior/result.json"); await writeNew(previousReceipt, { fixture: true });
  const input = resolve(base, "progress.json"); await writeNew(input, { schema: "worker-qualification-progress-v1", review: "verified", reason: "software_correction", evidence_sha256: ["d".repeat(64)] });
  const observerBinary = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer"); await writeFile(observerBinary, "binary fixture", { mode: 0o700 });
  const options = { suggestedDifficulty: "1000", observerBinary, firmwareRoot, gateRoot: resolve(base, "gate"), authorityDirectory: resolve(base, "authority"),
    privateRoot: resolve(base, "attempts/current"), previousReceipt, input, manifest: resolve(base, "manifest.json") };
  const previous = { context: { firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40) }, cleanup_confirmed: true,
    original_budget: { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false },
    next_ordinal: 16, total_charged_ms: 1200000, original_campaign_id: Buffer.alloc(16, 1).toString("base64url") };
  let inspections = 0, snapshotChecks = 0;
  const operations = { ignored: () => undefined, readPrevious: async () => previous,
    verifyArtifactSnapshot: async (root, context) => { assert.equal(root, resolve(base, "attempts/prior")); assert.equal(context, previous.context); snapshotChecks++; },
    inspectSources: async () => { assert(snapshotChecks > 0); inspections += 1; return { firmware_commit: "c".repeat(40), gate_commit: "b".repeat(40) }; } };
  return { base, options, previous, operations, inspections: () => inspections };
}

test("cadence preflight retains exhausted legacy accounting and allocates one exact normal successor", async t => {
  // Arrange
  const f = await fixture(t);
  // Act
  const result = await cadencePreflight(f.options, f.operations);
  const { context } = JSON.parse(await readFile(resolve(f.options.privateRoot, "context.json"), "utf8"));
  // Assert
  assert.deepEqual(result, { cadence_preflight_created: true, ordinal: 16, reserved_on_device: false, maximum_active_ms: 180000, device_effects: false });
  const retained = resolve(f.options.privateRoot, "cadence-observer.bin");
  assert.equal(await fileDigest(retained), context.cadence_observer.sha256);
  assert.equal((await stat(retained)).mode & 0o777, 0o600);
  assert.deepEqual(await readFile(retained), await readFile(f.options.observerBinary));
  assert.equal(context.expected_charged_ms, 1200000); assert.equal(context.qualification_attempt.maximumActiveMilliseconds, 180000);
  assert.equal(context.cadence_diagnostics_version, 2);
  assert.deepEqual(context.cadence_limits, CADENCE_LIMITS); assert.equal(f.previous.original_budget.charged_ms, 240000);
  assert(!(await readdir(resolve(f.base, "attempts"))).includes("campaign.json"));
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(f.base, "attempts/repeated") }, f.operations));
});

test("missing or nonactive cadence task blocks before source inspection or observer admission", async t => {
  // Arrange
  const f = await fixture(t);
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), `## Future\n### ${CADENCE_TASK} | fixture\n`);
  // Act / Assert
  await assert.rejects(cadencePreflight(f.options, f.operations), { code: "cadence_active_task_required" });
  assert.equal(f.inspections(), 0); assert.deepEqual(await readdir(resolve(f.base, "attempts")), ["prior"]);
});

test("active task duplicates cannot grant authority", async t => {
  // Arrange
  const f = await fixture(t);
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), `## Active\n### ${CADENCE_TASK} | first\n## Future\n### ${CADENCE_TASK} | duplicate\n`);
  // Act / Assert
  await assert.rejects(requireCadenceTask(f.options.firmwareRoot), { code: "cadence_active_task_required" });
});

test("unchanged runtime and unverified progress do not admit another allowance", async t => {
  // Arrange
  const f = await fixture(t);
  f.operations.inspectSources = async () => f.previous.context;
  // Act / Assert
  await assert.rejects(cadencePreflight(f.options, f.operations), { code: "cadence_unchanged_retry" });
  await writeFile(f.options.input, JSON.stringify({ schema: "worker-qualification-progress-v1", review: "pending", reason: "software_correction", evidence_sha256: ["d".repeat(64)] }));
  await assert.rejects(cadencePreflight(f.options, f.operations), { code: "cadence_verified_progress_required" });
});

test("cadence policy rejects mixing recovery modes or widening fixed limits", () => {
  // Arrange
  const context = { schema: CADENCE_SCHEMA, required_no_mining_cycles: 4, owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000,
    qualification_attempt: { purpose: "normal", maximumActiveMilliseconds: 180000 }, cadence_limits: CADENCE_LIMITS,
    cadence_observer: { path: "/fixture", sha256: "a".repeat(64) }, cadence_validator_sha256: "b".repeat(64) };
  // Act / Assert
  validateCadencePolicy(context);
  for (const change of [{ recovery_phase: "loss" }, { cycle_source: {} }, { owner_stack_minimum_bytes: 2048 },
    { cadence_limits: { ...CADENCE_LIMITS, maximum_interval_ms: 1501 } }, { qualification_attempt: { purpose: "normal", maximumActiveMilliseconds: 180001 } }]) {
    assert.throws(() => validateCadencePolicy({ ...context, ...change }));
  }
});

test("missing retained predecessor snapshot blocks before fresh source inspection", async t => {
  // Arrange
  const f = await fixture(t);
  delete f.operations.verifyArtifactSnapshot;
  // Act / Assert
  await assert.rejects(cadencePreflight(f.options, f.operations));
  assert.equal(f.inspections(), 0); assert.deepEqual(await readdir(resolve(f.base, "attempts")), ["prior"]);
});

test("an executable outside the canonical observer output is not admitted even with identical bytes", async t => {
  // Arrange
  const f = await fixture(t), alternate = resolve(f.base, "alternate-observer");
  await writeFile(alternate, await readFile(f.options.observerBinary), { mode: 0o700 });
  // Act / Assert
  await assert.rejects(cadencePreflight({ ...f.options, observerBinary: alternate }, f.operations), { code: "cadence_observer_not_canonical" });
  assert.deepEqual(await readdir(resolve(f.base, "attempts")), ["prior"]);
});

test("both live and historical context review reject a modified retained observer", async t => {
  // Arrange
  const f = await fixture(t); await cadencePreflight(f.options, f.operations);
  const { context } = JSON.parse(await readFile(resolve(f.options.privateRoot, "context.json"), "utf8"));
  await writeFile(resolve(f.options.privateRoot, "cadence-observer.bin"), "changed retained executable");
  // Act / Assert
  for (const historical of [false, true]) {
    await assert.rejects(validateCadenceContext(f.options.privateRoot, context, { historical }), { code: "cadence_observer_snapshot" });
  }
});

async function unissuedFixture(t) {
  const f = await fixture(t); await cadencePreflight(f.options, f.operations);
  const oldRoot = f.options.privateRoot, { context } = JSON.parse(await readFile(resolve(oldRoot, "context.json"), "utf8"));
  f.operations.verifyArtifactSnapshot = async () => undefined;
  await writeNew(resolve(oldRoot, "artifact-snapshot.json"), { fixture: "retained artifacts" });
  const final = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    status: "closed", connected: false, running: false, heartbeatSuppressed: false, renewalsConfirmed: 0, deviceRestorationConfirmed: false,
    deviceBaselineConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: true,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: Buffer.alloc(16, 2).toString("base64url"), device_identity_match: true,
      settings_match: true, authorization_high_water_match: true, mine_on_boot: false } };
  const records = [{ sequence: 1, state: { ...final, status: "failed", failure: "connect_failed", admissionFailureStage: "permission", serialFailureCategory: "operation_failed" } },
    { sequence: 2, state: final }];
  await writeFile(resolve(oldRoot, "iterative.samples.jsonl"), records.map(record => JSON.stringify(record) + "\n").join(""), { mode: 0o600 });
  await writeNew(resolve(oldRoot, "first-failure.json"), { schema: "worker-iterative-first-failure-v1", ordinal: 16, sequence: 1,
    browser: "connect_failed", serial: "operation_failed", admission: "permission" });
  await writeNew(resolve(oldRoot, "native-gesture-remediation.json"), { schema: "cadence-preparation-remediation-v1",
    original_boundary: { failure: "connect_failed", stage: "permission", serial: "operation_failed" },
    remediation: "foreground_native_accessibility_click", native_chooser_observed: true, exact_firmware_admitted: true,
    baseline_confirmed: true, ledger_read_method: "workerAcceptance.reviewQualificationAttempts", original_budget_read_method: "workerAcceptance.reviewBudget",
    private_values_exported: false, qualification_claimed: false });
  const input = { ledger: { schema: "worker-qualification-ledger-v1", next_ordinal: 16, total_charged_ms: 1200000, pending: false, last_completed_ordinal: 15 },
    original_budget: f.previous.original_budget, cleanup: { schema: "worker-unissued-cleanup-v1", source: "parent-observed", browser_closed: true,
      supervisor_exited: true, supervisor_exit_code: 0, listener_absent: true, owned_children_absent: true, serial_holders_absent: true } };
  await closeUnissued(oldRoot, input, f.operations);
  const successorInput = resolve(f.base, "successor-progress.json");
  await writeNew(successorInput, { schema: "worker-qualification-progress-v1", review: "verified", reason: "manual_remediation",
    evidence_sha256: [await fileDigest(resolve(oldRoot, "native-gesture-remediation.json"))] });
  const options = { ...f.options, input: successorInput, privateRoot: resolve(f.base, "attempts/successor"), supersedeUnissued: resolve(oldRoot, "unissued-closure.json") };
  f.operations.inspectSources = async () => ({ firmware_commit: "f".repeat(40), gate_commit: context.gate_commit });
  return { ...f, options, oldRoot, oldContext: context };
}

test("sealed unissued preparation reuses the unspent ordinal with fresh identity and an exclusive separate marker", async t => {
  // Arrange
  const f = await unissuedFixture(t), oldMarker = resolve(f.base, "attempts/ordinal-16.json"), oldHash = await fileDigest(oldMarker);
  const closureHash = await fileDigest(f.options.supersedeUnissued);
  // Act
  await cadencePreflight(f.options, f.operations);
  const { context } = JSON.parse(await readFile(resolve(f.options.privateRoot, "context.json"), "utf8"));
  // Assert
  assert.equal(context.preparation_attempt, 2); assert.equal(context.qualification_attempt.ordinal, 16);
  assert.notEqual(context.qualification_attempt.id, f.oldContext.qualification_attempt.id);
  assert.deepEqual(context.unissued_predecessor, { root: f.oldRoot, closure_sha256: closureHash });
  assert.equal(await fileDigest(oldMarker), oldHash); assert.equal(await fileDigest(f.options.supersedeUnissued), closureHash);
  assert.deepEqual(JSON.parse(await readFile(resolve(f.base, "attempts/ordinal-16-preparation-2.json"), "utf8")),
    { context_sha256: digest(JSON.stringify(context)), attempt_root: f.options.privateRoot });
  await validateCadenceContext(f.options.privateRoot, context, { operations: f.operations });
  await readUnissued(f.options.supersedeUnissued, f.operations);
  await assert.rejects(validateCadenceContext(f.oldRoot, f.oldContext, { operations: f.operations }), { code: "private_path_exists" });
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(f.base, "attempts/duplicate") }, f.operations), { code: "private_path_exists" });
});

test("unchanged pair, different charged predecessor and mutated closure cannot supersede preparation", async t => {
  // Arrange / Act / Assert
  for (const mode of ["pair", "predecessor", "closure"]) {
    const f = await unissuedFixture(t);
    if (mode === "pair") f.operations.inspectSources = async () => ({ firmware_commit: f.oldContext.firmware_commit, gate_commit: f.oldContext.gate_commit });
    if (mode === "predecessor") {
      const other = resolve(f.base, "attempts/other"); await mkdir(other, { mode: 0o700 });
      f.options.previousReceipt = resolve(other, "result.json"); await writeNew(f.options.previousReceipt, { another: "completed predecessor" });
    }
    if (mode === "closure") await writeFile(resolve(f.oldRoot, "native-gesture-remediation.json"), "{}");
    await assert.rejects(cadencePreflight(f.options, f.operations));
    assert(!(await readdir(resolve(f.base, "attempts"))).includes("ordinal-16-preparation-2.json"));
  }
});

test("ordinary cadence preflight cannot use manual remediation without sealed unissued lineage", async t => {
  // Arrange
  const f = await fixture(t);
  await writeFile(f.options.input, JSON.stringify({ schema: "worker-qualification-progress-v1", review: "verified", reason: "manual_remediation", evidence_sha256: ["d".repeat(64)] }));
  // Act / Assert
  await assert.rejects(cadencePreflight(f.options, f.operations), { code: "cadence_verified_progress_required" });
  assert.equal(f.inspections(), 0);
});
