import assert from "node:assert/strict";
import test from "node:test";
import { clientFixture, ATTEMPT, BINDING } from "./client.test-helper.mjs";
import { installV2Coordinator } from "./client.mjs";
const labels = f => f.calls.map(row => row.name ?? row.path);

test("Channel performs one asynchronous Start, tuple collection and normal close without mining", async () => {
  const f = clientFixture();
  const result = await f.api.run();
  assert.equal(result.phase_complete, true); assert.equal(result.hardware_qualified, false);
  assert.equal(f.counts.channelStart, 1); assert.equal(f.counts.shareStart, 0); assert.equal(f.counts.connect, 0);
  assert.equal(f.calls.filter(row => row.path === "/protocol/connection").length, 1);
  assert.ok(labels(f).indexOf("/fixture/start") < labels(f).indexOf("/start/claim"));
  assert.ok(labels(f).indexOf("/protocol/complete") < labels(f).indexOf("stop"));
  await assert.rejects(f.api.run(), /v2_client_start_consumed/u); assert.equal(f.counts.channelStart, 1);
  assert.equal(JSON.stringify(result).includes(ATTEMPT), false);
});
test("Share uses reviewed binding, selects actual ACK, retains exact headroom and returns before the recovery wait", async () => {
  const f = clientFixture("share");
  const result = await f.api.run();
  const order = labels(f);
  for (const [before, after] of [["cooling", "reviewed_binding"], ["reviewed_binding", "endpoint"], ["/observer/start", "/fixture/start"], ["/start/network", "sign"], ["sign", "load"], ["load", "share_start"], ["/fault/claim", "suppress"], ["/fault/confirm", "/observer/finish"], ["/observer/finish", "/protocol/complete"]]) assert.ok(order.indexOf(before) < order.indexOf(after), `${before} precedes ${after}`);
  assert.equal(f.counts.proof, 0); assert.equal(f.counts.shareStart, 1); assert.equal(f.counts.channelStart, 0); assert.equal(f.counts.connect, 0);
  assert.equal(f.sleeps.includes(145000), false); assert.equal(result.restoration_wait_remaining_ms, 139900);
  const confirmed = f.calls.find(row => row.path === "/fault/confirm").input;
  assert.equal(confirmed.headroom.leaseRemainingMs, 47000); assert.equal(confirmed.headroom.workGateRemainingMs, 123000);
  assert.equal(f.calls.find(row => row.path === "/start/network").input.controlSessionBindingSha256, BINDING);
  const exposed = JSON.stringify({ result, durable: f.durable, notices: f.notices });
  for (const secret of ["192.168.77.", "private-fixture-user", BINDING, '"stratum"', '"socket"']) assert.equal(exposed.includes(secret), false);
});
test("Share restoration keeps the original high-water mismatch and requires the post-work checkpoint", async () => {
  const f = clientFixture("share"), result = await f.api.run();
  f.advance(result.restoration_wait_remaining_ms); f.nativeReconnect();
  const restored = await f.api.restoreAndRecord();
  assert.equal(restored.hardware_qualified, false); assert.equal(restored.restoration_recorded, true);
  assert.equal(f.state.preservation.authorization_high_water_match, false);
  assert.equal(f.state.authorizationRecovery.matched, true); assert.equal(f.counts.connect, 0);
  assert.equal(f.calls.filter(row => row.path === "/accounting").at(-1).input.stage, "after");
});
test("early Share restoration performs no possession or accounting", async () => {
  const f = clientFixture("share"); await f.api.run(); f.nativeReconnect();
  const before = f.counts.proof;
  await assert.rejects(f.api.restoreAndRecord(), /v2_client_operation_failed/u);
  assert.equal(f.counts.proof, before); assert.equal(f.calls.some(row => row.path === "/accounting" && row.input.stage === "after"), false);
});
test("lost Start reply is never retried or mislabeled as delivered for cancellation", async () => {
  const f = clientFixture(); f.throwAt("channel_start");
  await assert.rejects(f.api.run(), /v2_client_operation_failed/u);
  assert.equal(f.counts.cancel, 0); assert.equal(labels(f).filter(value => value === "channel_start").length, 1);
  assert.ok(f.counts.close > 0);
  const failure = f.calls.find(row => row.path === "/client-failure").input;
  assert.deepEqual(failure, { phase: "start", code: "operation_failed" });
  assert.equal(JSON.stringify(f.notices).includes("private"), false);
});
test("failure after acknowledged Channel Start attempts cancellation and bounded normal cleanup", async () => {
  const f = clientFixture(); f.throwAt("status");
  await assert.rejects(f.api.run(), /v2_client_operation_failed/u);
  assert.equal(f.counts.cancel, 1); assert.equal(f.counts.close, 1);
  assert.equal(f.calls.filter(row => row.path === "/client-failure").length, 1);
});
test("each maximum-exchange probe is scoped and account/config outputs remain metadata only", async () => {
  const f = clientFixture();
  await f.api.recordAccounting("before-install");
  for (let index = 1; index <= 4; index++) assert.deepEqual(await f.api.recordCycle(index), { cycle_verified: true, index });
  assert.equal(labels(f).filter(value => value === "probe").length, 4);
  f.state.status = "closed"; f.state.connected = false; f.state.serialOwnershipReleased = true;
  assert.deepEqual(await f.api.configureCandidate(), { configured: true });
});
test("bootstrap removes manual work controls and the binding textarea without requesting Serial permission", () => {
  const removed = [], page = { workerAcceptance: {} }, notice = {};
  const document = { getElementById: id => ({ remove: () => removed.push(id) }), createElement: () => notice, body: { append() {} }, querySelector: () => null };
  installV2Coordinator(page, document, async () => { throw Error("unexpected_fetch"); });
  assert.ok(removed.includes("authorization-context")); assert.ok(removed.includes("start")); assert.equal(removed.includes("connect"), false);
  assert.deepEqual(Object.keys(page.v2Supervisor), ["flush", "recordAccounting", "configureCandidate", "recordCycle", "run", "restoreAndRecord"]);
});

