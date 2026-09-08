import test from "node:test";
import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { tmpdir } from "node:os";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { createUnreservedContinuation, validateUnreservedContinuation } from "./unreserved.mjs";
const ID = Buffer.alloc(16, 1).toString("base64url");
const ledger = { schema: "worker-qualification-ledger-v1", next_ordinal: 12, total_charged_ms: 1080000, pending: false, last_completed_ordinal: 11 };
function state(context) {
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "closed", connected: false, running: false, heartbeatSuppressed: false, renewalsConfirmed: 0,
    deviceRestorationConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: true,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: ID, device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false },
    qualification: { schema: "worker-qualification-v1", generation: 3, active_ms: 59069, generation_elapsed_ms: 61000, budget_reserved_ms: 240000,
      budget_complete: true, active_limit_ms: 180000, shutdown_budget_ms: 15550, work_gate_remaining_ms: 0, submitted: 5, accepted: 5, rejected: 0,
      nonce_work_correlations: 5, work_dispatched: 26, last_valid_heartbeat_ms: 59000, gate_closed_ms: 59200, shutdown_started_ms: 59300,
      safe_stop_complete: true, safe_stop_stage: "fan_paused", revocation_reason: "restoration_requested", voltage_volts: 5, power_watts: 0,
      chip_temp_celsius: 30, fan_rpm: 1500, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false,
      attempt: { schema: "worker-qualification-observation-v1", ordinal: 11, purpose: "normal", maximum_active_ms: 180000, reserved_ms: 180000, complete: true, active_ms: 59069 } } };
}
async function fixture(t) {
  const parent = await realpath(await mkdtemp(resolve(tmpdir(), "unreserved-fixture-"))); await chmod(parent, 0o700);
  t.after(() => rm(parent, { recursive: true, force: true }));
  const origin = resolve(parent, "attempt-012"), root = resolve(parent, "attempt-012-continuation-01"); await mkdir(origin, { mode: 0o700 });
  const context = { schema: "fixed-usb-iterative-context-v3", firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
    firmware_root: parent, gate_root: parent, supervisor_client_sha256: "f".repeat(64), expected_charged_ms: 1080000, suggested_difficulty: 1000, owner_stack_minimum_bytes: 4096,
    previous_receipt: resolve(parent, "previous.json"), progress_path: resolve(parent, "progress.json"), manifest: "old-manifest",
    qualification_attempt: { schema: "worker-qualification-attempt-v1", id: ID, ordinal: 12, purpose: "foreground_loss", maximumActiveMilliseconds: 30000 } };
  await writeNew(context.previous_receipt, {}); await writeNew(context.progress_path, {});
  const finalState = state(context), failed = { ...finalState, status: "failed", failure: "start_failed", serialFailureCategory: "command_rejected" };
  const hash = digest(JSON.stringify(context));
  await writeNew(resolve(origin, "context.json"), { context, sha256: hash });
  await writeNew(resolve(origin, "issued.json"), { schema: "worker-iterative-issuance-v1", context_sha256: hash, ordinal: 12, ledger_before: ledger, private_payload_persisted: false });
  await writeNew(resolve(origin, "consumed.json"), { ordinal: 12, delivery_attempted: true });
  await writeNew(resolve(origin, "first-failure.json"), { schema: "worker-iterative-first-failure-v1", ordinal: 12, sequence: 1, browser: "start_failed", serial: "command_rejected", admission: null });
  await writeFile(resolve(origin, "iterative.samples.jsonl"), [failed, finalState].map((state, index) => JSON.stringify({ sequence: index + 1, state }) + "\n").join(""), { mode: 0o600 });
  await writeNew(resolve(origin, "artifact-snapshot.json"), {});
  await writeNew(resolve(origin, "cooling.json"), {});
  for (let cycle = 1; cycle <= 4; cycle += 1) await writeNew(resolve(origin, `cycle-${cycle}.json`), { schema: "fixed-usb-cycle-report-v1", cycle,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, baseline_id: ID, browser_released: true, flash_success: true,
    runtime_identity_match: true, cleanup_complete: true, device_identity_match: true, settings_match: true, authorization_high_water_match: true,
    probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false });
  const diagnosticPath = resolve(origin, "diagnostic.json");
  await writeNew(diagnosticPath, { schema: "fixed-usb-diagnostic-export-v1", context_sha256: hash, hardware_authority: false,
    export: { schema: "worker-diagnostic-export-v1", observations: [{ category: "control_failure", authoritative: false, error: "admission_required" }] } });
  const flags = { browser_connection_closed: true, serial_nodes_released: true, supervisor_reaped: true, listener_released: true, restoration_confirmed: true };
  const observationPath = resolve(origin, "unreserved-recovery-observation.json");
  await writeNew(observationPath, { ...flags, next_ordinal: 12, total_charged_ms: 1080000, pending: false, last_completed_ordinal: 11, fault_executed: false });
  const cleanupPath = resolve(origin, "cleanup.json");
  await writeNew(cleanupPath, { schema: "worker-unreserved-cleanup-v1", context_sha256: hash, ...flags, source_observation_sha256: await fileDigest(observationPath) });
  const input = resolve(origin, "input.json");
  await writeNew(input, { ledger_after: ledger, final_state: finalState, diagnostic_export_path: diagnosticPath, cleanup_evidence_path: cleanupPath, progress_evidence_sha256: ["d".repeat(64)] });
  const prior = { result: "passed", context: { ...context, qualification_attempt: { ...context.qualification_attempt, purpose: "normal", ordinal: 11 } }, final_state: finalState,
    original_budget: { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false } };
  const operations = { validateContext: async () => {}, readPrevious: async () => prior, verifySnapshot: async () => {},
    validateDiagnostics: async (value) => value, checkDriver: () => {},
    inspectSources: async () => ({ snapshot: {}, sourceContext: context }), copyArtifacts: async () => {} };
  return { origin, root, context, input, operations, options: { originRoot: origin, privateRoot: root, input, qualificationSourceCommit: "e".repeat(40) } };
}
test("unreserved continuation preserves originals and identical allowance tuple", async (t) => {
  const f = await fixture(t), before = new Map();
  for (const name of await readdir(f.origin)) before.set(name, await fileDigest(resolve(f.origin, name)));
  const result = await createUnreservedContinuation(f.options, f.operations);
  assert.equal(result.status, "unverified_not_reserved"); assert.equal(result.charge_added_ms, 0); assert.equal(result.refund_ms, 0);
  for (const [name, sha] of before) assert.equal(await fileDigest(resolve(f.origin, name)), sha);
  const { context } = await readJson(resolve(f.root, "context.json"));
  assert.deepEqual(context.qualification_attempt, f.context.qualification_attempt);
  await validateUnreservedContinuation(f.root, context, f.operations);
  await assert.rejects(createUnreservedContinuation({ ...f.options, privateRoot: resolve(f.root, "../another") }, f.operations));
});
test("advanced, pending or unsafe recovery cannot create continuation", async (t) => {
  const f = await fixture(t), original = await readJson(f.input);
  for (const mutation of [{ ledger_after: { ...ledger, next_ordinal: 13 } }, { ledger_after: { ...ledger, pending: true } },
    { final_state: { ...original.final_state, running: true } }, { final_state: { ...original.final_state, deviceRestorationConfirmed: false } }]) {
    await writeFile(f.input, JSON.stringify({ ...original, ...mutation }));
    await assert.rejects(createUnreservedContinuation(f.options, f.operations));
    await assert.rejects(readFile(resolve(f.origin, "unreserved-continuation.json")), { code: "ENOENT" });
  }
});
test("fault, current ordinal evidence and wrong rejection are never treated as expiry", async (t) => {
  const f = await fixture(t);
  const fault = resolve(f.origin, "iterative.fault.json"); await writeNew(fault, {});
  await assert.rejects(createUnreservedContinuation(f.options, f.operations)); await rm(fault);
  const file = resolve(f.origin, "first-failure.json"), failure = await readJson(file);
  await writeFile(file, JSON.stringify({ ...failure, serial: "timeout" }));
  await assert.rejects(createUnreservedContinuation(f.options, f.operations), /first_failure/u);
});
test("changed source evidence and changed continuation identity invalidate lineage", async (t) => {
  const f = await fixture(t); await createUnreservedContinuation(f.options, f.operations);
  const { context } = await readJson(resolve(f.root, "context.json"));
  await assert.rejects(validateUnreservedContinuation(f.root, { ...context, suggested_difficulty: 0 }, f.operations), /context_changed/u);
  await writeFile(resolve(f.origin, "consumed.json"), '{"ordinal":12,"delivery_attempted":true,"extra":true}');
  await assert.rejects(validateUnreservedContinuation(f.root, context, f.operations), /evidence_changed/u);
});

