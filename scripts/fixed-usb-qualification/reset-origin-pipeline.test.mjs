import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import vm from "node:vm";
import { resetOriginFixture, resetDiagnostics } from "./reset-origin-fixtures.mjs";
import { recoveryState, recordRecoveryState } from "./cadence-startup-fixtures.mjs";
import { createResetOriginSupervisor } from "./reset-origin-server.mjs";
import { judgeResetOrigin, readResetOrigin } from "./reset-origin-judge.mjs";
import { readJson, writeNew } from "./contract.mjs";

async function pipeline(t) {
  let closeServer = async () => {};
  t.after(() => closeServer());
  const f = await resetOriginFixture(t);
  let clock = 100, nextTimer = 0, state = recoveryState(f.context.no_mining_context);
  const timers = new Map(), observers = [], calls = [];
  const stateNode = { textContent: JSON.stringify(state) };
  const diagnosticNode = { get textContent() { return JSON.stringify(resetDiagnostics(f.context, clock + 1000)); } };
  function publish() {
    stateNode.textContent = JSON.stringify(state);
    for (const notify of observers) queueMicrotask(notify);
  }
  function schedule(callback, delay) {
    const id = ++nextTimer;
    timers.set(id, { callback, at: clock + delay });
    if (delay <= 250) setImmediate(() => {
      const pending = timers.get(id);
      if (!pending) return;
      clock = pending.at;
      for (const [key, timer] of [...timers]) if (timer.at <= clock && timers.delete(key)) timer.callback();
    });
    return id;
  }
  await recordRecoveryState(f.root, f.context.no_mining_context, state);
  const server = await createResetOriginSupervisor({ privateRoot: f.root }, { ...f.operations, now: () => clock });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const origin = `http://127.0.0.1:${server.address().port}`;
  let serverClosed = false;
  closeServer = async () => {
    if (serverClosed) return;
    serverClosed = true;
    try { await server.closeQualificationResources(); }
    finally {
      const closed = once(server, "close");
      server.close(); server.closeAllConnections();
      await closed;
    }
  };
  const window = { workerAcceptance: {
    state: () => structuredClone(state),
    refresh: async () => { calls.push("status"); publish(); },
    reviewQualificationAttempts: async () => { calls.push("ledger"); return structuredClone(f.ledger); },
    reviewBudget: async () => { calls.push("budget"); return structuredClone(f.original); },
    close: async () => {
      calls.push("close");
      state = { ...state, status: "closing" }; publish();
      await new Promise(setImmediate);
      state = { ...state, status: "closed", connected: false, serialOwnershipReleased: true }; publish();
    },
  } };
  const context = vm.createContext({ window, document: {
    getElementById: () => undefined, createElement: () => ({}), body: { append() {} },
    querySelector: selector => selector === "#state" ? stateNode : selector === "#diagnostics" ? diagnosticNode : null,
  }, MutationObserver: class { constructor(callback) { this.callback = callback; } observe() { observers.push(this.callback); } },
  performance: { now: () => clock }, setTimeout: schedule, clearTimeout: id => timers.delete(id), AbortController,
  fetch: async (path, options = {}) => fetch(origin + path, { ...options,
    headers: { ...options.headers, ...(options.method === "POST" ? { Origin: origin } : {}) } }),
  });
  for (const name of ["no-mining-client.mjs", "reset-origin-client.mjs"])
    vm.runInContext(`{${await readFile(new URL(name, import.meta.url), "utf8")}\n}`, context, { filename: name });
  return { ...f, window, calls, timers, closeServer };
}

test("both production clients produce evidence accepted by the real supervisor and unchanged independent judge", { timeout: 120000 }, async t => {
  // Arrange
  const f = await pipeline(t);
  // Act
  await f.window.resetOriginObserver.run();
  await f.closeServer();
  const end = await readJson(resolve(f.root, "reset-origin-end.json"));
  const after = await readJson(resolve(f.root, "no-mining-accounting-after.json"));
  const cleanupPath = resolve(f.root, "cleanup.json");
  await writeNew(cleanupPath, { schema: "worker-reset-origin-cleanup-v1", source: "parent-observed", browser_closed: true,
    supervisor_exited: true, supervisor_exit_code: 0, listener_absent: true, owned_children_absent: true, serial_holders_absent: true });
  const result = await judgeResetOrigin(f.root, cleanupPath, f.operations);
  await readResetOrigin(resolve(f.root, "result.json"), f.operations);
  // Assert
  assert(end.observed_sequence < after.observed_sequence);
  assert.equal(result.result, "observed_stable");
  assert.equal(result.mining_authorized, false);
  assert.equal(result.reset_origin_resolved, false);
  assert.equal(f.calls.filter(value => value === "close").length, 1);
  assert.equal(f.calls.filter(value => value === "ledger").length, 2);
  assert.equal(f.calls.filter(value => value === "budget").length, 2);
  assert.equal(f.timers.size, 0);
});
