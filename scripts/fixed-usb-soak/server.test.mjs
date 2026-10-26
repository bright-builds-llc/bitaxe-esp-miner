import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { once } from "node:events";
import { mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { SOAK_CONTEXT_SCHEMA } from "./contract.mjs";
import { frozenPlan, SOAK_LIVE_CHANGES } from "./live-changes.mjs";
import { createSoakSupervisor } from "./server.mjs";
import { QualificationError } from "../fixed-usb-qualification/contract.mjs";

const context = { schema: SOAK_CONTEXT_SCHEMA, gate_commit: "a".repeat(40), firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
  suggested_difficulty: 1000, device_settings: { frequency: 485, coreVoltage: 1200, autofanspeed: 1 }, soak_judge: { path: "/nonexistent", sha256: "0".repeat(64) }, firmware_root: "", gate_root: "", gate_page_relative_path: "page.html" };
const state = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, status: "ready", connected: true, running: false,
  heartbeatSuppressed: false, renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: false,
  expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256 };
const proof = { schema: "worker-cooling-proof-v1", fan_duty_percent: 100, fan_rpm: 5000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false };
const restoration = { schema: "worker-cooling-baseline-v1", fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false };
const qualificationLedger = { schema: "worker-qualification-ledger-v1", next_ordinal: 34, total_charged_ms: 4260000, pending: false, last_completed_ordinal: 33 };
const soakLedger = { schema: "worker-soak-ledger-v2", next_ordinal: 1, total_charged_ms: 0, pending: false, last_completed_ordinal: 0 };
const binding = "A".repeat(43);
const preservation = (baselineId) => ({ schema: "worker-preservation-continuity-v1", baseline_id: baselineId, settings_match: true,
  authorization_high_water_match: false, device_identity_match: true, mine_on_boot: false });
/** The page that wrote the settings, and the page reloaded after the write. */
const beforeSettings = { ...state, preservation: preservation("A".repeat(22)) };
const afterSettings = { ...state, preservation: preservation(`${"B".repeat(21)}A`) };
const endpoint = { schema: "worker-telemetry-endpoint-v1", controlSessionBindingSha256: binding, ipv4: "192.168.0.2", httpPort: 80 };

/** Polls until `predicate` holds, so tests never depend on a fixed delay. */
async function eventually(predicate) {
  for (let attempt = 0; attempt < 200 && !(await predicate()); attempt++) await new Promise((done) => setTimeout(done, 10));
  assert.ok(await predicate(), "condition never held");
}

const PROFILES = { "upstream-default": { frequency: 485, coreVoltage: 1200, autofanspeed: 1 },
  "catalog-maximum-frequency": { frequency: 575, coreVoltage: 1200, autofanspeed: 1 },
  "upstream-default-bap": { frequency: 485, coreVoltage: 1200, autofanspeed: 1, manualFanSpeed: 100 } };

async function supervisor({ idlePassed = true, liveChanges = [], failLive = false, profile = "upstream-default", settings = PROFILES[profile],
  livePlan = null, bapPort = undefined, failBap = false, clientDrift = false, extraContext = {} } = {}) {
  const root = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "soak-server-")));
  const firmware = resolve(root, "firmware");
  await mkdir(resolve(firmware, "firmware/bitaxe/bwg"), { recursive: true });
  await writeFile(resolve(firmware, "firmware/bitaxe/bwg/deployment-trust.json"), "{}");
  const client = resolve(root, "bap_client");
  await writeFile(client, "frozen client");
  const clientSha256 = createHash("sha256").update(clientDrift ? "rebuilt client" : "frozen client").digest("hex");
  const signed = [], patches = [], bapChanges = [];
  // Completion waits for terminal samples on the clock; tests fast-forward it once the window is done.
  const clock = { now: Date.now(), fastForward: false };
  const server = await createSoakSupervisor({ privateRoot: root, context: { ...context, firmware_root: firmware, live_changes: liveChanges, live_plan: livePlan, device_profile: profile, device_settings: settings,
    bap_client: { path: client, sha256: clientSha256 }, ...extraContext }, bapPort }, {
    runBapChange: async (binary, port, ops, { allowed }) => {
      bapChanges.push({ binary: binary === client ? "frozen" : binary, port, parameters: ops.map((op) => op.parameter), allowed: allowed() });
      return { results: ops.map((op) => ({ parameter: op.parameter, passed: !failBap })), maybeFailure: failBap ? "bap_operation_failed" : undefined };
    },
    verifyFrozen: async () => undefined,
    readPool: async () => ({ endpoint: "stratum+tcp://pool.invalid:3333/", username: "u", password: "p" }),
    sign: async (operation, input) => { signed.push(input); return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "signed" }; },
    now: () => (clock.fastForward ? (clock.now += 10000) : clock.now),
    applyDeviceSettings: async (target, settings) => {
      patches.push({ ...target, settings });
      if (failLive && settings !== undefined && patches.length === 2) throw new QualificationError("soak_device_settings_patch");
      return { schema: "fixed-usb-soak-device-settings-v1", requested: {}, observed: {} };
    },
    createObserver: () => ({ start: async () => ({ observer_started: true }), idle: async () => ({ passed: idlePassed }), stop: async () => ({}), started: () => 1, journalPath: "/nonexistent" }),
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const origin = `http://127.0.0.1:${server.address().port}`;
  const call = async (path, maybeBody) => {
    const response = await fetch(origin + path, maybeBody === undefined ? { headers: { "sec-fetch-site": "same-origin" } }
      : { method: "POST", headers: { "content-type": "application/json", origin }, body: JSON.stringify(maybeBody) });
    return { status: response.status, value: await response.json() };
  };
  return { root, server, call, signed, patches, clock, bapChanges };
}

