import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createObserverRoutes } from "./observer.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";

async function fixture(t, options = {}) {
  const root = await mkdtemp(join(tmpdir(), "v2-observer-"));
  const context = { scope: "share", cadence_observer: { sha256: "a".repeat(64) } };
  const candidate = state(context); let now = 1000, connected = false, finishes = 0, maybePoll, cleared = false;
  const failures = [], fail = code => { if (failures.length === 0) failures.push(code); };
  const owner = { status: () => ({ connected, alive: connected, failed: false }),
    async start() { await options.beforeStart?.(); connected = true; await options.afterStart?.(); return { connected: true }; },
    async finish() { finishes++; connected = false; return { cleanupComplete: true, closed: true, reason: "requested", exitCode: 0 }; } };
  const journal = { lastState: () => ({ state: candidate, sequence: 1 }), async state() {} };
  const routes = createObserverRoutes(root, context, { now: () => now, ready() {}, failed: () => failures.length > 0, fail, journal,
    operations: { createObserver: () => owner, setInterval(callback, ms) {
      assert.equal(ms, 100); maybePoll = callback; return { unref() {} };
    }, clearInterval() { cleared = true; } } });
  const endpoint = { schema: "worker-telemetry-endpoint-v1", ipv4: "192.168.1.20", httpPort: 80,
    observedAtUs: 100, bootOrdinal: 2, generation: 3, controlSessionBindingSha256: Buffer.alloc(32, 4).toString("base64url") };
  async function start() { const challenge = await routes.handle("/observer/context", {});
    return routes.handle("/observer/start", { nonce: challenge.nonce, endpoint, state: candidate }); }
  t.after(async () => {
    try { await routes.finish(); } catch (error) { assert(options.expectedCleanupFailure, error.message); }
    await rm(root, { recursive: true });
  });
  return { root, routes, endpoint, start, candidate, failures, fail,
    poll() { maybePoll?.(); }, cleared: () => cleared,
    setNow(value) { now = value; }, disconnect() { connected = false; }, finishes: () => finishes };
}

test("fault tail is measured from the exact confirmation anchor and retained without endpoints", async t => {
  // Arrange
  const f = await fixture(t); await f.start(); f.setNow(1100); f.routes.markFault(1050);
  // Act / Assert
  f.setNow(6049); await assert.rejects(f.routes.handle("/observer/finish", {}), { code: "v2_observer_fault_tail" });
  f.setNow(6050); assert.deepEqual(await f.routes.handle("/observer/finish", {}), { observer_closed: true, tail_ms: 5000 });
  assert.equal(f.finishes(), 1);
  const stop = JSON.parse(await readFile(join(f.root, "observer-stop.json")));
  assert.equal(stop.tailMs, 5000);
  const claim = await readFile(join(f.root, "observer-start.claim.json"), "utf8");
  for (const secret of [f.endpoint.ipv4, f.endpoint.controlSessionBindingSha256, "httpPort"]) assert.equal(claim.includes(secret), false);
});

test("stale endpoint admission never starts the observer", async t => {
  const f = await fixture(t), challenge = await f.routes.handle("/observer/context", {}); f.setNow(6001);
  await assert.rejects(f.routes.handle("/observer/start", { nonce: challenge.nonce, endpoint: f.endpoint, state: f.candidate }),
    { code: "v2_observer_freshness" });
  assert.throws(() => f.routes.alive(), { code: "v2_observer_not_live" });
});

test("disconnect and future fault clocks fail instead of manufacturing a five-second tail", async t => {
  const f = await fixture(t); await f.start();
  assert.throws(() => f.routes.markFault(1001), { code: "v2_observer_fault_clock" });
  f.disconnect(); assert.throws(() => f.routes.markFault(1000), { code: "v2_observer_not_live" });
});

const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; };

test("finish joins a pending start before closing its future child", async t => {
  // Arrange
  const entered = deferred(), release = deferred();
  const f = await fixture(t, { beforeStart: async () => { entered.resolve(); await release.promise; } });
  const starting = f.start(); await entered.promise;
  // Act
  const closing = f.routes.finish();
  assert.equal(f.finishes(), 0);
  release.resolve();
  const results = await Promise.allSettled([starting, closing]);
  // Assert
  assert.equal(results[0].status, "rejected");
  assert.equal(results[0].reason.code, "v2_observer_stopping");
  assert.equal(results[1].status, "fulfilled");
  assert.equal(f.finishes(), 1);
});

test("interrupted startup exceeding cleanup bound still closes the actual owner", async t => {
  // Arrange
  const entered = deferred(), release = deferred();
  const f = await fixture(t, { expectedCleanupFailure: true,
    beforeStart: async () => { entered.resolve(); await release.promise; } });
  const starting = f.start(); await entered.promise;
  // Act
  const closing = f.routes.finish(); f.setNow(7000); release.resolve();
  const results = await Promise.allSettled([starting, closing]);
  // Assert: elapsed time is an unverified cleanup, never permission to leak.
  assert.equal(results[1].status, "rejected");
  assert.equal(results[1].reason.code, "v2_observer_cleanup");
  assert.equal(f.finishes(), 1);
});

test("a failed start is joined and cleaned without replacing its first cause", async t => {
  // Arrange
  const error = Object.assign(Error("closed test failure"), { code: "v2_test_start_failed" });
  const f = await fixture(t, { expectedCleanupFailure: true, afterStart() { throw error; } });
  // Act
  await assert.rejects(f.start(), { code: error.code });
  await assert.rejects(f.routes.finish(), { code: error.code });
  // Assert
  assert.equal(f.finishes(), 1);
  assert.deepEqual(f.failures, ["v2_observer_start_failed"]);
});

test("passive poll latches disconnect before a later unrelated failure", async t => {
  // Arrange
  const f = await fixture(t); await f.start();
  // Act
  f.disconnect(); f.poll(); f.fail("v2_browser_failed");
  // Assert
  assert.deepEqual(f.failures, ["v2_observer_disconnected"]);
});

test("route-entry observation catches a disconnect before the next timer tick", async t => {
  // Arrange
  const f = await fixture(t); await f.start();
  // Act
  f.disconnect(); f.routes.observeFailure();
  // Assert
  assert.deepEqual(f.failures, ["v2_observer_disconnected"]);
});

test("intentional finish clears polling and cannot produce a disconnect failure", async t => {
  // Arrange
  const f = await fixture(t); await f.start();
  // Act
  await f.routes.finish(); f.poll(); f.routes.observeFailure();
  // Assert
  assert(f.cleared()); assert.deepEqual(f.failures, []); assert.equal(f.finishes(), 1);
});
