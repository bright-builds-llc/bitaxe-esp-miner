import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { generateKeyPairSync } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { BUNDLE, digest, PAGE, writeNew } from "./contract.mjs";
import { loadContext } from "./preflight.mjs";
import { noMiningPreflight, inspectNoMiningSources, verifyNoMiningFrozen } from "./no-mining-context.mjs";
import { createNoMiningSupervisor } from "./no-mining-server.mjs";
import { readFrozenCampaign, saveNoMiningAccounting } from "./no-mining-accounting.mjs";
import { finishNoMining, validateNoMiningReview } from "./no-mining-judge.mjs";
import { validateNoMiningReadOnlyInterruption } from "./no-mining-interruption.mjs";
import { judgeWindow } from "./judge.mjs";
import { recordCycle } from "./store.mjs";
import { main } from "./main.mjs";

const SOURCE = "a".repeat(40), GATE = "b".repeat(40), REFERENCE = "c".repeat(40);
const BASELINE = Buffer.alloc(16, 3).toString("base64url");
const BINDING = Buffer.alloc(32, 4).toString("base64url");
function publicTrust() {
  const key = generateKeyPairSync("ed25519").publicKey.export({ format: "jwk" });
  const keys = [{ kid: "fixture-key", kty: "OKP", crv: "Ed25519", x: key.x, alg: "Ed25519", use: "sig", key_ops: ["verify"] }];
  return { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "fixture-update", audience: "bwg-reference-firmware-capability/0.2", role: "update_authority", keys },
    workLeaseAuthority: { issuer: "fixture-lease", audience: "bwg-worker-controller/0.4", role: "work_lease_authority", keys } };
}
async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "fixed-usb-fixture-")));
  t.after(() => rm(base, { recursive: true, force: true }));
  await chmod(base, 0o700);
  const firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), authorityDirectory = resolve(base, "authority");
  await Promise.all([mkdir(firmwareRoot), mkdir(gateRoot), mkdir(authorityDirectory, { mode: 0o700 })]);
  const manifest = resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json");
  await mkdir(dirname(manifest), { recursive: true });
  await mkdir(resolve(firmwareRoot, "firmware/bitaxe/bwg"), { recursive: true });
  const trust = publicTrust();
  await writeFile(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), JSON.stringify(trust));
  await writeFile(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-fixed-usb-serial-qualification | fixture\n### task-fixed-usb-worker-live-acceptance | fixture\n### task-fixed-usb-hello-resynchronization | fixture\n");
  for (const file of [PAGE, BUNDLE]) await mkdir(dirname(resolve(gateRoot, file)), { recursive: true });
  await writeFile(resolve(gateRoot, PAGE), "<!doctype html><html><body>Fixture page without device code</body></html>");
  await writeFile(resolve(gateRoot, BUNDLE), `const fixtureCommit = '${GATE}';`);
  const artifacts = [];
  for (const kind of ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"]) {
    const bytes = Buffer.from(`fixture-${kind}`);
    const path = kind === "partition_table" ? "firmware/bitaxe/partitions-ultra205.csv" : `${kind}.bin`;
    await writeFile(resolve(kind === "partition_table" ? firmwareRoot : dirname(manifest), path), bytes);
    artifacts.push({ kind, path, sha256: digest(bytes) });
  }
  const update_segments = [["bootloader", 0], ["partition_table_binary", 0x8000], ["firmware_ota_image", 0x10000], ["www_spiffs_image", 0x410000], ["otadata_initial", 0xf10000]]
    .map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`fixture-${artifact_kind}`) }));
  const manifestValue = { schema_version: 4, source_commit: SOURCE, reference_commit: REFERENCE, build_identity: { source_dirty: false },
    app_elf_sha256: artifacts[0].sha256, artifacts, update_segments };
  await writeFile(manifest, JSON.stringify(manifestValue));
  const options = { firmwareRoot, gateRoot, authorityDirectory, manifest, firmwareCommit: SOURCE, gateCommit: GATE, privateRoot: resolve(base, "attempt-001") };
  const operations = { cleanPushed: () => undefined, ignored: () => undefined, authorityCall: async () => trust };
  delete options.authorityDirectory;
  return { base, options, operations, trust, manifestValue };
}
function preservation() {
  return { schema: "worker-preservation-continuity-v1", baseline_id: BASELINE, device_identity_match: true,
    settings_match: true, authorization_high_water_match: true, mine_on_boot: false };
}
function state(context, extra = {}) {
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "ready", connected: true, running: false, heartbeatSuppressed: false, renewalsConfirmed: 0,
    deviceBaselineConfirmed: true, deviceRestorationConfirmed: false, deviceLeaseInactive: true, serialOwnershipReleased: false, preservation: preservation(), ...extra };
}
function cycle(context, index, extra = {}) {
  return { schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
    baseline_id: BASELINE, device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false,
    browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true, probe_request_bytes: 65536, probe_response_bytes: 65536, ...extra };
}
async function prepared(t, withCampaign = false) {
  const f = await fixture(t);
  if (withCampaign) {
    f.options.originalCampaignRecord = resolve(f.base, "original-campaign.json");
    await writeNew(f.options.originalCampaignRecord, { schema: "fixed-usb-campaign-v1", campaign_id: BASELINE });
  }
  await noMiningPreflight(f.options, f.operations);
  return { ...f, context: await loadContext(f.options.privateRoot) };
}
async function serving(t, withCampaign = false) {
  const f = await prepared(t, withCampaign);
  const server = await createNoMiningSupervisor({ ...f.options, context: f.context }, {
    verifyFrozen: () => verifyNoMiningFrozen(f.context, f.operations) });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(async () => { server.closeAllConnections(); await new Promise((done) => server.close(done)); });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const request = (route, maybeBody, requestOrigin = origin) => fetch(`${origin}${route}`, {
    method: maybeBody === undefined ? "GET" : "POST", headers: { Origin: requestOrigin, "Content-Type": "application/json" },
    body: maybeBody === undefined ? undefined : JSON.stringify(maybeBody) });
  return { ...f, request };
}
function review(context) {
  const ledger = { schema: "worker-qualification-ledger-v1", next_ordinal: 5, total_charged_ms: 300000, pending: false, last_completed_ordinal: 4 };
  const original = { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false };
  return { schema: "fixed-usb-no-mining-review-v1", ledger_before: ledger, ledger_after: { ...ledger },
    original_budget_before: original, original_budget_after: { ...original },
    recovery_before: state(context), recovery_after: state(context, { helloRecovery: { discardedRecords: 2, discardedBytes: 512, discardedReplies: 1 } }),
    final_state: state(context, { connected: false, serialOwnershipReleased: true, status: "closed" }),
    recovery_without_drain: true, cleanup_complete: true };
}