async function budgetReview(call, reviewState) {
  const { value: budget } = await call("/budget-review-context", {});
  assert.equal(budget.mode, "soak");
  return call("/budget-review", { nonce: budget.nonce, report: soakLedger, controlSessionBindingSha256: binding, state: reviewState });
}

async function reviewed(call, { withCooling = true } = {}) {
  await call("/activate", {});
  assert.deepEqual((await call("/observer/start", { endpoint, state: beforeSettings })).value, { observer_started: true });
  await call("/activate", {});
  if (withCooling) {
    const { value: cooling } = await call("/cooling-review-context", {});
    await call("/cooling-review", { nonce: cooling.nonce, proof, restoration, budget_before: qualificationLedger, budget_after: qualificationLedger, state: afterSettings });
  }
  const result = await budgetReview(call, afterSettings);
  assert.deepEqual(result.value, { budget_review_saved: true });
  return result;
}

test("a reviewed, cooled, idle-proven soak is signed once from the device ledger's next ordinal", async (t) => {
  // Arrange
  const { root, server, call, signed } = await supervisor();
  t.after(() => server.close());
  await reviewed(call);
  // Act
  const authorized = await call("/authorization-context", { controlSessionBindingSha256: binding });
  const first = await call("/window-artifacts");
  const second = await call("/window-artifacts");
  // Assert
  assert.deepEqual(authorized.value, { ready: true });
  assert.equal(first.value.grant.soakAllowance.ordinal, 1);
  assert.equal(first.value.renewals.length, 36);
  assert.equal(second.value.error, "soak_artifacts_unavailable");
  assert.equal(signed.length, 37);
  const issued = JSON.parse(await readFile(resolve(root, "issued.json"), "utf8"));
  assert.deepEqual([issued.ordinal, issued.private_payload_persisted, issued.preservation_baseline_id], [1, false, afterSettings.preservation.baseline_id]);
});

test("a budget review is refused before the soak's device settings are written", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  // Act
  const refused = await budgetReview(call, afterSettings);
  // Assert
  assert.equal(refused.value.error, "soak_device_settings_required");
});

test("a budget review is refused from the page that wrote the settings", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  await call("/observer/start", { endpoint, state: beforeSettings });
  // Act
  const refused = await budgetReview(call, beforeSettings);
  // Assert
  assert.equal(refused.value.error, "soak_settings_baseline_stale");
});

test("a budget review is refused when the reloaded page sees the settings change again", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  await call("/observer/start", { endpoint, state: beforeSettings });
  const changed = { ...afterSettings, preservation: { ...afterSettings.preservation, settings_match: false } };
  // Act
  const refused = await budgetReview(call, changed);
  // Assert
  assert.equal(refused.value.error, "soak_settings_baseline_stale");
});

test("settings are never written from a page that has not observed preservation", async (t) => {
  // Arrange
  const { server, call, patches } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  // Act
  const refused = await call("/observer/start", { endpoint, state });
  // Assert
  assert.equal(refused.value.error, "soak_preservation_unobserved");
  assert.equal(patches.length, 0);
});

test("a budget review is refused from a page that has not observed preservation", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  await call("/observer/start", { endpoint, state: beforeSettings });
  // Act
  const refused = await budgetReview(call, state);
  // Assert
  assert.equal(refused.value.error, "soak_preservation_unobserved");
});

