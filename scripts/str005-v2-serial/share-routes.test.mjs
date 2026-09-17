import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, readdir, readFile, rm, chmod, unlink } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createShareRoutes } from "./share-routes.mjs";
import { createJournal } from "./journal.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { state as baseState, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { sha256, check } from "./values.mjs";
const binding = Buffer.alloc(32, 9).toString("base64url"), attemptId = Buffer.alloc(16, 1).toString("base64url");
const proof = { schema: "worker-cooling-proof-v1", fan_duty_percent: 100, fan_rpm: 3000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false };
const restoration = { schema: "worker-cooling-baseline-v1", fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false };
function qualification() {
  return { schema: "worker-qualification-v1", generation: 7, active_ms: 0, generation_elapsed_ms: 0, budget_reserved_ms: 0,
    submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 0, last_valid_heartbeat_ms: 0,
    budget_complete: false, safe_stop_complete: false, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false,
    voltage_volts: 5, power_watts: 3, chip_temp_celsius: 35, fan_rpm: 3000, gate_closed_ms: null, shutdown_started_ms: null,
    safe_stop_stage: "not_started", revocation_reason: "none", active_limit_ms: null, shutdown_budget_ms: 15550, work_gate_remaining_ms: null };
}
function idle() {
  return { schema: "worker-stratum-v2-status-v1", scope: "share", state: "idle", connection: null, record: null,
    observation: { bootOrdinal: 10, workerGeneration: 7, serialTransportEpoch: 8, observedAtUs: 1000000,
      clockValid: true, stationIpv4: "192.168.1.10", wifiConnected: true, socket: null } };
}
async function fixture(t, scope = "share") {
  const root = await mkdtemp(join(tmpdir(), "v2-share-routes-")); t.after(() => rm(root, { recursive: true }));
  const context = { scope, attemptId, gate_commit: "a".repeat(40), firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
    client_sha256: "f".repeat(64), qualificationAttempt: { schema: "worker-qualification-attempt-v1", id: attemptId, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 }, expectedLedgerBefore: ledger };
  const contextSha256 = sha256(JSON.stringify(context)), state = { ...baseState(context), qualification: qualification() }, journal = await createJournal(root, context);
  let now = 1000, failure = false, observerAlive = true, scopeId = "challenge_synthetic", signs = 0, verifies = 0, maybeSignHook, maybeVerifyHook;
  for (let sequence = 1; sequence <= 9; sequence++) await journal.state("candidate", state, now);
  for (let index = 1; index <= 4; index++) await writeNew(join(root, `cycle-${index}.json`), { schema: "str005-v2-cycle-v1", contextSha256,
    beforeSequence: index * 2 - 1, afterSequence: index * 2, installReviewSha256: "d".repeat(64),
    report: { schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
      baseline_id: state.preservation.baseline_id, browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
      device_identity_match: true, settings_match: true, authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false } });
  const owner = { ready: { listenIpv4: "192.168.1.20", listenPort: 3333 },
    stratum: { profile: "bwg-worker-stratum-v2-standard/0.1", endpoint: "stratum+tcp://192.168.1.20:3333/", authorityPublicKey: Buffer.alloc(32, 2).toString("base64url"), userIdentity: "synthetic-private-user" },
    alive() {}, requireStartWindow() {}, validateStation(value) { check(value === "192.168.1.10", "v2_fixture_station_changed"); } };
  const operations = { now: () => now, ready() {}, failed: () => failure, journal, fixture: () => owner, activeScope: () => ({ challengeId: scopeId }),
    requireObserver(observation, suppliedBinding) { check(observerAlive, "v2_observer_not_live"); check(suppliedBinding === binding && observation.bootOrdinal === 10 && observation.workerGeneration === 7, "v2_observer_binding"); },
    async verify() { verifies++; await maybeVerifyHook?.(); },
    async sign(operation) { signs++; await readFile(join(root, "issuance.claim.json")); await maybeSignHook?.();
      return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "synthetic-private-authorization" }; } };
  const routes = createShareRoutes(root, context, operations);
  async function cool() {
    const challenge = await routes.handle("/cooling-review-context", {});
    return routes.handle("/cooling-review", { nonce: challenge.nonce, proof, restoration, budget_before: ledger, budget_after: ledger, state });
  }
  async function accounting() { await writeNew(join(root, "accounting-before.json"), { schema: "str005-v2-accounting-v1", contextSha256,
    observedSequence: journal.lastState().sequence, stage: "before", state, ledger, original_budget: original }); }
  async function review() {
    const challenge = await routes.handle("/budget-review-context", {});
    return routes.handle("/budget-review", { nonce: challenge.nonce, report: ledger, controlSessionBindingSha256: binding, state });
  }
  async function ready() { await cool(); await accounting(); await review(); await routes.handle("/start/network", { status: idle(), controlSessionBindingSha256: binding }); }
  return { root, context, state, journal, owner, operations, routes, cool, accounting, review, ready,
    dropObserver() { observerAlive = false; }, advance(ms) { now += ms; }, fail() { failure = true; }, clearFailure() { failure = false; }, replaceScope() { scopeId = "challenge_replacement"; routes.resetScope(); },
    counts: () => ({ signs, verifies }), onSign(fn) { maybeSignHook = fn; }, onVerify(fn) { maybeVerifyHook = fn; } };
}

