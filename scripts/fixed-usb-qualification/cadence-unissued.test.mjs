import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { CADENCE_LIMITS, CADENCE_SCHEMA, CADENCE_TASK } from "./cadence-contract.mjs";
import { closeUnissued, readUnissued } from "./cadence-unissued.mjs";
import { digest, fileDigest, nonce, readJson, writeNew } from "./contract.mjs";

async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-unissued-"))); await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, "current"), prior = resolve(base, "prior");
  await mkdir(root, { mode: 0o700 }); await mkdir(prior, { mode: 0o700 });
  await writeFile(resolve(base, "TASKS.md"), `## Active\n### ${CADENCE_TASK} | fixture\n`);
  const previousPath = resolve(prior, "result.json"), progress = resolve(base, "progress.json");
  await writeNew(previousPath, { fixture: "completed predecessor" });
  await writeNew(progress, { schema: "worker-qualification-progress-v1", review: "verified", reason: "software_correction", evidence_sha256: ["d".repeat(64)] });
  await writeFile(resolve(root, "cadence-observer.bin"), "fixture observer", { mode: 0o600 });
  const context = { schema: CADENCE_SCHEMA, firmware_root: base, firmware_commit: "c".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "e".repeat(64),
    owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000, required_no_mining_cycles: 4, cadence_limits: CADENCE_LIMITS,
    cadence_observer: { path: resolve(root, "cadence-observer.bin"), sha256: await fileDigest(resolve(root, "cadence-observer.bin")) },
    cadence_validator_sha256: "a".repeat(64), qualification_attempt: { schema: "worker-qualification-attempt-v1", id: nonce(), ordinal: 16, purpose: "normal", maximumActiveMilliseconds: 180000 },
    previous_receipt: previousPath, previous_receipt_sha256: await fileDigest(previousPath), expected_charged_ms: 1200000,
    original_campaign_id: nonce(), progress_path: progress, progress_sha256: await fileDigest(progress) };
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await writeNew(resolve(base, "ordinal-16.json"), { context_sha256: digest(JSON.stringify(context)), attempt_root: root });
  await writeNew(resolve(root, "artifact-snapshot.json"), { fixture: "retained artifacts" });
  const state = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "closed", connected: false, running: false, heartbeatSuppressed: false,
    renewalsConfirmed: 0, deviceRestorationConfirmed: false, deviceBaselineConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: true,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: nonce(), device_identity_match: true,
      settings_match: true, authorization_high_water_match: true, mine_on_boot: false },
    cadence: { schema: "worker-cadence-browser-v1", enabled: true, suppressionRequested: false } };
  const records = [{ sequence: 1, state: { ...state, status: "failed", failure: "connect_failed", admissionFailureStage: "permission", serialFailureCategory: "operation_failed" } },
    { sequence: 2, state }];
  const writeRecords = () => writeFile(resolve(root, "iterative.samples.jsonl"), records.map(record => JSON.stringify(record) + "\n").join(""), { mode: 0o600 });
  await writeRecords();
  await writeNew(resolve(root, "first-failure.json"), { schema: "worker-iterative-first-failure-v1", ordinal: 16, sequence: 1,
    browser: "connect_failed", serial: "operation_failed", admission: "permission" });
  await writeNew(resolve(root, "native-gesture-remediation.json"), { schema: "cadence-preparation-remediation-v1",
    original_boundary: { failure: "connect_failed", stage: "permission", serial: "operation_failed" },
    remediation: "foreground_native_accessibility_click", native_chooser_observed: true, exact_firmware_admitted: true,
    baseline_confirmed: true, ledger_read_method: "workerAcceptance.reviewQualificationAttempts", original_budget_read_method: "workerAcceptance.reviewBudget",
    private_values_exported: false, qualification_claimed: false });
  const input = { ledger: { schema: "worker-qualification-ledger-v1", next_ordinal: 16, total_charged_ms: 1200000, pending: false, last_completed_ordinal: 15 },
    original_budget: { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false },
    cleanup: { schema: "worker-unissued-cleanup-v1", source: "parent-observed", browser_closed: true, supervisor_exited: true, supervisor_exit_code: 0,
      listener_absent: true, owned_children_absent: true, serial_holders_absent: true } };
  const operations = { readPrevious: async () => ({ context: { firmware_commit: "a".repeat(40), gate_commit: context.gate_commit },
    next_ordinal: 16, total_charged_ms: 1200000, original_campaign_id: context.original_campaign_id }), verifyArtifactSnapshot: async () => undefined };
  return { base, root, context, input, operations, records, writeRecords, closure: resolve(root, "unissued-closure.json") };
}