async function appendState(f, observedState, sequence) {
  await writeNew(resolve(f.options.privateRoot, `no-mining-state-${String(sequence).padStart(4, "0")}.json`), {
    schema: "fixed-usb-no-mining-state-v1", context_sha256: digest(JSON.stringify(f.context)), sequence, state: observedState });
}
async function recordedReview(f, reviewed, interrupted = true) {
  await appendState(f, reviewed.recovery_before, 1);
  await saveNoMiningAccounting(f.options.privateRoot, f.context, { stage: "before", ledger: reviewed.ledger_before,
    original_budget: reviewed.original_budget_before, state: reviewed.recovery_before });
  if (interrupted) await appendState(f, state(f.context, { connected: false, status: "disconnected", serialOwnershipReleased: true }), 2);
  const afterSequence = interrupted ? 3 : 2;
  await appendState(f, reviewed.recovery_after, afterSequence);
  await saveNoMiningAccounting(f.options.privateRoot, f.context, { stage: "after", ledger: reviewed.ledger_after,
    original_budget: reviewed.original_budget_after, state: reviewed.recovery_after });
  await appendState(f, reviewed.final_state, afterSequence + 1);
}

test("no-mining preflight freezes exact sources and all artifacts without authority or campaign creation", async (t) => {
  // Arrange
  const f = await fixture(t);
  const checked = [];
  // Act
  await noMiningPreflight(f.options, { ...f.operations, cleanPushed: (root, commit) => checked.push([root, commit]),
    authorityCall: () => assert.fail("private authority must not be opened") });
  const context = await loadContext(f.options.privateRoot);
  // Assert
  assert.deepEqual(checked, [[f.options.firmwareRoot, SOURCE], [f.options.gateRoot, GATE]]);
  assert.equal(context.artifacts.length, 8);
  assert.equal(context.mining_authorized, false);
  assert.equal(context.campaign_id, undefined);
  assert.equal(context.window_limits_ms, undefined);
  assert(!(await readdir(f.base)).includes("campaign.json"));
});
test("no-mining admission rejects unpushed sources before creating an attempt", async (t) => {
  const f = await fixture(t);
  await assert.rejects(noMiningPreflight(f.options, { ...f.operations, cleanPushed: () => { throw new Error("source_not_pushed"); } }), /source_not_pushed/u);
  assert(!(await readdir(f.base)).includes("attempt-001"));
});
test("no-mining admission rejects changed package bytes", async (t) => {
  const f = await fixture(t);
  await writeFile(resolve(dirname(f.options.manifest), "firmware_ota_image.bin"), "changed");
  await assert.rejects(noMiningPreflight(f.options, f.operations), /artifact_digest/u);
  assert(!(await readdir(f.base)).includes("attempt-001"));
});
test("no-mining admission requires its one active task", async (t) => {
  const f = await fixture(t);
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-fixed-usb-hello-resynchronization | blocked\n");
  await assert.rejects(inspectNoMiningSources(f.options, f.operations), /active_task_missing/u);
});
test("no-mining server allows possession scope but has no authorization or grant routes", async (t) => {
  // Arrange
  const f = await serving(t);
  for (let index = 1; index <= 4; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  // Act
  const configured = await (await f.request("/context")).json();
  const scope = await (await f.request("/activate", {})).json();
  // Assert
  assert.equal(configured.expectedGateCommit, GATE);
  assert.match(scope.challengeId, /^challenge_/u);
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: BINDING })).status, 404);
  assert.equal((await f.request("/window-artifacts")).status, 404);
  for (const path of ["/advance", "/budget-review-context", "/completion-context", "/cooling-review-context"]) {
    assert.equal((await f.request(path, {})).status, 404);
  }
  assert.deepEqual(await (await f.request("/supervisor-state")).json(), { mode: "no-mining", mining_authorized: false, signing_available: false });
});
test("no-mining server rechecks frozen source before possession", async (t) => {
  const f = await serving(t);
  await writeFile(resolve(f.options.gateRoot, BUNDLE), `changed ${GATE}`);
  const response = await f.request("/activate", {});
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "frozen_source_drift");
});
test("no-mining server rejects cross-origin requests and records closed failure evidence", async (t) => {
  const f = await serving(t);
  assert.equal((await f.request("/activate", {}, "http://untrusted.invalid")).status, 400);
  const failed = state(f.context, { status: "failed", connected: false, failure: "connect_failed", admissionFailureStage: "hello", serialFailureCategory: "timeout" });
  assert.equal((await f.request("/record", { state: failed })).status, 200);
  const saved = JSON.parse(await readFile(resolve(f.options.privateRoot, "no-mining-state-0001.json"), "utf8"));
  assert.equal(saved.state.failure, "connect_failed");
});
test("no-mining recorder rejects work and raw evidence fields", async (t) => {
  const f = await serving(t);
  for (const changed of [{ running: true }, { renewalsConfirmed: 1 }, { status: "window_loaded" }, { raw_serial: "forbidden" }]) {
    assert.equal((await f.request("/record", { state: state(f.context, changed) })).status, 400);
  }
  assert(!(await readdir(f.options.privateRoot)).some((file) => file.startsWith("no-mining-state-")));
});
test("no-mining CLI rejects credential inputs before opening a source or secret", async () => {
  for (const command of ["no-mining-preflight", "no-mining-serve", "no-mining-judge"]) {
    for (const option of ["--authority-directory", "--pool-credentials"]) {
      await assert.rejects(main([command, "--private-root", "/absent", option, "/must-not-open"]), /command_arguments/u);
    }
  }
});
test("no-mining cycle validation rejects changed baseline and wrong maximum payload", async (t) => {
  const f = await prepared(t);
  await recordCycle(f.options.privateRoot, f.context, cycle(f.context, 1));
  await assert.rejects(recordCycle(f.options.privateRoot, f.context, cycle(f.context, 2, { baseline_id: Buffer.alloc(16, 7).toString("base64url") })), /cycle_baseline_changed/u);
  await assert.rejects(recordCycle(f.options.privateRoot, f.context, cycle(f.context, 2, { probe_request_bytes: 65535 })), /cycle_evidence_failed/u);
});
test("no-mining judge requires all four immutable cycles and unchanged reviewed accounting", async (t) => {
  // Arrange
  const f = await prepared(t, true);
  const inputPath = resolve(f.options.privateRoot, "review.json");
  await writeNew(inputPath, review(f.context));
  for (let index = 1; index <= 3; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /ENOENT/u);
  await recordCycle(f.options.privateRoot, f.context, cycle(f.context, 4));
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_accounting_evidence_missing/u);
  const reviewed = review(f.context);
  await recordedReview(f, reviewed);
  // Act
  const result = await finishNoMining(f.options.privateRoot, f.context, inputPath);
  // Assert
  assert.equal(result.no_mining_cycles, 4);
  assert.equal(result.accounting_unchanged, true);
  assert.equal(result.hardware_execution_claimed_by_supervisor, false);
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /EEXIST/u);
});
test("no-mining judge rejects changed ledger, drain recovery and unreleased ownership", async (t) => {
  const f = await prepared(t);
  for (const change of [
    (value) => { value.ledger_after.total_charged_ms += 30000; },
    (value) => { value.ledger_after.pending = true; },
    (value) => { value.recovery_without_drain = false; },
    (value) => { value.recovery_after.helloRecovery.discardedRecords = 0; },
    (value) => { value.recovery_after.helloRecovery.discardedBytes = 66561; },
    (value) => { value.final_state.serialOwnershipReleased = false; },
    (value) => { value.recovery_after.preservation.settings_match = false; },
  ]) {
    const value = review(f.context);
    change(value);
    assert.throws(() => validateNoMiningReview(value, f.context, cycle(f.context, 4)));
  }
});

