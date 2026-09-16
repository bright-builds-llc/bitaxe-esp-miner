import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { fixture, state } from "./test-fixture.mjs";
import { createSupervisor } from "./server.mjs";
import { admitted } from "./fixtures.mjs";

async function serving(t) {
  const f = await fixture(t);
  f.operations.processSnapshot = async () => [{ pid: process.pid, pgid: process.pid, ppid: 1, startedAt: "synthetic-start", state: "S", cpuPercent: 0 }];
  const server = await createSupervisor(f.options, f.operations);
  server.listen(0, "127.0.0.1"); await once(server, "listening"); await server.qualificationReady;
  const origin = `http://127.0.0.1:${server.address().port}`;
  t.after(async () => { server.closeAllConnections(); await new Promise((done) => server.close(done)); await server.closeQualificationResources(); });
  const request = (path, input) => fetch(`${origin}${path}`, input === undefined ? {} : {
    method: "POST", headers: { "Content-Type": "application/json", Origin: origin }, body: JSON.stringify(input) });
  return { ...f, server, origin, request };
}
test("actual HTTP page is HTML and context has the exact frozen Gate configuration shape", async (t) => {
  const f = await serving(t);
  const page = await f.request("/"); assert.match(page.headers.get("content-type"), /text\/html/u);
  assert.match(await page.text(), /^<!doctype html>/u);
  const context = await (await f.request("/context")).json();
  assert.deepEqual(Object.keys(context).sort(), ["expectedAppElfSha256", "expectedFirmwareSourceCommit", "expectedGateCommit", "noiseIdentities", "noiseQualification", "trust"].sort());
  assert.equal(context.noiseQualification, "before");
  assert.equal(context.noiseIdentities.before.firmwareSourceCommit, f.context.before_source.firmware_commit);
});
test("actual HTTP observer records published state without a signing or mining route", async (t) => {
  const f = await serving(t);
  const saved = await (await f.request("/record", { state: state(f.context, "before") })).json();
  assert.equal(saved.recorded, true);
  assert.equal((await f.request("/authorize", {})).status, 400);
  assert.equal((await f.request("/supervisor-state")).status, 200);
  assert.equal((await (await f.request("/supervisor-state")).json()).failed, true);
});
test("typed producer failure survives later malformed requests and attempted late success", async (t) => {
  const f = await serving(t), failed = admitted();
  failed.schema = "worker-noise-diagnostic-status-v2"; failed.state = "cancelling";
  failed.job.attemptId = f.context.attempt_id;
  failed.observation.observedAtUs = 5000;
  failed.job.firstFailure = { stage: "authority_verified", category: "authentication", detail: "wrong_authority", atUs: 2000 };
  assert.equal((await f.request("/noise/record", { status: failed })).status, 200);
  await f.server.closeQualificationResources();
  const original = await readFile(resolve(f.root, "failure.json"), "utf8");
  const failure = JSON.parse(original);
  assert.deepEqual(failure.cause, { stage: "authority_verified", category: "authentication", detail: "wrong_authority" });
  assert.equal(failure.provenance.kind, "device");
  await f.request("/unknown", {});
  const cleared = structuredClone(failed); cleared.job.firstFailure = null;
  assert.equal((await f.request("/noise/record", { status: cleared })).status, 400);
  assert.equal(await readFile(resolve(f.root, "failure.json"), "utf8"), original);
});
test("cross-origin writes are rejected before recording", async (t) => {
  const f = await serving(t);
  const result = await fetch(`${f.origin}/record`, { method: "POST", headers: { Origin: "http://invalid.example", "Content-Type": "application/json" }, body: JSON.stringify({ state: state(f.context, "before") }) });
  assert.equal(result.status, 400);
  await assert.rejects(readFile(resolve(f.root, "state-0001.json")), { code: "ENOENT" });
});
