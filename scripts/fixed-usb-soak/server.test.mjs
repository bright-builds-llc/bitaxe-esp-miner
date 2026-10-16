import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { SOAK_CONTEXT_SCHEMA } from "./contract.mjs";
import { createSoakSupervisor } from "./server.mjs";

const context = { schema: SOAK_CONTEXT_SCHEMA, gate_commit: "a".repeat(40), firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
  suggested_difficulty: 1000, firmware_root: "", gate_root: "", gate_page_relative_path: "page.html" };
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

async function supervisor({ idlePassed = true } = {}) {
  const root = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "soak-server-")));
  const firmware = resolve(root, "firmware");
  await mkdir(resolve(firmware, "firmware/bitaxe/bwg"), { recursive: true });
  await writeFile(resolve(firmware, "firmware/bitaxe/bwg/deployment-trust.json"), "{}");
  const signed = [], patches = [];
  const server = await createSoakSupervisor({ privateRoot: root, context: { ...context, firmware_root: firmware } }, {
    verifyFrozen: async () => undefined,
    readPool: async () => ({ endpoint: "stratum+tcp://pool.invalid:3333/", username: "u", password: "p" }),
    sign: async (operation, input) => { signed.push(input); return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "signed" }; },
    applyDeviceSettings: async (target) => (patches.push(target), { schema: "fixed-usb-soak-device-settings-v1", requested: {}, observed: {} }),
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
  return { root, server, call, signed, patches };
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
