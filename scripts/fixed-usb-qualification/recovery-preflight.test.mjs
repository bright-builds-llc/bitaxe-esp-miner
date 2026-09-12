import test from "node:test";
import assert from "node:assert/strict";
import { generateKeyPairSync } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { BUNDLE, digest, PAGE, readJson, writeNew } from "./contract.mjs";
import { verifyFrozen } from "./preflight.mjs";
import { recoveryPreflight, requireIterativeTask, requireReleasedState, validateRecoveryTransition, validateIterativePolicy } from "./iterative-preflight.mjs";
import { RECOVERY_SCHEMA } from "./recovery-judge.mjs";
import { requireSuccessorBaseline } from "./successor.mjs";
import { main } from "./main.mjs";

const FIRMWARE = "d".repeat(40), GATE = "e".repeat(40), ID = Buffer.alloc(16, 7).toString("base64url");
function publicTrust() {
  const key = generateKeyPairSync("ed25519").publicKey.export({ format: "jwk" });
  const keys = [{ kid: "fixture", kty: "OKP", crv: "Ed25519", x: key.x, alg: "Ed25519", use: "sig", key_ops: ["verify"] }];
  return { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "fixture-update", audience: "fixture-update", keys },
    workLeaseAuthority: { issuer: "fixture-work", audience: "fixture-work", keys } };
}
async function fixture(t, phase = "loss") {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "recovery-preflight-fixture-")));
  await chmod(base, 0o700); t.after(() => rm(base, { recursive: true, force: true }));
  const firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), authorityDirectory = resolve(base, "authority"), parent = resolve(base, "iterative");
  for (const path of [firmwareRoot, gateRoot, authorityDirectory, parent]) await mkdir(path, { mode: 0o700 });
  await writeFile(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-fixed-usb-hello-resynchronization | fixture\n");
  const trust = publicTrust(), trustPath = resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json");
  await mkdir(dirname(trustPath), { recursive: true }); await writeFile(trustPath, JSON.stringify(trust));
  for (const path of [PAGE, BUNDLE]) await mkdir(dirname(resolve(gateRoot, path)), { recursive: true });
  await writeFile(resolve(gateRoot, PAGE), "<html>Fixture</html>"); await writeFile(resolve(gateRoot, BUNDLE), `const commit='${GATE}';`);
  const manifest = resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json");
  await mkdir(dirname(manifest), { recursive: true });
  const kinds = ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"];
  const artifacts = [];
  for (const kind of kinds) {
    const bytes = Buffer.from(`fixture-${kind}`), path = `${kind}.bin`;
    await writeFile(resolve(kind === "partition_table" ? firmwareRoot : dirname(manifest), path), bytes);
    artifacts.push({ kind, path, sha256: digest(bytes) });
  }
  const update_segments = [["bootloader", 0], ["partition_table_binary", 0x8000], ["firmware_ota_image", 0x10000], ["www_spiffs_image", 0x410000], ["otadata_initial", 0xf10000]]
    .map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`fixture-${artifact_kind}`) }));
  const packageValue = { schema_version: 4, source_commit: FIRMWARE, reference_commit: "f".repeat(40), build_identity: { source_dirty: false },
    app_elf_sha256: artifacts[0].sha256, artifacts, update_segments };
  await writeFile(manifest, JSON.stringify(packageValue));
  const previousRoot = resolve(parent, "previous"), previousReceipt = resolve(previousRoot, "result.json");
  await mkdir(previousRoot, { mode: 0o700 });
  const previousContext = phase === "loss" ? { schema: "fixed-usb-iterative-context-v3", firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64), qualification_attempt: { purpose: "heartbeat_loss" } } : {
    schema: RECOVERY_SCHEMA, recovery_phase: "loss", firmware_commit: FIRMWARE, gate_commit: GATE, app_elf_sha256: packageValue.app_elf_sha256,
    qualification_attempt: { purpose: "diagnostic" } };
  const previous = { schema: "worker-iterative-result-v2", result: "passed", cleanup_confirmed: true, context: previousContext,
    context_sha256: digest(JSON.stringify(previousContext)), original_campaign_id: ID, next_ordinal: phase === "loss" ? 14 : 15,
    total_charged_ms: phase === "loss" ? 1140000 : 1170000, ...(phase === "resume" ? { judgment: { generation: 37, recovery_phase: "loss" } } : {}) };
  await writeNew(previousReceipt, { receipt: previous, sha256: digest(JSON.stringify(previous)) });
  await writeNew(resolve(previousRoot, "context.json"), { context: previousContext, sha256: previous.context_sha256 });
  const input = resolve(base, "progress.json");
  await writeNew(input, { schema: "worker-qualification-progress-v1", review: "verified", reason: phase === "loss" ? "software_correction" : "next_acceptance_window", evidence_sha256: ["9".repeat(64)] });
  const options = { firmwareRoot, gateRoot, authorityDirectory, manifest, firmwareCommit: FIRMWARE, gateCommit: GATE,
    privateRoot: resolve(parent, "new-attempt"), previousReceipt, recoveryPhase: phase, suggestedDifficulty: "1000", input };
  const calls = [];
  // Historical receipt validation is its existing independently tested boundary. This fixture supplies its admitted result.
  const operations = { ignored: () => undefined, cleanPushed: (root, commit) => calls.push([root, commit]),
    authorityCall: async (_gate, _directory, operation) => { assert.equal(operation, "public-trust"); return trust; },
    readPrevious: async path => { const record = await readJson(path); assert.equal(record.sha256, digest(JSON.stringify(record.receipt))); return record.receipt; } };
  return { base, parent, previousRoot, options, previous, operations, packageValue, calls };
}
function cycle(context, number) {
  return { schema: "fixed-usb-cycle-report-v1", cycle: number, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
    baseline_id: ID, browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
    device_identity_match: true, settings_match: true, authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false };
}
function baseline(context, closed = false) {
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: closed ? "closed" : "ready", connected: !closed, running: false, heartbeatSuppressed: false,
    renewalsConfirmed: 0, deviceBaselineConfirmed: true, deviceRestorationConfirmed: false, deviceLeaseInactive: true, serialOwnershipReleased: closed,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: ID, device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false } };
}
async function assertNoAttempt(f) { assert(!(await readdir(f.parent)).includes("new-attempt")); }

