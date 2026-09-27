import test from "node:test";
import assert from "node:assert/strict";
import { connect } from "node:net";
import { mkdtemp, rm, chmod, lstat, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { exchange, serveOperatorSocket } from "./operator-ipc.mjs";
import { createRequestStore } from "./operator-state.mjs";
const digest = "a".repeat(64), id = "b".repeat(32);
const request = { schema: "str005-v2-operator-request-v1", contextSha256: digest, requestId: id, action: "install", payload: { index: 0 } };
async function fixture(t) {
  const root = await mkdtemp(resolve(await realpath(tmpdir()), "v2op-test-")); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true })); return root;
}
test("exclusive request survives reconnect and never redispatches identical body", async t => {
  // Arrange
  const root = await fixture(t), store = createRequestStore(root, digest, [0]);
  // Act
  const first = await store.accept(request), duplicate = await store.accept(JSON.parse(JSON.stringify(request)));
  // Assert
  assert.equal(first.first, true); assert.equal(duplicate.first, false); assert.equal(duplicate.value.status, "pending");
  await store.finish(request); assert.equal((await store.get(id)).status, "succeeded");
});
test("conflicting request remains consumed and rejects changed action", async t => {
  const store = createRequestStore(await fixture(t), digest, [0]);
  await store.accept(request);
  await assert.rejects(store.accept({ ...request, action: "stop", payload: {} }), { code: "v2_operator_conflict" });
  assert.equal((await store.get(id)).status, "pending");
});
test("real Unix socket has protected mode and survives disconnected clients", async t => {
  const root = await fixture(t), path = resolve(root, "s"); let calls = 0;
  const server = await serveOperatorSocket(path, async () => { calls++; return { received: true }; });
  t.after(() => new Promise(done => server.close(done)));
  const client = connect(path); await new Promise(done => client.once("connect", done)); client.destroy();
  assert.deepEqual(await exchange(path, { hello: true }), { received: true });
  assert.equal(calls, 1); assert.equal((await lstat(path)).mode & 0o777, 0o600);
});
test("real Unix socket joins fragmented frame and rejects coalesced or oversized frames", async t => {
  const root = await fixture(t), path = resolve(root, "s"); let calls = 0;
  const server = await serveOperatorSocket(path, value => { calls++; return value; });
  t.after(() => new Promise(done => server.close(done)));
  const client = connect(path); await new Promise(done => client.once("connect", done));
  const received = new Promise(done => client.once("data", data => done(data.toString())));
  client.write('{"ok":'); client.write('true}\n'); assert.equal(await received, '{"ok":true}\n');
  for (const bytes of ['{}\n{}\n', 'x'.repeat(16385), 'bad\n']) {
    const bad = connect(path); await new Promise(done => bad.once("connect", done));
    const closed = new Promise(done => { bad.once("close", done); bad.on("error", () => undefined); });
    bad.write(bytes); await closed;
  }
  assert.equal(calls, 1);
});
test("unsafe socket directory is rejected before service starts", async t => {
  const root = await fixture(t); await chmod(root, 0o755);
  await assert.rejects(serveOperatorSocket(resolve(root, "s"), () => ({})));
});
test("another request ID cannot re-admit a completed phase", async t => {
  // Arrange: the original operation completed under its exclusive phase claim.
  const store = createRequestStore(await fixture(t), digest, [0]);
  await store.accept(request); await store.finish(request);
  // Act / Assert: a fresh ID cannot dispatch or turn the old cached completion into a new effect.
  await assert.rejects(store.accept({ ...request, requestId: "c".repeat(32) }), { code: "v2_operator_conflict" });
  assert.equal((await store.accept(request)).first, false);
  assert.equal((await store.get(id)).status, "succeeded");
});

test("concurrent result polling remains pending until complete bytes are published", async t => {
  // Arrange: delay only publication, after the real protected pending write closes.
  const root = await fixture(t);
  let publish, entered;
  const gate = new Promise(done => { publish = done; }), waiting = new Promise(done => { entered = done; });
  const store = createRequestStore(root, digest, [0], { async beforeResultPublish() { entered(); await gate; } });
  await store.accept(request);
  // Act: real result reads overlap an unfinished publication operation.
  const finishing = store.finish(request); await waiting;
  const observed = await Promise.all(Array.from({ length: 20 }, () => store.get(id)));
  publish(); await finishing;
  // Assert: readers never see incomplete JSON or a premature successful outcome.
  assert.ok(observed.every(value => value.status === "pending"));
  assert.equal((await store.get(id)).status, "succeeded");
});

test("interrupted result publication remains pending and cannot be republished", async t => {
  const root = await fixture(t);
  const store = createRequestStore(root, digest, [0], { beforeResultPublish() { throw Error("synthetic publication interruption"); } });
  await store.accept(request);
  await assert.rejects(store.finish(request), /synthetic publication interruption/u);
  assert.equal((await store.get(id)).status, "pending");
  assert.equal((await store.accept(request)).first, false);
  await assert.rejects(store.finish(request), { code: "EEXIST" });
  assert.equal((await lstat(resolve(root, `result-${id}.pending.json`))).mode & 0o777, 0o600);
});