test("current-ordinal activity, missing expiry diagnostic and unproved cleanup reject", async (t) => {
  const f = await fixture(t), input = await readJson(f.input), journal = resolve(f.origin, "iterative.samples.jsonl");
  const bytes = await readFile(journal), rows = bytes.toString().trim().split("\n").map(JSON.parse);
  rows[0].state.qualification.attempt.ordinal = 12;
  await writeFile(journal, rows.map(row => JSON.stringify(row) + "\n").join(""));
  await assert.rejects(createUnreservedContinuation(f.options, f.operations), /work_observed/u);
  await writeFile(journal, bytes);
  const diagnostic = await readJson(input.diagnostic_export_path);
  diagnostic.export.observations[0].error = "invalid_request";
  await writeFile(input.diagnostic_export_path, JSON.stringify(diagnostic));
  await assert.rejects(createUnreservedContinuation(f.options, f.operations), /admission_required_missing/u);
  diagnostic.export.observations[0].error = "admission_required";
  await writeFile(input.diagnostic_export_path, JSON.stringify(diagnostic));
  const cleanup = await readJson(input.cleanup_evidence_path); cleanup.supervisor_reaped = false;
  await writeFile(input.cleanup_evidence_path, JSON.stringify(cleanup));
  await assert.rejects(createUnreservedContinuation(f.options, f.operations), /cleanup_evidence/u);
});
test("nested origin and changed retained runtime cannot create a continuation", async (t) => {
  const f = await fixture(t);
  await assert.rejects(createUnreservedContinuation(f.options, { ...f.operations,
    inspectSources: async () => ({ sourceContext: f.context, snapshot: { firmware_commit: "f".repeat(40) } }) }), /context_changed/u);
  await assert.rejects(readFile(resolve(f.origin, "unreserved-rejection.json")), { code: "ENOENT" });
  const changed = { ...f.context, unreserved_continuation: {} };
  await writeFile(resolve(f.origin, "context.json"), JSON.stringify({ context: changed, sha256: digest(JSON.stringify(changed)) }));
  await assert.rejects(createUnreservedContinuation(f.options, f.operations), /origin_context/u);
});
test("identity drift, different retired generation and driver snapshot changes are denied", async (t) => {
  const f = await fixture(t), input = await readJson(f.input);
  for (const mutation of [
    { expectedAppElfSha256: "a".repeat(64) },
    { preservation: { ...input.final_state.preservation, device_identity_match: false } },
    { qualification: { ...input.final_state.qualification, generation: 4 } },
  ]) {
    await writeFile(f.input, JSON.stringify({ ...input, final_state: { ...input.final_state, ...mutation } }));
    await assert.rejects(createUnreservedContinuation(f.options, f.operations));
  }
});