/** A running soak sample whose work gate has reached `activeMs`. */
function mining(activeMs) {
  const qualification = { schema: "worker-qualification-v1", generation: 1, active_ms: activeMs, active_limit_ms: 600000, shutdown_budget_ms: 22550,
    work_gate_remaining_ms: 600000 - activeMs, generation_elapsed_ms: activeMs + 9000, budget_reserved_ms: 240000, budget_complete: false,
    submitted: 3, accepted: 3, rejected: 0, nonce_work_correlations: 3, work_dispatched: 5, last_valid_heartbeat_ms: activeMs, gate_closed_ms: null,
    shutdown_started_ms: null, safe_stop_stage: "not_started", safe_stop_complete: false, revocation_reason: "none", voltage_volts: 5.2,
    power_watts: 12, chip_temp_celsius: 60, fan_rpm: 3000, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true,
    watchdog_alive: true, mine_on_boot: false,
    soak: { schema: "worker-soak-observation-v1", ordinal: 1, maximum_active_ms: 600000, reserved_ms: 600000, complete: false, active_ms: activeMs } };
  return { ...afterSettings, status: "running", running: true, deviceLeaseInactive: false, renewalsConfirmed: 10, qualification };
}

test("a planned live change is sent once, when the work gate reaches it, and recorded", async (t) => {
  // Arrange
  const { root, server, call, patches } = await supervisor({ liveChanges: SOAK_LIVE_CHANGES });
  t.after(() => server.close());
  await reviewed(call);
  // Act
  for (const activeMs of [199000, 200000, 201000]) await call("/record", { state: mining(activeMs) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  // Assert
  assert.deepEqual(patches.map((target) => [target.ipv4, target.settings]), [[endpoint.ipv4, context.device_settings], [endpoint.ipv4, SOAK_LIVE_CHANGES[0].settings]]);
  const recorded = JSON.parse(await readFile(resolve(root, "live-change-1.json"), "utf8"));
  assert.deepEqual([recorded.index, recorded.active_ms, recorded.failure], [0, 200000, undefined]);
});

test("a soak without planned live changes never sends one", async (t) => {
  // Arrange
  const { server, call, patches } = await supervisor();
  t.after(() => server.close());
  await reviewed(call);
  // Act
  await call("/record", { state: mining(500000) });
  await call("/record", { state: mining(501000) });
  // Assert
  assert.equal(patches.length, 1);
});

test("a failed live change is recorded with its failure", async (t) => {
  // Arrange
  const { root, server, call } = await supervisor({ liveChanges: SOAK_LIVE_CHANGES, failLive: true });
  t.after(() => server.close());
  await reviewed(call);
  // Act
  await call("/record", { state: mining(200000) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  // Assert
  const recorded = JSON.parse(await readFile(resolve(root, "live-change-1.json"), "utf8"));
  assert.equal(recorded.failure, "soak_device_settings_patch");
});

test("a plan in the context that differs from the code's plan is refused", async () => {
  // Arrange
  const altered = [{ ...SOAK_LIVE_CHANGES[0], settings: { frequency: 575, coreVoltage: 1300 } }, SOAK_LIVE_CHANGES[1]];
  // Act / Assert
  await assert.rejects(supervisor({ liveChanges: altered }), /soak_live_changes_contract/u);
});

test("a soak that stops between its changes restores its settings and stays unverified", async (t) => {
  // Arrange
  const { root, server, call, patches, clock } = await supervisor({ liveChanges: SOAK_LIVE_CHANGES });
  t.after(() => server.close());
  await reviewed(call);
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/window-artifacts");
  await call("/record", { state: mining(200000) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  const { value: completion } = await call("/completion-context", {});
  clock.fastForward = true;
  // Act
  const completed = await call("/completion-review", { nonce: completion.nonce, ledger_after: soakLedger, final_state: afterSettings });
  // Assert
  assert.equal(completed.value.result, "unverified", JSON.stringify(completed.value));
  assert.deepEqual(patches.at(-1).settings, context.device_settings);
  const { result } = JSON.parse(await readFile(resolve(root, "result.json"), "utf8"));
  assert.ok(result.failures.includes("live_change_restored") && result.failures.includes("live_change_missing"));
  assert.deepEqual(result.settings_restore, { reason: "incomplete_live_changes", outcome: "restored" });
});

test("signing is refused until the idle WebSocket pre-phase passes", async (t) => {
  // Arrange
  const { server, call, signed } = await supervisor({ idlePassed: false });
  t.after(() => server.close());
  await reviewed(call);
  // Act
  const refused = await call("/authorization-context", { controlSessionBindingSha256: binding });
  // Assert
  assert.equal(refused.value.error, "soak_idle_proof_failed");
  assert.equal(signed.length, 0);
});

test("signing is refused without a cooling review", async (t) => {
  // Arrange
  const { server, call, signed } = await supervisor();
  t.after(() => server.close());
  await reviewed(call, { withCooling: false });
  // Act
  const refused = await call("/authorization-context", { controlSessionBindingSha256: binding });
  // Assert
  assert.equal(refused.value.error, "soak_cooling_required");
  assert.equal(signed.length, 0);
});

test("signing needs the binding of a fresh review", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await reviewed(call);
  // Act
  const refused = await call("/authorization-context", { controlSessionBindingSha256: `${"A".repeat(42)}E` });
  // Assert
  assert.equal(refused.value.error, "soak_fresh_review_required");
});

test("a pending soak ledger cannot be reviewed for signing", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  await call("/observer/start", { endpoint, state: beforeSettings });
  const { value: budget } = await call("/budget-review-context", {});
  // Act
  const refused = await call("/budget-review", { nonce: budget.nonce, report: { ...soakLedger, pending: true }, controlSessionBindingSha256: binding, state: afterSettings });
  // Assert
  assert.equal(refused.value.error, "soak_ledger_not_idle");
});

test("the page is configured for soak mode only", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  // Act
  const { value } = await call("/context");
  // Assert
  assert.deepEqual(Object.keys(value).sort(), ["expectedAppElfSha256", "expectedFirmwareSourceCommit", "expectedGateCommit", "soakQualification", "trust"]);
});

test("the catalog-maximum profile writes 575 MHz at 1200 mV with the automatic fan", async (t) => {
  // Arrange
  const { server, call, patches } = await supervisor({ profile: "catalog-maximum-frequency" });
  t.after(() => server.close());
  await call("/activate", {});
  // Act
  await call("/observer/start", { endpoint, state: beforeSettings });
  // Assert
  assert.deepEqual(patches[0].settings, { frequency: 575, coreVoltage: 1200, autofanspeed: 1 });
});

test("settings in the context that differ from their named profile are refused", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ profile: "catalog-maximum-frequency", settings: { frequency: 575, coreVoltage: 1300, autofanspeed: 1 } }),
    /soak_profile_contract/u);
});

