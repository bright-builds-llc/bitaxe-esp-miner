import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import vm from "node:vm";
import test from "node:test";
import { example, admitted } from "./fixtures.mjs";
import { state, ledger, original } from "./test-fixture.mjs";

async function browser(options = {}) {
  const context = { gate_commit: "b".repeat(40), firmware_commit: "a".repeat(40), app_elf_sha256: "c".repeat(64) };
  let current = state(context), callback, now = 0, binding = 0, started = 0, epoch = 2, jsonBodies = 0;
  const requests = [], calls = [];
  const output = { textContent: JSON.stringify(current) };
  function publish() { output.textContent = JSON.stringify(current); if (callback) queueMicrotask(callback); }
  const accepted = example().device; accepted.schema = "worker-noise-diagnostic-status-v2";
  accepted.job.resources.deadlineAtUs = accepted.job.authorityDeadlineUs + 5000000;
  const api = {
    state: () => structuredClone(current),
    refresh: async () => { calls.push("refresh"); publish(); },
    configure: async () => { calls.push("configure"); },
    reviewQualificationAttempts: async () => { binding++; calls.push("ledger"); return ledger; },
    reviewBudget: async () => { binding++; calls.push("original"); return original; },
    noiseDiagnosticPossession: async () => `private-binding-${++binding}`,
    noiseDiagnosticStatus: async (id, expected) => {
      assert.equal(expected, `private-binding-${binding}`);
      if (id === null) return { ...admitted(), schema: "worker-noise-diagnostic-status-v2", state: "idle", job: null };
      const result = structuredClone(accepted); result.observation.transportEpoch = epoch; return result;
    },
    noiseDiagnosticStart: async (_input, expected) => {
      assert.equal(expected, `private-binding-${binding}`); started++; calls.push("noise-start");
      if (options.failStart) throw new Error("ambiguous_start");
      return { ...admitted(), schema: "worker-noise-diagnostic-status-v2" };
    },
    noiseDiagnosticCancel: async () => { calls.push("cancel"); return { ...admitted(), schema: "worker-noise-diagnostic-status-v2", state: "cancelling" }; },
    stop: async () => { calls.push("stop"); current.status = "baseline_confirmed"; publish(); },
    close: async () => { calls.push("close"); current.status = "closed"; current.connected = false; current.serialOwnershipReleased = true; publish(); },
    probe: async () => {
      calls.push("probe"); if (options.probeWait) await options.probeWait;
      current.probe = { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }; publish(); return current.probe;
    },
  };
  const sandbox = { window: { workerAcceptance: api }, document: { getElementById: () => null, createElement: () => ({}),
    body: { append() {} }, querySelector: () => output },
    MutationObserver: class { constructor(fn) { callback = fn; } observe() {} },
    performance: { now: () => now }, setTimeout: (fn, ms) => { now += ms; queueMicrotask(fn); },
    fetch: async (path, init) => {
      const input = JSON.parse(init.body); requests.push({ path, input });
      let value = {};
      if (path === "/accounting-context") value = { campaignId: "synthetic-campaign" };
      if (path === "/start/claim") value = { ...example().start, schema: "worker-noise-diagnostic-start-v2" };
      if (path === "/restoration/context") value = { attemptId: example().start.attemptId };
      if (path === "/probe/claim") value = { probe_nonce: "fresh-probe-ticket" };
      return { ok: true, json: async () => { jsonBodies++; return value; } };
    },
  };
  vm.runInNewContext(await readFile(new URL("./client.mjs", import.meta.url), "utf8"), sandbox);
  return { driver: sandbox.window.noiseSupervisor, requests, calls, started: () => started, jsonBodies: () => jsonBodies,
    reconnect() { epoch++; current = state(context); publish(); } };
}
test("actual coordinator stops and closes before fresh reconnect/restoration accounting", async () => {
  const b = await browser();
  await b.driver.run();
  assert.deepEqual(b.calls.filter((name) => ["noise-start", "stop", "close"].includes(name)), ["noise-start", "stop", "close"]);
  await assert.rejects(b.driver.restoreAndRecord(), /noise_baseline_required/u);
  b.reconnect(); await b.driver.restoreAndRecord(); await b.driver.flush();
  assert.equal(b.requests.filter((row) => row.path === "/accounting").at(-1).input.stage, "after");
  assert.equal(b.calls.filter((name) => name === "stop").length, 1);
  assert.equal(b.jsonBodies(), b.requests.length);
  assert(!JSON.stringify(b.requests).includes("private-binding"));
  await assert.rejects(b.driver.run(), /noise_start_consumed/u);
});
test("ambiguous Start is never resent and cancellation remains bounded", async () => {
  const b = await browser({ failStart: true });
  await assert.rejects(b.driver.run(), /ambiguous_start/u);
  assert.equal(b.started(), 1); assert(b.calls.includes("cancel"));
  assert(b.requests.some((row) => row.path === "/client-failure"));
  await assert.rejects(b.driver.run(), /noise_start_consumed/u);
  assert.equal(b.started(), 1);
});
test("fresh probe receipt waits for the actual Gate probe promise", async () => {
  let release;
  const b = await browser({ probeWait: new Promise((resolve) => { release = resolve; }) });
  const running = b.driver.recordCycle(1);
  while (!b.calls.includes("probe")) await new Promise((resolve) => setImmediate(resolve));
  assert(!b.requests.some((row) => row.path === "/probe/complete"));
  release(); await running;
  const completed = b.requests.find((row) => row.path === "/probe/complete");
  assert.equal(completed.input.nonce, "fresh-probe-ticket");
  assert.equal(completed.input.probe.requestPayloadBytes, 65536);
});