test("existing campaign record is read-only, digest bound and absent from public context", async (t) => {
  // Arrange
  const f = await serving(t, true);
  const original = await readFile(f.options.originalCampaignRecord, "utf8");
  // Act
  const response = await f.request("/original-budget-context", {});
  const publicContext = await (await f.request("/context")).json();
  // Assert
  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), { campaignId: BASELINE });
  assert.equal(JSON.stringify(publicContext).includes(BASELINE), false);
  assert.equal(JSON.stringify(f.context).includes(BASELINE), false);
  assert.equal(f.context.original_campaign_record.sha256, digest(Buffer.from(original)));
  assert.equal(await readFile(f.options.originalCampaignRecord, "utf8"), original);
  assert.equal((await f.request("/original-budget-context", {}, "http://untrusted.invalid")).status, 400);
});
test("existing campaign record drift is rejected before browser accounting", async (t) => {
  const f = await serving(t, true);
  await writeFile(f.options.originalCampaignRecord, JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: Buffer.alloc(16, 8).toString("base64url") }));
  assert.equal((await f.request("/original-budget-context", {})).status, 400);
  await assert.rejects(readFrozenCampaign(f.context), /original_campaign_record_changed/u);
});
test("missing, unprotected or malformed campaign records cannot create a no-mining attempt", async (t) => {
  const f = await fixture(t);
  f.options.originalCampaignRecord = resolve(f.base, "missing-campaign.json");
  await assert.rejects(noMiningPreflight(f.options, f.operations), /ENOENT/u);
  await writeFile(f.options.originalCampaignRecord, JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: BASELINE }), { mode: 0o644 });
  await assert.rejects(noMiningPreflight(f.options, f.operations), /private_path_policy/u);
  await chmod(f.options.originalCampaignRecord, 0o600);
  await writeFile(f.options.originalCampaignRecord, JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: "invalid" }));
  await assert.rejects(noMiningPreflight(f.options, f.operations), /original_campaign_shape/u);
  assert(!(await readdir(f.base)).includes("attempt-001"));
});
test("accounting routes save immutable closed observations and reject changed charges", async (t) => {
  const f = await serving(t, true);
  const reviewed = review(f.context);
  const before = { stage: "before", ledger: reviewed.ledger_before, original_budget: reviewed.original_budget_before, state: reviewed.recovery_before };
  assert.equal((await f.request("/record", { state: before.state })).status, 200);
  assert.equal((await f.request("/accounting", before)).status, 200);
  assert.equal((await f.request("/accounting", before)).status, 400);
  const after = { ...before, stage: "after", ledger: { ...before.ledger, total_charged_ms: before.ledger.total_charged_ms + 30000 } };
  assert.equal((await f.request("/accounting", after)).status, 400);
  after.ledger = before.ledger;
  assert.equal((await f.request("/record", { state: after.state })).status, 200);
  assert.equal((await f.request("/accounting", after)).status, 200);
  const saved = await readFile(resolve(f.options.privateRoot, "no-mining-accounting-after.json"), "utf8");
  assert.equal(JSON.parse(saved).ledger.total_charged_ms, before.ledger.total_charged_ms);
  assert.equal(saved.includes('"campaign_id"'), false);
  assert.equal((await f.request("/authorization-context", {})).status, 404);
});