test("live changes are refused from a profile they would not return to", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ profile: "catalog-maximum-frequency", liveChanges: SOAK_LIVE_CHANGES }), /soak_live_changes_profile/u);
});

test("a catalog-maximum soak restores the upstream default once it completes", async (t) => {
  // Arrange
  const { root, server, call, patches, clock } = await supervisor({ profile: "catalog-maximum-frequency" });
  t.after(() => server.close());
  await reviewed(call);
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/window-artifacts");
  const { value: completion } = await call("/completion-context", {});
  clock.fastForward = true;
  // Act
  await call("/completion-review", { nonce: completion.nonce, ledger_after: soakLedger, final_state: afterSettings });
  // Assert
  assert.deepEqual(patches.map((patch) => patch.settings.frequency), [575, 485]);
  const { result } = JSON.parse(await readFile(resolve(root, "result.json"), "utf8"));
  assert.deepEqual(result.settings_restore, { reason: "profile_completed", outcome: "restored" });
});

const bapSoak = { profile: "upstream-default-bap", liveChanges: frozenPlan("bap"), livePlan: "bap", bapPort: "/dev/cu.usbserial-0001" };

test("a BAP plan needs the accessory adapter's port", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ ...bapSoak, bapPort: undefined }), /soak_bap_port/u);
});

test("a BAP plan runs only from its own profile", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ ...bapSoak, profile: "upstream-default" }), /soak_live_changes_profile/u);
});