test("historical lineage does not require a live driver checkout or active task", async (t) => {
  const f = await fixture(t);
  await createUnreservedContinuation(f.options, f.operations);
  const { context } = await readJson(resolve(f.root, "context.json"));
  const historical = { ...f.operations,
    checkDriver: () => { throw new Error("live source changed"); },
    validateContext: async (_root, _context, options) => { assert.equal(options.historical, true); },
  };
  await validateUnreservedContinuation(f.root, context, historical);
  // Creator still requires current task admission and retained-source verification.
  const fresh = await fixture(t);
  await assert.rejects(createUnreservedContinuation(fresh.options, { ...fresh.operations,
    validateContext: async (_root, _context, options) => { assert.equal(options.historical, false); throw new Error("inactive task"); },
  }), /inactive task/u);
  await assert.rejects(createUnreservedContinuation(fresh.options, { ...fresh.operations,
    inspectSources: async () => { throw new Error("unpublished driver"); },
  }), /unpublished driver/u);
});
test("continuation cannot replace its frozen client hash or other original fields", async (t) => {
  const f = await fixture(t);
  await createUnreservedContinuation(f.options, f.operations);
  const { context } = await readJson(resolve(f.root, "context.json"));
  for (const mutation of [{ supervisor_client_sha256: "0".repeat(64) }, { expected_charged_ms: 0 }, { progress_path: "/changed" }]) {
    await assert.rejects(validateUnreservedContinuation(f.root, { ...context, ...mutation }, f.operations), /context_changed/u);
  }
});

test("sealed history is independent of a later unavailable Gate diagnostic validator", async (t) => {
  const f = await fixture(t);
  let validations = 0;
  await createUnreservedContinuation(f.options, { ...f.operations, validateDiagnostics: async (value) => { validations += 1; return value; } });
  assert.equal(validations, 1);
  const { context } = await readJson(resolve(f.root, "context.json"));
  const history = { ...f.operations, validateDiagnostics: async () => { throw new Error("old Gate parser no longer available"); } };
  await validateUnreservedContinuation(f.root, context, history);
  const input = await readJson(f.input), diagnostic = await readFile(input.diagnostic_export_path);
  await writeFile(input.diagnostic_export_path, Buffer.concat([diagnostic, Buffer.from("\n")]));
  await assert.rejects(validateUnreservedContinuation(f.root, context, history), /evidence_changed/u);
});