test("loss preflight binds the existing ledger, exact eight artifacts and current client with only the Hello task active", async t => {
  // Arrange
  const f = await fixture(t), original = await readFile(f.options.previousReceipt, "utf8");
  // Act
  const result = await recoveryPreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  // Assert
  assert.equal(result.ordinal, 14); assert.equal(result.allowance_reserved_on_device, false);
  assert.equal(context.schema, RECOVERY_SCHEMA); assert.equal(context.recovery_phase, "loss");
  assert.equal(context.qualification_attempt.purpose, "diagnostic"); assert.equal(context.qualification_attempt.maximumActiveMilliseconds, 30000);
  assert.equal(context.expected_charged_ms, 1140000); assert.equal(context.artifacts.length, 8);
  assert.equal(context.cycle_source, undefined); assert.equal(context.original_campaign_id, ID);
  assert.equal(context.supervisor_client_sha256, digest(await readFile(new URL("./client.mjs", import.meta.url))));
  assert.deepEqual(f.calls, [[f.options.firmwareRoot, FIRMWARE], [f.options.gateRoot, GATE]]);
  assert.equal(await readFile(f.options.previousReceipt, "utf8"), original);
  assert(!(await readdir(f.parent)).some(name => ["campaign.json", "bootstrap.json"].includes(name)));
});
test("resume copies only the passed loss pair's four immutable cycles and advances one ordinal", async t => {
  const f = await fixture(t, "resume");
  for (let n = 1; n <= 4; n++) await writeNew(resolve(f.previousRoot, `cycle-${n}.json`), cycle(f.previous.context, n));
  const result = await recoveryPreflight(f.options, f.operations), { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  assert.equal(result.ordinal, 15); assert.equal(context.expected_charged_ms, 1170000); assert.equal(context.recovery_phase, "resume");
  assert.equal(context.cycle_source.root, f.previousRoot);
  assert.equal(context.recovery_loss_generation, 37);
  for (let n = 1; n <= 4; n++) assert.deepEqual(await readFile(resolve(f.options.privateRoot, `cycle-${n}.json`)), await readFile(resolve(f.previousRoot, `cycle-${n}.json`)));
});
test("loss refuses bootstrap or a different ledger parent before creating a child", async t => {
  const f = await fixture(t);
  await assert.rejects(recoveryPreflight({ ...f.options, previousReceipt: resolve(f.parent, "bootstrap.json") }, f.operations), /recovery_existing_chain_required/u);
  await assertNoAttempt(f);
  f.previous.schema = "worker-iterative-bootstrap-v1";
  await writeFile(f.options.previousReceipt, JSON.stringify({ receipt: f.previous, sha256: digest(JSON.stringify(f.previous)) }));
  await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_completed_predecessor/u);
  await assertNoAttempt(f);
});
test("loss refuses unchanged runtime identity or old cycle reuse", async t => {
  const f = await fixture(t);
  await assert.rejects(recoveryPreflight({ ...f.options, cyclesFrom: f.previousRoot }, f.operations), /recovery_cycle_source/u);
  Object.assign(f.previous.context, { firmware_commit: FIRMWARE, gate_commit: GATE, app_elf_sha256: f.packageValue.app_elf_sha256 });
  await writeFile(f.options.previousReceipt, JSON.stringify({ receipt: f.previous, sha256: digest(JSON.stringify(f.previous)) }));
  await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_loss_progress/u);
  await assertNoAttempt(f);
});
test("resume rejects an unverified loss or a changed runtime before copying cycles", async t => {
  const f = await fixture(t, "resume");
  f.previous.result = "unverified";
  await writeFile(f.options.previousReceipt, JSON.stringify({ receipt: f.previous, sha256: digest(JSON.stringify(f.previous)) }));
  await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_resume_progress/u);
  f.previous.result = "passed"; f.previous.context.gate_commit = "b".repeat(40);
  await writeFile(f.options.previousReceipt, JSON.stringify({ receipt: f.previous, sha256: digest(JSON.stringify(f.previous)) }));
  await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_resume_progress/u);
  await assertNoAttempt(f);
});
test("missing reused cycles or an occupied ordinal cannot allocate a new recovery context", async t => {
  const f = await fixture(t, "resume");
  await assert.rejects(recoveryPreflight(f.options, f.operations), /ENOENT/u);
  for (let n = 1; n <= 4; n++) await writeNew(resolve(f.previousRoot, `cycle-${n}.json`), cycle(f.previous.context, n));
  await writeNew(resolve(f.parent, "ordinal-15.json"), { occupied: true });
  await assert.rejects(recoveryPreflight(f.options, f.operations), /private_path_exists/u);
  await assertNoAttempt(f);
});
test("recovery requires the active Hello task and does not revive the archived preparation task", async t => {
  const f = await fixture(t);
  await assert.rejects(requireIterativeTask(f.options.firmwareRoot), /iterative_active_task_required/u);
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-fixed-usb-hello-resynchronization | fixture\n");
  await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_active_task_required/u);
  await assertNoAttempt(f);
});
test("v5 source verification detects public runtime drift while freezing client and authority trust", async t => {
  const f = await fixture(t); await recoveryPreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  await verifyFrozen(context, f.options.authorityDirectory, undefined, f.operations, f.options.privateRoot);
  await writeFile(resolve(f.options.gateRoot, BUNDLE), `changed bundle '${GATE}'`);
  await assert.rejects(verifyFrozen(context, f.options.authorityDirectory, undefined, f.operations, f.options.privateRoot), /frozen_source_drift/u);
});
test("cold baseline is admitted only for v5 initial readiness while final restoration remains required", async t => {
  const f = await fixture(t); await recoveryPreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  for (let n = 1; n <= 4; n++) await writeNew(resolve(f.options.privateRoot, `cycle-${n}.json`), cycle(context, n));
  await requireSuccessorBaseline(f.options.privateRoot, context, baseline(context));
  await assert.rejects(requireSuccessorBaseline(f.options.privateRoot, { ...context, schema: "fixed-usb-iterative-context-v3" }, baseline(context)), /successor_baseline/u);
  assert.throws(() => requireReleasedState(baseline(context, true), context), /iterative_cleanup_required/u);
  requireReleasedState({ ...baseline(context, true), deviceRestorationConfirmed: true }, context);
});
test("v5 policy excludes other purposes, fewer cycles and bootstrap-era progression", async () => {
  const context = { schema: RECOVERY_SCHEMA, recovery_phase: "loss", qualification_attempt: { purpose: "diagnostic", maximumActiveMilliseconds: 30000 },
    owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000, required_no_mining_cycles: 4 };
  assert.equal(validateIterativePolicy(context, true), true);
  assert.throws(() => validateIterativePolicy({ ...context, required_no_mining_cycles: 3 }), /recovery_cycle_policy/u);
  assert.throws(() => validateIterativePolicy({ ...context, qualification_attempt: { purpose: "normal", maximumActiveMilliseconds: 180000 } }), /recovery_policy/u);
  assert.throws(() => validateRecoveryTransition("loss", { schema: "worker-iterative-bootstrap-v1" }, {}, {}), /recovery_completed_predecessor/u);
});
test("recovery command rejects unrelated legacy authorization arguments before filesystem access", async () => {
  for (const flag of ["--purpose", "--retained-runtime-from", "--qualification-source-commit", "--pool-credentials"]) {
    await assert.rejects(main(["recovery-preflight", "--private-root", "/absent", flag, "forbidden"]), /command_arguments/u);
  }
  await assert.rejects(main(["recovery-preflight", "--private-root", "/absent"]), /preflight_argument_missing/u);
});

test("resume rejects a missing or zero prior loss generation before allocating the new ordinal", async t => {
  const f = await fixture(t, "resume");
  for (const maybeGeneration of [undefined, 0]) {
    f.previous.judgment = maybeGeneration === undefined ? {} : { generation: maybeGeneration };
    await writeFile(f.options.previousReceipt, JSON.stringify({ receipt: f.previous, sha256: digest(JSON.stringify(f.previous)) }));
    await assert.rejects(recoveryPreflight(f.options, f.operations), /recovery_loss_generation_missing/u);
    await assertNoAttempt(f);
  }
});