test("full no-mining result rejects continuity context without original accounting evidence", async (t) => {
  const f = await prepared(t);
  for (let index = 1; index <= 4; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  const inputPath = resolve(f.options.privateRoot, "review.json");
  await writeNew(inputPath, review(f.context));
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_accounting_evidence_missing/u);
  assert(!(await readdir(f.options.privateRoot)).includes("no-mining.result.json"));
});
test("same-session ready snapshots cannot establish fresh recovery", async (t) => {
  const f = await prepared(t, true);
  for (let index = 1; index <= 4; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  const reviewed = review(f.context), inputPath = resolve(f.options.privateRoot, "review.json");
  await writeNew(inputPath, reviewed);
  await recordedReview(f, reviewed, false);
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_interruption_missing/u);
  assert(!(await readdir(f.options.privateRoot)).includes("no-mining.result.json"));
});
test("accounting receipt state and journal sequence must match the reviewed recovery", async (t) => {
  const f = await prepared(t, true);
  for (let index = 1; index <= 4; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  const reviewed = review(f.context), inputPath = resolve(f.options.privateRoot, "review.json");
  await writeNew(inputPath, reviewed);
  await recordedReview(f, reviewed);
  const path = resolve(f.options.privateRoot, "no-mining-accounting-after.json");
  const original = JSON.parse(await readFile(path, "utf8"));
  const changed = structuredClone(original);
  changed.state.preservation.baseline_id = Buffer.alloc(16, 7).toString("base64url");
  await writeFile(path, JSON.stringify(changed));
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_accounting_integrity/u);
  original.observed_sequence = 1;
  await writeFile(path, JSON.stringify(original));
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_accounting_order/u);
});

