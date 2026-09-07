import test from "node:test";
import { once } from "node:events";
import { createSupervisor } from "./server.mjs";
import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { digest, writeNew } from "./contract.mjs";
import { createSuccessor, loadSuccessor, validateBudgetReview } from "./successor.mjs";
import { selectedWindow } from "./store.mjs";
import { signWindow } from "./authority.mjs";

const report = { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 1,
  completed_mask: 1, charged_ms: 180000, pending: false };
const baseline = Buffer.alloc(16, 1).toString("base64url");
async function fixture(t) {
  const base = await mkdtemp(resolve(tmpdir(), "successor-test-"));
  await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, "attempt-new"), predecessor = resolve(base, "attempt-old");
  await mkdir(root, { mode: 0o700 }); await mkdir(predecessor, { mode: 0o700 });
  const context = { required_no_mining_cycles: 4, window_limits_ms: [180000, 30000, 30000], firmware_root: root, gate_root: root, campaign_id: Buffer.alloc(16, 9).toString("base64url"), firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64) };
  const old = { ...context, firmware_commit: "d".repeat(40) };
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await writeNew(resolve(predecessor, "context.json"), { context: old, sha256: digest(JSON.stringify(old)) });
  await writeNew(resolve(predecessor, "window-0.consumed.json"), { window: 0, delivery_attempted: true });
  await writeNew(resolve(predecessor, "window-0.first-failure.json"), { schema: "fixed-usb-live-failure-review-v1",
    window: 0, first_failure: "start_failed", active_milliseconds: "unverified", mining_retry: false });
  const state = { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "ready", connected: true, running: false, heartbeatSuppressed: false,
    renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceLeaseInactive: true, serialOwnershipReleased: false,
    preservation: { schema: "worker-preservation-continuity-v1", baseline_id: baseline, settings_match: true,
      authorization_high_water_match: false, device_identity_match: true, mine_on_boot: false } };
  for (let cycle = 1; cycle <= 4; cycle += 1) await writeNew(resolve(root, `cycle-${cycle}.json`), {
    schema: "fixed-usb-cycle-report-v1", cycle, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
    baseline_id: baseline, browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
    device_identity_match: true, settings_match: true, authorization_high_water_match: true,
    probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false });
  const review = resolve(root, "budget-review-fixture.json");
  await writeNew(review, { schema: "fixed-usb-budget-review-v1", context_sha256: digest(JSON.stringify(context)), report, state });
  await mkdir(resolve(root, "firmware/bitaxe/bwg"), { recursive: true });
  await writeNew(resolve(root, "firmware/bitaxe/bwg/deployment-trust.json"), {});
  return { root, predecessor, context, review, state };
}

