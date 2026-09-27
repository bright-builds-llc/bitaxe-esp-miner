import assert from "node:assert/strict";
import test from "node:test";
import { clientFixture } from "./client.test-helper.mjs";
import { installNativeControls } from "./client-native-controls.mjs";
function fixture(maybeComposed) {
  const nodes = [], calls = [];
  const stop = { addEventListener(_, fn) { this.observeAbort = fn; } };
  const document = { getElementById(id) { return id === "stop" ? stop : null; }, createElement() { const node = { setAttribute() {}, append() {}, addEventListener(_, fn) { this.click = fn; } }; nodes.push(node); return node; }, body: { append() {} } };
  const state = maybeComposed?.state ?? { status: "ready", expectedFirmwareSourceCommit: "a".repeat(40), connected: true, running: false,
    deviceBaselineConfirmed: true, deviceLeaseInactive: true, preservation: { device_identity_match: true, settings_match: true, mine_on_boot: false } };
  const supervisor = maybeComposed?.api ?? {
    async recordAccounting(stage) { calls.push(stage); return { accounting_saved: true }; },
    async configureCandidate() { calls.push("configure"); return { configured: true }; },
    async recordCycle(index) { calls.push(index); return { cycle_verified: true, index }; },
    async run() { calls.push("run"); return { phase_complete: true, scope: "share", hardware_qualified: false, requires_native_reconnect: true, restoration_wait_remaining_ms: 139900, secret: "private-key" }; },
    async restoreAndRecord() { calls.push("restore"); return { restoration_recorded: true, accounting_recorded: true }; },
    async flush() { calls.push("flush"); },
  };
  const gate = maybeComposed?.gate ?? { async close() { calls.push("close"); Object.assign(state, { status: "closed", connected: false, serialOwnershipReleased: true }); } };
  const controls = installNativeControls(document, supervisor, gate, () => state, maybeComposed?.abort);
  const button = action => nodes.find(node => node.id === `v2-${action}`);
  const result = () => JSON.parse(nodes.find(node => node.id === "v2-native-result").textContent);
  async function prepare() {
    controls.observe(state); await button("accounting").click(); await button("close").click();
    await button("configure").click(); Object.assign(state, { status: "ready", connected: true }); controls.observe(state);
    for (let i = 0; i < 4; i++) await button("cycle").click();
  }
  return { controls, state, supervisor, button, result, calls, prepare, stop };
}
test("native controls expose literal readiness and fixed ordered cycle arguments", async () => {
  const f = fixture();
  assert.equal(f.result().status, "initializing"); await f.prepare();
  assert.deepEqual(f.calls, ["before-install", "close", "flush", "configure", 1, 2, 3, 4]);
  assert.equal(f.result().cycles_recorded, 4); assert.equal(f.button("run").disabled, false);
});
test("native control mutex prevents overlapping clicks and consumes Run permanently", async () => {
  const f = fixture(); await f.prepare(); let finish;
  const run = f.supervisor.run; f.supervisor.run = async () => { await new Promise(resolve => { finish = resolve; }); return run(); };
  const pending = f.button("run").click(); await f.button("run").click(); await f.button("close").click(); finish(); await pending; await f.button("run").click();
  assert.equal(f.calls.filter(value => value === "run").length, 1); assert.equal(f.result().run_consumed, true);
});
test("Run result retains allowlisted recovery timing without private metadata", async () => {
  const f = fixture(); await f.prepare(); await f.button("run").click();
  assert.equal(f.result().restoration_wait_remaining_ms, 139900); assert.equal(JSON.stringify(f.result()).includes("private-key"), false);
});
test("fresh page cannot recover cycle progress or run from displayed ready state", async () => {
  const f = fixture(); f.controls.observe(f.state); await f.button("run").click();
  assert.equal(f.calls.length, 0); assert.equal(f.result().status, "ready");
});
test("missing baseline prevents accounting and work", async () => {
  const f = fixture(); delete f.state.preservation; f.controls.observe(f.state);
  await f.button("accounting").click(); await f.button("run").click();
  assert.equal(f.calls.length, 0);
});
test("failure is redacted and effect controls stay consumed while close remains available", async () => {
  const f = fixture(); await f.prepare(); f.supervisor.run = async () => { throw Error("secret-private-endpoint"); }; await f.button("run").click();
  assert.equal(f.result().code, "operation_failed"); assert.equal(f.button("run").disabled, true); assert.equal(f.button("close").disabled, false);
  assert.equal(JSON.stringify(f.result()).includes("secret"), false);
});
test("close confirms release only after journal flush succeeds", async () => {
  const f = fixture(); f.controls.observe(f.state); f.supervisor.flush = async () => { throw Error(); }; await f.button("close").click();
  assert.equal(f.result().status, "failed"); assert.equal(f.result().journal_flushed, undefined);
});

test("native control module is served only at its fixed path with its inventoried digest", async () => {
  // Arrange.
  const { serveAsset } = await import("./server-assets.mjs");
  const { readFile } = await import("node:fs/promises");
  const { createHash } = await import("node:crypto");
  const path = "scripts/str005-v2-serial/client-native-controls.mjs", bytes = await readFile(path);
  const context = { firmware_root: process.cwd(), evaluator: [{ path, sha256: createHash("sha256").update(bytes).digest("hex") }] };
  let received;
  const response = { writeHead(status) { assert.equal(status, 200); }, end(value) { received = value; } };
  // Act.
  const served = await serveAsset("unused", context, "/client-native-controls.mjs", response);
  // Assert.
  assert.equal(served, true); assert.deepEqual(received, bytes);
  assert.equal(await serveAsset("unused", context, "/other-module.mjs", response), false);
  context.evaluator[0].sha256 = "0".repeat(64);
  await assert.rejects(serveAsset("unused", context, "/client-native-controls.mjs", response), /v2_asset_changed/u);
});


test("native controls compose the production coordinator through Channel and restoration", async () => {
  // Arrange.
  const composed = clientFixture(), f = fixture(composed); await f.prepare();
  // Act.
  await f.button("run").click(); composed.nativeReconnect(); f.controls.observe(f.state);
  await f.button("restore").click(); await f.button("close").click();
  // Assert.
  assert.equal(composed.counts.channelStart, 1); assert.equal(composed.counts.connect, 0);
  assert.equal(f.result().restoration_recorded, true); assert.equal(f.result().journal_flushed, true);
});

test("emergency Stop bypasses UI mutex and prevents a pending Channel start", async () => {
  // Arrange.
  const composed = clientFixture(), f = fixture(composed); await f.prepare();
  let release, entered;
  const entering = new Promise(resolve => { entered = resolve; }), review = composed.gate.reviewQualificationAttempts;
  composed.gate.reviewQualificationAttempts = async () => { entered(); await new Promise(resolve => { release = resolve; }); return review(); };
  // Act: the existing Gate Stop handler remains independently installed and callable.
  const pending = f.button("run").click(); await entering;
  f.stop.observeAbort(); await composed.gate.stop(); release(); await pending;
  // Assert.
  assert.equal(composed.counts.channelStart, 0); assert.equal(f.result().status, "failed");
  assert.equal(f.result().run_consumed, true);
  assert.ok(composed.calls.some(row => row.path === "/client-failure"));
});