test("a due BAP change runs its settings through the frozen client on the adapter port", async (t) => {
  // Arrange
  const { root, server, call, bapChanges, patches } = await supervisor(bapSoak);
  t.after(() => server.close());
  await reviewed(call);
  // Act
  await call("/record", { state: mining(180000) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  // Assert: the block-flag change was due first, and no HTTP PATCH followed the profile write.
  assert.deepEqual(bapChanges[0], { binary: "frozen", port: "/dev/cu.usbserial-0001",
    parameters: ["show_new_block", "show_new_block", "show_new_block", "show_new_block", "found_block", "found_block"], allowed: true });
  assert.equal(patches.length, 1);
  const recorded = JSON.parse(await readFile(resolve(root, "live-change-1.json"), "utf8"));
  assert.equal(recorded.operations.length, 6);
});

test("a failed BAP change is recorded with its failure", async (t) => {
  // Arrange
  const { root, server, call } = await supervisor({ ...bapSoak, failBap: true });
  t.after(() => server.close());
  await reviewed(call);
  // Act
  await call("/record", { state: mining(100000) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  // Assert
  const recorded = JSON.parse(await readFile(resolve(root, "live-change-1.json"), "utf8"));
  assert.equal(recorded.failure, "bap_operation_failed");
});

test("a BAP soak whose plan never ran needs no settings restore", async (t) => {
  // Arrange
  const { root, server, call, patches, clock } = await supervisor({ ...bapSoak, liveChanges: frozenPlan("bap") });
  t.after(() => server.close());
  await reviewed(call);
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/window-artifacts");
  const { value: completion } = await call("/completion-context", {});
  clock.fastForward = true;
  // Act
  await call("/completion-review", { nonce: completion.nonce, ledger_after: soakLedger, final_state: afterSettings });
  // Assert: the plan never ran, so only the profile write was sent, and nothing was restored.
  assert.equal(patches.length, 1);
  const { result } = JSON.parse(await readFile(resolve(root, "result.json"), "utf8"));
  assert.equal(result.settings_restore, undefined);
  assert.ok(result.failures.includes("live_change_missing"));
});

test("a BAP change never runs a client rebuilt after preflight", async (t) => {
  // Arrange
  const { root, server, call, bapChanges } = await supervisor({ ...bapSoak, clientDrift: true });
  t.after(() => server.close());
  await reviewed(call);
  // Act
  await call("/record", { state: mining(100000) });
  await eventually(() => readFile(resolve(root, "live-change-1.json")).then(() => true, () => false));
  // Assert
  const recorded = JSON.parse(await readFile(resolve(root, "live-change-1.json"), "utf8"));
  assert.deepEqual([recorded.failure, bapChanges.length], ["bap_client_drift", 0]);
});

test("an interrupting soak excuses a Connect refused before the observer and judges a resumable baseline", async (t) => {
  // Arrange: a stand-in window judge that echoes its arguments.
  const judgeRoot = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "soak-judge-")));
  const judgePath = resolve(judgeRoot, "soak_judge");
  const script = '#!/bin/sh\necho "{\\"schema\\":\\"soak-judge-v1\\",\\"passed\\":true,\\"failures\\":[],\\"last\\":\\"${5:-none}\\"}"\n';
  await writeFile(judgePath, script, { mode: 0o700 });
  const soakJudge = { path: judgePath, sha256: createHash("sha256").update(script).digest("hex") };
  const { root, server, call, clock } = await supervisor({ extraContext: { local_mining_interrupt: true, soak_judge: soakJudge } });
  t.after(() => server.close());
  await call("/record", { state: { ...state, status: "failed", connected: false, failure: "connect_failed", admissionFailureStage: "hello" } });
  await reviewed(call);
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/window-artifacts");
  const { value: completion } = await call("/completion-context", {});
  clock.fastForward = true;
  // Act
  await call("/completion-review", { nonce: completion.nonce, ledger_after: soakLedger, final_state: afterSettings });
  // Assert
  const { result } = JSON.parse(await readFile(resolve(root, "result.json"), "utf8"));
  assert.equal(result.window_judgement.last, "--local-mining-interrupt");
  assert.equal(result.refused_connect_records, 1);
  assert.ok(!result.failures.includes("browser_failure") && !result.failures.includes("interrupt_refusal_unobserved"), JSON.stringify(result.failures));
});

const gamma601Soak = { profile: "gamma601-first", settings: { frequency: 400, coreVoltage: 1150, autofanspeed: 1 },
  extraContext: { board: "601", local_mining_interrupt: true } };

test("a 601 soak writes its own profile and never restores 485 MHz at 1200 mV", async (t) => {
  // Arrange
  const { root, server, call, patches, clock } = await supervisor(gamma601Soak);
  t.after(() => server.close());
  await reviewed(call);
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/window-artifacts");
  const { value: completion } = await call("/completion-context", {});
  clock.fastForward = true;
  // Act
  await call("/completion-review", { nonce: completion.nonce, ledger_after: soakLedger, final_state: afterSettings });
  // Assert
  assert.deepEqual(patches.map((patch) => patch.settings), [gamma601Soak.settings]);
  const { result } = JSON.parse(await readFile(resolve(root, "result.json"), "utf8"));
  assert.equal(result.settings_restore, undefined);
});

test("a 601 context carrying the 205's profile is refused", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ extraContext: { board: "601", local_mining_interrupt: true } }), /soak_board_profile/u);
});

test("a context naming an unknown board is refused", async () => {
  // Arrange / Act / Assert
  await assert.rejects(supervisor({ ...gamma601Soak, extraContext: { board: "999" } }), /soak_board/u);
});