test("successor consumes no reservation and keeps original failure immutable", async (t) => {
  // Arrange
  const f = await fixture(t);
  const original = await readFile(resolve(f.predecessor, "window-0.first-failure.json"));
  // Act
  await createSuccessor(f.root, f.context, f.predecessor, f.review);
  // Assert
  assert.equal(await selectedWindow(f.root), 1);
  assert.deepEqual(await readFile(resolve(f.predecessor, "window-0.first-failure.json")), original);
  await assert.rejects(readFile(resolve(f.root, "window-0.result.json")), { code: "ENOENT" });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review));
});
test("a synthetic passing predecessor result cannot authorize successor", async (t) => {
  const f = await fixture(t);
  await writeNew(resolve(f.predecessor, "window-0.result.json"), { browser_report_accepted: true });
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review), /private_path_exists/u);
});
test("changed predecessor evidence invalidates existing successor", async (t) => {
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review);
  await writeFile(resolve(f.predecessor, "window-0.consumed.json"), '{"window":0,"delivery_attempted":true,"changed":true}');
  await assert.rejects(loadSuccessor(f.root, f.context), /evidence_changed/u);
});
test("different campaign cannot reuse consumed window", async (t) => {
  const f = await fixture(t);
  await assert.rejects(createSuccessor(f.root, { ...f.context, campaign_id: Buffer.alloc(16, 2).toString("base64url") }, f.predecessor, f.review), /campaign_identity/u);
});
test("missing current-image cycle keeps successor blocked", async (t) => {
  const f = await fixture(t);
  await rm(resolve(f.root, "cycle-4.json"));
  await assert.rejects(createSuccessor(f.root, f.context, f.predecessor, f.review), { code: "ENOENT" });
});
test("pending, refunded, mismatched and already consumed remaining windows reject", () => {
  for (const mutation of [{ pending: true }, { charged_ms: 0 }, { campaign_match: false }, { reserved_mask: 3 },
    { completed_mask: 0 }, { charged_ms: 180001 }, { reserved_mask: 9 }]) {
    assert.throws(() => validateBudgetReview({ ...report, ...mutation }), /successor_budget_state/u);
  }
  assert.deepEqual(validateBudgetReview(report), report);
  const second = { ...report, reserved_mask: 3, completed_mask: 3, charged_ms: 210000 };
  assert.deepEqual(validateBudgetReview(second, 2), second);
});
test("successor renewal happens before short-window shutdown headroom without extending reservation", async () => {
  const input = { campaignId: baseline, challengeId: "challenge_fixture", binding: Buffer.alloc(32, 1).toString("base64url"),
    stratum: { endpoint: "stratum+tcp://fixture.invalid:1234/", username: "fixture", password: "fixture" }, successor: true,
    sign: async (operation) => ({ profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }) };
  for (const index of [1, 2]) {
    const value = await signWindow({ ...input, index });
    assert.equal(value.grant.acceptanceCampaign.maximumActiveMilliseconds, 30000);
    assert.equal(value.grant.renewAfterMilliseconds, 5000);
    assert(5000 + 6000 < 30000 - 15550);
    assert(value.renewals.every((renewal) => renewal.renewAfterMilliseconds === 5000 && !Object.hasOwn(renewal, "acceptanceCampaign")));
  }
  await assert.rejects(signWindow({ ...input, index: 0 }), /successor_signing_window/u);
});

test("forged remaining-window result cannot skip directly to heartbeat run", async (t) => {
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review);
  await writeNew(resolve(f.root, "window-1.result.json"), { browser_report_accepted: true });
  await assert.rejects(selectedWindow(f.root));
});
test("fresh private review binds issuance to the current possession and is consumed once", async (t) => {
  // Arrange
  const f = await fixture(t);
  await createSuccessor(f.root, f.context, f.predecessor, f.review);
  let clock = 1000, signs = 0;
  const server = await createSupervisor({ privateRoot: f.root, context: f.context }, {
    verifyFrozen: async () => ({}), now: () => clock,
    readPool: async () => ({ endpoint: "stratum+tcp://fixture.invalid:1234/", username: "fixture", password: "fixture" }),
    sign: async (operation) => { signs += 1; return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }; },
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  t.after(() => new Promise((resolveDone) => { server.close(resolveDone); server.closeAllConnections(); }));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (route, body) => fetch(origin + route, { method: "POST", headers: { Origin: origin, "Content-Type": "application/json" }, body: JSON.stringify(body) });
  const binding = Buffer.alloc(32, 4).toString("base64url");
  let lastNonce;
  const review = async () => {
    const challenge = await (await post("/budget-review-context", {})).json();
    lastNonce = challenge.nonce;
    const response = await post("/budget-review", { nonce: challenge.nonce, report, controlSessionBindingSha256: binding, state: f.state });
    if (response.ok) {
      const receipt = await response.clone().json();
      const saved = await readFile(resolve(f.root, receipt.review_file), "utf8");
      assert(!saved.includes(binding));
      assert(!saved.includes(f.context.campaign_id));
    }
    return response;
  };
  await post("/activate", {});
  // Act / Assert: no review and wrong connection are closed failures.
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal((await review()).status, 200);
  assert.equal((await post("/budget-review", { nonce: lastNonce, report, controlSessionBindingSha256: binding, state: f.state })).status, 400);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: Buffer.alloc(32, 5).toString("base64url") })).status, 400);
  assert.equal(signs, 0);
  assert.equal((await review()).status, 200);
  await post("/record", { state: { ...f.state, connected: false, serialOwnershipReleased: true } });
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal((await review()).status, 200);
  clock += 45000;
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
  assert.equal(signs, 0);
  assert.equal((await review()).status, 200);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 200);
  assert(signs > 0);
  assert.equal((await post("/authorization-context", { controlSessionBindingSha256: binding })).status, 400);
});