test("Channel host horizon cannot be hidden by a late terminal reply", async () => {
  const f = clientFixture(), original = f.gate.stratumV2Status;
  f.gate.stratumV2Status = async (...args) => { if (args[1] !== null) f.advance(125001); return original(...args); };
  await assert.rejects(f.api.run(), /v2_client_observation_horizon/u);
  assert.equal(f.counts.channelStart, 1);
  assert.equal(f.calls.some(row => row.path === "/protocol/complete"), false);
});
test("Share cannot report restoration from a checkpoint for another generation", async () => {
  const f = clientFixture("share"), result = await f.api.run();
  f.advance(result.restoration_wait_remaining_ms); f.nativeReconnect();
  f.state.preservation.authorization_high_water_match = true;
  f.state.authorizationRecovery.generation = 999;
  await assert.rejects(f.api.restoreAndRecord(), /v2_client_operation_failed/u);
  assert.equal(f.calls.some(row => row.path === "/restoration"), false);
});

test("Share guards actual Start invocation against conservative fixture-request deadline", async () => {
  // Arrange.
  const f = clientFixture("share"), sign = f.gate.prepareStartAuthorization;
  f.gate.prepareStartAuthorization = async () => { const result = await sign(); f.advance(10001); return result; };
  // Act / Assert.
  await assert.rejects(f.api.run());
  assert.equal(f.counts.shareStart, 0);
  assert.equal(f.calls.some(row => row.path === "/share/start-observed"), false);
});

test("Share retains final idle observation and independent invocation/reply timing", async () => {
  // Arrange.
  const f = clientFixture("share"), start = f.gate.startWindow;
  f.gate.startWindow = async () => { const result = await start(); f.advance(20000); return result; };
  // Act.
  await f.api.run();
  // Assert: a 20-second synchronous reply does not falsely violate the ten-second invocation limit.
  const observed = f.calls.find(row => row.path === "/share/start-observed").input;
  assert.equal(observed.status.state, "idle");
  assert.deepEqual(observed.timing, { fixtureRequestAtPageMs: 0, startInvokedAtPageMs: 0, startRepliedAtPageMs: 20000 });
  assert.equal(f.calls.filter(row => row.name === "share_start").length, 1);
});

test("Share late successful Start reply remains unverified and never writes timing success", async () => {
  // Arrange.
  const f = clientFixture("share"), start = f.gate.startWindow;
  f.gate.startWindow = async () => { const result = await start(); f.advance(30001); return result; };
  // Act / Assert.
  await assert.rejects(f.api.run(), /v2_client_timeout/u);
  assert.equal(f.counts.shareStart, 1);
  assert.equal(f.calls.some(row => row.path === "/share/start-observed"), false);
});