test("Channel rejects all work/cooling routes without reading signing keys", async t => {
  const f = await fixture(t, "channel");
  for (const path of ["/budget-review-context", "/cooling-review-context", "/authorization-context", "/start/network", "/window-artifacts"])
    await assert.rejects(f.routes.handle(path, {}, path === "/window-artifacts" ? "GET" : "POST"), { code: "v2_share_route_forbidden" });
  assert.equal(f.counts().signs, 0);
});
test("actual protected prerequisites gate one issuance and one delivery without persisting private inputs", async t => {
  // Arrange
  const f = await fixture(t); await f.ready();
  // Act
  assert.deepEqual(await f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { ready: true });
  const artifacts = await f.routes.handle("/window-artifacts", undefined, "GET");
  // Assert
  assert.equal(artifacts.renewals.length, 9); assert.equal(artifacts.grant.qualificationAttempt.ordinal, 18);
  assert.equal(f.counts().signs, 10);
  await assert.rejects(f.routes.handle("/window-artifacts", undefined, "GET"));
  const files = (await Promise.all((await readdir(f.root)).map(name => readFile(join(f.root, name), "utf8")))).join("\n");
  for (const forbidden of [binding, "192.168.1.10", "192.168.1.20", "synthetic-private-user", "synthetic-private-authorization", '"authorityPublicKey"', '"stratum"']) assert.equal(files.includes(forbidden), false);
  assert.equal(JSON.parse(await readFile(join(f.root, "consumed.json"))).deviceReservationObserved, false);
});
test("fresh review records the actual submitted state despite an intervening published-state update", async t => {
  const f = await fixture(t); await f.cool(); await f.accounting();
  const challenge = await f.routes.handle("/budget-review-context", {});
  await f.journal.state("candidate", { ...f.state, qualification: { ...f.state.qualification, fan_rpm: 3100 } }, 1000);
  await f.routes.handle("/budget-review", { nonce: challenge.nonce, report: ledger, controlSessionBindingSha256: binding, state: f.state });
  assert.equal(f.journal.lastState().state.qualification.fan_rpm, 3000);
});
test("expired and reused review nonces cannot authorize issuance", async t => {
  const f = await fixture(t); await f.cool(); await f.accounting();
  const challenge = await f.routes.handle("/budget-review-context", {}); f.advance(45000);
  const input = { nonce: challenge.nonce, report: ledger, controlSessionBindingSha256: binding, state: f.state };
  await assert.rejects(f.routes.handle("/budget-review", input), { code: "v2_share_review_expired" });
  await assert.rejects(f.routes.handle("/budget-review", input), { code: "v2_share_review_expired" });
  assert.equal(f.counts().signs, 0);
});
test("pending reservation or ledger drift blocks review", async t => {
  const f = await fixture(t); await f.cool(); await f.accounting();
  for (const change of [{ pending: true }, { total_charged_ms: 1740000 }, { next_ordinal: 19 }]) {
    const challenge = await f.routes.handle("/budget-review-context", {});
    await assert.rejects(f.routes.handle("/budget-review", { nonce: challenge.nonce, report: { ...ledger, ...change }, controlSessionBindingSha256: binding, state: f.state }));
  }
  assert.equal(f.counts().signs, 0);
});
test("missing cooling, cycles or accounting is never replaced by a caller pass flag", async t => {
  const f = await fixture(t); const challenge = await f.routes.handle("/budget-review-context", {});
  await assert.rejects(f.routes.handle("/budget-review", { nonce: challenge.nonce, report: ledger, controlSessionBindingSha256: binding, state: f.state }));
  await unlink(join(f.root, "cycle-4.json")); await assert.rejects(f.routes.handle("/cooling-review-context", {}));
  assert.equal(f.counts().signs, 0);
});
test("the actual fixture expected peer is checked rather than its listener IP", async t => {
  const f = await fixture(t); await f.cool(); await f.accounting(); await f.review();
  const status = idle(); status.observation.stationIpv4 = f.owner.ready.listenIpv4;
  await assert.rejects(f.routes.handle("/start/network", { status, controlSessionBindingSha256: binding }), { code: "v2_fixture_station_changed" });
});
test("stale network observation and unsafe prerequisite permissions block signing", async t => {
  const f = await fixture(t); await f.ready(); f.advance(5001);
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { code: "v2_share_network_stale" });
  await f.review(); await f.routes.handle("/start/network", { status: idle(), controlSessionBindingSha256: binding });
  await chmod(join(f.root, "cycle-3.json"), 0o644);
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding })); assert.equal(f.counts().signs, 0);
});
test("failure during signing preserves the consumed claim and blocks delivery or reset-based retry", async t => {
  const f = await fixture(t); await f.ready(); f.onSign(() => f.fail());
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { code: "v2_terminal_failure" });
  assert.equal(f.counts().signs, 1); await readFile(join(f.root, "issuance.claim.json"));
  f.clearFailure(); f.replaceScope();
  await assert.rejects(f.routes.handle("/budget-review-context", {}), { code: "v2_share_issuance_consumed" });
  await assert.rejects(f.routes.handle("/window-artifacts", undefined, "GET"));
});
test("scope replacement during awaited verification prevents signing", async t => {
  const f = await fixture(t); await f.ready(); f.onVerify(() => f.replaceScope());
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }));
  assert.equal(f.counts().signs, 0);
});
test("signing stops between calls when readiness ages out", async t => {
  const f = await fixture(t); await f.ready(); f.onSign(() => f.advance(5001));
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { code: "v2_share_network_stale" });
  assert.equal(f.counts().signs, 1);
});
test("an interrupted existing issuance claim cannot be overwritten by a fresh route owner", async t => {
  const f = await fixture(t); await f.ready();
  await writeNew(join(f.root, "issuance.claim.json"), { interrupted: true });
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }));
  assert.equal(f.counts().signs, 0);
  const replacement = createShareRoutes(f.root, f.context, f.operations);
  await assert.rejects(replacement.handle("/budget-review-context", {}));
});