test("a later reopen invalidates the earlier closed cleanup observation", async (t) => {
  // Arrange
  const f = await prepared(t, true);
  for (let index = 1; index <= 4; index += 1) await recordCycle(f.options.privateRoot, f.context, cycle(f.context, index));
  const reviewed = review(f.context), inputPath = resolve(f.options.privateRoot, "review.json");
  await writeNew(inputPath, reviewed);
  await recordedReview(f, reviewed);
  await appendState(f, reviewed.recovery_after, 5);
  // Act / Assert
  await assert.rejects(finishNoMining(f.options.privateRoot, f.context, inputPath), /no_mining_final_cleanup_missing/u);
  assert(!(await readdir(f.options.privateRoot)).includes("no-mining.result.json"));
});

test("canonical cold baseline qualifies without claiming restoration occurred", async (t) => {
  // Arrange
  const f = await prepared(t, true), reviewed = review(f.context);
  await appendState(f, reviewed.recovery_before, 1);
  // Act
  const receipt = await saveNoMiningAccounting(f.options.privateRoot, f.context, { stage: "before", ledger: reviewed.ledger_before,
    original_budget: reviewed.original_budget_before, state: reviewed.recovery_before });
  const result = validateNoMiningReview(reviewed, f.context, cycle(f.context, 4));
  // Assert
  assert.equal(reviewed.recovery_before.deviceBaselineConfirmed, true);
  assert.equal(reviewed.recovery_before.deviceRestorationConfirmed, false);
  assert.deepEqual(receipt, { accounting_saved: true, stage: "before" });
  assert.equal(result.accounting_unchanged, true);
});
test("no-mining accounting rejects missing or false baseline confirmation", async (t) => {
  const f = await prepared(t, true), reviewed = review(f.context);
  for (const maybeConfirmed of [undefined, false]) {
    const observed = { ...reviewed.recovery_before };
    if (maybeConfirmed === undefined) delete observed.deviceBaselineConfirmed;
    else observed.deviceBaselineConfirmed = maybeConfirmed;
    await assert.rejects(saveNoMiningAccounting(f.options.privateRoot, f.context, { stage: "before", ledger: reviewed.ledger_before,
      original_budget: reviewed.original_budget_before, state: observed }), /no_mining_accounting_baseline/u);
  }
});
test("no-mining review requires baseline confirmation on recovery and final cleanup", async (t) => {
  const f = await prepared(t, true);
  for (const stage of ["recovery_before", "recovery_after", "final_state"]) {
    for (const maybeConfirmed of [undefined, false]) {
      const reviewed = review(f.context);
      if (maybeConfirmed === undefined) delete reviewed[stage].deviceBaselineConfirmed;
      else reviewed[stage].deviceBaselineConfirmed = maybeConfirmed;
      assert.throws(() => validateNoMiningReview(reviewed, f.context, cycle(f.context, 4)), /no_mining_safe_baseline_missing/u);
    }
  }
});

