import assert from "node:assert/strict";
import test from "node:test";
import { createServer } from "node:http";
import { connect } from "node:net";
import { once } from "node:events";
import { body } from "../fixed-usb-qualification/http.mjs";
import { createSupervisorCleanup } from "./supervisor-cleanup.mjs";

const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; };
const cause = code => Object.assign(Error(code), { code });

test("closing resources unblocks the active request before its queue is joined", async () => {
  // Arrange
  const request = deferred(), order = [];
  const close = createSupervisorCleanup({ stop() { order.push("stop"); },
    async closeFixture() { order.push("fixture"); request.resolve(); },
    async closeObserver() { order.push("observer"); },
    async settleQueue() { await request.promise; order.push("queue"); },
    fail() { assert.fail("no cleanup failure"); }, async settled() { order.push("settled"); } });
  // Act
  const first = close(), second = close(); assert.equal(first, second); await first;
  // Assert
  assert.deepEqual(order, ["stop", "fixture", "observer", "queue", "settled"]);
});

test("fixture cleanup rejection never skips observer cleanup or failure flush", async () => {
  // Arrange
  const fixtureError = cause("v2_fixture_cleanup_failed"), observer = deferred(), failures = [], order = [];
  const close = createSupervisorCleanup({ stop() {}, async closeFixture() { throw fixtureError; },
    async closeObserver() { order.push("observer"); await observer.promise; },
    async settleQueue() { order.push("queue"); },
    fail(code) { if (failures.length === 0) failures.push(code); }, async settled() { order.push("settled"); } });
  // Act
  const closing = close(); assert.deepEqual(order, ["observer"]); observer.resolve();
  await assert.rejects(closing, { code: fixtureError.code });
  // Assert
  assert.deepEqual(order, ["observer", "queue", "settled"]);
  assert.deepEqual(failures, [fixtureError.code]);
});

test("the first observed cleanup failure survives a later second-owner failure", async () => {
  // Arrange
  const release = deferred(), first = cause("v2_observer_cleanup_failed"), failures = [];
  const close = createSupervisorCleanup({ stop() {},
    async closeFixture() { await release.promise; throw cause("v2_fixture_cleanup_failed"); },
    async closeObserver() { throw first; }, async settleQueue() {}, async settled() {},
    fail(code) { if (failures.length === 0) failures.push(code); } });
  // Act
  const closing = close(); await Promise.resolve(); release.resolve();
  await assert.rejects(closing, { code: first.code });
  // Assert
  assert.deepEqual(failures, [first.code]);
});

test("stop failure still attempts both owners and preserves that earlier failure", async () => {
  // Arrange
  const first = cause("v2_supervisor_stop_failed"), calls = [], failures = [];
  const close = createSupervisorCleanup({ stop() { throw first; },
    async closeFixture() { calls.push("fixture"); }, async closeObserver() { calls.push("observer"); },
    async settleQueue() {}, async settled() {}, fail(code) { failures.push(code); } });
  // Act / Assert
  await assert.rejects(close(), { code: first.code });
  assert.deepEqual(calls, ["fixture", "observer"]); assert.deepEqual(failures, [first.code]);
});

test("an actual unfinished HTTP body is aborted before waiting for the handler", async t => {
  // Arrange
  const entered = deferred(); let queue = Promise.resolve(), aborted = false;
  const server = createServer(request => {
    queue = (async () => { entered.resolve(); try { await body(request); } catch { aborted = true; } })();
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  const socket = connect(server.address().port, "127.0.0.1"); await once(socket, "connect");
  t.after(() => { socket.destroy(); server.closeAllConnections(); server.close(); });
  socket.write("POST /record HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 100\r\n\r\n{");
  await entered.promise;
  const closed = once(server, "close"), resourceCalls = [];
  const cleanup = createSupervisorCleanup({ stop() { server.close(); server.closeAllConnections(); },
    async closeFixture() { resourceCalls.push("fixture"); }, async closeObserver() { resourceCalls.push("observer"); },
    settleQueue: () => queue, fail() { assert.fail("unexpected cleanup failure"); }, async settled() {} });
  // Act
  await cleanup(); await closed;
  // Assert
  assert(aborted); assert.deepEqual(resourceCalls, ["fixture", "observer"]);
});