test("unissued closure preserves all original bytes and seals failure without a hardware pass", async t => {
  // Arrange
  const f = await fixture(t), before = new Map();
  for (const name of await readdir(f.root)) before.set(name, await fileDigest(resolve(f.root, name)));
  // Act
  assert.deepEqual(await closeUnissued(f.root, f.input, f.operations), { result: "unissued", outcome: "continue_after_manual_remediation", ordinal: 16, hardware_pass: false, device_effects: false });
  const receipt = await readUnissued(f.closure, f.operations);
  // Assert
  assert.equal(receipt.first_failure.details.browser, "connect_failed"); assert.equal(receipt.final_sequence, 2);
  for (const [name, hash] of before) assert.equal(await fileDigest(resolve(f.root, name)), hash);
  assert.equal(receipt.inventory.length, before.size); assert(!receipt.inventory.some(row => row.path === "unissued-closure.json"));
  await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "private_path_exists" });
});

test("issued, consumed, completed or cadence-started preparations cannot be closed unissued", async t => {
  // Arrange / Act / Assert
  for (const name of ["issued.json", "consumed.json", "result.json", "cadence-idle-arm.json", "cadence-usb-arm.json", "cadence-mining-arm.json", "cadence-observer.jsonl", "cadence-idle.json"]) {
    const f = await fixture(t); await writeNew(resolve(f.root, name), { fixture: true });
    await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "private_path_exists" });
    assert(!(await readdir(f.root)).includes("unissued-closure.json"));
  }
});

test("funded device observation rejects an unissued claim even when issuance files are absent", async t => {
  // Arrange
  const f = await fixture(t);
  f.records.at(-1).state.qualification = { schema: "worker-qualification-v1", generation: 1, active_ms: 0, generation_elapsed_ms: 0,
    budget_reserved_ms: 240000, submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 0, last_valid_heartbeat_ms: 0,
    budget_complete: true, safe_stop_complete: true, voltage_fresh: false, power_fresh: false, temperature_fresh: false, fan_fresh: false,
    watchdog_alive: true, mine_on_boot: false, voltage_volts: null, power_watts: null, chip_temp_celsius: null, fan_rpm: null,
    gate_closed_ms: null, shutdown_started_ms: null, safe_stop_stage: "not_started", revocation_reason: "none",
    active_limit_ms: 180000, shutdown_budget_ms: 15550, work_gate_remaining_ms: 0,
    attempt: { schema: "worker-qualification-observation-v1", ordinal: 16, purpose: "normal", maximum_active_ms: 180000, reserved_ms: 180000, complete: true, active_ms: 0 } };
  await f.writeRecords();
  // Act / Assert
  await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "cadence_unissued_work_observed" });
  assert(!(await readdir(f.root)).includes("unissued-closure.json"));
});

test("reserved or advanced accounting cannot masquerade as unused ordinal", async t => {
  // Arrange / Act / Assert
  for (const change of [{ pending: true }, { next_ordinal: 17 }, { total_charged_ms: 1380000 }, { last_completed_ordinal: 14 }]) {
    const f = await fixture(t); Object.assign(f.input.ledger, change);
    await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "iterative_ledger_admission" });
  }
  const f = await fixture(t); f.input.original_budget.charged_ms++;
  await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "original_budget_not_complete" });
});

test("unclean parent observations and nonbaseline or running journal fail before closure", async t => {
  // Arrange / Act / Assert
  for (const key of ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"]) {
    const f = await fixture(t); f.input.cleanup[key] = false;
    await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "cadence_unissued_cleanup" });
  }
  for (const change of [{ running: true }, { deviceBaselineConfirmed: false }, { connected: true }, { serialOwnershipReleased: false }]) {
    const f = await fixture(t); Object.assign(f.records.at(-1).state, change); await f.writeRecords();
    await assert.rejects(closeUnissued(f.root, f.input, f.operations));
  }
});

test("changed first failure, artifact proof or insecure file modes cannot be sealed", async t => {
  // Arrange / Act / Assert
  const f = await fixture(t); await writeFile(resolve(f.root, "first-failure.json"), "{}");
  await assert.rejects(closeUnissued(f.root, f.input, f.operations), { code: "cadence_unissued_first_failure" });
  const g = await fixture(t); g.operations.verifyArtifactSnapshot = async () => { throw new Error("snapshot rejected"); };
  await assert.rejects(closeUnissued(g.root, g.input, g.operations), /snapshot rejected/u);
  const h = await fixture(t); await chmod(resolve(h.root, "first-failure.json"), 0o644);
  await assert.rejects(closeUnissued(h.root, h.input, h.operations), { code: "private_path_policy" });
});

test("closure hash, original inventory contents and membership are revalidated", async t => {
  // Arrange / Act / Assert
  for (const mode of ["closure", "file", "added", "removed", "directory_mode"]) {
    const f = await fixture(t); await closeUnissued(f.root, f.input, f.operations);
    if (mode === "closure") { const saved = await readJson(f.closure); saved.receipt.hardware_pass = true; await writeFile(f.closure, JSON.stringify(saved)); }
    if (mode === "file") await writeFile(resolve(f.root, "first-failure.json"), "{}");
    if (mode === "added") await writeNew(resolve(f.root, "extra.json"), {});
    if (mode === "removed") await rm(resolve(f.root, "first-failure.json"));
    if (mode === "directory_mode") await chmod(f.root, 0o755);
    await assert.rejects(readUnissued(f.closure, f.operations));
  }
});