test("baseline confirmation alone does not satisfy live-mining restoration evidence", () => {
  // Arrange
  const records = [{ sequence: 1, state: { running: true, deviceBaselineConfirmed: true, deviceRestorationConfirmed: false,
    deviceLeaseInactive: true, qualification: { generation: 1, safe_stop_complete: false, budget_reserved_ms: 180000 } } }];
  // Act / Assert
  assert.throws(() => judgeWindow(0, records), /device_restoration_ack_missing/u);
});

test("credits-only stale records do not establish old control-reply recovery", async (t) => {
  const f = await prepared(t, true), reviewed = review(f.context);
  reviewed.recovery_after.helloRecovery.discardedReplies = 0;
  assert.throws(() => validateNoMiningReview(reviewed, f.context, cycle(f.context, 4)), /no_mining_stale_recovery_missing/u);
});
test("legacy aggregate discard counts cannot establish control-reply recovery", async (t) => {
  const f = await prepared(t, true), reviewed = review(f.context);
  delete reviewed.recovery_after.helloRecovery.discardedReplies;
  assert.throws(() => validateNoMiningReview(reviewed, f.context, cycle(f.context, 4)), /no_mining_stale_recovery_missing/u);
});
test("discarded reply counts cannot exceed the observed record count", async (t) => {
  const f = await prepared(t, true), reviewed = review(f.context);
  reviewed.recovery_after.helloRecovery.discardedReplies = 3;
  assert.throws(() => validateNoMiningReview(reviewed, f.context, cycle(f.context, 4)), /hello_recovery_reply_shape/u);
});

