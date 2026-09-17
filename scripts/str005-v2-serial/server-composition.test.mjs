import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture } from "./context-fixtures.mjs";
import { createSupervisor } from "./server.mjs";
import { createV2Coordinator } from "./client.mjs";
import { readJournal } from "./journal.mjs";
import { ledger, original, state } from "../str005-noise-serial/test-fixture.mjs";

async function serving(t) {
  const f = await contextFixture(t);
  f.operations.processSnapshot = async () => [{ pid: process.pid, pgid: process.pid, ppid: 1,
    startedAt: "synthetic-http-owner", state: "S", cpuPercent: 0 }];
  const server = await createSupervisor(f.options, f.operations);
  server.listen(0, "127.0.0.1"); await once(server, "listening"); await server.qualificationReady;
  const origin = `http://127.0.0.1:${server.address().port}`;
  t.after(async () => {
    server.closeAllConnections(); await new Promise(done => server.close(done)); await server.closeQualificationResources();
  });
  const request = async (path, input, method = "POST") => {
    const response = await fetch(`${origin}${path}`, { method, headers: { "Content-Type": "application/json", Origin: origin },
      ...(method === "POST" ? { body: JSON.stringify(input) } : {}) });
    const value = await response.json();
    if (!response.ok) throw Object.assign(new Error(value.error), { code: value.error });
    return value;
  };
  return { ...f, server, origin, request };
}

test("actual HTTP supervisor and browser coordinator retain final ordered states after accounting", async t => {
  // Arrange: only the physical Gate device is simulated; HTTP, coordinator and journal are production modules.
  const f = await serving(t); let current = state(f.context, "before"), coordinator;
  const gate = { async reviewQualificationAttempts() { return structuredClone(ledger); }, async reviewBudget() { return structuredClone(original); },
    async refresh() { coordinator.observe(structuredClone(current)); return structuredClone(current); } };
  coordinator = createV2Coordinator({ gate, request: f.request, published: () => structuredClone(current), now: () => 100, sleep: async () => {} });
  // Act.
  assert.deepEqual(await coordinator.supervisor.recordAccounting("before-install"), { accounting_saved: true, stage: "before-install" });
  current = state(f.context, "before", true); coordinator.observe(current); await coordinator.supervisor.flush();
  assert.deepEqual((await f.request("/supervisor-state", undefined, "GET")), { scope: "channel", phase: "before", failed: false });
  await f.server.closeQualificationResources();
  const rows = await readJournal(f.root, f.context);
  // Assert.
  assert.equal(rows.length, 2); assert.equal(rows.at(-1).state.status, "closed");
  assert.equal(rows.at(-1).state.serialOwnershipReleased, true);
  const accounting = JSON.parse(await readFile(resolve(f.root, "accounting-before-install.json")));
  assert.equal(accounting.observedSequence, 1);
});

test("actual HTTP record failure latches source sequence and preserves later cleanup journal", async t => {
  // Arrange.
  const f = await serving(t), failed = { ...state(f.context, "before"), failure: "probe_failed", serialFailureCategory: "wire_bound" };
  // Act.
  const recorded = await f.request("/record", { state: failed });
  await f.request("/record", { state: state(f.context, "before", true) });
  await assert.rejects(f.request("/start/claim", { status: {} }), { code: "v2_browser_failed" });
  assert.equal((await f.request("/supervisor-state", undefined, "GET")).failed, true);
  await f.server.closeQualificationResources();
  // Assert.
  const failure = JSON.parse(await readFile(resolve(f.root, "failure.json")));
  assert.equal(failure.code, "v2_browser_failed"); assert.equal(failure.sourceSequence, recorded.sequence);
  const rows = await readJournal(f.root, f.context);
  assert.equal(rows.length, 2); assert.equal(rows.at(-1).state.status, "closed");
});

test("actual HTTP Channel exposes no signing route and rejects cross-origin control before journaling", async t => {
  // Arrange.
  const f = await serving(t);
  // Act / Assert.
  const config = await f.request("/context", undefined, "GET");
  assert.equal(config.stratumV2Qualification, "before"); assert.equal(config.stratumV2Scope, "channel");
  const response = await fetch(`${f.origin}/record`, { method: "POST", headers: { Origin: "http://invalid.example", "Content-Type": "application/json" }, body: JSON.stringify({ state: state(f.context, "before") }) });
  assert.equal(response.status, 400);
  await assert.rejects(readFile(resolve(f.root, "state-0001.json")), { code: "ENOENT" });
  await assert.rejects(f.request("/authorization-context", {}), { code: "v2_origin_rejected" });
});