test("dead observer blocks issuance even while fixture and reviewed binding remain live", async t => {
  const f = await fixture(t); await f.ready(); f.dropObserver();
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { code: "v2_observer_not_live" });
  assert.equal(f.counts().signs, 0);
});
test("observer loss during awaited signing prevents further grants and any delivery", async t => {
  const f = await fixture(t); await f.ready(); f.onSign(() => f.dropObserver());
  await assert.rejects(f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding }), { code: "v2_observer_not_live" });
  assert.equal(f.counts().signs, 1);
  await assert.rejects(f.routes.handle("/window-artifacts", undefined, "GET"));
});
test("observer loss after issuance still blocks artifact delivery", async t => {
  const f = await fixture(t); await f.ready(); await f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding });
  f.dropObserver();
  await assert.rejects(f.routes.handle("/window-artifacts", undefined, "GET"), { code: "v2_observer_not_live" });
  assert.equal(f.counts().signs, 10);
});

test("observed Start retains only post-load safe network metadata and separate page-clock bounds", async t => {
  // Arrange.
  const f = await fixture(t); await f.ready();
  await f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding });
  await f.routes.handle("/window-artifacts", undefined, "GET");
  await writeNew(join(f.root, "fixture-ready.json"), { readyAtMs: 1000 });
  const running = { ...f.state, status: "running", running: true, deviceLeaseInactive: false, deviceBaselineConfirmed: false };
  await f.journal.state("candidate", running, 1000);
  const status = idle(); status.observation.observedAtUs++;
  // Act.
  const result = await f.routes.handle("/share/start-observed", { status, timing: {
    fixtureRequestAtPageMs: 0.5, startInvokedAtPageMs: 10000.5, startRepliedAtPageMs: 40000.5 } });
  // Assert.
  assert.deepEqual(result, { start_observed: true });
  const text = await readFile(join(f.root, "share-start-observed.json"), "utf8"), saved = JSON.parse(text);
  assert.equal(saved.networkObservedAtDeviceUs, 1000001);
  for (const value of [binding, "192.168.", "stratum", "authorityPublicKey"]) assert.equal(text.includes(value), false);
  await assert.rejects(f.routes.handle("/share/start-observed", { status, timing: saved.timing }), { code: "EEXIST" });
});

test("observed Start rejects timing overrun, changed session and unissued contexts", async t => {
  // Arrange.
  const f = await fixture(t), timing = { fixtureRequestAtPageMs: 0, startInvokedAtPageMs: 1, startRepliedAtPageMs: 2 };
  await assert.rejects(f.routes.handle("/share/start-observed", { status: idle(), timing }), { code: "v2_share_start_issuance" });
  await f.ready(); await f.routes.handle("/authorization-context", { controlSessionBindingSha256: binding });
  await f.routes.handle("/window-artifacts", undefined, "GET");
  // Act / Assert.
  await assert.rejects(f.routes.handle("/share/start-observed", { status: idle(), timing: { ...timing, startInvokedAtPageMs: 10001 } }), { code: "v2_start_fixture_deadline" });
  await assert.rejects(f.routes.handle("/share/start-observed", { status: idle(), timing: { ...timing, startRepliedAtPageMs: 30002 } }), { code: "v2_start_reply_deadline" });
  const changed = idle(); changed.observation.serialTransportEpoch++;
  await assert.rejects(f.routes.handle("/share/start-observed", { status: changed, timing }), { code: "v2_share_start_binding" });
});