function interruptionInput(context) {
  return { receipt: { schema: "worker-read-interruption-v1", interrupted: true, request_consumed: true, response_pending: true, ownership_released: true },
    before: state(context), after: state(context, { status: "closed", connected: false, serialOwnershipReleased: true }) };
}
function interruptionRecords(context, input) {
  return [input.before, input.after].map((state, index) => ({ schema: "fixed-usb-no-mining-state-v1",
    context_sha256: digest(JSON.stringify(context)), sequence: index + 1, state }));
}
async function postInterruptionStates(f, input) {
  for (const state of [input.before, input.after]) assert.equal((await f.request("/record", { state })).status, 200);
}
test("consumed read-only interruption records bind exact ordered browser states and digests", async (t) => {
  // Arrange
  const f = await serving(t), input = interruptionInput(f.context);
  await postInterruptionStates(f, input);
  // Act
  const response = await f.request("/read-only-interruption", input);
  // Assert
  assert.equal(response.status, 200);
  const saved = JSON.parse(await readFile(resolve(f.options.privateRoot, "no-mining-read-only-interruption.json"), "utf8"));
  assert.deepEqual(saved, validateNoMiningReadOnlyInterruption(input, f.context, interruptionRecords(f.context, input)));
  assert.equal(saved.before_sequence, 1);
  assert.equal(saved.after_sequence, 2);
  assert.equal(saved.receipt_sha256, digest(JSON.stringify(input.receipt)));
  assert.equal(saved.hardware_execution_claimed_by_supervisor, false);
});
test("missing request consumption is preserved as unqualified without a passing receipt", async (t) => {
  const f = await serving(t), input = interruptionInput(f.context);
  input.receipt.request_consumed = false;
  await postInterruptionStates(f, input);
  const response = await f.request("/read-only-interruption", input);
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "read_only_interruption_not_observed");
  const attempt = JSON.parse(await readFile(resolve(f.options.privateRoot, "no-mining-read-only-interruption-attempt.json"), "utf8"));
  assert.equal(attempt.outcome, "unqualified");
  assert.equal(attempt.input.receipt.request_consumed, false);
  assert(!(await readdir(f.options.privateRoot)).includes("no-mining-read-only-interruption.json"));
});
test("already-received replies retain the ready no-interruption outcome without retry", async (t) => {
  const f = await serving(t), input = interruptionInput(f.context);
  input.receipt.interrupted = false;
  input.receipt.response_pending = false;
  input.receipt.ownership_released = false;
  input.after = input.before;
  await postInterruptionStates(f, input);
  assert.equal((await f.request("/read-only-interruption", input)).status, 400);
  const path = resolve(f.options.privateRoot, "no-mining-read-only-interruption-attempt.json");
  const before = await readFile(path, "utf8");
  assert.equal(JSON.parse(before).input.after.status, "ready");
  assert.equal((await f.request("/read-only-interruption", input)).status, 400);
  assert.equal(await readFile(path, "utf8"), before);
  assert(!(await readdir(f.options.privateRoot)).includes("no-mining-read-only-interruption.json"));
});
test("read-only interruption cannot qualify a nonbaseline or active-lease observation", async (t) => {
  const f = await prepared(t);
  for (const change of [{ deviceBaselineConfirmed: false }, { deviceLeaseInactive: false }, { running: true }]) {
    const input = interruptionInput(f.context);
    Object.assign(input.before, change);
    assert.throws(() => validateNoMiningReadOnlyInterruption(input, f.context, interruptionRecords(f.context, input)), /read_only_interruption_baseline/u);
  }
});
test("read-only interruption requires actual release and matching ordered journal observations", async (t) => {
  const f = await prepared(t), input = interruptionInput(f.context);
  const unreleased = structuredClone(input);
  unreleased.after.serialOwnershipReleased = false;
  assert.throws(() => validateNoMiningReadOnlyInterruption(unreleased, f.context, interruptionRecords(f.context, unreleased)), /read_only_interruption_release/u);
  const mismatch = interruptionRecords(f.context, input);
  mismatch[0].state = { ...input.before, helloRecovery: { discardedRecords: 0, discardedBytes: 10, discardedReplies: 0 } };
  assert.throws(() => validateNoMiningReadOnlyInterruption(input, f.context, mismatch), /read_only_interruption_journal/u);
  const reopened = interruptionRecords(f.context, input);
  reopened.push({ ...reopened[0], sequence: 3 });
  assert.throws(() => validateNoMiningReadOnlyInterruption(input, f.context, reopened), /read_only_interruption_journal/u);
});
test("duplicate read-only interruption cannot replace the immutable qualified receipt", async (t) => {
  const f = await serving(t), input = interruptionInput(f.context);
  await postInterruptionStates(f, input);
  assert.equal((await f.request("/read-only-interruption", input)).status, 200);
  const path = resolve(f.options.privateRoot, "no-mining-read-only-interruption.json"), before = await readFile(path, "utf8");
  assert.equal((await f.request("/read-only-interruption", input)).status, 400);
  assert.equal(await readFile(path, "utf8"), before);
});
test("read-only interruption rejects cross-origin and raw fields before persistence", async (t) => {
  const f = await serving(t), input = interruptionInput(f.context);
  await postInterruptionStates(f, input);
  assert.equal((await f.request("/read-only-interruption", input, "http://untrusted.invalid")).status, 400);
  assert.equal((await f.request("/read-only-interruption", { ...input, raw_serial: "forbidden" })).status, 400);
  assert(!(await readdir(f.options.privateRoot)).some((name) => name.startsWith("no-mining-read-only-interruption")));
});
